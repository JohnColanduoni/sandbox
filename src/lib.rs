#[macro_use] extern crate log;
#[macro_use] extern crate cfg_if;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json as json;
extern crate futures;
extern crate tokio_reactor;
extern crate tokio_current_thread;
extern crate sandbox_ipc as ipc;

cfg_if! {
    if #[cfg(target_os = "windows")] {
        #[macro_use] extern crate winhandle;
        extern crate winapi;
        extern crate crsio2;
    } else if #[cfg(target_os = "macos")] {
        extern crate libc;
    }
}

mod tokio {
    pub(crate) use tokio_reactor as reactor;
    pub(crate) use tokio_current_thread as current_thread;
}

mod services;
mod policy;
mod command;

#[cfg_attr(target_os = "windows", path = "os/windows.rs")]
#[cfg_attr(target_os = "macos", path = "os/macos/mod.rs")]
mod platform;

pub use services::{Services, BrokerServices, TargetServices};
pub use command::Command;
pub use policy::{Policy, PolicyBuilder, PolicyPreset};

pub mod os {
    #[cfg(target_os = "macos")]
    pub mod macos {
        pub use platform::policy::{PolicyBuilderExt};
    }
}

use std::io;

pub fn init() -> io::Result<Services> {
    Ok(match platform::init()? {
        platform::Services::Broker(broker) => Services::Broker(BrokerServices::new(broker)),
        platform::Services::Target(target) => Services::Target(TargetServices::new(target)),
    })
}
