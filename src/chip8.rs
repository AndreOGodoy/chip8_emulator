use std::error::Error;
use std::fs::read;
use std::path::Path;

use rand::Rng;

const PROGRAM_START: usize = 0x200;
const WINDOW_SIZE: (usize, usize) = (64, 32);

const FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0,
    0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80,
    0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80,
    0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];
pub struct Chip8 {
    opcode: u16,

    memory: [u8; 4096],

    v: [u8; 16],

    i: u16,

    pc: usize,

    pub display: [u8; WINDOW_SIZE.0 * WINDOW_SIZE.1],

    delay_timer: u8,
    sound_timer: u8,

    stack: [u16; 16],

    stack_pointer: usize,

    key: [u8; 16],

    pub draw_flag: bool,
}

impl Chip8 {
    pub fn new() -> Self {
        Chip8::default()
    }

    pub fn default() -> Self {
        Chip8 {
            pc: PROGRAM_START,
            opcode: 0,
            i: 0,
            stack: [0; 16],
            stack_pointer: 0,
            memory: [0; 4096],
            delay_timer: 0,
            sound_timer: 0,
            display: [0; 64 * 32],
            key: [0; 16],
            v: [0; 16],
            draw_flag: false,
        }
    }

    pub fn load_program<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let program_as_binary = read(path)?;
        self.memory[PROGRAM_START..program_as_binary.len() + PROGRAM_START]
            .clone_from_slice(&program_as_binary[..]);

        self.memory[0x50..0x50 + 80].clone_from_slice(&FONT_SET[..]);

