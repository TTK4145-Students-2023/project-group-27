pub const N_FLOORS: u8 = 4;
pub const N_BUTTONS: u8 = 3;
pub const ADDR: &str = "localhost:15657";

pub struct ElevInputDevice {
    pub floor_sensor: fn()          -> u8,
    pub request_button: fn(u8,u8)   -> u8,
    pub stop_button: fn()           -> u8,
    pub obstruction: fn()           -> u8,
}

pub struct ElevOutputDevice {
    pub floor_indicator: fn(u8)             -> (),
    pub request_button_light: fn(u8,u8,u8)  -> (),
    pub door_light: fn(u8)                  -> (),
    pub stop_button_light: fn(u8)           -> (),
    pub motor_direction: fn(u8)             -> (),
}

