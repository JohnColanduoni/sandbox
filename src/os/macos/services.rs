use super::policy::Policy;

use std::{io, process, panic};

use futures::prelude::*;
use tokio::reactor::{Reactor, Background as BackgroundReactor};
use tokio::current_thread::block_on_all;
use ipc::{MessageChannel, ChildRawMessageChannel};

pub struct BrokerServices {
    pub(in platform) event_loop: BackgroundReactor,
}

pub struct TargetServices {
    event_loop: BackgroundReactor,
    channel: Option<MessageChannel<TargetMessage, BrokerMessage>>,
}

impl BrokerServices {
    pub fn new() -> io::Result<Self> {
        let event_loop = Reactor::new()?.background()?;
        Ok(BrokerServices {
            event_loop
        })
    }
}

impl TargetServices {
    pub fn new(channel: ChildRawMessageChannel) -> io::Result<Self> {
        let event_loop = Reactor::new()?.background()?;
        let channel = MessageChannel::from_raw(channel.into_channel(event_loop.handle())?, MAX_MESSAGE_SIZE)?;
        Ok(TargetServices {
            event_loop,
            channel: Some(channel),
        })
    }

    pub fn lockdown(&mut self) -> io::Result<()> {
        if self.channel.is_none() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "sandboxed process has already been locked down"));
        }
        debug!("receiving policy from broker");
        let (msg, channel) = block_on_all(self.channel.take().unwrap().into_future().map_err(|(err, _)| err))?;
        debug!("policy has been received");
        if let Some(BrokerMessage::PolicySpec(policy)) = msg {
            policy.enact()?;
        } else {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid initial message from broker"));
        }
        Ok(())
    }
}

pub(in platform) const MAX_MESSAGE_SIZE: usize = 16384;

#[derive(Serialize, Deserialize)]
pub(in platform) enum BrokerMessage {
    PolicySpec(Policy), // FIXME: support policies longer than MAX_MESSAGE_SIZE
}

#[derive(Serialize, Deserialize)]
pub(in platform) enum TargetMessage {
}
