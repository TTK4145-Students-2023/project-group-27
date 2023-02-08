use super::elevator;
use super::elevator_io_types;

fn requests_above(e: elevator::Elevator) -> bool {
    for f in e.floor+1..elevator_io_types::N_FLOORS {
        for btn in 0..elevator_io_types::N_BUTTONS {
            if e.requests[f as usize][btn as usize] {
                return true;
            }
        }
    }
    return false;
}