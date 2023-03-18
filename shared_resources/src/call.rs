use driver_rust::elevio::elev;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub enum Call {
    HallUp = 0,
    HallDown = 1,
    Cab = 2,
}

impl Call {
    pub fn from_elev_constant(elev_constant: u8) -> Option<Self> {
        match elev_constant {
            elev::HALL_UP => Some(Call::HallUp),
            elev::HALL_DOWN => Some(Call::HallDown),
            elev::CAB => Some(Call::Cab),
            _ => None,
        }
    }

    pub fn num_calls() -> u8 {
        3
    }

    pub fn num_hall_calls() -> u8 {
        2
    }

    pub fn as_elev_constant(self) -> u8 {
        match self {
            Call::HallUp => elev::HALL_UP,
            Call::HallDown => elev::HALL_DOWN,
            Call::Cab => elev::CAB,
        }
    }

    pub fn iter() -> impl Iterator<Item = Call> {
        [Call::HallUp, Call::HallDown, Call::Cab].iter().copied()
    }

    pub fn iter_hall() -> impl Iterator<Item = Call> {
        [Call::HallUp, Call::HallDown].iter().copied()
    }
}
