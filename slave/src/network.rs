use std::thread::spawn;
use std::net;
use std::process;
use std::collections::HashMap;
use std::time::Duration;

use crossbeam_channel::{Sender, Receiver, unbounded, select};
use network_rust::udpnet;
use driver_rust::elevio::{elev, poll};

use crate::config::{self, UPDATE_PORT, COMMAND_PORT};

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
    pub new_hall_orders: Vec<HallOrder>
}

fn get_id() -> String {
    let local_ip = net::TcpStream::connect("8.8.8.8:53")
        .unwrap()
        .local_addr()
        .unwrap()
        .ip();
    format!("{}",local_ip)
}

pub fn init(
    hall_button_rx: Receiver<poll::CallButton>,
    hall_request_tx: Sender<[[bool; 2]; config::ELEV_NUM_FLOORS as usize]>,
    elevator_state_rx: Receiver<(String,u8,u8)>,
    cab_requests_rx: Receiver<[bool; config::ELEV_NUM_FLOORS as usize]>,
) {
    spawn(move || main(
        hall_button_rx, 
        hall_request_tx, 
        elevator_state_rx,
        cab_requests_rx,
    ));
}

fn main(
    hall_button_rx: Receiver<poll::CallButton>,
    hall_requests_tx: Sender<[[bool; 2]; config::ELEV_NUM_FLOORS as usize]>,
    elevator_state_rx: Receiver<(String,u8,u8)>,
    cab_requests_rx: Receiver<[bool; config::ELEV_NUM_FLOORS as usize]>,
) {
    const SEND_UPDATE_FREQUENCY: f64 = 0.25;
    
    let (elevator_message_tx, elevator_message_rx) = unbounded::<ElevatorMessage>();
    spawn(move || {
        if udpnet::bcast::tx(UPDATE_PORT, elevator_message_rx).is_err() {
            process::exit(1);
        }
    });
    
    let (command_tx, command_rx) = unbounded::<HashMap<String, [[bool; 2]; config::ELEV_NUM_FLOORS as usize]>>();
    spawn(move || {
        if udpnet::bcast::rx(COMMAND_PORT, command_tx).is_err() {
            process::exit(1);
        }
    });
    
    let id = get_id();
    
    let mut behaviour = String::from("idle");
    let mut floor = 0;
    let mut direction = String::from("up");
    let mut cab_requests = [false; config::ELEV_NUM_FLOORS as usize];
    let mut new_hall_orders: Vec<HallOrder> = Vec::new();
    
    loop {
        select! {
            recv(command_rx) -> command => {
                let hall_requests = command.unwrap()[&id];
                for index in 0..new_hall_orders.len() {
                    let floor = new_hall_orders[index].floor;
                    let call = new_hall_orders[index].call;
                    if hall_requests[floor as usize][call as usize] {
                        new_hall_orders.remove(index);
                    }
                }
                hall_requests_tx.send(hall_requests).unwrap();
            },
            recv(hall_button_rx) -> hall_request => {
                new_hall_orders.push(HallOrder {
                    floor: hall_request.as_ref().unwrap().floor,
                    call: hall_request.unwrap().call,
                });
            },
            recv(elevator_state_rx) -> state => {
                behaviour = state.clone().unwrap().0;
                floor = state.clone().unwrap().1;
                direction = match state.unwrap().2 {
                    elev::DIRN_UP => "up".to_string(),
                    elev::DIRN_DOWN => "down".to_string(),
                    _ => panic!("illegal direction"),
                };
            },
            recv(cab_requests_rx) -> msg => {
                cab_requests = msg.unwrap();
            },
            default(Duration::from_secs_f64(SEND_UPDATE_FREQUENCY)) => {
                elevator_message_tx.send(ElevatorMessage { 
                    id: id.clone(), 
                    behaviour: behaviour.clone(), 
                    floor: floor, 
                    direction: direction.clone(), 
                    cab_requests: cab_requests, 
                    new_hall_orders: new_hall_orders.clone()
                }).unwrap();
            }
        }
    }
}
