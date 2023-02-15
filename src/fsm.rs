use driver_rust::elevio::elev::DIRN_STOP;
use driver_rust::elevio::poll;
use driver_rust::elevio::elev;
use crossbeam_channel::{select, Receiver, Sender};

use crate::requests::{self, ElevatorBehaviour, DirnBehaviourPair};

pub fn main(
    elevator: elev::Elevator, 
    call_button_rx: Receiver<poll::CallButton>,
    floor_sensor_rx: Receiver<u8>,
    doors_closing_rx: Receiver<bool>,
    obstruction_rx: Receiver<bool>,
    doors_activate_tx: Sender<bool>
) {
    let mut elevator_behaviour: ElevatorBehaviour = ElevatorBehaviour::Idle;
    let mut dirn: u8 = DIRN_STOP;
    loop {
        select! {
            recv(call_button_rx) -> call_button => {
                on_request_button_press(
                    elevator.clone(),
                    &mut elevator_behaviour,
                    call_button.as_ref().unwrap().floor, 
                    call_button.unwrap().call
                );
            },
            recv(floor_sensor_rx) -> floor => {
                on_floor_arrival(
                    elevator.clone(),
                    &mut elevator_behaviour,
                    floor.unwrap()
                );
            },
            recv(doors_closing_rx) -> _ => {
                doors_activate_tx.send(false).unwrap();
                on_door_timeout(
                    elevator.clone(),
                    &mut elevator_behaviour,
                    &mut dirn,
                    doors_activate_tx.clone()
                );
            },
            recv(obstruction_rx) -> _ => {
                doors_activate_tx.send(true).unwrap();
            },
        }
    }
}

fn on_request_button_press(
    elevator: elev::Elevator, 
    elevator_behaviour: &mut ElevatorBehaviour, 
    f: u8, 
    b: u8
) {
    
}

fn on_floor_arrival(
    elevator: elev::Elevator, 
    elevator_behaviour: &mut ElevatorBehaviour, 
    f: u8
) {

}

fn on_door_timeout(
    elevator: elev::Elevator, 
    elevator_behaviour: &mut ElevatorBehaviour, 
    dirn: &mut u8, 
    doors_activate_tx: Sender<bool>
) {
    // switch(elevator.behaviour){
    //     case EB_DoorOpen:;
    //         DirnBehaviourPair pair = requests_chooseDirection(elevator);
    //         elevator.dirn = pair.dirn;
    //         elevator.behaviour = pair.behaviour;
            
    //         switch(elevator.behaviour){
    //         case EB_DoorOpen:
    //             timer_start(elevator.config.doorOpenDuration_s);
    //             elevator = requests_clearAtCurrentFloor(elevator);
    //             setAllLights(elevator);
    //             break;
    //         case EB_Moving:
    //         case EB_Idle:
    //             outputDevice.doorLight(0);
    //             outputDevice.motorDirection(elevator.dirn);
    //             break;
    //         }
            
    //         break;
    //     default:
    //         break;
    //     }
    match elevator_behaviour {
        ElevatorBehaviour::DoorOpen => {
            let pair: DirnBehaviourPair = requests::choose_direction(elevator.clone());
            *dirn = pair.dirn;
            *elevator_behaviour = pair.behaviour;
            match elevator_behaviour {
                ElevatorBehaviour::DoorOpen => {
                    doors_activate_tx.send(true).unwrap();
                    requests::clear_at_current_floor(elevator);
                },
                ElevatorBehaviour::Moving | ElevatorBehaviour::Idle => {

                },
            }
        },
        _ => (),
    }
}
