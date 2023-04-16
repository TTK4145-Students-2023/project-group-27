/// ----- NETWORK MODULE -----
/// This module is responsible for collecting hall requests from the io module,
/// states from the FSM module, and sending these to the master node. It also
/// parses messages from the master node and delivers this elevator's orders
/// as decided by master to the fsm module for execution.

use std::thread::spawn;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crossbeam_channel::{Sender, Receiver, unbounded, select};
use network_rust::udpnet;

use shared_resources::config::SlaveConfig;
use shared_resources::request::Request;
use shared_resources::elevator_message::ElevatorMessage;
use shared_resources::request_buffer::RequestBuffer;

use crate::utilities::elevator_status::ElevatorStatus;
use crate::utilities::master_message::MasterMessage;

pub fn main(
    config: SlaveConfig,
    hall_button_rx: Receiver<Request>,
    master_hall_requests_tx: Sender<MasterMessage>,
    elevator_status_rx: Receiver<ElevatorStatus>,
) {
    const TIMEOUT_BUFFERED_HALL_REQUESTS: u64 = 5;

    let (elevator_message_tx, elevator_message_rx) = unbounded::<ElevatorMessage>();
    {
        let elevator_message_rx = elevator_message_rx.clone();
        spawn(move || {
            if udpnet::bcast::tx(config.network.update_port, elevator_message_rx, false).is_err() {
                panic!("Could not establish sending connection with master. Port {} already in use?", config.network.update_port);
            }
        });
    }
    let (pp_update_tx, pp_update_rx) = unbounded::<ElevatorStatus>();
    spawn(move || {
        println!("{:#?}", config.network.pp_update_port);
        if udpnet::bcast::tx(config.network.pp_update_port, pp_update_rx, true).is_err() {
            panic!("Could not establish sending connection to process pair backup. Port {} already in use?", config.network.pp_update_port);
        }
    });
    
    let (command_tx, command_rx) = unbounded::<HashMap<String, Vec<Vec<bool>>>>();
    spawn(move || {
        if udpnet::bcast::rx(config.network.command_port, command_tx).is_err() {
            panic!("Could not establish receiving connection with master. Port {} already in use?", config.network.command_port);
        }
    });

    let num_floors = config.elevator.num_floors;

    let mut hall_request_buffer = RequestBuffer::new(TIMEOUT_BUFFERED_HALL_REQUESTS);
    let mut elevator_behaviour = ElevatorStatus::new(num_floors);
    
    const MASTER_TIMEOUT: u64 = 3;
    let mut last_seen_master = Instant::now();
    let mut master_connected = false;
    let mut last_master_message = MasterMessage { 
        our_hall_requests: Vec::new(), 
        all_hall_requests: Vec::new() 
    };

    loop {
        select! {
            recv(command_rx) -> msg => {
                // decode command message from master
                let message = msg.unwrap();
                let master_message = MasterMessage::parse(
                    message, 
                    num_floors, 
                    config.elevnum.to_string().clone()
                );
                hall_request_buffer.remove_confirmed_requests(&master_message.all_hall_requests);
                master_hall_requests_tx.send(master_message.clone()).unwrap();
                last_master_message = master_message;
                last_seen_master = Instant::now();
                master_connected = true;
            },
            recv(hall_button_rx) -> hall_request => {
                // append new hall order to queue
                hall_request_buffer.insert_new_request(hall_request.unwrap());
            },
            recv(elevator_status_rx) -> elevator_behaviour_msg => {
                elevator_behaviour = elevator_behaviour_msg.unwrap();
            } 
            default(Duration::from_secs_f64(0.1)) => (),
        }
        hall_request_buffer.remove_timed_out_orders();
        let message = generate_elevator_message(
            config.elevnum.to_string().clone(),
            elevator_behaviour.clone(),
            &hall_request_buffer
        );
        elevator_message_tx.send(message).unwrap();
        pp_update_tx.send(elevator_behaviour.clone()).unwrap();
        if master_connected && last_seen_master.elapsed() > Duration::from_secs(MASTER_TIMEOUT) {
            master_connected = false;
            master_hall_requests_tx.send(MasterMessage {
                our_hall_requests: last_master_message.all_hall_requests.clone(),
                all_hall_requests: last_master_message.all_hall_requests.clone(),
            }).unwrap();
        }
    }
}

pub fn generate_elevator_message(
    id: String, 
    elevator_behaviour: ElevatorStatus, 
    request_buffer: &RequestBuffer
) -> ElevatorMessage {
    ElevatorMessage {
        id: id,
        behaviour: elevator_behaviour.behaviour.as_string(),
        floor: elevator_behaviour.floor,
        direction: elevator_behaviour.direction.as_string().unwrap(),
        cab_requests: elevator_behaviour.requests.get_cab_requests(),
        new_hall_orders: request_buffer.get_new_requests(),
    }
}
