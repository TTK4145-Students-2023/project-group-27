use std::time::Duration;
use crossbeam_channel::{self, select};

pub fn main(s: crossbeam_channel::Sender<String>, r: crossbeam_channel::Receiver<String>) {
    const TIMER_DURATION: f64 = 3.0;
    let mut active: bool = false;

    loop {
        select! {
            recv(r) -> msg => {
                match msg.unwrap().as_str() {
                    "start timer" => active = true,
                    "stop timer" => active = false,
                    _ => ()
                }
            },
            default(Duration::from_secs_f64(TIMER_DURATION)) => {
                if active {
                    s.send(String::from("timed out")).unwrap();
                    active = false;
                }
            },
        }
    }
}
