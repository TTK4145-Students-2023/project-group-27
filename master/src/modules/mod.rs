use std::thread;
use std::io::Result;
use std::fs;
use std::path::PathBuf;

use crossbeam_channel::unbounded;
use shared_resources::config::MasterConfig;

mod network;
mod process_pair;

use crate::utilities::debug;

pub fn run() -> Result<()> {
    let config = MasterConfig::get();
    let num_floors = config.elevator.num_floors;
    let (hall_requests_tx, hall_requests_rx) = unbounded();
    let (connected_elevators_tx, connected_elevators_rx) = unbounded();

    let program_dir = PathBuf::from("./.");
    let program_path: String = fs::canonicalize(&program_dir).unwrap().into_os_string().into_string().unwrap();
    let process_pair_port = config.network.pp_port;
    let process_pair_handle = thread::spawn(move || process_pair::process_pair(process_pair_port));
    process_pair_handle.join().unwrap();

    process_pair::spawn_process_pair(program_path);

    thread::spawn(move || network::main(
        config,
        hall_requests_tx,
        connected_elevators_tx,
    ));
    
    thread::spawn(move || debug::main(
        num_floors,
        hall_requests_rx,
        connected_elevators_rx
    ));

    loop { }
}
