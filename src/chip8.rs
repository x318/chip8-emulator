use crate::fontset::FONTSET;
use crate::CHIP8_HEIGHT;
use crate::CHIP8_RAM;
use crate::CHIP8_WIDTH;
use crate::OPCODE_SIZE;
use rand::prelude::*;
use std::{fs, io};

const START_ADRESS: usize = 0x200;
const FONTSET_START_ADRESS: u16 = 0x50;

pub struct Chip8 {
    registers: [u8; 16],
    ram: [u8; CHIP8_RAM],
    stack: [usize; 16],
    index: usize,
    pc: usize,
    sp: usize,
    delay_timer: u8,
    sound_timer: u8,
    vram: [[u8; CHIP8_WIDTH]; CHIP8_HEIGHT],
    vram_changed: bool,
    keypad: [bool; 16],
    keypad_waiting: bool,
    keypad_register: usize,
}

pub struct OutputState<'a> {
    pub vram: &'a [[u8; CHIP8_WIDTH]; CHIP8_HEIGHT],
    pub vram_changed: bool,
    pub beep: bool,
}

enum ProgramCounter {
    Next,
    Skip,
    Jump(usize),
}

impl ProgramCounter {
    fn skip_if(condition: bool) -> ProgramCounter {
        if condition {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }
}

impl Chip8 {
    pub fn new() -> Self {
        let mut cpu = Chip8 {
            pc: START_ADRESS,
            delay_timer: 0,
            index: 0,
            sound_timer: 0,
            sp: 0,
            registers: [0; 16],
            ram: [0; CHIP8_RAM],
            stack: [0; 16],
            vram: [[0; CHIP8_WIDTH]; CHIP8_HEIGHT],
            vram_changed: false,
            keypad: [false; 16],
            keypad_waiting: false,
            keypad_register: 0,
        };

        for i in 0..FONTSET.len() {
            cpu.ram[FONTSET_START_ADRESS as usize + i] = FONTSET[i];
        }

        cpu
    }

    pub fn load_rom(&mut self, file_path: &str) -> io::Result<()> {
        let buffer = fs::read(file_path)?;

        for i in 0..buffer.len() {
            self.ram[START_ADRESS as usize + i] = buffer[i];
        }

        Ok(())
    }

    pub fn tick(&mut self, keypad: [bool; 16]) -> OutputState {
        self.keypad = keypad;
        self.vram_changed = false;

        if self.keypad_waiting {
            for i in 0..keypad.len() {
                if keypad[i] {
                    self.keypad_waiting = false;
                    self.registers[self.keypad_register] = i as u8;
                    break;
                }
            }
        } else {
            if self.delay_timer > 0 {
                self.delay_timer -= 1
            }
            if self.sound_timer > 0 {
                self.sound_timer -= 1
            }
            let opcode = self.get_opcode();
            self.run_opcode(opcode);
        }

        OutputState {
            vram: &self.vram,
            vram_changed: self.vram_changed,
            beep: self.sound_timer > 0,
        }
    }

    fn get_opcode(&self) -> u16 {
        (self.ram[self.pc] as u16) << 8 | (self.ram[self.pc + 1] as u16)
    }

