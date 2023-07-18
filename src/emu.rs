use rand::random;
use std::{println, unimplemented};

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

const START_ADDR: u16 = 0x200;

const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_regs: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    delay_timer: u8,
    sound_timer: u8,
}

impl Emu {
    pub fn new() -> Self {
        let mut emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_regs: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            delay_timer: 0,
            sound_timer: 0,
        };
        emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        emu
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = start + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    pub fn tick(&mut self) {
        let op = self.fetch();
        self.execute(op);
    }

    fn fetch(&mut self) -> u16 {
        let byte1 = self.ram[self.pc as usize];
        let byte2 = self.ram[self.pc as usize + 1];
        let op: u16 = ((byte1 as u16) << 8) | (byte2 as u16);
        self.pc += 2;
        op
    }

    fn execute(&mut self, op: u16) {
        let d1 = (op & 0xF000) >> 12;
        let d2 = (op & 0x0F00) >> 8;
        let d3 = (op & 0x00F0) >> 4;
        let d4 = op & 0x000F;

        match (d1, d2, d3, d4) {
            // NOP
            (0, 0, 0, 0) => return,
            // CLS
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }
            // 00EE - return from subroutine
            (0, 0, 0xE, 0xE) => {
                self.pc = self.pop();
            }
            // 1NNN - jump
            (1, _, _, _) => {
                self.pc = op & 0xFFF;
            }
            // 2NNN - call NNN
            (2, _, _, _) => {
                self.push(self.pc);
                self.pc = op & 0xFFF;
            }
            // SKIP if VX == NN
            (3, _, _, _) => {
                let vx = d2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_regs[vx] == nn {
                    self.pc += 2;
                }
            }
            // SKIP if VX != NN
            (4, _, _, _) => {
                let vx = d2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_regs[vx] != nn {
                    self.pc += 2;
                }
            }
            // SKIP if VX == VY
            (5, _, _, 0) => {
                let vx = d2 as usize;
                let vy = d3 as usize;
                if self.v_regs[vx] == self.v_regs[vy] {
                    self.pc += 2;
                }
            }
            // VX = NN
            (6, _, _, _) => {
                let vx = d2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_regs[vx] = nn;
            }
            // VX += NN
            (7, _, _, _) => {
                let vx = d2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_regs[vx] = self.v_regs[vx].wrapping_add(nn);
            }
            // VX = VY
            (8, _, _, 0) => {
                let vx = d2 as usize;
                let vy = d3 as usize;
                self.v_regs[vx] = self.v_regs[vy];
            }
            // VX |= VY
            (8, _, _, 1) => {
                let vx = d2 as usize;
                let vy = d3 as usize;
                self.v_regs[vx] |= self.v_regs[vy];
            }
            // VX &= VY
            (8, _, _, 2) => {
                let vx = d2 as usize;
                let vy = d3 as usize;
                self.v_regs[vx] &= self.v_regs[vy];
            }
            // VX ^= VY
            (8, _, _, 3) => {
                let vx = d2 as usize;
                let vy = d3 as usize;
                self.v_regs[vx] ^= self.v_regs[vy];
            }
            // VX += VY
            (8, _, _, 4) => {
                let vx = d2 as usize;
                let vy = d3 as usize;
                let (sum, overflow) = self.v_regs[vx].overflowing_add(self.v_regs[vy]);
                self.v_regs[vx] = sum;
                self.v_regs[0xF] = if overflow { 1 } else { 0 };
            }
            // VX -= VY
            (8, _, _, 5) => {
                let vx = d2 as usize;
                let vy = d3 as usize;
                let (sub, overflow) = self.v_regs[vx].overflowing_sub(self.v_regs[vy]);
                self.v_regs[vx] = sub;
                self.v_regs[0xF] = if overflow { 0 } else { 1 };
            }
            // VX >>= 1
            (8, _, _, 6) => {
                let vx = d2 as usize;
                let lsb = self.v_regs[vx] & 1;
                self.v_regs[vx] >>= 1;
                self.v_regs[0xF] = lsb;
            }
            // VX = VY - VX
            (8, _, _, 7) => {
                let vx = d2 as usize;
                let vy = d3 as usize;
                let (sub, overflow) = self.v_regs[vy].overflowing_sub(self.v_regs[vx]);
                self.v_regs[vx] = sub;
                self.v_regs[0xF] = if overflow { 0 } else { 1 };
            }
            // VX <<= 1
            (8, _, _, 0xE) => {
                let vx = d2 as usize;
                let msb = (self.v_regs[vx] >> 7) & 1;
                self.v_regs[vx] <<= 1;
                self.v_regs[0xF] = msb;
            }
            // SKIP if VX != VY
            (9, _, _, 0) => {
                let vx = d2 as usize;
                let vy = d3 as usize;
                if self.v_regs[vx] != self.v_regs[vy] {
                    self.pc += 2;
                }
            }
            // SET I reg to NNN
            (0xA, _, _, _) => {
                self.i_reg = op & 0xFFF;
            }
            // JUMP to V0 + NNN
            (0xB, _, _, _) => {
                self.pc = (self.v_regs[0] as u16) + (op & 0xFFF);
            }
            // VX = rand() & NN
            (0xC, _, _, _) => {
                let vx = d2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_regs[vx] = random::<u8>() & nn;
            }
            // Draw sprite
            (0xD, _, _, _) => {
                let vx = d2 as usize;
                let vy = d3 as usize;
                let sprite_height = d4;

                let (x_coord, y_coord) = (self.v_regs[vx] as u16, self.v_regs[vy] as u16);

                let mut flipped = false;
                for y_line in 0..sprite_height {
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];
                    for x_line in 0..8 {
                        let mask = 0b1000_0000 >> x_line;
                        let pixel = pixels & mask;
                        if pixel != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;
                            let idx = x + SCREEN_WIDTH * y;
                            flipped |= self.screen[idx]; // mark flipped if not already
                            self.screen[idx] ^= true;
                        }
                    }
                }

