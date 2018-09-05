extern crate rand;

use std::{io};

use self::rand::OsRng;

pub fn os_rng() -> bool {
    match OsRng::new() {
        Ok(_) => true,
        Err(mut err) => {
            if let Some(err) = err.take_cause() {
                if let Some(err) = err.downcast_ref::<io::Error>() {
                    if err.kind() == io::ErrorKind::PermissionDenied {
                        false
                    } else {
                        panic!("unexpected error {}", err)
                    }
                } else {
                    panic!("unexpected error {}", err)
                }
            } else {
                panic!("unexpected error {}", err)
            }
        }
    }
}