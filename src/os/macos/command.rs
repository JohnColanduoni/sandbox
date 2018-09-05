use ::command::{Command, EnvAction};
use super::{CHANNEL_ENV_VAR};
use super::services::{MAX_MESSAGE_SIZE, BrokerMessage, TargetMessage};

use std::{io, env, mem, ptr, str, slice};
use std::io::{Read, Write};
use std::process::{self, Command as StdCommand, Child as StdChild, ExitStatus};
use std::fs::File;
use std::os::unix::prelude::*;

use futures::prelude::*;
use ipc::{RawMessageChannel, MessageChannel, ProcessHandle};
use tokio::current_thread::block_on_all;
use json;
use libc::{self, c_char, c_int, c_void};

pub struct Child {
    process_id: i32,
    error_rx: Option<File>,
    exit_status: Option<ExitStatus>,
    resumed: bool,
    channel: Option<MessageChannel<BrokerMessage, TargetMessage>>,
    policy: ::Policy,
}

impl Child {
    pub fn spawn(services: &mut ::BrokerServices, command: &mut Command) -> io::Result<Self> {
        let mut std_command = StdCommand::new(&command.program);
        std_command.env_clear();

        std_command.args(&command.arguments);
        if let Some(current_dir) = command.current_dir.as_ref() {
            std_command.current_dir(current_dir);
        }
        for (k, action) in command.envs.iter() {
            match action {
                EnvAction::Value(value) => {
                    std_command.env(k, value);
                },
                EnvAction::Inherit => if let Some(value) = env::var_os(k) {
                    std_command.env(k, value);
                },
            }
        }

        let (channel, (process_id, error_rx)) = RawMessageChannel::establish_with_child_custom(services.inner.event_loop.handle(), |child_channel| {
            std_command
                .env(CHANNEL_ENV_VAR, json::to_string(&child_channel).unwrap());
            // TODO: this doesn't matter because ProcessHandles don't do anything on macOS, but we should probably make this clearer
            Ok((ProcessHandle::current()?, do_spawn(&mut std_command, child_channel.as_raw_fd())?))
        })?;

        let channel = MessageChannel::<BrokerMessage, TargetMessage>::from_raw(channel, MAX_MESSAGE_SIZE)?;

        Ok(Child {
            process_id,
            error_rx: Some(error_rx),
            exit_status: None,
            resumed: false,
            channel: Some(channel),
            policy: command.policy.clone(),
        })
    }

    pub fn id(&self) -> u32 {
        self.process_id as u32
    }

    pub fn run(&mut self) -> io::Result<()> {
        if let Some(status) = self.exit_status {
            return Err(io::Error::new(io::ErrorKind::Other, "process has already exited"));
        }
        if self.resumed {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "process has already been resumed"));
        }
        let mut status: c_int = 0;
        // Wait for process to either exit or suspend itself with SIGSTOP
        unsafe { try_libc!(pid: libc::waitpid(self.process_id, &mut status, libc::WUNTRACED)); }
        if !unsafe { libc::WIFSTOPPED(status) } {
            let status = ExitStatus::from_raw(status);
            self.exit_status = Some(status);
            error!("spawned sandbox process exited before pausing for exec with status: {}", status);
            if let Some(error) = self.check_early_error() {
                return Err(error);
            } else {
                return Err(io::Error::new(io::ErrorKind::Other, "process has already exited"));
            }
        }
        // Resume process
        unsafe {
            try_libc!(libc::kill(self.process_id as i32, libc::SIGCONT));
        }
        self.resumed = true;

        // Send policy
        debug!("sending policy to sandboxed process");
        block_on_all(self.channel.take().unwrap().send(BrokerMessage::PolicySpec(self.policy.0.inner.clone())))?;
        debug!("policy successfully sent");

        Ok(())
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        if let Some(status) = self.exit_status {
            return Ok(status);
        }
        let mut status: c_int = 0;
        unsafe {
            try_libc!(pid: libc::waitpid(self.process_id, &mut status, 0), "waitpid failed: {}");
        }
        let status = ExitStatus::from_raw(status);
        self.exit_status = Some(status);
        if let Some(error) = self.check_early_error() {
            return Err(error);
        }
        Ok(status)
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        if let Some(status) = self.exit_status {
            return Ok(Some(status));
        }
        let mut status: c_int = 0;
        let pid = unsafe {
            try_libc!(pid: libc::waitpid(self.process_id, &mut status, libc::WNOHANG), "waitpid failed: {}")
        };
        if pid == 0 {
            Ok(None)
        } else {
            let status = ExitStatus::from_raw(status);
            self.exit_status = Some(status);
            if let Some(error) = self.check_early_error() {
                return Err(error);
            }
            Ok(Some(status))
        }
    }

    pub fn kill(&mut self) -> io::Result<()> {
        if let Some(status) = self.exit_status {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "process has already exited"));
        }
        unsafe { try_libc!(libc::kill(self.process_id, libc::SIGKILL)); }
        Ok(())
    }

    // Checks if a pre-exec or exec error was reported. Should only be called when we know the
    // child has exited as otherwise it could block.
    fn check_early_error(&mut self) -> Option<io::Error> {
        let mut bytes = [0u8; 4];
        if let Some(mut error_rx) = self.error_rx.take().and_then(|mut x| x.read_exact(&mut bytes).ok()) {
            // Reconstitute error
            let errno = (((bytes[0] as u32) << 24) | ((bytes[1] as u32) << 16) | ((bytes[2] as u32) << 8) | ((bytes[3] as u32) << 0)) as i32;
            return Some(io::Error::from_raw_os_error(errno))
        } else {
            return None;
        }
    }
}

