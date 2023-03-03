use std::thread::spawn;
use std::process;

use crossbeam_channel::{Receiver, unbounded};
use network_rust::udpnet;

use crate::config::{STATE_PORT, ORDER_PORT};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct HallOrder {
    pub id: String,
    pub floor: u8,
    pub call: u8
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ElevatorState {
    pub id: String,
    pub behaviour: String,
    pub floor: u8,
    pub direction: String,
    pub cab_requests: [bool; 4]
}

pub fn init() -> (
    Receiver<HallOrder>,
    Receiver<ElevatorState> 
) {
    let (hall_order_tx, hall_order_rx) = unbounded();
    let (elevator_state_tx, elevator_state_rx) = unbounded();

    spawn(move || {
        if udpnet::bcast::rx(ORDER_PORT, hall_order_tx).is_err() {
            process::exit(1);
        }
    });
    spawn(move || {
        if udpnet::bcast::rx(STATE_PORT, elevator_state_tx).is_err() {
            process::exit(1);
        }
    });
    (hall_order_rx, elevator_state_rx)
}
