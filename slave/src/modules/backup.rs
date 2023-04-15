use std::process;
use std::thread;
use std::time::Duration;
use std::process::Command;

use crossbeam_channel::{select, unbounded};
use network_rust::udpnet::{self, bcast::BcError};

use crate::utilities::elevator_status::ElevatorStatus;

pub fn backup(num_floors: u8, backup_port: u16, ack_port: u16) -> ElevatorStatus {
    println!("SLAVE BACKUP ON PORT: {:#?}\n------------------------",backup_port);
    let mut backup_data: ElevatorStatus = ElevatorStatus::new(num_floors);

    let (backup_recv_tx, backup_recv_rx) = unbounded::<ElevatorStatus>();
    let (backup_ack_tx, backup_ack_rx) = unbounded::<ElevatorStatus>();
    thread::Builder::new().name("backup_recieve_from_elevator".to_string()).spawn(move || {
        match udpnet::bcast::rx(backup_port, backup_recv_tx) {
            Err(BcError::IOError(_e)) => process::exit(1),
            _ => (),
        }
    }).ok();

    thread::Builder::new().name("backup_ack_to_elevator".to_string()).spawn(move || {
        match udpnet::bcast::tx(ack_port, backup_ack_rx) {
            Err(BcError::IOError(_e)) => process::exit(1),
            _ => (),
        }

    }).ok();

    loop {
        select! {
            recv(backup_recv_rx) -> data => {
                backup_data = data.clone().unwrap();
                backup_ack_tx.send(backup_data.clone()).unwrap()
            },
            default(Duration::from_secs(2)) => {
                break;
            }
        }
    }
    backup_data
}

pub fn spawn_backup(
    program_path: String, 
    elevnum: u8,
    server_port: u16,
) {
    if cfg!(target_os = "linux") {
        Command::new("gnome-terminal")
            .arg("--")
            .arg("/bin/sh")
            .arg("-c")
            .arg("cd ".to_owned()
                + &program_path
                + " && "
                + "cargo run"
                + " --"
                + " --elevnum "
                + &elevnum.to_string()
                + " --serverport "
                + &server_port.to_string())
            .output()
            .expect("failed to start backup");
    } else if cfg!(target_os = "macos") {
        Command::new("osascript")
            .arg("-e")
            .arg("tell app \"Terminal\" to do script \"cd ".to_owned() 
                + &program_path
                + " && "
                + "cargo run"
                + " --"
                + " --elevnum "
                + &elevnum.to_string()
                + " --serverport "
                + &server_port.to_string() + "\"")
            .output()
            .expect("failed to start backup");
    }
}
