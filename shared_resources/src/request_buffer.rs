use std::time::{Instant, Duration};

use crate::request::Request;

#[derive(Debug, Clone)]
pub struct RequestBuffer {
    new_requests: Vec<Request>,
    new_request_timers: Vec<Instant>,
    timeout: u64,
}

impl RequestBuffer {
    pub fn new(timeout: u64) -> Self {
        RequestBuffer { 
            new_requests: Vec::new(), 
            new_request_timers: Vec::new(),
            timeout: timeout,
        }
    }

    pub fn get_new_requests(&self) -> Vec<Request> {
        self.new_requests.clone()
    }

    pub fn insert_new_request(&mut self, request: Request) {
        self.new_requests.push(request);
        self.new_request_timers.push(Instant::now());
    }

    pub fn remove_confirmed_requests(&mut self, all_hall_requests: &Vec<Vec<bool>>) {
        for index in (0..self.new_requests.len()).rev() {
            let floor = self.new_requests[index].floor;
            let call = self.new_requests[index].call;
            if all_hall_requests[floor as usize][call as usize] {
                self.new_requests.remove(index);
                self.new_request_timers.remove(index);
            }
        }
    }

    pub fn remove_timed_out_orders(&mut self) {
        for index in (0..self.new_requests.len()).rev() {
            if self.new_request_timers[index].elapsed() > Duration::from_secs(self.timeout) {
                self.new_requests.remove(index);
                self.new_request_timers.remove(index);
            }
        }
    }
}
