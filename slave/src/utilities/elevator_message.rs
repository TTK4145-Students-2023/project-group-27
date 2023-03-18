use super::request_buffer::RequestBuffer;
use super::request::Request;
use super::elevator_behaviour::ElevatorBehaviour;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ElevatorMessage {
    pub id: String,
    pub behaviour: String,
    pub floor: u8,
    pub direction: String,
    pub cab_requests: Vec<bool>,
    pub new_hall_orders: Vec<Request>,
    pub served_hall_orders: Vec<Request>,
}

impl ElevatorMessage {
    pub fn new(id: String, elevator_behaviour: ElevatorBehaviour, request_buffer: &RequestBuffer) -> Self {
        ElevatorMessage {
            id: id,
            behaviour: elevator_behaviour.behaviour.as_string(),
            floor: elevator_behaviour.floor,
            direction: elevator_behaviour.direction.as_string().unwrap(),
            cab_requests: elevator_behaviour.requests.get_cab_requests(),
            new_hall_orders: request_buffer.get_new_requests(),
            served_hall_orders: request_buffer.get_served_requests(),
        }
    }
}
