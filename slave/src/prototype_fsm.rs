

use std::time::Duration;

use crossbeam_channel::{Receiver, Sender, select, tick};
use driver_rust::elevio::elev::{elev, poll, self, DIRN_DOWN};

use crate::config::ElevatorSettings;

#[derive(PartialEq, Debug)]
enum State {
    Idle,
    Moving,
    DoorOpen,
}

#[derive(Clone, Debug)]
pub struct ElevatorStatus {
    pub orders: Vec<Vec<bool>>,
    pub cleared_request: poll::CallButton,
    pub state: String,
    pub floor: u8,
    pub direction: u8
}

pub fn main(
    elevator_settings: ElevatorSettings,
    cab_button_rx: Receiver<poll::CallButton>, // From io
    our_hall_requests_rx: Receiver<Vec<[bool; 2]>>, // From network
    all_hall_requests_rx: Receiver<Vec<[bool; 2]>>, // From network
    doors_closing_rx: Receiver<bool>, // From doors
    floor_sensor_rx: Receiver<u8>, // From io
    cab_requests_tx: Sender<Vec<bool>>, // To io
    button_light_tx: Sender<(u8,u8,bool)>, // To io
    doors_activate_tx: Sender<bool>, // To doors
    motor_direction_tx: Sender<u8>, // To io
    floor_indicator_tx: Sender<u8>, // To io
    // orders_tx: Sender<Vec<Vec<bool>>>, // To debug
    // cleared_request_tx: Sender<poll::CallButton>, // To network
    // elevator_state_tx: Sender<(String,u8,u8)>, // To network
    elevator_status_tx: Sender<ElevatorStatus>
) {

    let timer = tick(Duration::from_secs_f64(0.1));
    let n_buttons = elevator_settings.num_buttons;
    let n_floors = elevator_settings.num_floors;

    let mut floor: u8 = 0;
    let mut direction: u8 = DIRN_DOWN;
    let mut state: State = State::Moving;
    let mut orders = vec![vec![false; n_buttons as usize]; n_floors as usize];

    // CLEAR ALL LIGHTS
    for floor in 0..n_floors {
        for btn in elev::HALL_UP..=elev::CAB {
            button_light_tx.send((floor, btn, false)).unwrap();
        }
    }

    loop {
        select! {
            recv(timer) -> _ => {
                orders_tx.send(ElevatorStatus {
                    /* 
                    
                    Fill out content
                    
                    */
                }).unwrap();
            }
        }
        elevator_status_tx.send(/* object */).unwrap();
    }
}