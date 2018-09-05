extern crate sandbox;
extern crate env_logger;

mod cases;

use cases::TestCases;

use std::{env, io, process};
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
        .env("SANDBOX_TEST_HOME", env::home_dir().unwrap())
        .env_inherit("RUST_LOG");
    let mut child = command.spawn(&mut broker).unwrap();
    child.run().unwrap();
    let exit_code = child.wait().unwrap();
    assert!(exit_code.success(), "subprocess returned {}", exit_code);
}

fn run_target(mut target: TargetServices) {
    target.lockdown();

    let mut cases = TestCases {
        os_rng: true,
        .. TestCases::none()
    };
    if cfg!(target_os = "macos") {
        cases.open_nonexistent_file_home = true; // macOS sandbox doesn't blind open() to files that do not exist
    }
    if !cases.run() {
        process::exit(1);
    }
}