/// ----- HALL REQUEST ASSIGNER -----
/// This stateless module provides some abstraction when calling the
/// hall_request_assigner algorithm provided. It takes elevator states and
/// active hall requests as parameters and returns a data structure containing
/// which elevators are to serve which orders.

use std::collections::HashMap;
use std::process::Command;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HRAElevState {
    pub behaviour: String, 
    pub floor: u8,    
    pub direction: String,  
    pub cab_requests: Vec<bool>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HRAInput {
    pub hall_requests: Vec<Vec<bool>>,
    pub states: HashMap<String, HRAElevState>
}

pub fn assign_orders(
    exec_path: &String,
    hall_requests: Vec<Vec<bool>>,
    states: HashMap<String, HRAElevState>
) -> Result<HashMap<String, Vec<Vec<bool>>>, serde_json::Error> {
    let input = HRAInput {
        hall_requests: hall_requests,
        states: states
    };
    let json_arg = serde_json::to_string(&input).unwrap();
    let command = "./".to_owned() + exec_path + " -i '" + json_arg.as_str() + "'";
    let result = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to call hall request assigner");
    let str_result = String::from_utf8(result.stdout).unwrap();
    serde_json::from_str(&str_result)
}
