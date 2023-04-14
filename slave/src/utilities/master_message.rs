use std::collections::HashMap;

use shared_resources::call::Call;

pub struct MasterMessage {
    pub our_hall_requests: Vec<Vec<bool>>,
    pub all_hall_requests: Vec<Vec<bool>>,
}

impl MasterMessage {
    pub fn parse(message: HashMap<String, Vec<Vec<bool>>>, num_floors: u8, id: String) -> Self {
        let mut all_hall_requests = vec![vec![false; Call::num_hall_calls() as usize]; num_floors as usize];
        for (_, requests) in &message {
            for floor in 0..num_floors {
                for btn in Call::iter_hall() {
                    if requests[floor as usize][btn as usize] {
                        all_hall_requests[floor as usize][btn as usize] = true;
                    }
                }
            }
        }
        let empty_vec = vec![vec![false; Call::num_hall_calls() as usize]; num_floors as usize];
        let our_hall_requests = match message.get(&id) {
            Some(hr) => hr,
            None => &empty_vec, // master does not yet know about this elevator -> we're not assigned any requests
        };
        MasterMessage { 
            our_hall_requests: our_hall_requests.clone(), 
            all_hall_requests: all_hall_requests,
        }
    }
}
