extern crate sandbox;

use std::{env};

use sandbox::{Services, BrokerServices, TargetServices, Command, Policy};

fn main() {
    match sandbox::init().unwrap() {
        Services::Broker(broker) => run_broker(broker),
        Services::Target(target) => run_target(target),
    }
}

fn run_broker(mut broker: BrokerServices) {
    let policy = Policy::compute_only(&mut broker).unwrap();
    let mut command = Command::new(env::current_exe().unwrap(), &policy);
    let mut child = command.spawn(&mut broker).unwrap();
    child.run().unwrap();
    let exit_code = child.wait().unwrap();
    assert!(exit_code.success(), "subprocess returned {}", exit_code);
}

fn run_target(target: TargetServices) {
    
}