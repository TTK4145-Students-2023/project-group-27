/// ----- REQUESTS MODULE -----
/// This module is responsible for keeping track of this elevators orders,
/// calculating when the elevator should stop at a floor and what direction
/// to choose next, and the sending this to the FSM module.

use std::time::Duration;
use std::vec;

use crossbeam_channel::{Sender, Receiver, select};
use driver_rust::elevio::{poll, elev};

use crate::config::ElevatorSettings;

pub fn main(
    elevator_settings: ElevatorSettings,
    cab_button_rx: Receiver<poll::CallButton>, 
    our_hall_requests_rx: Receiver<Vec<[bool; 2]>>, 
    all_hall_requests_rx: Receiver<Vec<[bool; 2]>>, 
    cleared_request_tx: Sender<poll::CallButton>, 
    button_light_tx: Sender<(u8,u8,bool)>,
    should_stop_tx: Sender<bool>,
    next_direction_tx: Sender<u8>,
    cab_requests_tx: Sender<Vec<bool>>,
    elevator_data_rx: Receiver<(u8,u8,bool)>,
    orders_tx: Sender<Vec<Vec<bool>>>,
) {
    // CLEAR ALL LIGHTS
    for floor in 0..elevator_settings.num_floors {
        for btn in elev::HALL_UP..=elev::CAB {
            button_light_tx.send((floor, btn, false)).unwrap();
        }
    }

    const UPDATE_FREQ: f64 = 0.1;
    let mut orders = vec![vec![false; elevator_settings.num_buttons as usize]; elevator_settings.num_floors as usize];

    loop {
        select! {
            recv(cab_button_rx) -> msg => { 
                // received cab order -> add to orders
                let destination = msg.as_ref().unwrap().floor;
                orders[destination as usize][elev::CAB as usize] = true;
                button_light_tx.send((destination, elev::CAB, true)).unwrap();
                let mut cab_requests = vec![false; elevator_settings.num_floors.clone() as usize];
                for floor in 0..elevator_settings.num_floors {
                    cab_requests[floor as usize] = orders[floor as usize][elev::CAB as usize];
                }
                // send cab orders to network module
                cab_requests_tx.send(cab_requests).unwrap();
                //TODO: Clear cab order which is assigned on same floor as current_floor 
            },
            recv(our_hall_requests_rx) -> msg => {
                // collect this elevator's hall requests from network module and store locally
                for floor in 0..elevator_settings.num_floors {
                    for btn in elev::HALL_UP..=elev::HALL_DOWN {
                        orders[floor as usize][btn as usize] = msg.clone().unwrap()[floor as usize][btn as usize];
                    }
                }
            },
            recv(all_hall_requests_rx) -> msg => {
                // collect all hall requests from network module and set button lights
                for floor in 0..elevator_settings.num_floors {
                    for btn in elev::HALL_UP..=elev::HALL_DOWN {
                        button_light_tx.send((floor, btn, msg.clone().unwrap()[floor as usize][btn as usize])).unwrap();
                    }
                }
            },
            recv(elevator_data_rx) -> data => {
                // collect elevator data from FSM
                let floor = data.unwrap().0;
                let direction = data.unwrap().1;
                let is_stopped = data.unwrap().2;
                if should_stop(elevator_settings.clone(), orders.clone(), floor, direction) && !is_stopped {
                    // send stop message to FSM if the elevator should stop at current floor
                    should_stop_tx.send(true).unwrap();
                    orders[floor as usize][elev::CAB as usize] = false;
                    button_light_tx.send((floor, elev::CAB, false)).unwrap();
                    // send message to network module that order has been served
                    cleared_request_tx.send(poll::CallButton {
                        floor: floor,
                        call: if direction == elev::DIRN_UP { elev::HALL_UP } else { elev::HALL_DOWN },
                    }).unwrap();
                    // if no further orders in direction -> the order in opposite direction is also served
                    if !further_requests_in_direction(elevator_settings.clone(), orders.clone(), floor, direction) {
                        cleared_request_tx.send(poll::CallButton {
                            floor: floor,
                            call: if direction == elev::DIRN_UP { elev::HALL_DOWN } else { elev::HALL_UP },
                        }).unwrap();
                    }
                }
                let next_direction = next_direction(elevator_settings.clone(), orders.clone(), floor, direction);
                next_direction_tx.send(next_direction).unwrap();
                //TODO: Clear cab order which is assigned on same floor as current_floor
            },
            default(Duration::from_secs_f64(UPDATE_FREQ)) => {
                orders_tx.send(orders.clone()).unwrap();
            },
        }
    }
}

fn should_stop(
    elevator_settings: ElevatorSettings,
    orders: Vec<Vec<bool>>,
    floor: u8,
    dirn: u8
) -> bool {
    if cab_request_at_floor(orders.clone(), floor)
    || requests_in_direction_at_this_floor(orders.clone(), floor, dirn)
    || !further_requests_in_direction(elevator_settings, orders.clone(), floor, dirn) {
        return true
    }
    false
}

fn cab_request_at_floor(
    orders: Vec<Vec<bool>>,
    floor: u8
) -> bool {
    orders[floor as usize][elev::CAB as usize]
}

fn requests_in_direction_at_this_floor(
    orders: Vec<Vec<bool>>,
    floor: u8,
    dirn: u8,
) -> bool {
    let hall_button = if dirn == elev::DIRN_UP { elev::HALL_UP } else { elev::HALL_DOWN };
    orders[floor as usize][hall_button as usize]
}

fn further_requests_in_direction(
    elevator_settings: ElevatorSettings,
    orders: Vec<Vec<bool>>,
    floor: u8,
    dirn: u8,
) -> bool {
    let range = if dirn == elev::DIRN_UP { (floor+1)..elevator_settings.num_floors } else { 0..floor };
    for f in range {
        for btn in elev::HALL_UP..=elev::CAB {
            if orders[f as usize][btn as usize] {
                return true
            }
        }
    }
    false
}

fn next_direction(
    elevator_settings: ElevatorSettings,
    orders: Vec<Vec<bool>>,
    floor: u8,
    last_direction: u8
) -> u8 {
    let other_direction = if last_direction == elev::DIRN_UP { elev::DIRN_DOWN } else { elev::DIRN_UP };
    if further_requests_in_direction(elevator_settings.clone(), orders.clone(), floor, last_direction) {
        return last_direction
    } else if further_requests_in_direction(elevator_settings.clone(), orders.clone(), floor, other_direction) {
        return other_direction
    }
    elev::DIRN_STOP
}
