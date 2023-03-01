use std::thread::spawn;
use std::time::Duration;

use crossbeam_channel::{Sender, Receiver, unbounded, select};
use driver_rust::elevio::{poll, elev};
// ---
//use std::process;
//use std::thread::*;
//use std::process::Command;
//use std::env;
//use network_rust::udpnet;
// ---

use crate::config::{ELEV_NUM_FLOORS, ELEV_NUM_BUTTONS};
use crate::network::{HallOrder, hall_order};

pub fn init(
    elevator: elev::Elevator,
    call_button_rx: Receiver<poll::CallButton>,
    floor_sensor_rx: Receiver<u8>,
    send_to_master_tx: Sender<HallOrder>
) -> (
    Receiver<bool>, 
    Receiver<u8>
) {
    
    // CLEAR ALL LIGHTS
    for floor in 0..ELEV_NUM_FLOORS {
        for btn in elev::HALL_UP..=elev::CAB {
            elevator.call_button_light(floor, btn, false);
        }
    }

    let (requests_should_stop_tx, requests_should_stop_rx) = unbounded();
    let (requests_next_direction_tx, requests_next_direction_rx) = unbounded();
    // ---
    
    // ---
    spawn(move || main(
        elevator,
        call_button_rx, 
        floor_sensor_rx,
        requests_should_stop_tx,
        requests_next_direction_tx,
        // ---
        send_to_master_tx
        // ---
    ));
    // --- Thread for sending Hall orders through UDP Broadcast
    
    // ---
    (requests_should_stop_rx, requests_next_direction_rx)
}

fn main(
    elevator: elev::Elevator,
    call_button_rx: Receiver<poll::CallButton>, 
    floor_sensor_rx: Receiver<u8>,
    requests_should_stop_tx: Sender<bool>,
    requests_next_direction_tx: Sender<u8>,
    // ---
    send_to_master_tx: Sender<HallOrder>
    // ---
    
) {
    let send_new_direction_freq: Duration = Duration::from_secs_f64(0.5);

    let mut orders = [[false; ELEV_NUM_BUTTONS as usize]; ELEV_NUM_FLOORS as usize];
    let mut last_floor: u8 = 0;
    let mut last_direction: u8 = elev::DIRN_DOWN;

    loop {
        select! {
            recv(call_button_rx) -> msg => {
                // WHEN WE RECIEVE A NEW ORDER -> ADD TO MATRIX
                let destination = msg.as_ref().unwrap().floor;
                let button = msg.unwrap().call;
                // THIS PART CHECKS IF ORDER IS FROM CAB, ELEVATOR TREATS OWN CAB ORDERS
                if button == elev::CAB {
                    orders[destination as usize][button as usize] = true;
                    elevator.call_button_light(destination, button, true);
                    println!("Recieved order | floor: {}, dirn: {}", destination, button);
                    if destination == last_floor && !elevator.floor_sensor().is_none() {
                        requests_should_stop_tx.send(true).unwrap();
                    }  
                }
                else {
                    send_to_master_tx.send(hall_order(destination, button)).unwrap();
                    println!("Sending order to master | floor: {}, button: {}", destination, button);
                }
                // ---
                
            },
            recv(floor_sensor_rx) -> floor => {
                // WHEN WE PASS A FLOOR -> CHECK IF WE SHOULD STOP
                // IF WE STOP: SEND MESSAGE AND CLEAR ORDER
                last_floor = floor.unwrap();
                elevator.floor_indicator(last_floor);
                if should_stop(orders, floor.unwrap(), last_direction) {
                    requests_should_stop_tx.send(true).unwrap();
                    clear_order(elevator.clone(), &mut orders, floor.unwrap(), last_direction);
                }
            },
            default(send_new_direction_freq) => {
                // SPAM NEW DIRECTION, FSM WILL IGNORE IF OBSOLETE
                let next_direction = next_direction(orders, last_floor, last_direction);
                requests_next_direction_tx.send(next_direction).unwrap();
                if !elevator.floor_sensor().is_none() {
                    clear_order(elevator.clone(), &mut orders, last_floor, next_direction);
                }
                last_direction = if next_direction != elev::DIRN_STOP { next_direction } else { last_direction };
            }
        }
    }
}

fn clear_order(
    elevator: elev::Elevator,
    orders: &mut [[bool; ELEV_NUM_BUTTONS as usize]; ELEV_NUM_FLOORS as usize],
    floor: u8,
    dirn: u8
) {
    if floor == 0 || floor == ELEV_NUM_FLOORS - 1 {
        for btn in elev::HALL_UP..=elev::CAB {
            orders[floor as usize][btn as usize] = false;
            elevator.call_button_light(floor, btn, false);
        }
    } else {
        let button = if dirn == elev::DIRN_UP { elev::HALL_UP } else { elev::HALL_DOWN };
        orders[floor as usize][button as usize] = false;
        elevator.call_button_light(floor, button, false);
        orders[floor as usize][elev::CAB as usize] = false;
        elevator.call_button_light(floor, elev::CAB, false);
    }
}

fn should_stop(
    orders: [[bool; ELEV_NUM_BUTTONS as usize]; ELEV_NUM_FLOORS as usize],
    floor: u8,
    dirn: u8
) -> bool {
    if cab_request_at_floor(orders, floor)
    || requests_in_travel_direction(orders, floor, dirn)
    || !further_requests_in_direction(orders, floor, dirn) {
        return true
    }
    false
}

fn cab_request_at_floor(
    orders: [[bool; ELEV_NUM_BUTTONS as usize]; ELEV_NUM_FLOORS as usize],
    floor: u8
) -> bool {
    orders[floor as usize][elev::CAB as usize]
}

fn requests_in_travel_direction(
    orders: [[bool; ELEV_NUM_BUTTONS as usize]; ELEV_NUM_FLOORS as usize],
    floor: u8,
    dirn: u8,
) -> bool {
    let hall_button = if dirn == elev::DIRN_UP { elev::HALL_UP } else { elev::HALL_DOWN };
    orders[floor as usize][hall_button as usize]
}

fn further_requests_in_direction(
    orders: [[bool; ELEV_NUM_BUTTONS as usize]; ELEV_NUM_FLOORS as usize],
    floor: u8,
    dirn: u8,
) -> bool {
    let range = if dirn == elev::DIRN_UP { (floor+1)..ELEV_NUM_FLOORS } else { 0..floor };
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
    orders: [[bool; ELEV_NUM_BUTTONS as usize]; ELEV_NUM_FLOORS as usize],
    floor: u8,
    last_direction: u8
) -> u8 {
    let other_direction = if last_direction == elev::DIRN_UP { elev::DIRN_DOWN } else { elev::DIRN_UP };
    if further_requests_in_direction(orders, floor, last_direction) {
        return last_direction
    } else if further_requests_in_direction(orders, floor, other_direction) {
        return other_direction
    }
    elev::DIRN_STOP
}
