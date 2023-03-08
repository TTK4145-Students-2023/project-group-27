use std::collections::HashMap;
use std::process::Command;

use crate::config;

const EXEC_NAME: &str = "hall_request_assigner";

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HRAElevState {
    pub behaviour: String, 
    pub floor: u8,    
    pub direction: String,  
    pub cab_requests: [bool; config::ELEV_NUM_FLOORS as usize]
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HRAInput {
    pub hall_requests: [[bool; 2]; config::ELEV_NUM_FLOORS as usize],
    pub states: HashMap<String, HRAElevState>
}

pub fn assign_orders(
    hall_requests: [[bool; 2]; config::ELEV_NUM_FLOORS as usize],
    states: HashMap<String, HRAElevState>
) -> HashMap<String, [[bool; 2]; config::ELEV_NUM_FLOORS as usize]> {
    let input = HRAInput {
        hall_requests: hall_requests,
        states: states
    };
    let json_arg = serde_json::to_string(&input).unwrap();
    let command = "./".to_owned() + EXEC_NAME + " -i '" + json_arg.as_str() + "'";
    let result = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to call hall request assigner");
    let str_result = String::from_utf8(result.stdout).unwrap();
    let output: HashMap<String, [[bool; 2]; config::ELEV_NUM_FLOORS as usize]> = serde_json::from_str(&str_result).unwrap();
    output
}
