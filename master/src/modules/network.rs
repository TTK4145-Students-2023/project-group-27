/// ----- NETWORK MODULE -----
/// This module is responsible for collecting states and hall requests from the 
/// slave nodes and uses the hall_request_assigner algorithm to distribute orders
/// among the connected elevators and UDP broadcasts the result. 

use std::thread::spawn;
use std::collections::HashMap;
use std::time::{Instant, Duration};

use crossbeam_channel::{unbounded, select, Sender, tick};
use network_rust::udpnet;

use shared_resources::config::MasterConfig;
use shared_resources::call::Call;
use shared_resources::elevator_message::ElevatorMessage;

use crate::utilities::hall_request_assigner::*;

pub fn main(
    backup_data: Vec<Vec<bool>>,
    config: MasterConfig,
    hall_requests_tx: Sender<Vec<Vec<bool>>>,
    connected_elevators_tx: Sender<HashMap<String, ElevatorData>>,
    backup_send_tx: Sender<Vec<Vec<bool>>>
) {
    let (command_tx, command_rx) = unbounded::<HashMap<String, Vec<Vec<bool>>>>();
    for port in config.network.command_ports {
        let command_rx = command_rx.clone();
        spawn(move || {
            if udpnet::bcast::tx(port, command_rx, false).is_err() {
                panic!("Could not establish sending connection with slave. Port {} already in use?", port);
            }
        });
    }

    let (elevator_message_tx, elevator_message_rx) = unbounded::<ElevatorMessage>();
    for port in config.network.update_ports {
        let elevator_message_tx = elevator_message_tx.clone();
        spawn(move || {
            if udpnet::bcast::rx(port, elevator_message_tx).is_err() {
                panic!("Could not establish receiving connection from slave. Port {} already in use?", port);
            }
        });
    }
    
    let timeout: f64 = 2.5*config.elevator.num_floors as f64;

    let hra_exec_path = config.hall_request_assigner.exec_path;
    let update_freq = Duration::from_secs_f64(0.1);
    let timer = tick(update_freq);

    let mut connected_elevators: HashMap<String, ElevatorData> = HashMap::new();
    let mut hall_requests = backup_data;
    let mut output: HashMap<String, Vec<Vec<bool>>> = HashMap::new();

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

                if behaviour != "moving" || connected_elevators.contains_key(&id.clone()) {
                    connected_elevators.insert(id.clone(), ElevatorData{
                        state: HRAElevState { 
                            behaviour: behaviour.clone(), 
                            floor: floor, 
                            direction: direction.clone(), 
                            cab_requests: cab_requests
                        },
                        last_seen: if behaviour != "moving" {
                            Instant::now()
                        } else {
                            connected_elevators[&id].last_seen
                        }
                    });
                }
                
                // collect new hall orders
                for order in msg.clone().unwrap().new_hall_orders {
                    hall_requests[order.floor as usize][order.call as usize] = true;
                }

                // remove served hall orders
                if behaviour == "doorOpen" {
                    let call = if direction == "up" { Call::HallUp } else { Call::HallDown };
                    hall_requests[floor as usize][call as usize] = false;
                }

                // assign hall orders
                let mut states = HashMap::new();
                for (id, data) in connected_elevators.clone() {
                    states.insert(id, data.state);
                }
                output = match assign_orders(&hra_exec_path, hall_requests.clone(), states) {
                    Ok(result) => result,
                    Err(_) => continue, // give up and try again next time :)
                };

                // broadcast assigned orders

                hall_requests_tx.send(hall_requests.clone()).unwrap();
            },
            recv(timer) -> _ => {
                // remove lost elevators
                for id in connected_elevators.clone().keys() {
                    if connected_elevators[id].last_seen.elapsed() > Duration::from_secs_f64(timeout) {
                        connected_elevators.remove(id);
                    }
                }
                connected_elevators_tx.send(connected_elevators.clone()).unwrap();
            }
        }
        command_tx.send(output.clone()).unwrap();

        backup_send_tx.send(hall_requests.clone()).unwrap();
    }
}
