

#[macro_use] mod macros;
pub mod policy;
mod services;
mod command;

pub use self::policy::{Policy, PolicyBuilder};
pub use self::services::{BrokerServices, TargetServices};
pub use self::command::{Child};

use std::{io, env};

use json;
use ipc::{MessageChannel, ChildRawMessageChannel, RawMessageChannel};

pub enum Services {
    Broker(BrokerServices),
    Target(TargetServices),
}

// We just use a static UUID to prevent accidental collisions with the environment, not as a 
// security feature
// TODO: maybe use a static file descriptor number instead?
const CHANNEL_ENV_VAR: &str = "SANDBOX_CHANNEL_ac15e9d0-52bd-4c49-a152-db8d6b8ea202";

pub fn init() -> io::Result<Services> {
    if let Some(channel_str_os) = env::var_os(CHANNEL_ENV_VAR) {
        let channel: ChildRawMessageChannel = channel_str_os.to_str()
            .and_then(|x| json::from_str(x).ok())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid sandbox IPC channel passed in environment variable"))?;
        env::remove_var(CHANNEL_ENV_VAR);

        Ok(Services::Target(TargetServices::new(channel)?))
    } else {
        Ok(Services::Broker(BrokerServices::new()?))
    }
}