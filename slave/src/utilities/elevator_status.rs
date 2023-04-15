use shared_resources::call::Call;

use crate::utilities::direction::Direction;
use crate::utilities::request_collection::RequestCollection;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug, Clone)]
pub enum Behaviour {
    Idle,
    Moving,
    DoorOpen,
}

impl Behaviour {
    pub fn as_string(&self) -> String {
        match self {
            Behaviour::Idle => String::from("idle"),
            Behaviour::Moving => String::from("moving"),
            Behaviour::DoorOpen => String::from("doorOpen"),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ElevatorStatus {
    pub requests: RequestCollection,
    pub behaviour: Behaviour,
    pub floor: u8,  
    pub direction: Direction,
}

impl ElevatorStatus {
    pub fn new(num_floors: u8) -> Self {
        ElevatorStatus { 
            requests: RequestCollection::new(num_floors), 
            behaviour: Behaviour::Moving, 
            floor: 0, 
            direction: Direction::Down,
        }
    }
    
    pub fn serve_requests_here(&mut self) {
        self.requests.clear_request(Call::Cab, self.floor);
        self.requests.clear_request(self.direction.to_call().unwrap(), self.floor);
    }
    
    pub fn should_stop(&self) -> bool {
        self.requests.should_stop(self.floor, self.direction)
    }

    pub fn requests_at_this_floor(&self) -> bool {
        for request in self.requests.get_requests_at_floor(self.floor) {
            if request {
                return true
            }
        }
        false
    }

    pub fn update_direction(&mut self) {
        if self.requests.next_direction(self.floor, self.direction).is_some() {
            self.direction = self.requests.next_direction(self.floor, self.direction).unwrap();
        }
    }
}
