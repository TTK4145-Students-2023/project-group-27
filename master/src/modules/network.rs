/// ----- NETWORK MODULE -----
/// This module is responsible for collecting states and hall requests from the 
/// slave nodes and uses the hall_request_assigner algorithm to distribute orders
/// among the connected elevators and UDP broadcasts the result. 

use std::collections::HashMap;
use std::time::{Instant, Duration};
use std::thread;
use std::process;

use crossbeam_channel::{unbounded, select, Sender, tick};
use network_rust::udpnet;

use shared_resources::config::MasterConfig;
use shared_resources::call::Call;
use shared_resources::elevator_message::ElevatorMessage;
use shared_resources::request_buffer::RequestBuffer;
use shared_resources::request::Request;

use crate::utilities::hall_request_assigner::*;

pub fn main(
    config: MasterConfig,
    hall_requests_tx: Sender<Vec<Vec<bool>>>,
    connected_elevators_tx: Sender<HashMap<String, ElevatorData>>,
) {
    let (command_tx, command_rx) = unbounded::<HashMap<String, Vec<Vec<bool>>>>();
    for port in config.network.command_ports {
        let command_rx = command_rx.clone();
        thread::spawn(move || {
            if udpnet::bcast::tx(port, command_rx, false).is_err() {
                panic!("Could not establish sending connection with slave. Port {} already in use?", port);
            }
        });
    }

    let (elevator_message_tx, elevator_message_rx) = unbounded::<ElevatorMessage>();
    for port in config.network.update_ports {
        let elevator_message_tx = elevator_message_tx.clone();
        thread::spawn(move || {
            if udpnet::bcast::rx(port, elevator_message_tx).is_err() {
                panic!("Could not establish receiving connection from slave. Port {} already in use?", port);
            }
        });
    }

    let (backup_changed_requests_tx, backup_changed_requests_rx) = unbounded::<(Vec<Request>,Vec<Request>)>();
    thread::Builder::new().name("master_to_backup".to_string()).spawn(move || {
        if udpnet::bcast::tx(config.network.backup_update_port, backup_changed_requests_rx, false).is_err() {
            process::exit(1);
        }
    }).unwrap();
 
    let (backup_confirmed_orders_tx, backup_confirmed_orders_rx) = unbounded::<Vec<Vec<bool>>>();
    thread::Builder::new().name("backup_to_master".to_string()).spawn(move || {
        if udpnet::bcast::rx(config.network.backup_ack_port, backup_confirmed_orders_tx).is_err() {
            process::exit(1);
        }
    }).unwrap();
 
    let (process_pair_tx, process_pair_rx) = unbounded::<bool>();
    thread::Builder::new().name("master_to_process_pair".to_string()).spawn(move || {
        if udpnet::bcast::tx(config.network.pp_port, process_pair_rx, true).is_err() {
            process::exit(1);
        }
    }).unwrap();

    const SLAVE_TIMEOUT: f64 = 4.0;

    let hra_exec_path = config.hall_request_assigner.exec_path;
    let update_freq = Duration::from_secs_f64(0.1);
    let timer = tick(update_freq);

    let mut connected_elevators: HashMap<String, ElevatorData> = HashMap::new();
    let mut hall_requests = vec![vec![false; Call::num_hall_calls() as usize]; config.elevator.num_floors as usize];
    let mut output: HashMap<String, Vec<Vec<bool>>> = HashMap::new();

    const BUFFER_TIMEOUT: u64 = 2;
    let mut new_hall_request_buffer = RequestBuffer::new(BUFFER_TIMEOUT);
    let mut served_hall_request_buffer = RequestBuffer::new(BUFFER_TIMEOUT);

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
                connected_elevators.entry(id.clone()).or_insert(ElevatorData{
                    state: HRAElevState { 
                        behaviour: behaviour.clone(), 
                        floor: floor, 
                        direction: direction.clone(), 
                        cab_requests: cab_requests.clone(),
                    },
                    last_seen: Instant::now(),
                    last_available: Instant::now(),
                });
                connected_elevators.insert(id.clone(), ElevatorData{
                    state: HRAElevState { 
                        behaviour: behaviour.clone(), 
                        floor: floor, 
                        direction: direction.clone(), 
                        cab_requests: cab_requests,
                    },
                    last_seen: Instant::now(),
                    last_available: 
                        if behaviour != connected_elevators[&id].state.behaviour // changed state
                            || behaviour == "idle" // is idle
                            || floor != connected_elevators[&id].state.floor // moved to another floor
                            || direction != connected_elevators[&id].state.direction
                        { Instant::now() } else { connected_elevators[&id].last_available },
                });
                
                // remove served hall orders
                if behaviour == "doorOpen" {
                    let call = if direction == "up" { Call::HallUp } else { Call::HallDown };
                    served_hall_request_buffer.insert_new_request(Request { floor: floor, call: call });
                }
                
                // collect new hall orders
                for request in msg.clone().unwrap().new_hall_orders {
                    new_hall_request_buffer.insert_new_request(request);
                }

                // assign hall orders only to available elevators
                let mut states = HashMap::new();
                for (id, data) in connected_elevators.clone() {
                    if data.last_available.elapsed() < Duration::from_secs_f64(SLAVE_TIMEOUT) {
                        states.insert(id, data.state);
                    }
                }
                output = match assign_orders(&hra_exec_path, hall_requests.clone(), states) {
                    Ok(result) => result,
                    Err(_) => continue,
                };

                // broadcast assigned orders
                hall_requests_tx.send(hall_requests.clone()).unwrap();

                // send new and served orders to backup
                backup_changed_requests_tx.send((
                    new_hall_request_buffer.get_new_requests(),
                    served_hall_request_buffer.get_new_requests()
                )).unwrap();
            },
            // receive from backup
            recv(backup_confirmed_orders_rx) -> msg => {
                hall_requests = msg.unwrap();
                new_hall_request_buffer.remove_confirmed_requests(&hall_requests);
                let mut inverted_hall_requests = hall_requests.clone();
                for floor in 0..config.elevator.num_floors {
                    for call in 0..Call::num_hall_calls() {
                        inverted_hall_requests[floor as usize][call as usize] = !inverted_hall_requests[floor as usize][call as usize];
                    }
                }
                served_hall_request_buffer.remove_confirmed_requests(&inverted_hall_requests);
            },
            recv(timer) -> _ => {
                // remove lost elevators
                for id in connected_elevators.clone().keys() {
                    if connected_elevators[id].last_seen.elapsed() > Duration::from_secs_f64(SLAVE_TIMEOUT) {
                        connected_elevators.remove(id);
                    }
                }
                connected_elevators_tx.send(connected_elevators.clone()).unwrap();
            }
        }
        command_tx.send(output.clone()).unwrap();
        process_pair_tx.send(true).unwrap(); // ping process pair
    }
}
