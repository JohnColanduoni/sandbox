#[macro_use] extern crate log;
#[macro_use] extern crate cfg_if;

cfg_if! {
    if #[cfg(target_os = "windows")] {
        #[macro_use] extern crate winhandle;
        extern crate winapi;
        extern crate crsio2;
    }
}

mod services;
mod policy;
mod command;

#[cfg_attr(target_os = "windows", path = "os/windows.rs")]
mod platform;

pub use services::{Services, BrokerServices, TargetServices};
pub use command::Command;
pub use policy::{Policy, PolicyPreset};

use std::io;

pub fn init() -> io::Result<Services> {
    Ok(match platform::init()? {
        platform::Services::Broker(broker) => Services::Broker(BrokerServices::new(broker)),
        platform::Services::Target(target) => Services::Target(TargetServices::new(target)),
    })
}
