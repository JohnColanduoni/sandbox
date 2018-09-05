use ::{PolicyPreset};

use std::{io, ptr};
use std::collections::HashMap;
use std::fmt::Write;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

#[derive(Clone, Serialize, Deserialize)]
pub struct Policy {
    profile: String,
    parameters: HashMap<CString, CString>,
}

pub struct PolicyBuilder {
    default_access: Access,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Access {
    Allow,
    Deny,
}

impl PolicyBuilder {
    pub fn new(broker: &mut ::BrokerServices, preset: PolicyPreset) -> Self {
        match preset {
            PolicyPreset::ComputeOnly => {
                PolicyBuilder {
                    default_access: Access::Deny,
                }
            },
            PolicyPreset::Unrestricted => {
                PolicyBuilder {
                    default_access: Access::Allow,
                }
            },
        }
    }

    pub fn build(self) -> io::Result<Policy> {
        let mut profile = String::new();
        let mut parameters = HashMap::new();
        writeln!(profile, "(version 1)").unwrap();
        match self.default_access {
            Access::Allow => writeln!(profile, "(allow default)").unwrap(),
            Access::Deny => writeln!(profile, "(deny default)").unwrap(),
        }
        if cfg!(debug_assertions) {
            writeln!(profile, r#"(debug deny)"#).unwrap();
        }
        Ok(Policy { 
            profile,
            parameters,
        })
    }
}

impl Policy {
    pub(crate) fn enact(&self) -> io::Result<()> {
        let profile_cstr = CString::new(self.profile.clone()).expect("invalid characters in profile");
        let mut params_list = Vec::new();
        for (k, v) in self.parameters.iter() {
            params_list.push(k.as_ptr());
            params_list.push(v.as_ptr());
        }
        params_list.push(ptr::null());

        let mut errorbuf = ptr::null_mut();
        debug!("activating macOS sandbox profile {:?}", profile_cstr);
        if unsafe { sandbox_init_with_parameters(profile_cstr.as_ptr(), 0, params_list.as_ptr(), &mut errorbuf) != 0 } {
            if !errorbuf.is_null() {
                let errorbuf_cstr = unsafe { CStr::from_ptr(errorbuf) };
                error!("sandbox_init_with_parameters() failed: {:?}", errorbuf_cstr);
                let error = io::Error::new(io::ErrorKind::InvalidInput, format!("sandbox_init_with_parameters() failed: {:?}", errorbuf_cstr));
                unsafe { sandbox_free_error(errorbuf); }
                Err(error)
            } else {
                let message = format_args!("sandbox_init_with_parameters() failed for an unknown reason");
                error!("{}", message);
                Err(io::Error::new(io::ErrorKind::InvalidInput, format!("{}", message)))
            }
        } else {
            Ok(())
        }
    }
}

pub trait PolicyBuilderExt {
    fn set_default_access(&mut self, access: Access) -> &mut Self;
}

impl PolicyBuilderExt for ::PolicyBuilder {
    fn set_default_access(&mut self, access: Access) -> &mut Self {
        self.inner.default_access = access;
        self
    }
}

extern "C" {
    fn sandbox_init_with_parameters(profile: *const c_char, flags: u64, parameters: *const *const c_char, errorbuf: *mut *mut c_char) -> c_int;
    fn sandbox_free_error(errorbuf: *mut c_char);
}