        Ok(())
    }

    pub fn emulate_cycle(&mut self) {
        self.opcode = self.fetch_opcode();

        let nibbles = (
            self.opcode & 0xF000,
            self.opcode & 0x0F00,
            self.opcode & 0x00F0,
            self.opcode & 0x000F,
        );

        let f = nibbles.0 >> 12;
        let x = nibbles.1 >> 8;
        let y = nibbles.2 >> 4;
        let n = nibbles.3;
        let kk = self.opcode & 0x00FF;
        let nnn = self.opcode & 0x0FFF;

        println!("Opcode is: {:#0x}", self.opcode);

        self.pc = match (f, x, y, n) {
            (0x0, _, _, 0x0) => self.clear_screen(),
            (0x0, _, _, 0xE) => self.return_from_subroutine(),
            (0x1, _, _, _) => self.jump_to_address(nnn),
            (0x2, _, _, _) => self.call_address(nnn),
            (0x3, _, _, _) => self.skip_if_equal(self.v[x as usize].into(), kk),
            (0x4, _, _, _) => self.skip_if_diff(self.v[x as usize].into(), kk),
            (0x5, _, _, _) => {
                self.skip_if_equal(self.v[x as usize].into(), self.v[y as usize].into())
            }
            (0x6, _, _, _) => self.insert_on_register(x, kk),
            (0x7, _, _, _) => self.add_to_register(x, kk),
            (0x8, _, _, 0x0) => self.insert_on_register(x, self.v[y as usize].into()),
            (0x8, _, _, 0x1) => self.register_or(x, y),
            (0x8, _, _, 0x2) => self.register_and(x, y),
            (0x8, _, _, 0x3) => self.register_xor(x, y),
            (0x8, _, _, 0x4) => self.register_carry_add(x, y),
            (0x8, _, _, 0x5) => self.register_borrow_sub(x, y),
            (0x8, _, _, 0x6) => self.register_shr_1(x),
            (0x8, _, _, 0x7) => self.register_sub_rev(x, y),
            (0x8, _, _, 0xE) => self.register_shl_1(x),
            (0x9, _, _, 0x0) => {
                self.skip_if_diff(self.v[x as usize] as u16, self.v[y as usize] as u16)
            }
            (0xA, _, _, _) => self.insert_on_register(self.i, nnn),
            (0xB, _, _, _) => self.jump_to_address(nnn + self.v[0x000] as u16),
            (0xC, _, _, _) => self.register_random_end(x, kk),
            (0xD, _, _, _) => self.draw_sprite(x, y, n),
            (0xE, _, _, 0xE) => self.skip_if_key_pressed(x),
            (0xE, _, _, 0x1) => self.skip_if_key_not_pressed(x),
            (0xF, _, 0x0, 0x7) => self.insert_on_register(x, self.delay_timer as u16),
            (0xF, _, 0x0, 0xA) => self.wait_for_key_press(x),
            (0xF, _, 0x1, 0x5) => self.set_delay_timer(x),
            (0xF, _, 0x1, 0x8) => self.set_sound_timer(x),
            (0xF, _, 0x1, 0xE) => self.index_add(x),
            (0xF, _, 0x2, 0x9) => self.set_index_sprite_location(x),
            (0xF, _, 0x3, 0x3) => self.memory_store_bcd(x),
            (0xF, _, 0x5, 0x5) => self.read_registers(x),
            (0xF, _, 0x6, 0x5) => self.store_registers(x),
            _ => {
                println!("Unknown opcode: {:#0x}", self.opcode);
                self.pc
            }
        };

        match (self.delay_timer, self.sound_timer) {
            (a, _) if a > 0 => self.delay_timer -= 1,
            (_, b) if b > 0 => {
                if b == 1 {
                    println!("BEEP!");
                }
                self.sound_timer -= 1;
            }
            _ => (),
        }
    }

    fn clear_screen(&mut self) -> usize {
        self.display.iter_mut().for_each(|byte| *byte = 0);
        self.draw_flag = true;
        self.pc + 2
    }

    fn return_from_subroutine(&mut self) -> usize {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer].into()
    }

    fn jump_to_address(&mut self, address: u16) -> usize {
        address.into()
    }

    fn call_address(&mut self, address: u16) -> usize {
        self.stack[self.stack_pointer] = self.pc as u16;
        self.stack_pointer += 1;
        address.into()
    }

    fn skip_if_equal(&mut self, a: u16, b: u16) -> usize {
        if a == b {
            self.pc + 4
        } else {
            self.pc + 2
        }
    }

    fn skip_if_diff(&mut self, a: u16, b: u16) -> usize {
        if a != b {
            self.pc + 4
        } else {
            self.pc + 2
        }
    }

    fn insert_on_register(&mut self, x: u16, kk: u16) -> usize {
        self.v[x as usize] = kk as u8;
        self.pc + 2
    }

    fn add_to_register(&mut self, x: u16, kk: u16) -> usize {
        self.v[x as usize] = (self.v[x as usize] as u16 + kk as u16) as u8;
        self.pc + 2
    }

    fn register_or(&mut self, x: u16, y: u16) -> usize {
        self.v[x as usize] |= self.v[y as usize];
        self.pc + 2
    }

    fn register_and(&mut self, x: u16, y: u16) -> usize {
        self.v[x as usize] &= self.v[y as usize];
        self.pc + 2
    }

    fn register_xor(&mut self, x: u16, y: u16) -> usize {
        self.v[x as usize] ^= self.v[y as usize];
        self.pc + 2
    }

    fn register_carry_add(&mut self, x: u16, y: u16) -> usize {
        let sum = self.v[x as usize] as u16 + self.v[y as usize] as u16;
        self.v[0xF] = (sum > 0xFF) as u8;
        self.v[x as usize] = (sum & 0x00FF) as u8;
        self.pc + 2
    }

    fn register_borrow_sub(&mut self, x: u16, y: u16) -> usize {
        let sub = self.v[x as usize] + self.v[y as usize];
        self.v[0xF] = (self.v[x as usize] > self.v[y as usize]) as u8;
        self.v[x as usize] = sub;
        self.pc + 2
    }

    fn register_shr_1(&mut self, x: u16) -> usize {
        self.v[0xF] = self.v[x as usize] & 0x000F;
        self.v[x as usize] >>= 1;
        self.pc + 2
    }

    fn register_sub_rev(&mut self, x: u16, y: u16) -> usize {
        let sub = self.v[y as usize] - self.v[x as usize];
        self.v[0xF] = (self.v[y as usize] > self.v[x as usize]) as u8;
        self.v[x as usize] = sub;
        self.pc + 2
    }

    fn register_shl_1(&mut self, x: u16) -> usize {
        self.v[0xF] = (self.v[x as usize] >> 7) as u8;
        self.v[x as usize] <<= 1;
        self.pc + 2
    }

    fn register_random_end(&mut self, x: u16, kk: u16) -> usize {
        let mut rng = rand::thread_rng();
        self.v[x as usize] = (rng.gen_range(0, 255) & kk) as u8;
        self.pc + 2
    }

    fn draw_sprite(&mut self, x: u16, y: u16, n: u16) -> usize {
        self.v[0xF] = 0;
        for byte in 0..n as usize {
            let pos = self.memory[self.i as usize + byte];
            for bit in 0..8 as usize {
                if pos & (0x80 >> bit) != 0x0 {
                    let pixel = (self.v[x as usize] as u16
                        + bit as u16
                        + (self.v[y as usize] as u16 + byte as u16)
                        << 6)
                        % 2028;
                    self.v[0xF] |= self.display[pixel as usize] & 1;
                    self.display[pixel as usize] ^= self.display[pixel as usize];
                }
            }
        }
        self.draw_flag = true;
        self.pc + 2
    }

    fn skip_if_key_pressed(&mut self, x: u16) -> usize {
        if self.key[self.v[x as usize] as usize] == 1 {
            self.pc + 4
        } else {
            self.pc + 2
        }
    }

    fn skip_if_key_not_pressed(&mut self, x: u16) -> usize {
        if self.key[self.v[x as usize] as usize] == 0 {
            self.pc + 4
        } else {
            self.pc + 2
        }
    }

    fn wait_for_key_press(&mut self, x: u16) -> usize {
        if let Some(&key_state) = self.key.iter().find(|&&key| key == 1) {
            self.v[x as usize] = key_state;
            self.pc + 2
        } else {
            self.pc
        }
    }

    fn set_delay_timer(&mut self, x: u16) -> usize {
        self.delay_timer = self.v[x as usize];
        self.pc + 2
    }

    fn set_sound_timer(&mut self, x: u16) -> usize {
        self.sound_timer = self.v[x as usize];
        self.pc + 2
    }

    fn index_add(&mut self, x: u16) -> usize {
        self.i += self.v[x as usize] as u16;
        self.pc + 2
    }

    fn set_index_sprite_location(&mut self, x: u16) -> usize {
        self.i = self.v[x as usize] as u16 * 5;
        self.pc + 2
    }

    fn memory_store_bcd(&mut self, x: u16) -> usize {
        self.memory[self.i as usize] = (x / 100) as u8;
        self.memory[self.i as usize + 1] = (x / 10) as u8 % 10;
        self.memory[self.i as usize + 2] = x as u8 % 10;
        self.pc + 2
    }

    fn read_registers(&mut self, x: u16) -> usize {
        for index in 0..=x as usize {
            self.v[index] = self.memory[self.i as usize + index];
        }
        self.pc + 2
    }

    fn store_registers(&mut self, x: u16) -> usize {
        for index in 0..=x as usize {
            self.memory[self.i as usize + index] = self.v[index];
        }
        self.pc + 2
    }

    fn fetch_opcode(&mut self) -> u16 {
        (self.memory[self.pc as usize] as u16) << 8 | self.memory[self.pc as usize + 1] as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chip8_default_init() {
        let chip8_debug = Chip8::new();

        assert_eq!(chip8_debug.pc, 0x200);

        assert_eq!(chip8_debug.stack_pointer, 0);
        assert_eq!(chip8_debug.sound_timer, 0);
        assert_eq!(chip8_debug.delay_timer, 0);
        assert_eq!(chip8_debug.i, 0);
        assert_eq!(chip8_debug.opcode, 0);
        assert_eq!(chip8_debug.draw_flag, false);

        assert!(chip8_debug.memory.iter().all(|&byte| byte == 0));
        assert!(chip8_debug.display.iter().all(|&byte| byte == 0));
        assert!(chip8_debug.key.iter().all(|&byte| byte == 0));
        assert!(chip8_debug.v.iter().all(|&byte| byte == 0));
        assert!(chip8_debug.stack.iter().all(|&byte| byte == 0));
    }

    #[test]
    fn test_load_program_err() {
        let mut chip8_debug = Chip8::new();
        assert!(chip8_debug.load_program("path/do/not/exist").is_err());
    }

    #[test]
    fn test_load_program_ok() {
        let mut chip8_debug = Chip8::new();
        let result = chip8_debug.load_program("roms/pong.rom");

        assert!(result.is_ok());

        let program_as_bytes = read("roms/pong.rom").unwrap();
        let slice_as_vec = chip8_debug.memory[0x200..0x200+program_as_bytes.len()].to_vec();

        assert_eq!(program_as_bytes, slice_as_vec);

        let fontset = vec![
            0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80,
            0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0,
            0x10, 0xF0, 0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90,
            0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0,
            0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0,
            0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
        ];
        let slice_as_vec = chip8_debug.memory[0x50..0x50+80].to_vec();

        assert_eq!(fontset, slice_as_vec);
    }

    #[test]
    fn test_emulate_cycle() {
        let mut chip8_debug = Chip8::new();
        chip8_debug.load_program("roms/pong.rom").unwrap();
        
        let mut old_pc = chip8_debug.pc;
        
        chip8_debug.emulate_cycle();
        assert_eq!(chip8_debug.opcode, 0x6A02);
        assert_eq!(chip8_debug.pc, old_pc + 2);
        
        
        old_pc = chip8_debug.pc;
        chip8_debug.emulate_cycle();
        assert_eq!(chip8_debug.opcode, 0x6B0C);
        assert_eq!(chip8_debug.pc, old_pc + 2);
    }

    #[test]
    fn test_clear_screen() {
        let mut chip8_debug = Chip8::new();
    
        chip8_debug.display.iter_mut().step_by(4).for_each(|byte| *byte += 2);
        assert!(!chip8_debug.display.iter().all(|&byte| byte == 0));

        chip8_debug.clear_screen();
        assert!(chip8_debug.display.iter().all(|&byte| byte == 0));        
    }

    #[test]
    fn test_jump_to_address() {
        let mut chip8_debug = Chip8::new();

        let new_address = 0xFFF;
        chip8_debug.pc = chip8_debug.jump_to_address(0xFFF);
        assert_eq!(chip8_debug.pc, new_address);
        
        let new_address = 0x0000;
        chip8_debug.pc = chip8_debug.jump_to_address(0x0000);
        assert_eq!(chip8_debug.pc, new_address);
    }

    #[test]
    fn test_call_and_return_from_subroutine() {
        let mut chip8_debug = Chip8::new();

        let old_sp = chip8_debug.stack_pointer;
        let old_pc = chip8_debug.pc;

        assert_eq!(chip8_debug.call_address(0xABC), 0xABC);
        assert_eq!(chip8_debug.stack_pointer, old_sp + 1);
        
        let old_sp = chip8_debug.stack_pointer;

        assert_eq!(chip8_debug.return_from_subroutine(), old_pc);
        assert_eq!(chip8_debug.stack_pointer, old_sp - 1);
    }

    #[test]
    fn test_skip_if_equal_and_different() {
        let mut chip8_debug = Chip8::new();

        chip8_debug.pc = chip8_debug.jump_to_address(0x200 + 0xabc);
        
        let old_pc = chip8_debug.pc;
        chip8_debug.pc = chip8_debug.skip_if_equal(0x300, 0x300);
        assert_eq!(chip8_debug.pc, old_pc + 4);
        
        let old_pc = chip8_debug.pc;
        chip8_debug.pc = chip8_debug.skip_if_equal(0x300, 0x200);
        assert_eq!(chip8_debug.pc, old_pc + 2);
        
        let old_pc = chip8_debug.pc;
        chip8_debug.pc = chip8_debug.skip_if_diff(0x300, 0x200);
        assert_eq!(chip8_debug.pc, old_pc + 4);
        
        let old_pc = chip8_debug.pc;
        chip8_debug.pc = chip8_debug.skip_if_diff(0x300, 0x300);
        assert_eq!(chip8_debug.pc, old_pc + 2);
    }

    #[test]
    fn test_register_insert_add_operations() {
        let mut chip8_debug = Chip8::new();

        chip8_debug.insert_on_register(0x05, 0x00FF);
        assert_eq!(chip8_debug.v[5], 255);

        chip8_debug.add_to_register(0x04, 0x00FA);
        assert_eq!(chip8_debug.v[4], 0x00FA);

        chip8_debug.add_to_register(0x04, 0x0001);
        assert_eq!(chip8_debug.v[4], 0x00FB);

        chip8_debug.insert_on_register(0x0000, 0x0001); 
    }
}
