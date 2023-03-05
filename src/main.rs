mod chip8;
mod console_display;
mod emulator;
mod fontset;

use std::error::Error;

use emulator::Emulator;

pub const OPCODE_SIZE: usize = 2;
pub const CHIP8_WIDTH: usize = 64;
pub const CHIP8_HEIGHT: usize = 32;
pub const CHIP8_RAM: usize = 4096;

pub type MyResult<T> = Result<T, Box<dyn Error>>;

fn main() -> MyResult<()> {
    Emulator::new().run("roms/PONG", 2)?;
    Ok(())
}
