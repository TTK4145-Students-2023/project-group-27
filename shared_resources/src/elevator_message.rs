use super::request::Request;

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
