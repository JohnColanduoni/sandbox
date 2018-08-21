use ::{platform, BrokerServices, Policy};

use std::{io};
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::{ExitStatus};

/// 
/// # Differences from `std::process::Command`
/// 
/// ## Program Path
/// 
/// The program path must be an absolute path; relative paths will be rejected.
/// 
/// ## Environment Variables
/// 
/// By default, `sandbox::Command` does not pass any environment variables to the child process.
/// Environment variables can either be specified explicitly or be flagged for inheritance by
/// `env_inherit`.
pub struct Command {
    pub(crate) program: PathBuf,
    pub(crate) policy: Policy,
    pub(crate) arguments: Vec<OsString>,
    pub(crate) envs: HashMap<OsString, EnvAction>,
    pub(crate) current_dir: Option<PathBuf>,
}

pub struct Child {
    inner: platform::Child,
}

impl Command {
    pub fn new(program: impl AsRef<Path>, policy: &Policy) -> Self {
        Command {
            program: program.as_ref().to_owned(),
            policy: policy.clone(),
            arguments: Default::default(),
            envs: Default::default(),
            current_dir: None,
        }
    }

    pub fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.arguments.push(arg.as_ref().to_owned());
        self
    }

    pub fn args(&mut self, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> &mut Self {
        self.arguments.extend(args.into_iter().map(|x| x.as_ref().to_owned()));
        self
    }

    pub fn env(&mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> &mut Self {
        self.envs.insert(key.as_ref().to_owned(), EnvAction::Value(value.as_ref().to_owned()));
        self
    }

    pub fn envs(&mut self, vars: impl IntoIterator<Item = (impl AsRef<OsStr>, impl AsRef<OsStr>)>) -> &mut Self {
        for (k, v) in vars {
            self.env(k, v);
        }
        self
    }

    pub fn env_remove(&mut self, key: impl AsRef<OsStr>) -> &mut Self {
        self.envs.remove(key.as_ref());
        self
    }

    pub fn env_clear(&mut self) -> &mut Self {
        self.envs.clear();
        self
    }

    pub fn env_inherit(&mut self, key: impl AsRef<OsStr>) -> &mut Self {
        self.envs.insert(key.as_ref().to_owned(), EnvAction::Inherit);
        self
    }

    pub fn current_dir(&mut self, dir: impl AsRef<Path>) -> &mut Self {
        self.current_dir = Some(dir.as_ref().to_owned());
        self
    }

    pub fn spawn(&mut self, services: &mut BrokerServices) -> io::Result<Child> {
        let inner = platform::Child::spawn(services, self)?;
        Ok(Child { inner })
    }
}

impl Child {
    pub fn id(&self) -> u32 {
        self.inner.id()
    }

    /// Causes the process to resume after the initial pause after being launched.
    pub fn run(&mut self) -> io::Result<()> {
        self.inner.run()
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        self.inner.wait()
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.inner.try_wait()
    }

    pub fn kill(&mut self) -> io::Result<()> {
        self.inner.kill()
    }
}

pub(crate) enum EnvAction {
    Inherit,
    Value(OsString),
}