use ::{platform, BrokerServices};

use std::{io};
use std::sync::Arc;

#[derive(Clone)]
pub struct Policy(pub(crate) Arc<_Policy>);

pub(crate) struct _Policy {
    pub(crate) inner: platform::Policy,
}

pub struct PolicyBuilder {
    pub(crate) inner: platform::PolicyBuilder,
}

pub enum PolicyPreset {
    ComputeOnly,
    Unrestricted,
}

impl Policy {
    pub fn builder(broker: &mut BrokerServices, preset: PolicyPreset) -> PolicyBuilder {
        PolicyBuilder::new(broker, preset)
    }

    pub fn compute_only(broker: &mut BrokerServices) -> io::Result<Policy> {
        PolicyBuilder::new(broker, PolicyPreset::ComputeOnly).build()
    }
}

impl PolicyBuilder {
    pub fn new(broker: &mut BrokerServices, preset: PolicyPreset) -> Self {
        PolicyBuilder {
            inner: platform::PolicyBuilder::new(broker, preset)
        }
    }

    pub fn build(self) -> io::Result<Policy> {
        Ok(Policy(Arc::new(_Policy {
            inner: self.inner.build()?,
        })))
    }
}

