use std::thread::spawn;
use std::net;
use std::process;
//use std::thread::*;
use crossbeam_channel::{Sender, unbounded};
//use driver_rust::elevio::{poll, elev};
use network_rust::udpnet;

use crate::config::{PORT};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct HallOrder {
    pub address: String,
    pub floor: u8,
    pub call: u8
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ElevatorState {
    pub address: String,
    pub behavior: String,
    pub floor: u8,
    pub direction: String,
    pub cab_requests: [bool; 4]
}

fn my_addr() -> String {
    let local_ip = net::TcpStream::connect("8.8.8.8:53")
        .unwrap()
        .local_addr()
        .unwrap()
        .ip();
    format!("{}",local_ip)
}

pub fn hall_order(destination: u8, button: u8) -> HallOrder {
    HallOrder {
        address: my_addr(),
        floor: destination,
        call: button 
    }
}

pub fn init() -> Sender<HallOrder> {
    let (send_to_master_tx, send_to_master_rx) = unbounded::<HallOrder>();
    spawn(move || {
        if udpnet::bcast::tx(PORT, send_to_master_rx).is_err() {
            process::exit(1);
        }
    });
    send_to_master_tx
}

