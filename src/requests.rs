use driver_rust::elevio::elev as d_elevator;
use super::elevator;
use super::elevator_io_types;

struct DirnBehaviourPair {
    dirn: u8,
    behaviour: elevator::ElevatorBehaviour,
}

fn requests_above(e: &elevator::Elevator) -> bool {
    for f in e.floor+1..elevator_io_types::N_FLOORS {
        for btn in 0..elevator_io_types::N_BUTTONS {
            if e.requests[f as usize][btn as usize] {
                return true;
            }
        }
    }
    return false;
}

fn requests_here(e: &elevator::Elevator) -> bool {
    for btn in 0..elevator_io_types::N_BUTTONS {
        if e.requests[e.floor as usize][btn as usize] {
            return true;
        }
    }
    return false;
}

fn requests_below(e: &elevator::Elevator) -> bool {
    for f in 0..e.floor {
        for btn in 0..elevator_io_types::N_BUTTONS {
            if e.requests[f as usize][btn as usize] {
                return true;
            }
        }
    }
    return false;
}

fn requests_chooseDirection(e: &elevator::Elevator) -> DirnBehaviourPair {
    match e.dirn {
        d_elevator::DIRN_UP     => return { if requests_above(e) {DirnBehaviourPair {dirn: d_elevator::DIRN_UP, behaviour: elevator::ElevatorBehaviour::EbMoving}} 
                                            else if requests_here(e) {DirnBehaviourPair {dirn: d_elevator::DIRN_DOWN, behaviour: elevator::ElevatorBehaviour::EbDoorOpen}}
                                            else if requests_below(e) {DirnBehaviourPair {dirn: d_elevator::DIRN_DOWN, behaviour: elevator::ElevatorBehaviour::EbMoving}}
                                            else {DirnBehaviourPair {dirn: d_elevator::DIRN_STOP, behaviour: elevator::ElevatorBehaviour::EbIdle}}},
        //d_elevator::DIRN_DOWN   => return {},
        //d_elevator::DIRN_STOP   => return {},
        _ => {DirnBehaviourPair {dirn: d_elevator::DIRN_UP, behaviour: elevator::ElevatorBehaviour::EbMoving}},
    }
}