use driver_rust::elevio::elev;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Down,
    Stop,
    Up,
}

impl Direction {
    pub fn as_elev_constant(self) -> u8 {
        match self {
            Direction::Down => elev::DIRN_DOWN,
            Direction::Stop => elev::DIRN_STOP,
            Direction::Up => elev::DIRN_UP,
        }
    }

    pub fn as_string(self) -> Option<String> {
        match self {
            Direction::Down => Some(String::from("down")),
            Direction::Up => Some(String::from("up")),
            Direction::Stop => None,
        }
    }
}