                if flipped {
                    self.v_regs[0xF] = 1;
                } else {
                    self.v_regs[0xF] = 0;
                }
            }
            // Skip if key is pressed
            (0xE, _, 9, 0xE) => {
                let x = d2 as usize;
                let vx = self.v_regs[x];
                if self.keys[vx as usize] {
                    self.pc += 2;
                }
            }
            // Skip if key is not pressed
            (0xE, _, 0xA, 1) => {
                let x = d2 as usize;
                let vx = self.v_regs[x];
                if !self.keys[vx as usize] {
                    self.pc += 2;
                }
            }
            // Store delay timer to vx
            (0xF, _, 0, 7) => {
                let x = d2 as usize;
                self.v_regs[x] = self.delay_timer;
            }
            // wait for key press
            (0xF, _, 0, 0xA) => {
                let x = d2 as usize;
                let mut pressed = false;
                for idx in 0..self.keys.len() {
                    if self.keys[idx] {
                        self.v_regs[x] = idx as u8;
                        pressed = true;
                        break;
                    }
                }
                // if not presssed - repeat the op
                if !pressed {
                    self.pc -= 2;
                }
            }
            // Get delay timer from VX
            (0xF, _, 1, 5) => {
                let x = d2 as usize;
                self.delay_timer = self.v_regs[x];
            }
            // Get sound timer from VX
            (0xF, _, 1, 8) => {
                let x = d2 as usize;
                self.sound_timer = self.v_regs[x];
            }
            // I += VX
            (0xF, _, 1, 0xE) => {
                let x = d2 as usize;
                self.i_reg = self.i_reg.wrapping_add(self.v_regs[x] as u16);
            }
            // I = character from font
            (0xF, _, 2, 9) => {
                let x = d2 as usize;
                let c = self.v_regs[x] as u16;
                self.i_reg = c * 5; // each char takes 5 bytes
            }
            // BCD
            (0xF, _, 3, 3) => {
                let x = d2 as usize;
                let vx = self.v_regs[x] as f32;

                let hundreds = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0) as u8;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[self.i_reg as usize + 1] = tens;
                self.ram[self.i_reg as usize + 2] = ones;
            }
            // Store V to RAM
            (0xF, _, 5, 5) => {
                let x = d2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.ram[i + idx] = self.v_regs[idx];
                }
            }
            // Load V from RAM
            (0xF, _, 6, 5) => {
                let x = d2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.v_regs[idx] = self.ram[i + idx]
                }
            }
            (_, _, _, _) => {
                unimplemented!("Unimplemented op code {:X}|{:X}|{:X}|{:X}", d1, d2, d3, d4)
            }
        }
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("BEEP")
            }

            self.sound_timer -= 1;
        }
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }
}
