use std::io::{stdout, Stdout, Write};

use crossterm::{cursor, terminal, Result, ExecutableCommand};

use super::elevator_behaviour::ElevatorBehaviour;

const STATUS_SIZE: u16 = 24;

pub struct Debug {
    stdout: Stdout,
    num_floors: u8,
}

impl Debug {
    pub fn new(num_floors: u8) -> Self {
        Debug { 
            stdout: stdout(),
            num_floors: num_floors,
        }
    }

    pub fn printstatus(&mut self, elevator_behaviour: &ElevatorBehaviour) -> Result<()> {
        self.stdout.execute(cursor::MoveUp(STATUS_SIZE))?;
        self.stdout.execute(terminal::Clear(terminal::ClearType::FromCursorDown))?;
    
        writeln!(self.stdout, "+---------------------------------------------------+")?;
        writeln!(self.stdout, "| ORDERS FOR THIS ELEVATOR                          |")?;
        writeln!(self.stdout, "+------------+------------+------------+------------+")?;
        writeln!(self.stdout, "| {0:<10} | {1:<10} | {2:<10} | {3:<10} |", "FLOOR", "HALL UP", "HALL DOWN", "CAB")?;
        for i in (0..self.num_floors).rev() {
            writeln!(self.stdout, "+------------+------------+------------+------------+")?;
            let floor_i = elevator_behaviour.requests.get_requests_at_floor(i);
            writeln!(self.stdout, "| {0:<10} | {1:<10} | {2:<10} | {3:<10} |", i, floor_i[0],  floor_i[1],  floor_i[2])?;
        }
        writeln!(self.stdout, "+------------+------------+------------+------------+\n\n")?;
    
        writeln!(self.stdout, "+-------------------------+")?;
        writeln!(self.stdout, "| STATE MACHINE           |")?;
        writeln!(self.stdout, "+------------+------------+")?;
        writeln!(self.stdout, "| {0:<10} | {1:<10} |", "STATE", elevator_behaviour.behaviour.as_string())?;
        writeln!(self.stdout, "+------------+------------+")?;
        writeln!(self.stdout, "| {0:<10} | {1:<10} |", "FLOOR", elevator_behaviour.floor)?;
        writeln!(self.stdout, "+------------+------------+")?;
        writeln!(self.stdout, "| {0:<10} | {1:<10} |", "DIRECTION", elevator_behaviour.direction.as_string().unwrap())?;
        writeln!(self.stdout, "+------------+------------+")?;
    
        Ok(())
    }
}
