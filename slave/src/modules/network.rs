/// ----- NETWORK MODULE -----
/// This module is responsible for collecting requests from the request module,
/// states from the FSM module, and sending these to the master node. It also
/// parses messages from the master node and distributes this elevator's orders
/// to the requests node.

use std::thread::spawn;
use std::net;
use std::process;
use std::collections::HashMap;
use std::time::Duration;

use crossbeam_channel::{Sender, Receiver, unbounded, select, tick};
use network_rust::udpnet;

use shared_resources::config::{ElevatorConfig, NetworkConfig};
use shared_resources::request::Request;
use shared_resources::elevator_message::ElevatorMessage;

use crate::utilities::request_buffer::RequestBuffer;
use crate::utilities::elevator_behaviour::ElevatorBehaviour;
use crate::utilities::master_message::MasterMessage;

pub fn main(
    elevator_settings: ElevatorConfig,
    network_config: NetworkConfig,
    hall_button_rx: Receiver<Request>,
    master_hall_requests_tx: Sender<MasterMessage>,
    elevator_behaviour_rx: Receiver<ElevatorBehaviour>,
) {
    let update_master = tick(Duration::from_secs_f64(0.25));

    const TIMEOUT: u64 = 5;

    let (elevator_message_tx, elevator_message_rx) = unbounded::<ElevatorMessage>();
    spawn(move || {
        if udpnet::bcast::tx(network_config.update_port, elevator_message_rx).is_err() {
            panic!("Could not establish sending connection with master. Port {} already in use?", network_config.update_port);
        }
    });
    
    let (command_tx, command_rx) = unbounded::<HashMap<String, Vec<Vec<bool>>>>();
    spawn(move || {
        if udpnet::bcast::rx(network_config.command_port, command_tx).is_err() {
            panic!("Could not establish receiving connection with master. Port {} already in use?", network_config.command_port);
        }
    });
    
    let id = get_id();
    let num_floors = elevator_settings.num_floors;

    let mut hall_request_buffer = RequestBuffer::new(TIMEOUT);
    let mut elevator_behaviour = ElevatorBehaviour::new(num_floors);
    
    loop {
        select! {
            recv(command_rx) -> msg => {
                // decode command message from master
                let message = msg.unwrap();
                let master_message = MasterMessage::parse(message, num_floors, id.clone());
                hall_request_buffer.remove_confirmed_requests(&master_message.all_hall_requests);
                master_hall_requests_tx.send(master_message).unwrap();
            },
            recv(hall_button_rx) -> hall_request => {
                // append new hall order to queue
                hall_request_buffer.insert_new_request(hall_request.unwrap());
            },
            recv(elevator_behaviour_rx) -> elevator_behaviour_msg => {
                elevator_behaviour = elevator_behaviour_msg.unwrap();
                hall_request_buffer.insert_served_requests(elevator_behaviour.pop_served_requests());
            } 
            recv(update_master) -> _ => {
                // remove timed out orders
                hall_request_buffer.remove_timed_out_orders();
                // send state and collected orders to master
                let message = generate_elevator_message(
                    id.clone(),
                    elevator_behaviour.clone(),
                    &hall_request_buffer
                );
                elevator_message_tx.send(message).unwrap();
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

pub fn generate_elevator_message(
    id: String, 
    elevator_behaviour: ElevatorBehaviour, 
    request_buffer: &RequestBuffer
) -> ElevatorMessage {
    ElevatorMessage {
        id: id,
        behaviour: elevator_behaviour.behaviour.as_string(),
        floor: elevator_behaviour.floor,
        direction: elevator_behaviour.direction.as_string().unwrap(),
        cab_requests: elevator_behaviour.requests.get_cab_requests(),
        new_hall_orders: request_buffer.get_new_requests(),
        served_hall_orders: request_buffer.get_served_requests(),
    }
}

