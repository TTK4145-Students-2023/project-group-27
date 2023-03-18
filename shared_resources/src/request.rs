use driver_rust::elevio::poll;

use super::call::Call;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Request {
    pub floor: u8,
    pub call: Call
}

impl Request {
    pub fn from_elev(call_button: poll::CallButton) -> Self {
        Request {
            floor: call_button.floor,
            call: Call::from_elev_constant(call_button.call).unwrap(),
        }
    }
}
