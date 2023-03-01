use std::collections::HashMap;
use std::process::Command;

use crate::config;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct HRAElevState {
    behaviour: String, 
    floor: u8,    
    direction: String,  
    cab_requests: [bool; config::ELEV_NUM_FLOORS as usize]
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct HRAInput {
    hall_requests: Vec<[bool; 2]>,
    states: HashMap<String, HRAElevState>
}

pub fn test_hall_assigner() {
    let exec_name = "hall_request_assigner";

    let test_input = HRAInput {
        hall_requests: Vec::from([[false, false], [true, false], [false, false], [false, true]]),
        states: HashMap::from([
            (String::from("one"), HRAElevState{
                behaviour: String::from("moving"),
                floor: 2,
                direction: String::from("up"),
                cab_requests: [false, false, false, true],
            }),
            (String::from("two"), HRAElevState{
                behaviour: String::from("idle"),
                floor: 0,
                direction: String::from("stop"),
                cab_requests: [false, false, false, false],
            }),
        ]),
    };

    let command = "./".to_owned() + exec_name + " -i '" + serde_json::to_string(&test_input).unwrap().as_str() + "'";
    // THIS ONLY WORKS FOR UNIX
    let result = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to call hall request assigner");

    let expected = "{\"one\":[[false,false],[false,false],[false,false],[false,true]],\"two\":[[false,false],[true,false],[false,false],[false,false]]}\n";
    let actual = String::from_utf8(result.stdout).unwrap();
    assert_eq!(expected, actual)
}
