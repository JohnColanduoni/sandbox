use ::{Command, PolicyPreset};

use std::{io};
use std::process::ExitStatus;
use std::os::windows::process::ExitStatusExt;

use winapi::shared::winerror::{WAIT_TIMEOUT};
use winapi::um::winbase::{INFINITE, WAIT_OBJECT_0};
use winapi::um::synchapi::{WaitForSingleObject};
use winapi::um::processthreadsapi::{GetExitCodeProcess, TerminateProcess};
use crsio2::{self, TokenLevel};

macro_rules! try_crsio2 {
    ($x:expr) => {
        match $x {
            ::std::result::Result::Ok(v) => v,
            ::std::result::Result::Err(err) => {
                return ::std::result::Result::Err(::std::io::Error::new(::std::io::ErrorKind::Other, format!("crsio2 error: {}", err)));
            }
        }
    }
}

pub struct Child {
    inner: crsio2::TargetProcess,
}

pub struct Policy {
    inner: crsio2::Policy,
}

pub struct PolicyBuilder {
    inner: crsio2::Policy,
}

pub struct BrokerServices {
    inner: crsio2::BrokerServices,
}

pub struct TargetServices {
    inner: crsio2::TargetServices,
}

pub enum Services {
    Broker(BrokerServices),
    Target(TargetServices),
}

pub fn init() -> io::Result<Services> {
    match try_crsio2!(crsio2::init()) {
        crsio2::Services::Broker(broker) => {
            Ok(Services::Broker(BrokerServices {
                inner: broker,
            }))
        },
        crsio2::Services::Target(target) => {
            Ok(Services::Target(TargetServices {
                inner: target,
            }))
        },
    }
}

impl Child {
    pub fn spawn(services: &mut ::BrokerServices, command: &mut Command) -> io::Result<Self> {
        let inner = try_crsio2!(services.inner.inner.spawn_target(
            &command.program,
            "", // FIXME
            &command.policy.0.inner.inner,
        ));

        Ok(Child {
            inner,
        })
    }

    pub fn id(&self) -> u32 {
        self.inner.get_process_id()
    }

    pub fn run(&mut self) -> io::Result<()> {
        try_crsio2!(self.inner.resume());
        Ok(())
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        unsafe {
            let handle = self.inner.get_process_handle();
            match WaitForSingleObject(handle, INFINITE) {
                self::WAIT_OBJECT_0 => {},
                _ => return Err(io::Error::last_os_error()),
            }
            let mut status = 0;
            winapi_bool_call!(GetExitCodeProcess(handle, &mut status))?;
            Ok(ExitStatus::from_raw(status))
        }
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        unsafe {
            let handle = self.inner.get_process_handle();
            match WaitForSingleObject(handle, 0) {
                self::WAIT_OBJECT_0 => {},
                self::WAIT_TIMEOUT => return Ok(None),
                _ => return Err(io::Error::last_os_error()),
            }
            let mut status = 0;
            winapi_bool_call!(GetExitCodeProcess(handle, &mut status))?;
            Ok(Some(ExitStatus::from_raw(status)))
        }
    }

    pub fn kill(&mut self) -> io::Result<()> {
        unsafe {
            let handle = self.inner.get_process_handle();
            winapi_bool_call!(TerminateProcess(handle, 1))?;
            Ok(())
        }
    }
}

impl PolicyBuilder {
    pub fn new(broker: &mut ::BrokerServices, preset: PolicyPreset) -> Self {
        let mut policy = broker.inner.inner.create_policy();
        match preset {
            PolicyPreset::ComputeOnly => {
                policy.set_token_level(TokenLevel::RestrictedSameAccess, TokenLevel::Lockdown).expect("failed to set token level");
                // FIXME
                PolicyBuilder { inner: policy }
            },
            PolicyPreset::Unrestricted => {
                unimplemented!()
            },
        }
    }

    pub fn build(self) -> io::Result<Policy> {
        Ok(Policy {
            inner: self.inner,
        })
    }
}

