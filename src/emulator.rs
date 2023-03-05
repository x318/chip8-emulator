use std::{thread, time::Duration};

use crate::{chip8::Chip8, console_display::ConsoleDisplay, MyResult};

pub struct Emulator;

impl Emulator {
    pub fn new() -> Self {
        Emulator {}
    }

    pub fn run(&self, rom: &str, tick_rate: u64) -> MyResult<()> {
        let mut cpu = Chip8::new();
        cpu.load_rom(rom)?;

        let mut display = ConsoleDisplay::new()?;

        loop {
            let output = cpu.tick([false; 16]);

            if output.vram_changed {
                display.draw(output.vram)?;
            }

            thread::sleep(Duration::from_millis(tick_rate));
        }
    }
}