    fn run_opcode(&mut self, opcode: u16) {
        let nibbles = (
            (opcode & 0xF000) >> 12 as u8,
            (opcode & 0x0F00) >> 8 as u8,
            (opcode & 0x00F0) >> 4 as u8,
            (opcode & 0x000F) as u8,
        );
        let nnn = (opcode & 0x0FFF) as usize;
        let kk = (opcode & 0x00FF) as u8;
        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;
        let n = nibbles.3 as usize;

        let pc_change = match nibbles {
            (0x00, 0x00, 0x0e, 0x00) => self.op_00e0(),
            (0x00, 0x00, 0x0e, 0x0e) => self.op_00ee(),
            (0x01, _, _, _) => self.op_1nnn(nnn),
            (0x02, _, _, _) => self.op_2nnn(nnn),
            (0x03, _, _, _) => self.op_3xkk(x, kk),
            (0x04, _, _, _) => self.op_4xkk(x, kk),
            (0x05, _, _, 0x00) => self.op_5xy0(x, y),
            (0x06, _, _, _) => self.op_6xkk(x, kk),
            (0x07, _, _, _) => self.op_7xkk(x, kk),
            (0x08, _, _, 0x00) => self.op_8xy0(x, y),
            (0x08, _, _, 0x01) => self.op_8xy1(x, y),
            (0x08, _, _, 0x02) => self.op_8xy2(x, y),
            (0x08, _, _, 0x03) => self.op_8xy3(x, y),
            (0x08, _, _, 0x04) => self.op_8xy4(x, y),
            (0x08, _, _, 0x05) => self.op_8xy5(x, y),
            (0x08, _, _, 0x06) => self.op_8x06(x),
            (0x08, _, _, 0x07) => self.op_8xy7(x, y),
            (0x08, _, _, 0x0e) => self.op_8x0e(x),
            (0x09, _, _, 0x00) => self.op_9xy0(x, y),
            (0x0a, _, _, _) => self.op_annn(nnn),
            (0x0b, _, _, _) => self.op_bnnn(nnn),
            (0x0c, _, _, _) => self.op_cxkk(x, kk),
            (0x0d, _, _, _) => self.op_dxyn(x, y, n),
            (0x0e, _, 0x09, 0x0e) => self.op_ex9e(x),
            (0x0e, _, 0x0a, 0x01) => self.op_exa1(x),
            (0x0f, _, 0x00, 0x07) => self.op_fx07(x),
            (0x0f, _, 0x00, 0x0a) => self.op_fx0a(x),
            (0x0f, _, 0x01, 0x05) => self.op_fx15(x),
            (0x0f, _, 0x01, 0x08) => self.op_fx18(x),
            (0x0f, _, 0x01, 0x0e) => self.op_fx1e(x),
            (0x0f, _, 0x02, 0x09) => self.op_fx29(x),
            (0x0f, _, 0x03, 0x03) => self.op_fx33(x),
            (0x0f, _, 0x05, 0x05) => self.op_fx55(x),
            (0x0f, _, 0x06, 0x05) => self.op_fx65(x),
            _ => ProgramCounter::Next,
        };

        match pc_change {
            ProgramCounter::Next => self.pc += OPCODE_SIZE,
            ProgramCounter::Skip => self.pc += 2 * OPCODE_SIZE,
            ProgramCounter::Jump(addr) => self.pc = addr,
        }
    }
}

// operations
impl Chip8 {
    fn op_00e0(&mut self) -> ProgramCounter {
        for y in 0..CHIP8_HEIGHT {
            for x in 0..CHIP8_WIDTH {
                self.vram[y][x] = 0;
            }
        }
        self.vram_changed = true;
        ProgramCounter::Next
    }

    fn op_00ee(&mut self) -> ProgramCounter {
        self.sp -= 1;
        ProgramCounter::Jump(self.stack[self.sp])
    }

    fn op_1nnn(&mut self, nnn: usize) -> ProgramCounter {
        ProgramCounter::Jump(nnn)
    }

    fn op_2nnn(&mut self, nnn: usize) -> ProgramCounter {
        self.stack[self.sp] = self.pc + OPCODE_SIZE;
        self.sp += 1;
        ProgramCounter::Jump(nnn)
    }

    fn op_3xkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        ProgramCounter::skip_if(self.registers[x] == kk)
    }

