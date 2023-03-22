use shared_resources::call::Call;

use crate::utilities::direction::Direction;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Requests {
    requests: Vec<Vec<bool>>,
    num_floors: u8,
}

impl Requests {
    pub fn new(num_floors: u8) -> Self {
        Requests {
            num_floors: num_floors,
            requests: vec![vec![false; Call::num_calls() as usize]; num_floors as usize],
        }
    }

    pub fn add_order(&mut self, floor: u8, call: Call) {
        self.requests[floor as usize][call as usize] = true;
    }

    pub fn clear_cab_request(&mut self, floor: u8) {
        self.requests[floor as usize][Call::Cab as usize] = false;
    }

    pub fn update_hall_requests(&mut self, our_hall_requests: Vec<Vec<bool>>) {
        for floor in 0..self.num_floors {
            for btn in Call::iter_hall() {
                self.requests[floor as usize][btn as usize] = our_hall_requests[floor as usize][btn as usize];
            }
        }
    }

    pub fn get_cab_requests(&self) -> Vec<bool> {
        let mut cab_requests = vec![false; self.num_floors as usize];
        for floor in 0..self.num_floors {
            cab_requests[floor as usize] = self.requests[floor as usize][Call::Cab as usize];
        }
        cab_requests
    }

    pub fn should_stop(&self, floor: u8, direction: Direction) -> bool {
        if Self::cab_request_at_floor(self, floor)
        || Self::requests_in_direction_at_this_floor(self, floor, direction)
        || !Self::further_requests_in_direction(self, floor, direction) {
            return true
        }
        false
    }

    pub fn cab_request_at_floor(&self, floor: u8) -> bool {
        self.requests[floor as usize][Call::Cab as usize]
    }

    pub fn requests_in_direction_at_this_floor(&self, floor: u8, direction: Direction) -> bool {
        let hall_button = if direction == Direction::Up { Call::HallUp } else { Call::HallDown };
        self.requests[floor as usize][hall_button as usize]
    }

    pub fn further_requests_in_direction(&self, floor: u8, direction: Direction) -> bool {
        let range = if direction == Direction::Up { (floor+1)..self.num_floors } else { 0..floor };
        for f in range {
            for call in Call::iter() {
                if self.requests[f as usize][call as usize] {
                    return true
                }
            }
        }
        false
    }

    pub fn next_direction(&self, floor: u8, last_direction: Direction) -> Option<Direction> {
        let other_direction = if last_direction == Direction::Up { Direction::Down } else { Direction::Up };
        if Self::further_requests_in_direction(self, floor, last_direction) {
            return Some(last_direction)
        } else if Self::further_requests_in_direction(self, floor, other_direction) {
            return Some(other_direction)
        }
        None
    }

    pub fn get_requests_at_floor(&self, floor: u8) -> Vec<bool> {
        self.requests[floor as usize].clone()
    }
}
