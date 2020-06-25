use std::error::Error;
use std::fs::read;
use std::path::Path;
use std::primitive::u8;

use rand::Rng;

const PROGRAM_START: usize = 0x200;

pub struct Chip8 {
    //Stores the current 2-byte opcode
    opcode: u16,

    //Emulates the 4K total memory
    memory: [u8; 4096],

    //Emulates the 16 2-byte general purpose registers. VF = v[15] shoud not be used. Its a flag for some instructions
    v: [u8; 16],

    //Index register that can have a value from 0x000 to 0xFFF. Store some memory address
    i: u16,

    //Program counter that can have a value from 0x000 to 0xFFF. Shows where the program currently is
    pc: u16,

    //0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
    //0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
    //0x200-0xFFF - Program ROM and work RAM

    //Black and white screen of 64 x 32 = 2048 pixels
    gfx: [u8; 64 * 32],

    //Special registers that decrementes by one at a rate of 60Hz when above 0
    delay_timer: u8,
    sound_timer: u8,

    //Stack to help jumping to subroutines. Not originally specificated
    stack: [u16; 16],

    //Holds a position on the stack
    stack_pointer: u16,

    //Keypad. Each position of the array holds one key's state
    key: [u8; 16],
}

impl Chip8 {
    pub fn new() -> Self {
        Chip8::default()
    }

    pub fn default() -> Self {
        Chip8 {
            pc: PROGRAM_START as u16,
            opcode: 0,
            i: 0,
            stack: [0; 16],
            stack_pointer: 0,
            memory: [0; 4096],
            delay_timer: 0,
            sound_timer: 0,
            gfx: [0; 64 * 32],
            key: [0; 16],
            v: [0; 16],
        }
    }

