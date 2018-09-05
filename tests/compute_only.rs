extern crate sandbox;
extern crate env_logger;

use std::{env, io};
use std::path::PathBuf;
use std::fs::File;

use sandbox::{Services, BrokerServices, TargetServices, Command, Policy};

fn main() {
    env_logger::init();
    match sandbox::init().unwrap() {
        Services::Broker(broker) => run_broker(broker),
        Services::Target(target) => run_target(target),
    }
}

fn run_broker(mut broker: BrokerServices) {
    let policy = Policy::compute_only(&mut broker).unwrap();
    let mut command = Command::new(env::current_exe().unwrap(), &policy);
    command
        .env("HOME", env::home_dir().unwrap())
        .env_inherit("RUST_LOG");
    let mut child = command.spawn(&mut broker).unwrap();
    child.run().unwrap();
    let exit_code = child.wait().unwrap();
    assert!(exit_code.success(), "subprocess returned {}", exit_code);
}

fn run_target(mut target: TargetServices) {
    target.lockdown();

    let home_dir = PathBuf::from(env::var_os("HOME").unwrap());

    // Check that we can't access files in the user's home directory
    match File::open(home_dir.join(".profile")) {
        Ok(_) => panic!("opening file in home directory succeeded"),
        Err(ref err) if err.kind() == io::ErrorKind::PermissionDenied => {},
        Err(err) => panic!("opening file in home directory failed with unexpected error: {}", err),
    }
}