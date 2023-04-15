use std::thread;
use std::io::Result;
use std::process;
use std::fs;
use std::path::PathBuf;

use crossbeam_channel::unbounded;
use shared_resources::config::MasterConfig;

mod network;
mod backup;

use crate::utilities::debug;
use network_rust::udpnet;

pub fn run() -> Result<()> {
    let config = MasterConfig::get();
    let num_floors = config.elevator.num_floors;
    let (hall_requests_tx, hall_requests_rx) = unbounded();
    let (connected_elevators_tx, connected_elevators_rx) = unbounded();

    let program_dir = PathBuf::from("./.");
    let program_path: String = fs::canonicalize(&program_dir).unwrap().into_os_string().into_string().unwrap();
    println!("{:#?}",program_path);
    let backup_port = config.network.backup_port;
    //let ack_port = config.network.ack_port;
    let handle = thread::spawn(move || backup::backup(config.elevator.num_floors, backup_port));
    let backup_data = handle.join().unwrap();
    // BECOME PRIMARY, CREATE NEW BACKUP

    backup::spawn_backup(program_path);

    let (backup_send_tx, backup_send_rx) = unbounded::<Vec<Vec<bool>>>();
    {
        thread::spawn(move || network::main(
            backup_data,
            config,
            hall_requests_tx,
            connected_elevators_tx,
            backup_send_tx
        ));
    }
    

    thread::spawn(move || debug::main(
        num_floors,
        hall_requests_rx,
        connected_elevators_rx
    ));

    {
        thread::Builder::new().name("master to backup".to_string()).spawn(move || {
            if udpnet::bcast::tx(backup_port, backup_send_rx).is_err() {
                // crash program if creating the socket fails (`bcast:tx` will always block if the
                // initialization succeeds)
                process::exit(1);
            }
        })?;
    }

    loop { }
}
