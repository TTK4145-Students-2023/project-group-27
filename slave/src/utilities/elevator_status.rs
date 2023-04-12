use shared_resources::request::Request;
use shared_resources::call::Call;

use crate::utilities::direction::Direction;
use crate::utilities::requests::Requests;

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
    pub requests: Requests,
    pub behaviour: Behaviour,
    pub floor: u8,  
    pub direction: Direction,
    served_requests: Vec<Request>,
}

impl ElevatorStatus {
    pub fn new(num_floors: u8) -> Self {
        ElevatorStatus { 
            requests: Requests::new(num_floors), 
            behaviour: Behaviour::Moving, 
            floor: 0, 
            direction: Direction::Down,
            served_requests: Vec::new(),
        }
    }
    
    pub fn serve_requests_here(&mut self) {
        self.requests.clear_request(Call::Cab, self.floor);
        let call = if self.direction == Direction::Up { Call::HallUp } else { Call::HallDown };
        self.requests.clear_request(call, self.floor);
        self.served_requests.clear();
        //self.served_requests = Vec::new()
        self.served_requests.push(Request {
            floor: self.floor,
            call: if self.direction == Direction::Up { Call::HallUp } else { Call::HallDown },
        });
        // if no further orders in direction -> the order in opposite direction is also served
        // if !self.requests.further_requests_in_direction(self.floor, self.direction) {
        //     self.served_requests.push(Request {
        //         floor: self.floor,
        //         call: if self.direction == Direction::Up { Call::HallDown } else { Call::HallUp },
        //     });
        // }
    }
    
    pub fn pop_served_requests(&mut self) -> Vec<Request> {
        let served_requests = self.served_requests.clone();
        self.served_requests.clear();
        served_requests
    }
    
    pub fn should_stop(&self) -> bool {
        self.requests.should_stop(self.floor, self.direction)
    }

    pub fn next_direction(&self) -> Option<Direction> {
        self.requests.next_direction(self.floor, self.direction)
    }

    pub fn current_floor_has_requests(&self) -> bool {
        for call in self.requests.get_requests_at_floor(self.floor) {
            if call {
                return true
            }
        }
        false
    }
}
