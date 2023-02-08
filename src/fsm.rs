// use lazy_static::lazy_static;
// use parking_lot::Mutex;

// use super::elevator;
// use super::elevator_io_device;
// use super::requests;
// use super::timer;

// lazy_static! {
//     static ref ELEVATOR: Mutex<Elevator> = Mutex::new(elevator_uninitialized());
//     static ref OUTPUT_DEVICE: Mutex<ElevOutputDevice> = Mutex::new(elevio_getOutputDevice());
// }

// fn setAllLights(es: Elevator) {
//     for floor in 0..N_FLOORS {
//         for btn in 0..N_BUTTONS {
//             OUTPUT_DEVICE.requestButtonLight(floor, btn, es.requests[floor as usize][btn as usize]);
//         }
//     }
// }

// fn fsm_onInitBetweenFloors() {
//     OUTPUT_DEVICE.motorDirection(D_Down);
//     ELEVATOR.dirn = D_Down;
//     ELEVATOR.behaviour = EB_Moving;
// }

// fn fsm_onRequestButtonPress(btn_floor: u8, btn_type: Button) {
//     print!("\n\n%s(%d, %s)\n", "fsm_onRequestButtonPress", btn_floor, elevio_button_toString(btn_type));
//     elevator_print(ELEVATOR);

//     match ELEVATOR.behaviour {
//         EB_DoorOpen => {
//             OUTPUT_DEVICE.doorLight(true);
//             timer_start(ELEVATOR.config.doorOpenDuration_s);
//             ELEVATOR = requests_clearAtCurrentFloor(ELEVATOR);
//         },
//         EB_Moving => ELEVATOR.requests[btn_floor as usize][btn_type as usize] = true,
//         EB_Idle => {
//             ELEVATOR.requests[btn_floor as usize][btn_type as usize] = true;
//             pair = requests_chooseDirection(ELEVATOR);
//             ELEVATOR.dirn = pair.dirn;
//             ELEVATOR.behaviour = pair.behaviour;
//             match pair.behaviour {
//                 EB_DoorOpen => {
//                     OUTPUT_DEVICE.doorLight(true);
//                     timer_start(ELEVATOR.config.doorOpenDuration_s);
//                     ELEVATOR = requests_clearAtCurrentFloor(ELEVATOR);
//                 },
//                 EB_Moving => OUTPUT_DEVICE.motorDirection(ELEVATOR.dirn),        
//                 EB_Idle => (),
//             }
//         }
//     }

//     setAllLights(ELEVATOR); 
//     print!("\nNew state:\n");
//     elevator_print(ELEVATOR);
// }


// fn fsm_onFloorArrival(newFloor: u8){
//     print!("\n\n%s(%d)\n", "fsm_onFloorArrival", newFloor);
//     elevator_print(ELEVATOR);
    
//     ELEVATOR.floor = newFloor;
    
//     OUTPUT_DEVICE.floorIndicator(ELEVATOR.floor);
    
//     match ELEVATOR.behaviour {
//         EB_Moving => if requests_shouldStop(ELEVATOR) {
//             OUTPUT_DEVICE.motorDirection(D_Stop);
//             OUTPUT_DEVICE.doorLight(1);
//             ELEVATOR = requests_clearAtCurrentFloor(ELEVATOR);
//             timer_start(ELEVATOR.config.doorOpenDuration_s);
//             setAllLights(ELEVATOR);
//             ELEVATOR.behaviour = EB_DoorOpen;
//         },
//         _ => (),
//     }
    
//     print!("\nNew state:\n");
//     elevator_print(ELEVATOR); 
// }

// fn fsm_onDoorTimeout() {
//     printf("\n\n%s()\n", "fsm_onDoorTimeout");
//     elevator_print(ELEVATOR);
    
//     match ELEVATOR.behaviour {
//         EB_DoorOpen => {
//             pair = requests_chooseDirection(ELEVATOR);
//             ELEVATOR.dirn = pair.dirn;
//             ELEVATOR.behaviour = pair.behaviour;
            
//             match ELEVATOR.behaviour {
//                 EB_DoorOpen => {
//                     timer_start(ELEVATOR.config.doorOpenDuration_s);
//                     ELEVATOR = requests_clearAtCurrentFloor(ELEVATOR);
//                     setAllLights(ELEVATOR);
//                 },
//                 EB_Moving => (),
//                 EB_Idle => {
//                     OUTPUT_DEVICE.doorLight(false);
//                     OUTPUT_DEVICE.motorDirection(ELEVATOR.dirn);
//                 },
//             }
//         },
//         _ => (),
//     }
    
//     printf("\nNew state:\n");
//     elevator_print(ELEVATOR);
// }
