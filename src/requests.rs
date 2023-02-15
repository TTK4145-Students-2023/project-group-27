use driver_rust::elevio::elev;

pub struct DirnBehaviourPair {
    pub dirn: u8,
    pub behaviour: ElevatorBehaviour
}

pub enum ElevatorBehaviour {
    Idle,
    Moving,
    DoorOpen
}

pub fn choose_direction(elevator: elev::Elevator) -> DirnBehaviourPair {
    DirnBehaviourPair {dirn: 0, behaviour: ElevatorBehaviour::Idle}
}
