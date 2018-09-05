use ::{platform};

use std::{panic, process};

pub struct BrokerServices {
    pub(crate) inner: platform::BrokerServices,
}

pub struct TargetServices {
    pub(crate) inner: platform::TargetServices,
}

pub enum Services {
    Broker(BrokerServices),
    Target(TargetServices),
}

impl BrokerServices {
    pub(crate) fn new(inner: platform::BrokerServices) -> Self {
        BrokerServices { inner }
    }
}

impl TargetServices {
    pub(crate) fn new(inner: platform::TargetServices) -> Self {
        TargetServices { inner }
    }

    pub fn lockdown(&mut self) {
        // If lockdown fails for any reason, force process to exit immediately
        let this = panic::AssertUnwindSafe(self);
        if let Err(err) = panic::catch_unwind(move || {
            if let Err(err) = this.0.inner.lockdown() {
                error!("error when trying to lockdown sandbox: {}", err);
                process::abort();
            }
        }) {
            error!("panic when trying to lockdown sandbox: {:?}", err);
            process::abort();
        }
    }
}