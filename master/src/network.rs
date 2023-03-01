use std::thread::spawn;
// use std::net;
use std::process;
//use std::thread::*;
use crossbeam_channel::{Receiver, unbounded};
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


pub fn init() -> (
    Receiver<HallOrder>,
    Receiver<ElevatorState> 

){
    let (receive_hall_order_tx, receive_hall_order_rx) = unbounded::<HallOrder>();
    let (receive_elevator_state_tx, receive_elevator_state_rx) = unbounded::<ElevatorState>();
    println!("Hello world");
    spawn(move || {
        if udpnet::bcast::rx(PORT, receive_hall_order_tx).is_err() {
            process::exit(1);
        }
    });
    spawn(move || {
        if udpnet::bcast::rx(PORT, receive_elevator_state_tx).is_err() {
            process::exit(1);
        }
    });

    ( receive_hall_order_rx, receive_elevator_state_rx)
}