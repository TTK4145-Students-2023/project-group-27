use super::elevator_io_types;

pub struct Config{
    pub clear_request_variant: u8,
    pub door_open_duration_s: f64,
}

pub struct Elevator{
    pub floor: u8,
    pub dirn: u8,
    pub requests: [[bool; elevator_io_types::N_FLOORS as usize]; elevator_io_types::N_BUTTONS as usize],
    pub behaviour: u8,
    pub config: Config,
}

