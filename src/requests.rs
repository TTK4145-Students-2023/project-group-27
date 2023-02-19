use crossbeam_channel::{Sender, Receiver, unbounded, select};
use std::thread::spawn;

use driver_rust::elevio::{poll, elev};

use crate::config::{ELEV_NUM_FLOORS, ELEV_NUM_BUTTONS};

pub fn init(
    elevator: elev::Elevator,
    call_button_rx: Receiver<poll::CallButton>,
    floor_sensor_rx: Receiver<u8>,
) -> (
    Receiver<bool>, 
    Receiver<u8>,
    Sender<bool>
) {
    let (requests_should_stop_tx, requests_should_stop_rx) = unbounded();
    let (requests_new_direction_tx, requests_new_direction_rx) = unbounded();
    let (requests_next_direction_tx, requests_next_direction_rx) = unbounded();
    spawn(move || main(
        elevator.clone(),
        call_button_rx.clone(), 
        floor_sensor_rx.clone(),
        requests_should_stop_tx.clone(),
        requests_next_direction_tx.clone(),
        requests_new_direction_rx.clone()
    ));
    (requests_should_stop_rx, requests_next_direction_rx, requests_new_direction_tx)
}

fn main(
    elevator: elev::Elevator,
    call_button_rx: Receiver<poll::CallButton>, 
    floor_sensor_rx: Receiver<u8>,
    requests_should_stop_tx: Sender<bool>,
    requests_next_direction_tx: Sender<u8>,
    requests_new_direction: Receiver<bool>
) {
    let mut orders = [[false; ELEV_NUM_BUTTONS as usize]; ELEV_NUM_FLOORS as usize];
    let mut last_floor: u8 = 0;
    let mut last_direction: u8 = elev::DIRN_DOWN;

    loop {
        select! {
            recv(call_button_rx) -> msg => {
                // WHEN WE RECIEVE A NEW ORDER -> ADD TO MATRIX
                let floor = msg.as_ref().unwrap().floor;
                let dirn = msg.unwrap().call;

                orders[floor as usize][dirn as usize] = true;
                elevator.call_button_light(floor, dirn, true);
                println!("Recieved order | floor: {}, dirn: {}", floor, dirn);
            },
            recv(floor_sensor_rx) -> floor => {
                // WHEN WE PASS A FLOOR -> CHECK IF WE SHOULD STOP
                // IF WE STOP: SEND MESSAGE AND CLEAR ORDER
                last_floor = floor.unwrap().clone();
                elevator.floor_indicator(last_floor);
                println!("reached floor: {}", floor.unwrap());
                if should_stop(orders.clone(), floor.unwrap().clone(), last_direction.clone()) {
                    requests_should_stop_tx.send(true).unwrap();
                    clear_order(elevator.clone(), &mut orders, floor.unwrap().clone(), last_direction.clone());
                }
            },
            recv(requests_new_direction) -> _ => {
                // WHEN THE DOORS CLOSE -> WHAT DIRECTION TO GO TO NEXT?
                let next_direction = next_direction(orders.clone(), last_floor.clone(), last_direction.clone());
                requests_next_direction_tx.send(next_direction.clone()).unwrap();
                clear_order(elevator.clone(), &mut orders, last_floor, next_direction);
                last_direction = if next_direction.clone() != elev::DIRN_STOP { next_direction.clone() } else { last_direction };
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
