use std::thread::spawn;
use std::net;
use std::process;
use std::collections::HashMap;
use std::time::Duration;

use crossbeam_channel::{Sender, Receiver, unbounded, select};
use network_rust::udpnet;
use driver_rust::elevio::{elev, poll};

use crate::config::{ElevatorSettings, NetworkConfig};

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
    pub cab_requests: Vec<bool>,
    pub new_hall_orders: Vec<HallOrder>,
    pub served_hall_orders: Vec<HallOrder>,
}

fn get_id() -> String {
    let local_ip = net::TcpStream::connect("8.8.8.8:53")
        .unwrap()
        .local_addr()
        .unwrap()
        .ip();
    format!("{}#{}",local_ip, process::id())
}

pub fn main(
    elevator_settings: ElevatorSettings,
    network_config: NetworkConfig,
    hall_button_rx: Receiver<poll::CallButton>,
    our_hall_requests_tx: Sender<Vec<[bool; 2]>>,
    all_hall_requests_tx: Sender<Vec<[bool; 2]>>,
    cleared_request_rx: Receiver<poll::CallButton>, 
    elevator_state_rx: Receiver<(String,u8,u8)>,
    cab_requests_rx: Receiver<Vec<bool>>,
) {
    const SEND_UPDATE_FREQUENCY: f64 = 0.1;
    
    let (elevator_message_tx, elevator_message_rx) = unbounded::<ElevatorMessage>();
    spawn(move || {
        if udpnet::bcast::tx(network_config.update_port, elevator_message_rx).is_err() {
            process::exit(1);
        }
    });
    
    let (command_tx, command_rx) = unbounded::<HashMap<String, Vec<[bool; 2]>>>();
    spawn(move || {
        if udpnet::bcast::rx(network_config.command_port, command_tx).is_err() {
            process::exit(1);
        }
    });
    
    let id = get_id();
    
    // FSM states
    let mut behaviour = String::from("idle");
    let mut floor = 0;
    let mut direction = String::from("up");
    
    // request states
    let mut cab_requests = vec![false; elevator_settings.num_floors as usize];
    let mut new_hall_orders: Vec<HallOrder> = Vec::new();
    let mut served_hall_orders: Vec<HallOrder> = Vec::new();
    
    loop {
        select! {
            recv(command_rx) -> msg => {
                // decode command message from master
                let command = msg.unwrap(); 

                // collect all hall orders and send to request module
                let mut all_hall_requests = vec![[false; 2]; elevator_settings.num_floors as usize];
                for (_, requests) in command.clone() {
                    for floor in 0..elevator_settings.num_floors {
                        for btn in elev::HALL_UP..=elev::HALL_DOWN {
                            if requests[floor as usize][btn as usize] {
                                all_hall_requests[floor as usize][btn as usize] = true;
                            }
                        }
                    }
                }
                all_hall_requests_tx.send(all_hall_requests.clone()).unwrap();
                
                // collect hall requests to be served from this elevator
                let our_hall_requests = match command.get(&id) {
                    Some(hr) => hr,
                    None => continue, // master does not yet know about this elevator -> discard message
                };
                // remove unconfirmed new hall orders from queue
                for index in (0..new_hall_orders.len()).rev() {
                    let floor = new_hall_orders[index].floor;
                    let call = new_hall_orders[index].call;
                    if all_hall_requests[floor as usize][call as usize] {
                        new_hall_orders.remove(index);
                    }
                }
                // remove unconfirmed served hall orders from queue
                for index in (0..served_hall_orders.len()).rev() {
                    let floor = served_hall_orders[index].floor;
                    let call = served_hall_orders[index].call;
                    if !all_hall_requests[floor as usize][call as usize] {
                        served_hall_orders.remove(index);
                    }
                }
                // pass hall requests to requests module
                our_hall_requests_tx.send(our_hall_requests.clone()).unwrap();
            },
            recv(hall_button_rx) -> hall_request => {
                // append new hall order to queue
                new_hall_orders.push(HallOrder {
                    floor: hall_request.as_ref().unwrap().floor,
                    call: hall_request.unwrap().call,
                });
            },
            recv(cleared_request_rx) -> cleared_request => {
                // append served hall order to queue
                served_hall_orders.push(HallOrder {
                    floor: cleared_request.as_ref().unwrap().floor,
                    call: cleared_request.unwrap().call,
                });
            },
            recv(elevator_state_rx) -> state => {
                // collect elevator state info from FSM
                behaviour = state.clone().unwrap().0;
                floor = state.clone().unwrap().1;
                direction = match state.unwrap().2 {
                    elev::DIRN_UP => "up".to_string(),
                    elev::DIRN_DOWN => "down".to_string(),
                    _ => panic!("illegal direction"),
                };
            },
            recv(cab_requests_rx) -> msg => {
                // collect this elevator's cab orders
                cab_requests = msg.unwrap();
            },
            default(Duration::from_secs_f64(SEND_UPDATE_FREQUENCY)) => {
                // send state and collected orders to master
                elevator_message_tx.send(ElevatorMessage { 
                    id: id.clone(), 
                    behaviour: behaviour.clone(), 
                    floor: floor, 
                    direction: direction.clone(), 
                    cab_requests: cab_requests.clone(), 
                    new_hall_orders: new_hall_orders.clone(),
                    served_hall_orders: served_hall_orders.clone(),
                }).unwrap();
            }
        }
    }
}
