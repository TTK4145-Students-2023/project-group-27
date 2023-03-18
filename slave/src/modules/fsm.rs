/// ----- FSM MODULE -----
/// This module is the finite state machine controlling the elevator.
/// It receives events from other modules and switches states based
/// on these.

use std::time::Duration;

use crossbeam_channel::{select, Receiver, Sender, tick};

use crate::utilities::elevator_behaviour::{ElevatorBehaviour, Behaviour};
use crate::utilities::config::ElevatorSettings;
use crate::utilities::master_message::MasterMessage;
use crate::utilities::call::Call;
use crate::utilities::direction::Direction;
use crate::utilities::request::Request;

pub fn main(
    elevator_settings: ElevatorSettings,
    floor_sensor_rx: Receiver<u8>,
    floor_indicator_tx: Sender<u8>,
    button_light_tx: Sender<(Request,bool)>,
    doors_closing_rx: Receiver<bool>,
    doors_activate_tx: Sender<bool>,
    cab_button_rx: Receiver<u8>,
    motor_direction_tx: Sender<Direction>,
    master_hall_requests_rx: Receiver<MasterMessage>,
    elevator_behaviour_tx: Sender<ElevatorBehaviour>,
) {
    let timer = tick(Duration::from_secs_f64(0.25));

    let num_floors = elevator_settings.num_floors;
    let mut elevator_behaviour = ElevatorBehaviour::new(num_floors);

    loop {
        select! {
            recv(cab_button_rx) -> msg => {
                let floor = msg.unwrap();
                elevator_behaviour.requests.add_order(floor, Call::Cab);
                button_light_tx.send((Request{ floor: floor, call: Call::Cab }, true)).unwrap();
            },
            recv(master_hall_requests_rx) -> msg => {
                let message = msg.unwrap();
                elevator_behaviour.requests.update_hall_requests(message.our_hall_requests);
                for floor in 0..num_floors {
                    for call in Call::iter() {
                        button_light_tx.send((
                            Request{ floor: floor, call: call }, 
                            message.all_hall_requests[floor as usize][call as usize],
                        )).unwrap();
                    }
                }
            },
            recv(floor_sensor_rx) -> msg => {
                elevator_behaviour.floor = msg.unwrap();
                floor_indicator_tx.send(elevator_behaviour.floor).unwrap();
                if elevator_behaviour.should_stop() {
                    elevator_behaviour.behaviour = match elevator_behaviour.behaviour {
                        Behaviour::Idle | Behaviour::Moving => {
                            motor_direction_tx.send(Direction::Stop).unwrap();
                            doors_activate_tx.send(true).unwrap();
                            elevator_behaviour.serve_requests_here();
                            button_light_tx.send((Request { floor: elevator_behaviour.floor, call: Call::Cab }, false)).unwrap();
                            Behaviour::DoorOpen
                        },
                        Behaviour::DoorOpen => elevator_behaviour.behaviour,
                    }
                }
            },
            recv(doors_closing_rx) -> _ => {
                elevator_behaviour.behaviour = match elevator_behaviour.behaviour {
                    Behaviour::DoorOpen => Behaviour::Idle,
                    Behaviour::Idle | Behaviour::Moving => elevator_behaviour.behaviour,
                }
            },
            recv(timer) -> _ => {
                elevator_behaviour.behaviour = match elevator_behaviour.behaviour {
                    Behaviour::Idle => {
                        let next_direction = elevator_behaviour.next_direction();
                        if next_direction.is_some() {
                            motor_direction_tx.send(next_direction.unwrap()).unwrap();
                            elevator_behaviour.direction = next_direction.unwrap();
                            Behaviour::Moving
                        } else if elevator_behaviour.current_floor_has_requests()
                            && elevator_behaviour.should_stop() {
                            doors_activate_tx.send(true).unwrap();
                            elevator_behaviour.serve_requests_here();
                            button_light_tx.send((Request { floor: elevator_behaviour.floor, call: Call::Cab }, false)).unwrap();
                            Behaviour::DoorOpen
                        }else {
                            elevator_behaviour.behaviour
                        }
                    },
                    Behaviour::Moving | Behaviour::DoorOpen => elevator_behaviour.behaviour,
                }
            }
        }
        elevator_behaviour_tx.send(elevator_behaviour.clone()).unwrap();
        elevator_behaviour.flush_served_requests();
    }
}
