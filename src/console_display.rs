use crossterm::{
    cursor, queue,
    style::{self, Stylize},
    terminal::SetSize,
    ExecutableCommand,
};
use std::io::{stdout, Write};

use crate::CHIP8_WIDTH;
use crate::{MyResult, CHIP8_HEIGHT};
use crossterm::terminal::{Clear, ClearType};

pub struct ConsoleDisplay;

impl ConsoleDisplay {
    pub fn new() -> MyResult<Self> {
        Self::prepare()?;
        Ok(Self {})
    }

    fn prepare() -> MyResult<()> {
        let mut stdout = stdout();
        stdout
            .execute(SetSize(CHIP8_WIDTH as u16, CHIP8_HEIGHT as u16))?
            .execute(Clear(ClearType::All))?
            .execute(cursor::Hide)?;

        Ok(())
    }

    pub fn draw(&mut self, pixels: &[[u8; CHIP8_WIDTH]; CHIP8_HEIGHT]) -> MyResult<()> {
        let mut stdout = stdout();

        for (y, row) in pixels.iter().enumerate() {
            for (x, &col) in row.iter().enumerate() {
                if col == 0 {
                    queue!(
                        stdout,
                        cursor::MoveTo(x as u16, y as u16),
                        style::Print(" ")
                    )?;
                } else {
                    queue!(
                        stdout,
                        cursor::MoveTo(x as u16, y as u16),
                        style::PrintStyledContent("█".green())
                    )?;
                }
            }
        }

        stdout.flush()?;

        Ok(())
    }
}
