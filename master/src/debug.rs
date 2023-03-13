use std::io::{stdout, Stdout, Write};
use std::collections::HashMap;
use std::time::Instant;

use crossbeam_channel::{Receiver, select};
use crossterm::{cursor, terminal, Result, ExecutableCommand};

use crate::config;
use crate::network::ElevatorData;

const STATUS_SIZE: u16 = 20;

pub fn main(
    hall_requests_rx: Receiver<[[bool; 2]; config::ELEV_NUM_FLOORS as usize]>,
    connected_elevators_rx: Receiver<HashMap<String, ElevatorData>>,
) -> Result<()> {
    let mut stdout = stdout();

    let mut hall_requests = [[false; 2]; config::ELEV_NUM_FLOORS as usize];
    let mut connected_elevators: HashMap<String, ElevatorData> = HashMap::new();

    loop {
        select! {
            recv(hall_requests_rx) -> msg => {
                hall_requests = msg.unwrap();
                printstatus(&mut stdout, hall_requests, connected_elevators.clone())?;
            },
            recv(connected_elevators_rx) -> msg => {
                connected_elevators = msg.unwrap();
                printstatus(&mut stdout, hall_requests, connected_elevators.clone())?;
            },
        }
    }
}

fn printstatus(
    stdout: &mut Stdout,
    hall_requests: [[bool; 2]; config::ELEV_NUM_FLOORS as usize],
    connected_elevators: HashMap<String, ElevatorData>,
) -> Result<()> {
    
    stdout.execute(terminal::Clear(terminal::ClearType::FromCursorDown))?;

    writeln!(stdout, "+--------------------------------------+")?;
    writeln!(stdout, "| HALL ORDERS                          |")?;
    writeln!(stdout, "+------------+------------+------------+")?;
    writeln!(stdout, "| {0:<10} | {1:<10} | {2:<10} |", "FLOOR", "HALL UP", "HALL DOWN")?;
    for i in (0..config::ELEV_NUM_FLOORS).rev() {
        writeln!(stdout, "+------------+------------+------------+")?;
        writeln!(stdout, "| {0:<10} | {1:<10} | {2:<10} |", i, hall_requests[i as usize][0],  hall_requests[i as usize][1])?;
    }
    writeln!(stdout, "+------------+------------+------------+\n\n")?;

    writeln!(stdout, "+---------------------------------------------------------------------+")?;
    writeln!(stdout, "| CONNECTED ELEVATORS                                                 |")?;
    writeln!(stdout, "+-----------------+------------+------------+------------+------------+")?;
    writeln!(stdout, "| {0:<15} | {1:<10} | {2:<10} | {3:<10} | {4:<10} |", "ID", "LAST SEEN", "STATE", "FLOOR", "DIRECTION")?;
    writeln!(stdout, "+-----------------+------------+------------+------------+------------+")?;
    for (id, elev) in &connected_elevators {
        writeln!(stdout, "| {0:<15} | {1:>8}ms | {2:<10} | {3:<10} | {4:<10} |", 
        id, 
        Instant::now().duration_since(elev.last_seen).as_millis(), 
        elev.state.behaviour, 
        elev.state.floor, 
        elev.state.direction)?;
        writeln!(stdout, "+-----------------+------------+------------+------------+------------+")?;
    }

    stdout.execute(cursor::MoveUp(STATUS_SIZE + 2 * connected_elevators.len() as u16))?;
    Ok(())
}
