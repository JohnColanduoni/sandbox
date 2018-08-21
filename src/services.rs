use ::{platform};

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
}