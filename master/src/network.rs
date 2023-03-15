/// ----- NETWORK MODULE -----
/// This module is responsible for collecting states and hall requests from the 
/// slave nodes and uses the hall_request_assigner algorithm to distribute orders
/// among the connected elevators and UDP broadcasts the result. 

use std::thread::spawn;
use std::process;
use std::collections::HashMap;
use std::time::{Instant, Duration};

use crossbeam_channel::{unbounded, select, Sender};
use network_rust::udpnet;

use crate::config::{self, UPDATE_PORTS, COMMAND_PORTS};
use crate::hall_request_assigner::*;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct HallOrder {
    pub floor: u8,
    pub call: u8
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ElevatorMessage {
    pub id: String,
    pub behaviour: String,
    pub floor: u8,
    pub direction: String,
    pub cab_requests: [bool; config::ELEV_NUM_FLOORS as usize],
    pub new_hall_orders: Vec<HallOrder>,
    pub served_hall_orders: Vec<HallOrder>,
}

#[derive(Clone)]
pub struct ElevatorData {
    pub state: HRAElevState,
    pub last_seen: Instant
}

pub fn main(
    hall_requests_tx: Sender<[[bool; 2]; config::ELEV_NUM_FLOORS as usize]>,
    connected_elevators_tx: Sender<HashMap<String, ElevatorData>>,
) {
    let (command_tx, command_rx) = unbounded::<HashMap<String, [[bool; 2]; config::ELEV_NUM_FLOORS as usize]>>();
    for port in COMMAND_PORTS {
        let command_rx = command_rx.clone();
        spawn(move || {
            if udpnet::bcast::tx(port, command_rx).is_err() {
                process::exit(1);
            }
        });
    }

    let (elevator_message_tx, elevator_message_rx) = unbounded::<ElevatorMessage>();
    for port in UPDATE_PORTS {
        let elevator_message_tx = elevator_message_tx.clone();
        spawn(move || {
            if udpnet::bcast::rx(port, elevator_message_tx).is_err() {
                process::exit(1);
            }
        });
    }

    const UPDATE_FREQ: f64 = 0.1;
    const TIMEOUT: f64 = 5.0;

    let mut connected_elevators: HashMap<String, ElevatorData> = HashMap::new();
    let mut hall_requests = [[false; 2]; config::ELEV_NUM_FLOORS as usize];
    
    loop {
        select! {
            recv(elevator_message_rx) -> msg => {
                // decode message
                let id = msg.clone().unwrap().id;
                let behaviour = msg.clone().unwrap().behaviour;
                let floor = msg.clone().unwrap().floor;
                let direction = msg.clone().unwrap().direction;
                let cab_requests = msg.clone().unwrap().cab_requests;

                // update elevator information data structure
                connected_elevators.insert(id, ElevatorData{
                    state: HRAElevState { 
                        behaviour: behaviour, 
                        floor: floor, 
                        direction: direction, 
                        cab_requests: cab_requests
                    },
                    last_seen: Instant::now()
                });

                // collect new hall orders
                for order in msg.clone().unwrap().new_hall_orders {
                    hall_requests[order.floor as usize][order.call as usize] = true;
                }

                // remove served hall orders
                for order in msg.clone().unwrap().served_hall_orders {
                    hall_requests[order.floor as usize][order.call as usize] = false;
                }

                // assign hall orders
                let mut states = HashMap::new();
                for (id, data) in connected_elevators.clone() {
                    states.insert(id, data.state);
                }
                let output = match assign_orders(hall_requests, states) {
                    Ok(result) => result,
                    Err(_) => continue, // give up and try again next time :)
                };

                // broadcast assigned orders
                command_tx.send(output).unwrap();

                hall_requests_tx.send(hall_requests).unwrap();
            },
            default(Duration::from_secs_f64(UPDATE_FREQ)) => {
                // remove lost elevators
                for id in connected_elevators.clone().keys() {
                    if Instant::now().duration_since(connected_elevators[id].last_seen) > Duration::from_secs_f64(TIMEOUT) {
                        connected_elevators.remove(id);
                    }
                }
                connected_elevators_tx.send(connected_elevators.clone()).unwrap();
            }
        }
    }
}
