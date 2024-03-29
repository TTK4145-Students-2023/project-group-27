/// ----- DEBUG MODULE -----
/// This module receives information about the elevators currently
/// connected to the master and the active hall requests, and does 
/// a formatted print to the console.

use std::io::{stdout, Stdout, Write};
use std::collections::HashMap;
use std::time::Instant;

use crossbeam_channel::{Receiver, select};
use crossterm::{cursor, terminal, Result, ExecutableCommand};

use shared_resources::call::Call;

use crate::utilities::hall_request_assigner::ElevatorData;

const STATUS_SIZE: u16 = 20;
const TIMEOUT: u128 = 4000 as u128;

pub fn main(
    num_floors: u8,
    hall_requests_rx: Receiver<Vec<Vec<bool>>>,
    connected_elevators_rx: Receiver<HashMap<String, ElevatorData>>,
) -> Result<()> {
    let mut stdout = stdout();

    let mut hall_requests = vec![vec![false; Call::num_hall_calls() as usize]; num_floors as usize];
    let mut connected_elevators: HashMap<String, ElevatorData> = HashMap::new();

    loop {
        select! {
            recv(hall_requests_rx) -> msg => {
                hall_requests = msg.unwrap();
                printstatus(num_floors, &mut stdout, &hall_requests, connected_elevators.clone())?;
            },
            recv(connected_elevators_rx) -> msg => {
                connected_elevators = msg.unwrap();
                printstatus(num_floors, &mut stdout, &hall_requests, connected_elevators.clone())?;
            },
        }
    }
}

fn printstatus(
    num_floors: u8,
    stdout: &mut Stdout,
    hall_requests: &Vec<Vec<bool>>,
    connected_elevators: HashMap<String, ElevatorData>,
) -> Result<()> {
    
    stdout.execute(terminal::Clear(terminal::ClearType::FromCursorDown))?;

    writeln!(stdout, "+--------------------------------------+")?;
    writeln!(stdout, "| HALL ORDERS                          |")?;
    writeln!(stdout, "+------------+------------+------------+")?;
    writeln!(stdout, "| {0:<10} | {1:<10} | {2:<10} |", "FLOOR", "HALL UP", "HALL DOWN")?;
    for i in (0..num_floors).rev() {
        writeln!(stdout, "+------------+------------+------------+")?;
        writeln!(stdout, "| {0:<10} | {1:<10} | {2:<10} |", i, hall_requests[i as usize][0],  hall_requests[i as usize][1])?;
    }
    writeln!(stdout, "+------------+------------+------------+\n\n")?;

    writeln!(stdout, "+-----------------------------------------------------------------------------+")?;
    writeln!(stdout, "| CONNECTED ELEVATORS                                                         |")?;
    writeln!(stdout, "+------------+------------+------------+------------+------------+------------+")?;
    writeln!(stdout, "| {0:<10} | {1:<10} | {2:<10} | {3:<10} | {4:<10} | {5:<10} |", "ID", "LAST SEEN", "AVAILABLE", "STATE", "FLOOR", "DIRECTION")?;
    writeln!(stdout, "+------------+------------+------------+------------+------------+------------+")?;
    for (id, elev) in &connected_elevators {
        writeln!(stdout, "| {0:<10} | {1:>8}ms | {2:<10} | {3:<10} | {4:<10} | {5:<10} |", 
        id, 
        Instant::now().duration_since(elev.last_seen).as_millis(),
        if Instant::now().duration_since(elev.last_available).as_millis() > TIMEOUT { "NO" } else { "YES"},
        elev.state.behaviour, 
        elev.state.floor, 
        elev.state.direction)?;
        writeln!(stdout, "+------------+------------+------------+------------+------------+------------+")?;
    }

    stdout.execute(cursor::MoveUp(STATUS_SIZE + 2 * connected_elevators.len() as u16))?;
    Ok(())
}