    pub fn load_program<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let program_as_binary = read(path)?;
        self.memory[PROGRAM_START..program_as_binary.len() + PROGRAM_START]
            .clone_from_slice(&program_as_binary[..]);
        Ok(())
    }

    pub fn emulate_cycle(&mut self) {
        self.opcode = self.fetch_opcode();

        println!("Opcode is: {:#0x}", self.opcode);

        match self.opcode & 0xF000 {
            //Search for the first 4 bis
            0x0000 => match self.opcode & 0x00FF {
                0x00E0 => {
                    self.gfx.iter_mut().for_each(|a| *a = 0);
                    self.increment_program_counter();
                }
                0x00EE => {
                    self.pc = self.stack[self.stack_pointer as usize];
                    self.stack_pointer -= 1;
                }
                _ => println!("Unknown Opcode: {}", self.opcode),
            },

            0xA000 => {
                self.i = self.opcode & 0x0FFF;
                self.increment_program_counter()
            }

            0xB000 => {
                self.pc = (self.opcode & 0x0FFF) + self.v[0x000 as u8 as usize] as u16;
            }

            0xC000 => {
                let mut rng = rand::thread_rng();
                let random: u8 = rng.gen_range(0, 255);
                self.v[(self.opcode & 0x0F00) as usize] = random & (self.opcode & 0x00FF) as u8;
                self.increment_program_counter();
            }

            0xD000 => {
                self.v[0xF] = 0;
                let y = self.v[((self.opcode & 0x00F0) >> 4) as usize] as usize;
                for byte in 0..(self.opcode & 0x000F) as usize {
                    let pixel = self.memory[self.i as usize + byte];
                    for bit in 0..8 as usize {
                        let x = self.v[(self.opcode & 0x0F00 >> 8) as usize] as usize;
                        self.v[0xF] = self.gfx[(x + bit + (y + byte) * 64) as usize] & 1;
                        self.gfx[(x + bit + (y + byte) * 64) as usize] ^= 1;
                    }
                }
                self.increment_program_counter();
            }

            0xE000 => match self.opcode & 0x00FF {
                0x009E => {
                    let x = self.opcode & 0x0F00;
                    if self.key[self.v[x as usize] as usize] == 0xFF {
                        self.skip_next_instruction();
                    } else {
                        self.increment_program_counter();
                    }
                }
                0x00A1 => {
                    let x = self.opcode & 0x0F00;
                    if self.key[self.v[x as usize] as usize] == 0x00 {
                        self.skip_next_instruction();
                    } else {
                        self.increment_program_counter();
                    }
                }
                _ => println!("Unknown opcode: {}", self.opcode)
            },

            0xF000 => match self.opcode & 0x00FF {
                0x0007 => {
                    self.v[(self.opcode & 0x0F00) as u8 as usize] = self.delay_timer;
                    self.increment_program_counter();
                }
                0x000A => {
                    unimplemented!();
                }
                0x0015 => {
                    self.delay_timer = self.v[(self.opcode & 0x0F00) as u8 as usize];
                    self.increment_program_counter();
                }
                0x0018 => {
                    self.sound_timer = self.v[(self.opcode & 0x0F00) as u8 as usize];
                    self.increment_program_counter();
                }
                0x001E => {
                    self.i += self.v[(self.opcode & 0x0F00) as u8 as usize] as u16;
                    self.increment_program_counter();
                }
                0x0029 => {
                    self.i = (self.v[((self.opcode & 0x0F00) >> 8 ) as usize] * 5) as u16;
                    self.increment_program_counter();
                }
                0x0033 => {
                    let x = self.opcode & 0x0F00;
                    self.memory[self.i as usize] = (x/ 100) as u8;
                    self.memory[self.i as usize + 1] = ((x % 100) / 10) as u8;
                    self.memory[self.i as usize + 2] = (x % 10) as u8;
                    self.increment_program_counter();
                }
                0x0055 => {
                    let x = (self.opcode & 0x0F00) as usize;
                    self.memory[self.i as usize..].clone_from_slice(&self.v[0x0000 as usize..x]);
                    self.increment_program_counter();
                }
                0x0065 => {
                    let x = (self.opcode & 0x0F00 >> 8) as usize;
                    for iter in 0..x as usize {
                        self.v[iter] = self.memory[self.i as usize + x];
                    }
                    self.increment_program_counter();
                }
                _ => println!("Unknown opcode: {}", self.opcode),
            },

            0x1000 => {
                self.pc = self.opcode & 0x0FFF;
            }

            0x2000 => {
                self.stack[self.stack_pointer as usize] = self.pc;
                self.stack_pointer += 1;
                self.pc = self.opcode & 0x0FFF;
            }

            0x3000 => {
                if self.v[((self.opcode & 0x0F00) as u8) as usize] == (self.opcode & 0x00FF) as u8 {
                    self.skip_next_instruction();
                } else {
                    self.increment_program_counter();
                }
            }

            0x4000 => {
                if self.v[((self.opcode & 0x0F00) as u8) as usize] != (self.opcode & 0x00FF) as u8 {
                    self.skip_next_instruction();
                } else {
                    self.increment_program_counter();
                }
            }

            0x5000 => {
                if self.v[((self.opcode & 0x0F00) as u8) as usize]
                    == self.v[((self.opcode & 0x00F0) as u8) as usize]
                {
                    self.skip_next_instruction();
                } else {
                    self.increment_program_counter();
                }
            }

            0x6000 => {
                self.v[((self.opcode & 0x0F00) as u8) as usize] = (self.opcode & 0x00FF) as u8;
                self.increment_program_counter();
            }

            0x7000 => {
                let x = ((self.opcode & 0x0F00) as u8) as usize;
                self.v[x] += (self.opcode & 0x00FF) as u8;
                self.increment_program_counter();
            }

            0x8000 => match self.opcode & 0x000F {
                0x0000 => {
                    self.v[(self.opcode & 0x0F00) as u8 as usize] =
                        self.v[(self.opcode & 0x00F0) as u8 as usize];
                    self.increment_program_counter();
                }

                0x0001 => {
                    self.v[(self.opcode & 0x0F00) as u8 as usize] |=
                        self.v[(self.opcode & 0x00F0) as u8 as usize];
                    self.increment_program_counter();
                }

                0x0002 => {
                    self.v[(self.opcode & 0x0F00) as u8 as usize] &=
                        self.v[(self.opcode & 0x00F0) as u8 as usize];
                    self.increment_program_counter();
                }

                0x0003 => {
                    self.v[(self.opcode & 0x0F00) as u8 as usize] ^=
                        self.v[(self.opcode & 0x00F0) as u8 as usize];
                    self.increment_program_counter();
                }

                0x0004 => {
                    let sum = (self.v[(self.opcode & 0x0F00) as u8 as usize]
                        + self.v[(self.opcode & 0x00F0) as u8 as usize])
                        as u16;
                    if sum > 0x00FF {
                        self.v[0xF] = 1;
                    } else {
                        self.v[0xF] = 0;
                    }

                    self.v[(self.opcode & 0x0F00) as u8 as usize] = sum as u8;
                    self.increment_program_counter();
                }

                0x0005 => {
                    let sub = self.v[(self.opcode & 0x0F00) as u8 as usize]
                        - self.v[(self.opcode & 0x00F0) as u8 as usize];
                    if self.v[(self.opcode & 0x0F00) as u8 as usize]
                        > self.v[(self.opcode & 0x00F0) as u8 as usize]
                    {
                        self.v[0xF] = 1;
                    } else {
                        self.v[0xF] = 0;
                    }

                    self.v[(self.opcode & 0x0F00) as u8 as usize] = sub as u8;
                    self.increment_program_counter();
                }

                0x0006 => {
                    let shr = self.v[(self.opcode & 0x0F00) as u8 as usize] >> 1;

                    if self.v[(self.opcode & 0x0F00) as u8 as usize] & 0x000F != 0x0000 {
                        self.v[0xF] = 1;
                    } else {
                        self.v[0xF] = 0
                    }

                    self.v[(self.opcode & 0x0F00) as u8 as usize] = shr;
                    self.increment_program_counter();
                }

                0x0007 => {
                    let sub = self.v[(self.opcode & 0x00F0) as u8 as usize]
                        - self.v[(self.opcode & 0x0F00) as u8 as usize];

                    if self.v[(self.opcode & 0x00F0) as u8 as usize]
                        > self.v[(self.opcode & 0x0F00) as u8 as usize]
                    {
                        self.v[0xF] = 1;
                    } else {
                        self.v[0xF] = 0
                    }

                    self.v[(self.opcode & 0x0F00) as u8 as usize] = sub;
                    self.increment_program_counter();
                }

                0x000E => {
                    let shl = self.v[(self.opcode & 0x0F00) as u8 as usize] << 1;

                    if self.v[(self.opcode & 0x0F00) as u8 as usize] as u16 & 0xF000 != 0x0000 {
                        self.v[0xF] = 1;
                    } else {
                        self.v[0xF] = 0
                    }

                    self.v[(self.opcode & 0x0F00) as u8 as usize] = shl;
                    self.increment_program_counter();
                }

                _ => unimplemented!(),
            },

            0x9000 => {
                if self.v[(self.opcode & 0x0F00) as u8 as usize]
                    != self.v[(self.opcode & 0x00F0) as u8 as usize]
                {
                    self.skip_next_instruction();
                } else {
                    self.increment_program_counter();
                }
            }

            0xD000 => self.increment_program_counter(),

            0xF000 => self.increment_program_counter(),

            _ => println!("Unknown opcode: {}", self.opcode),
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

    fn increment_program_counter(&mut self) {
        self.pc += 2;
    }

    fn skip_next_instruction(&mut self) {
        self.pc += 4;
    }

    fn fetch_opcode(&mut self) -> u16 {
        (self.memory[self.pc as usize] as u16) << 8 | self.memory[self.pc as usize + 1] as u16
    }
}
