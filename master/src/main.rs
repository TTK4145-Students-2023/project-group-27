use std::thread;
use std::io::Result;
use std::process;
use std::process::Command;
use std::time::Duration;
use std::fs;
use std::path::PathBuf;

use crossbeam_channel::{select, unbounded};
use shared_resources::config::MasterConfig;
use shared_resources::call::Call;

use master::network;
use master::utilities::debug;
use network_rust::udpnet;
use network_rust::udpnet::bcast::BcError;


fn backup(num_floors: u8, backup_port: u16) -> Vec<Vec<bool>> {
    println!("BACKUP MODE for master: {:#?}\n-----------------",backup_port);
    let mut backup_data: Vec<Vec<bool>> = vec![vec![false; Call::num_hall_calls() as usize]; num_floors as usize];

    let (backup_recv_tx, backup_recv_rx) = unbounded::<Vec<Vec<bool>>>();
    thread::Builder::new().name("backup_recieve_from_master".to_string()).spawn(move || {
        match udpnet::bcast::rx(backup_port, backup_recv_tx) {
            Err(BcError::IOError(_e)) => process::exit(1),
            _ => (),
        }
    }).ok();

    // thread::Builder::new().name("backup_ack_to_elevator".to_string()).spawn(move || {
    //     match udpnet::bcast::tx(ack_port, backup_ack_rx) {
    //         Err(BcError::IOError(_e)) => process::exit(1),
    //         _ => (),
    //     }
    // }).ok();

    //let config = shared_resources::config::MasterConfig::get();
    //let num_floors = config.elevator.num_floors;
    //let mut backup_debug = Debug::new(num_floors);

    loop {
        select! {
            recv(backup_recv_rx) -> data => {
                //backup_debug.printstatus(&data.clone().unwrap()).unwrap();
                backup_data = data.clone().unwrap();
                //backup_ack_tx.send(backup_data.clone()).unwrap()
            },
            default(Duration::from_secs(2)) => {
                break;
            }
        }
    }
    backup_data
}

fn main() -> Result<()> {
    let config = MasterConfig::get();
    let num_floors = config.elevator.num_floors;
    let (hall_requests_tx, hall_requests_rx) = unbounded();
    let (connected_elevators_tx, connected_elevators_rx) = unbounded();

    let program_dir = PathBuf::from("./.");
    let program_path: String = fs::canonicalize(&program_dir).unwrap().into_os_string().into_string().unwrap();
    println!("{:#?}",program_path);
    let backup_port = config.network.backup_port;
    //let ack_port = config.network.ack_port;
    let handle = thread::spawn(move || backup(config.elevator.num_floors, backup_port));
    let backup_data = handle.join().unwrap();
    // BECOME PRIMARY, CREATE NEW BACKUP

    if cfg!(target_os = "linux") {
        Command::new("gnome-terminal")
            .arg("--")
            .arg("/bin/sh")
            .arg("-c")
            .arg("cd ".to_owned()
                + &program_path
                + " && "
                + "cargo run")
            .output()
            .expect("failed to start backup");
    } else if cfg!(target_os = "macos") {
        Command::new("osascript")
            .arg("-e")
            .arg("tell app \"Terminal\" to do script \"cd ".to_owned() 
                + &program_path
                + " && "
                + "cargo run")
            .output()
            .expect("failed to start backup");
    }

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
