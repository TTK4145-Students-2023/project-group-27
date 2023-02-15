use driver_rust::elevio::elev;
use crossbeam_channel::{select};

enum State {
    Idle,
    Moving,
    DoorOpen
}

pub fn main(elevator: elev::Elevator) {
    let mut state: State = State::Idle;
    loop {
        select! {

        }
    }
}
