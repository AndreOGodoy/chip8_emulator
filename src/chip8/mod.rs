use std::error::Error;
use std::fs::read;
use std::path::Path;

mod emulated_cpu;
use emulated_cpu::EmulatedCpu;

mod emulated_memory;
use emulated_memory::EmulatedMemory;

mod emulated_keypad;
use emulated_keypad::EmulatedKeypad;

mod emulated_graphics;
use emulated_graphics::EmulatedGraphics;

mod emulated_timers;
use emulated_timers::EmulatedTimers;

const PROGRAM_START: usize = 0x200;

pub enum ExecutionState {
    Hold,
    Skip,
    Continue,
    ReturnTo(usize),
    JumpTo(usize),
}

pub struct Chip8 {
    memory: EmulatedMemory,
    cpu: EmulatedCpu,
    keypad: EmulatedKeypad,
    timers: EmulatedTimers,
    
    pub graphics: EmulatedGraphics,
    pub draw_flag: bool,

    pc: usize,
}

impl Chip8 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn default() -> Self {
        Chip8 {
            pc: PROGRAM_START,
            memory: EmulatedMemory::new(),
            timers: EmulatedTimers::new(),
            draw_flag: false,
            cpu: EmulatedCpu::new(),
            keypad: EmulatedKeypad::new(),
            graphics: EmulatedGraphics::new(),
        }
    }

    pub fn load_program<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let program_as_binary = read(path)?;

        self.memory.mem_array[PROGRAM_START..program_as_binary.len() + PROGRAM_START]
            .clone_from_slice(&program_as_binary[..]);

        Ok(())
    }

    pub fn emulate_cycle(&mut self) {
        let opcode = self.fetch_opcode();

        println!("Opcode is: {:#0x}", opcode);

        let state = self.execute_opcode(opcode);

        println!("Pc value is: {}", self.pc);
        self.pc = match state {
            ExecutionState::Hold => self.pc,
            ExecutionState::Skip => self.pc + 4,
            ExecutionState::Continue => self.pc + 2,
            ExecutionState::JumpTo(address) => address,
            ExecutionState::ReturnTo(address) => address + 2,
        };

        self.timers.tick();
    }

    fn skip_if_equal<T: PartialEq>(&mut self, a: T, b: T) -> ExecutionState {
        if a == b {
            ExecutionState::Skip
        } else {
            ExecutionState::Continue
        }
    }

    fn skip_if_diff<T: PartialEq>(&mut self, a: T, b: T) -> ExecutionState {
        if a != b {
            ExecutionState::Skip
        } else {
            ExecutionState::Continue
        }
    }

    fn draw(&mut self, vx: u8, vy: u8, n: u8) -> ExecutionState {
        let sprite = &self.memory.mem_array[self.memory.index..self.memory.index + n as usize];
        self.cpu.register[0xF] = self.graphics.draw_sprite(vx, vy, n, sprite) as u8;

        self.draw_flag = true;

        ExecutionState::Continue
    }

    fn set_index_sprite_location(&mut self, vx: u8) -> ExecutionState {
        self.memory.index = vx as usize * 5;

        ExecutionState::Continue
    }

    fn read_registers(&mut self, x: u8) -> ExecutionState {
        for index in 0..=x as usize {
            self.cpu.register[index] = self.memory.mem_array[self.memory.index as usize + index];
        }

        ExecutionState::Continue
    }

    fn store_registers(&mut self, x: u8) -> ExecutionState {
        for index in 0..=x as usize {
            self.memory.mem_array[self.memory.index as usize + index] = self.cpu.register[index];
        }

        ExecutionState::Continue
    }

    fn fetch_opcode(&mut self) -> u16 {
        (self.memory.mem_array[self.pc as usize] as u16) << 8
            | self.memory.mem_array[self.pc as usize + 1] as u16
    }

    pub fn press_key(&mut self, key: u8) {
        self.keypad.press_key(key);
    }

    pub fn release_key(&mut self, key: u8) {
        self.keypad.release_key(key);
    }

    fn execute_opcode(&mut self, opcode: u16) -> ExecutionState {
        let f = ((opcode & 0xF000) >> 12) as u8;
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;
        let n = (opcode & 0x000F) as u8;
        let kk = (opcode & 0x00FF) as u8;
        let nnn = (opcode & 0x0FFF) as usize;

        let vx = self.cpu.register[x as usize];
        let vy = self.cpu.register[y as usize];

        match (f, x, y, n) {
            (0x0, 0x0, 0xE, 0xE) => self.memory.return_from_subroutine(),
            (0x0, _, 0xE, 0x0) => self.graphics.clear_display(),
            (0x0, _, _, _) => ExecutionState::Continue,
            (0x1, _, _, _) => self.memory.jump_to_address(nnn),
            (0x2, _, _, _) => self.memory.call_subroutine(nnn, self.pc),
            (0x3, _, _, _) => self.skip_if_equal(vx, kk),
            (0x4, _, _, _) => self.skip_if_diff(vx, kk),
            (0x5, _, _, _) => self.skip_if_equal(vx, vy),
            (0x6, _, _, _) => self.cpu.set_register(x, kk),
            (0x7, _, _, _) => self.cpu.register_add_value(x, kk),
            (0x8, _, _, 0) => self.cpu.set_register(x, vy),
            (0x8, _, _, 1) => self.cpu.register_or(x, y),
            (0x8, _, _, 2) => self.cpu.register_and(x, y),
            (0x8, _, _, 3) => self.cpu.register_xor(x, y),
            (0x8, _, _, 4) => self.cpu.register_carry_add(x, y),
            (0x8, _, _, 5) => self.cpu.register_borrow_sub(x, y),
            (0x8, _, _, 6) => self.cpu.register_shr(x),
            (0x8, _, _, 7) => self.cpu.register_borrow_sub_rev(x, y),
            (0x8, _, _, 0xE) => self.cpu.register_shl(x),
            (0x9, _, _, _) => self.skip_if_diff(vx, vy),
            (0xA, _, _, _) => self.memory.set_index(nnn),
            (0xB, _, _, _) => self
                .memory
                .jump_to_address(nnn + self.cpu.register[0x0] as usize),
            (0xC, _, _, _) => self.cpu.register_random_and(x, kk),
            (0xD, _, _, _) => self.draw(vx, vy, n),
            (0xE, _, _, 0xE) => self.keypad.skip_if_pressed(vx),
            (0xE, _, _, 0x1) => self.keypad.skip_if_released(vx),
            (0xF, _, _, 0x7) => self.cpu.set_register(x, self.timers.get_delay_timer()),
            (0xF, _, _, 0xA) => self.keypad.wait_for_key(&mut self.cpu.register[x as usize]),
            (0xF, _, 0x1, 0x5) => self.timers.set_delay_timer(vx),
            (0xF, _, _, 0x8) => self.timers.set_sound_timer(vx),
            (0xF, _, _, 0xE) => self.memory.index_add(x),
            (0xF, _, _, 0x9) => self.set_index_sprite_location(vx),
            (0xF, _, _, 0x3) => self.memory.memory_store_bcd(vx),
            (0xF, _, 0x5, 0x5) => self.store_registers(x),
            (0xF, _, 0x6, 0x5) => self.read_registers(x),
            _ => panic!("Uknown opcode: {}", opcode),
        }
    }
}
