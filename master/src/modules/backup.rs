use std::thread;
use std::process;
use std::process::Command;
use std::time::Duration;

use crossbeam_channel::{select, unbounded};
use shared_resources::call::Call;

use network_rust::udpnet;
use network_rust::udpnet::bcast::BcError;

pub fn backup(num_floors: u8, backup_port: u16) -> Vec<Vec<bool>> {
    println!("MASTER BACKUP ON PORT: {:#?}\n---------------------", backup_port);
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

pub fn spawn_backup(
    program_path: String
) {
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
}