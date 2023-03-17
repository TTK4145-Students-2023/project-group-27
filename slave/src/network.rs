/// ----- NETWORK MODULE -----
/// This module is responsible for collecting requests from the request module,
/// states from the FSM module, and sending these to the master node. It also
/// parses messages from the master node and distributes this elevator's orders
/// to the requests node.

use std::thread::spawn;
use std::net;
use std::process;
use std::collections::HashMap;
use std::time::{Instant, Duration};

use crossbeam_channel::{Sender, Receiver, unbounded, select, tick};
use network_rust::udpnet;
use driver_rust::elevio::{elev, poll};

use crate::config::{ElevatorSettings, NetworkConfig};
use crate::prototype_fsm::ElevatorStatus;
use crate::prototype_fsm::HallRequests;

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

#[derive(Debug, Clone)]
pub struct HallOrderBuffer {
    new_orders: Vec<HallOrder>,
    served_orders: Vec<HallOrder>,
    new_order_timers: Vec<Instant>,
    served_order_timers: Vec<Instant>,
    timeout: u64,
}

impl HallOrderBuffer {
    pub fn new(timeout: u64) -> Self {
        HallOrderBuffer { 
            new_orders: Vec::new(), 
            served_orders: Vec::new(),
            new_order_timers: Vec::new(),
            served_order_timers: Vec::new(),
            timeout: timeout,
        }
    }
    pub fn get_new_orders(&self) -> Vec<HallOrder> {
        self.new_orders.clone()
    }
    pub fn get_served_orders(&self) -> Vec<HallOrder> {
        self.served_orders.clone()
    }
    pub fn insert_new_order(&mut self, order: HallOrder) {
        self.new_orders.push(order);
        self.new_order_timers.push(Instant::now());
    }
    pub fn insert_served_order(&mut self, order: HallOrder) {
        self.served_orders.push(order);
        self.served_order_timers.push(Instant::now());
    }
    pub fn remove_confirmed_orders(&mut self, all_hall_requests: &Vec<[bool; 2]>) {
        for index in (0..self.new_orders.len()).rev() {
            let floor = self.new_orders[index].floor;
            let call = self.new_orders[index].call;
            if all_hall_requests[floor as usize][call as usize] {
                self.new_orders.remove(index);
                self.new_order_timers.remove(index);
            }
        }
        for index in (0..self.served_orders.len()).rev() {
            let floor = self.served_orders[index].floor;
            let call = self.served_orders[index].call;
            if !all_hall_requests[floor as usize][call as usize] {
                self.served_orders.remove(index);
                self.served_order_timers.remove(index);
            }
        }
    }
    pub fn remove_timed_out_orders(&mut self) {
        for index in (0..self.new_orders.len()).rev() {
            if self.new_order_timers[index].elapsed() > Duration::from_secs(self.timeout) {
                self.new_orders.remove(index);
                self.new_order_timers.remove(index);
            }
        }
        for index in (0..self.served_orders.len()).rev() {
            if self.served_order_timers[index].elapsed() > Duration::from_secs(self.timeout) {
                self.served_orders.remove(index);
                self.served_order_timers.remove(index);
            }
        }
    }
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
    cleared_request_rx: Receiver<poll::CallButton>, 
    //elevator_state_rx: Receiver<(String,u8,u8)>,
    elevator_status_rx: Receiver<ElevatorStatus>,
    cab_requests_rx: Receiver<Vec<bool>>,
    // our_hall_requests_tx: Sender<Vec<[bool; 2]>>,
    // all_hall_requests_tx: Sender<Vec<[bool; 2]>>
    hall_requests_tx: Sender<HallRequests>
) {
    let update_master = tick(Duration::from_secs_f64(0.1));

    const TIMEOUT: u64 = 5;

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
    let mut hall_order_buffer = HallOrderBuffer::new(TIMEOUT);
    
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

                let empty_buffer = vec![[false; 2]; elevator_settings.num_floors as usize];

                let our_hall_requests = match command.get(&id) {
                    Some(hr) => hr,
                    None => &empty_buffer, // master does not yet know about this elevator -> discard message
                };

                let hall_requests = HallRequests {
                    our_requests: our_hall_requests.clone(),
                    all_requests: all_hall_requests.clone()
                };

                hall_order_buffer.remove_confirmed_orders(&all_hall_requests);

                hall_requests_tx.send(hall_requests.clone()).unwrap();

                // all_hall_requests_tx.send(all_hall_requests.clone()).unwrap();
                
                // // collect hall requests to be served from this elevator
                // let our_hall_requests = match command.get(&id) {
                //     Some(hr) => hr,
                //     None => continue, // master does not yet know about this elevator -> discard message
                // };
                
                // hall_order_buffer.remove_confirmed_orders(&all_hall_requests);

                // // pass hall requests to requests module
                // our_hall_requests_tx.send(our_hall_requests.clone()).unwrap();
            },
            recv(hall_button_rx) -> hall_request => {
                // append new hall order to queue
                hall_order_buffer.insert_new_order(HallOrder {
                    floor: hall_request.as_ref().unwrap().floor,
                    call: hall_request.unwrap().call,
                });
            },
            recv(cleared_request_rx) -> cleared_request => {
                // append served hall order to queue
                hall_order_buffer.insert_served_order(HallOrder {
                    floor: cleared_request.as_ref().unwrap().floor,
                    call: cleared_request.unwrap().call,
                });
            },
            recv(elevator_status_rx) -> status => {
                // collect elevator state info from FSM
                behaviour = status.clone().unwrap().behavior;
                floor = status.clone().unwrap().floor;
                direction = match status.clone().unwrap().direction {
                    elev::DIRN_UP => "up".to_string(),
                    elev::DIRN_DOWN => "down".to_string(),
                    _ => panic!("illegal direction, found {}", status.unwrap().direction),
                };
            },
            recv(cab_requests_rx) -> msg => {
                // collect this elevator's cab orders
                cab_requests = msg.unwrap();
            },
            recv(update_master) -> _ => {
                // remove timed out orders
                hall_order_buffer.remove_timed_out_orders();
                // send state and collected orders to master
                elevator_message_tx.send(ElevatorMessage { 
                    id: id.clone(), 
                    behaviour: behaviour.clone(), 
                    floor: floor, 
                    direction: direction.clone(), 
                    cab_requests: cab_requests.clone(), 
                    new_hall_orders: hall_order_buffer.get_new_orders(),
                    served_hall_orders: hall_order_buffer.get_served_orders(),
                }).unwrap();
            }
        }
    }
}