fn do_spawn(command: &mut StdCommand, ipc_fd: c_int) -> io::Result<(i32, File)> {
    unsafe {
        // Any code that allocates needs to be done before fork (due to bugs in pthread_fork on some platforms)
        let fd_dir = try_libc!(ptr: libc::opendir(b"/dev/fd\0".as_ptr() as *const c_char)); // FIXME: release this
        let (mut error_tx, error_rx) = anon_pipe()?; // Used to transmit error between fork and exec in child

        match try_libc!(pid: libc::fork(), "fork failed: {}") {
            0 => {
                mem::drop(error_rx);
                let err = do_exec(command, fd_dir, &[0, 1, 2, ipc_fd, error_tx.as_raw_fd()]);
                let errno = err.raw_os_error().unwrap_or(libc::EINVAL) as u32;
                // If we get this far there was an error, emit the code to our parent via pipe
                assert!(error_tx.write(&[
                    (errno >> 24) as u8,
                    (errno >> 16) as u8,
                    (errno >>  8) as u8,
                    (errno >>  0) as u8,
                ]).is_ok());
                process::abort()
            },
            pid => {
                mem::drop(error_tx);
                // FIXME: check status of child process
                return Ok((pid, error_rx));
            },
        }
    }
}

unsafe fn do_exec(command: &mut StdCommand, fd_dir: *mut libc::DIR, excluded_fds: &[c_int]) -> io::Error {
    if let Err(err) = before_exec(fd_dir, excluded_fds) {
        return err;
    }

    libc::raise(libc::SIGSTOP);
    command.exec()
}

// WARNING: No allocation is allowed in this function
unsafe fn before_exec(fd_dir: *mut libc::DIR, excluded_fds: &[c_int]) -> io::Result<()> {
    // Close all file descriptors other than stdin, stdout, and stderr
    loop {
        let mut entry: libc::dirent = mem::zeroed();
        let mut result: *mut libc::dirent = ptr::null_mut();
        (libc::readdir_r(fd_dir, &mut entry, &mut result));
        if result.is_null() {
            break
        }
        if let Some(fd) = str::from_utf8(slice::from_raw_parts((*result).d_name.as_ptr() as *const u8, (*result).d_namlen as usize)).ok()
            .and_then(|x| x.parse::<c_int>().ok()) {
            if !excluded_fds.iter().any(|&x| x == fd) {
                try_libc!(fd: libc::close(fd));
            }
        }
    }

    Ok(())
}

fn anon_pipe() -> io::Result<(File, File)> {
    unsafe {
        let mut pipe_fds: [c_int; 2] = [0; 2];
        try_libc!(libc::pipe(pipe_fds.as_mut_ptr()));
        let pipe_read = File::from_raw_fd(pipe_fds[0]);
        let pipe_write = File::from_raw_fd(pipe_fds[1]);
        for &fd in pipe_fds.iter() {
            try_libc!(libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC));
        }
        Ok((pipe_write, pipe_read))
    }
}