    fn op_4xkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        ProgramCounter::skip_if(self.registers[x] != kk)
    }

    fn op_5xy0(&mut self, x: usize, y: usize) -> ProgramCounter {
        ProgramCounter::skip_if(self.registers[x] == self.registers[y])
    }

    fn op_6xkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        self.registers[x] = kk;
        ProgramCounter::Next
    }

    fn op_7xkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        let vx = self.registers[x] as u16;
        let val = kk as u16;
        let result = vx + val;
        self.registers[x] = result as u8;
        ProgramCounter::Next
    }

    fn op_8xy0(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.registers[x] = self.registers[y];
        ProgramCounter::Next
    }

    fn op_8xy1(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.registers[x] |= self.registers[y];
        ProgramCounter::Next
    }

    fn op_8xy2(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.registers[x] &= self.registers[y];
        ProgramCounter::Next
    }

    fn op_8xy3(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.registers[x] ^= self.registers[y];
        ProgramCounter::Next
    }

    fn op_8xy4(&mut self, x: usize, y: usize) -> ProgramCounter {
        let vx = self.registers[x] as u16;
        let vy = self.registers[y] as u16;
        let result = vx + vy;
        self.registers[x] = result as u8;
        self.registers[0x0f] = if result > 0xFF { 1 } else { 0 };
        ProgramCounter::Next
    }

    fn op_8xy5(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.registers[0x0f] = if self.registers[x] > self.registers[y] {
            1
        } else {
            0
        };
        self.registers[x] = self.registers[x].wrapping_sub(self.registers[y]);
        ProgramCounter::Next
    }

    fn op_8x06(&mut self, x: usize) -> ProgramCounter {
        self.registers[0x0f] = self.registers[x] & 1;
        self.registers[x] >>= 1;
        ProgramCounter::Next
    }

    fn op_8xy7(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.registers[0x0f] = if self.registers[y] > self.registers[x] {
            1
        } else {
            0
        };
        self.registers[x] = self.registers[y].wrapping_sub(self.registers[x]);
        ProgramCounter::Next
    }

    fn op_8x0e(&mut self, x: usize) -> ProgramCounter {
        self.registers[0x0f] = (self.registers[x] & 0b10000000) >> 7;
        self.registers[x] <<= 1;
        ProgramCounter::Next
    }

    fn op_9xy0(&mut self, x: usize, y: usize) -> ProgramCounter {
        ProgramCounter::skip_if(self.registers[x] != self.registers[y])
    }

    fn op_annn(&mut self, nnn: usize) -> ProgramCounter {
        self.index = nnn;
        ProgramCounter::Next
    }

    fn op_bnnn(&mut self, nnn: usize) -> ProgramCounter {
        ProgramCounter::Jump((self.registers[0] as usize) + nnn)
    }

    fn op_cxkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        let mut rng = rand::thread_rng();
        self.registers[x] = rng.gen::<u8>() & kk;
        ProgramCounter::Next
    }

    fn op_dxyn(&mut self, x: usize, y: usize, n: usize) -> ProgramCounter {
        self.registers[0x0f] = 0;
        for byte in 0..n {
            let y = (self.registers[y] as usize + byte) % CHIP8_HEIGHT;
            for bit in 0..8 {
                let x = (self.registers[x] as usize + bit) % CHIP8_WIDTH;
                let color = (self.ram[self.index + byte] >> (7 - bit)) & 1;
                self.registers[0x0f] |= color & self.vram[y][x];
                self.vram[y][x] ^= color;
            }
        }
        self.vram_changed = true;
        ProgramCounter::Next
    }

    fn op_ex9e(&mut self, x: usize) -> ProgramCounter {
        ProgramCounter::skip_if(self.keypad[self.registers[x] as usize])
    }

    fn op_exa1(&mut self, x: usize) -> ProgramCounter {
        ProgramCounter::skip_if(!self.keypad[self.registers[x] as usize])
    }

    fn op_fx07(&mut self, x: usize) -> ProgramCounter {
        self.registers[x] = self.delay_timer;
        ProgramCounter::Next
    }

    fn op_fx0a(&mut self, x: usize) -> ProgramCounter {
        self.keypad_waiting = true;
        self.keypad_register = x;
        ProgramCounter::Next
    }

    fn op_fx15(&mut self, x: usize) -> ProgramCounter {
        self.delay_timer = self.registers[x];
        ProgramCounter::Next
    }

    fn op_fx18(&mut self, x: usize) -> ProgramCounter {
        self.sound_timer = self.registers[x];
        ProgramCounter::Next
    }

    fn op_fx1e(&mut self, x: usize) -> ProgramCounter {
        self.index += self.registers[x] as usize;
        self.registers[0x0f] = if self.index > 0x0F00 { 1 } else { 0 };
        ProgramCounter::Next
    }

    fn op_fx29(&mut self, x: usize) -> ProgramCounter {
        self.index = (self.registers[x] as usize) * 5;
        ProgramCounter::Next
    }

    fn op_fx33(&mut self, x: usize) -> ProgramCounter {
        self.ram[self.index] = self.registers[x] / 100;
        self.ram[self.index + 1] = (self.registers[x] % 100) / 10;
        self.ram[self.index + 2] = self.registers[x] % 10;
        ProgramCounter::Next
    }

    fn op_fx55(&mut self, x: usize) -> ProgramCounter {
        for i in 0..x + 1 {
            self.ram[self.index + i] = self.registers[i];
        }
        ProgramCounter::Next
    }

    fn op_fx65(&mut self, x: usize) -> ProgramCounter {
        for i in 0..x + 1 {
            self.registers[i] = self.ram[self.index + i];
        }
        ProgramCounter::Next
    }
}
