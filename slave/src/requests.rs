use std::thread::spawn;

use crossbeam_channel::{Sender, Receiver, unbounded, select};
use driver_rust::elevio::{poll, elev};

use crate::config::{ELEV_NUM_FLOORS, ELEV_NUM_BUTTONS};

pub fn init(
    cab_button_rx: Receiver<poll::CallButton>,
    button_light_tx: Sender<(u8,u8,bool)>,
) -> (
    Receiver<bool>, 
    Receiver<u8>,
    Sender<[[bool; 2]; ELEV_NUM_FLOORS as usize]>,
    Receiver<[bool; ELEV_NUM_FLOORS as usize]>,
    Sender<(u8,u8,bool)>
) {
    // CLEAR ALL LIGHTS
    for floor in 0..ELEV_NUM_FLOORS {
        for btn in elev::HALL_UP..=elev::CAB {
            button_light_tx.send((floor, btn, false)).unwrap();
        }
    }

    let (requests_should_stop_tx, requests_should_stop_rx) = unbounded();
    let (requests_next_direction_tx, requests_next_direction_rx) = unbounded();
    let (hall_requests_tx, hall_requests_rx) = unbounded();
    let (cab_requests_tx, cab_requests_rx) = unbounded();
    let (elevator_data_tx, elevator_data_rx) = unbounded();

    spawn(move || main(
        cab_button_rx, 
        hall_requests_rx,
        button_light_tx,
        requests_should_stop_tx,
        requests_next_direction_tx,
        cab_requests_tx,
        elevator_data_rx
    ));
    (requests_should_stop_rx, 
     requests_next_direction_rx, 
     hall_requests_tx,
     cab_requests_rx,
     elevator_data_tx)
}

fn main(
    cab_button_rx: Receiver<poll::CallButton>, 
    hall_requests_rx: Receiver<[[bool; 2]; ELEV_NUM_FLOORS as usize]>, 
    button_light_tx: Sender<(u8,u8,bool)>,
    requests_should_stop_tx: Sender<bool>,
    requests_next_direction_tx: Sender<u8>,
    cab_requests_tx: Sender<[bool; ELEV_NUM_FLOORS as usize]>,
    elevator_data_rx: Receiver<(u8,u8,bool)>
) {
    let mut orders = [[false; ELEV_NUM_BUTTONS as usize]; ELEV_NUM_FLOORS as usize];

    loop {
        select! {
            recv(cab_button_rx) -> msg => {
                let destination = msg.as_ref().unwrap().floor;
                orders[destination as usize][elev::CAB as usize] = true;
                button_light_tx.send((destination, elev::CAB, true)).unwrap();
                let mut cab_requests = [false; ELEV_NUM_FLOORS as usize];
                for floor in 0..ELEV_NUM_FLOORS {
                    cab_requests[floor as usize] = orders[floor as usize][elev::CAB as usize];
                }
                cab_requests_tx.send(cab_requests).unwrap();
            },
            recv(hall_requests_rx) -> msg => {
                for floor in 0..ELEV_NUM_FLOORS {
                    for btn in elev::HALL_UP..=elev::HALL_DOWN {
                        orders[floor as usize][btn as usize] = msg.unwrap()[floor as usize][btn as usize];
                        println!("{:#?}", orders);
                        button_light_tx.send((floor, btn, orders[floor as usize][btn as usize])).unwrap();
                    }
                }
            },
            recv(elevator_data_rx) -> data => {
                let last_floor = data.unwrap().0;
                let last_direction = data.unwrap().1;
                let is_stopped = data.unwrap().2;
                if should_stop(orders, last_floor, last_direction) && !is_stopped {
                    requests_should_stop_tx.send(true).unwrap();
                    clear_order(button_light_tx.clone(), &mut orders, last_floor, last_direction);
                }
                let next_direction = next_direction(orders, last_floor, last_direction);
                requests_next_direction_tx.send(next_direction).unwrap();
                // if !elevator.floor_sensor().is_none() {
                //     clear_order(button_light_tx, &mut orders, last_floor, next_direction);
                // }
            },
        }
    }
}

fn clear_order(
    button_light_tx: Sender<(u8,u8,bool)>,
    orders: &mut [[bool; ELEV_NUM_BUTTONS as usize]; ELEV_NUM_FLOORS as usize],
    floor: u8,
    dirn: u8
) {
    let call_in_direction = if dirn == elev::DIRN_UP { elev::HALL_UP } else { elev::HALL_DOWN };
    orders[floor as usize][call_in_direction as usize] = false;
    button_light_tx.send((floor, call_in_direction, false)).unwrap();
    orders[floor as usize][elev::CAB as usize] = false;
    button_light_tx.send((floor, elev::CAB, false)).unwrap();
    if floor == 0 || floor == ELEV_NUM_FLOORS - 1 {
        let call_in_other_direction = if call_in_direction == elev::HALL_UP { elev::HALL_DOWN } else { elev::HALL_UP };
        orders[floor as usize][call_in_other_direction as usize] = false;
        button_light_tx.send((floor, call_in_other_direction, false)).unwrap();
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
