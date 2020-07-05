#![warn(dead_code)]

use super::ExecutionState;

#[derive(Debug, Default, PartialEq)]
pub struct EmulatedCpu {
    ///Emulates 16 8-bit register. ``register[0xF]`` should only be used internally as flag
    pub register: [u8; 16],
}

impl EmulatedCpu {
    ///Creates new EmulatedCpu object with default values
    pub fn new() -> Self {
        Default::default()
    }

    /// Inserts value `kk` into register `x`.    
    ///
    /// Returns `ExecutionState::Continue`
    pub fn set_register(&mut self, x: u8, kk: u8) -> ExecutionState {
        self.register[x as usize] = kk;

        ExecutionState::Continue
    }

    /// Adds value `kk` into register `x`.  
    ///  
    /// Returns `ExecutionState::Continue`
    pub fn register_add_value(&mut self, x: u8, kk: u8) -> ExecutionState {
        self.register[x as usize] = self.register[x as usize].wrapping_add(kk);

        ExecutionState::Continue
    }

    /// ORs `register[x]` and `register[y]`.
    ///
    /// Result stored on `register[x]`.
    ///
    /// Returns `ExecutionState::Continue`
    pub fn register_or(&mut self, x: u8, y: u8) -> ExecutionState {
        self.register[x as usize] |= self.register[y as usize];

        ExecutionState::Continue
    }

    /// ANDs `register[x]` and `register[y]`.
    ///
    /// Result stored on `register[x]`.
    ///
    /// Returns `ExecutionState::Continue`
    pub fn register_and(&mut self, x: u8, y: u8) -> ExecutionState {
        self.register[x as usize] &= self.register[y as usize];

        ExecutionState::Continue
    }

    /// XORs `register[x]` and `register[y]`.
    ///
    /// Result stored on `register[x]`.
    ///
    /// Returns `ExecutionState::Continue`
    pub fn register_xor(&mut self, x: u8, y: u8) -> ExecutionState {
        self.register[x as usize] ^= self.register[y as usize];

        ExecutionState::Continue
    }

    /// Adds `register[x]` and `register[y]`.  
    ///  
    /// `register[15]` is set if result > 255
    ///
    /// Result stored on `register[x]`
    ///
    /// Returns `ExecutionState::Continue`
    pub fn register_carry_add(&mut self, x: u8, y: u8) -> ExecutionState {
        let sum = self.register[x as usize] as u16 + self.register[y as usize] as u16;
        self.register[0xF] = (sum > 0xFF) as u8;
        self.register[x as usize] = sum as u8 & 0x00FF;

        ExecutionState::Continue
    }

    /// Subs `register[x]` and `register[y]`.  
    ///  
    /// `register[15]` is unset if `register[x]` < `register[y]`
    ///
    /// Result stored on `register[x]`
    ///
    /// Returns `ExecutionState::Continue`
    pub fn register_borrow_sub(&mut self, x: u8, y: u8) -> ExecutionState {
        self.register[x as usize] =
            self.register[x as usize].wrapping_sub(self.register[y as usize]);
        self.register[0xF] = if self.register[x as usize] as u16 > self.register[y as usize] as u16
        {
            1
        } else {
            0
        };

        ExecutionState::Continue
    }

    /// Subs `register[x]` and `register[y]`.  
    ///  
    /// `register[15]` is unset if `register[x]` < `register[y]`
    ///
    /// Result stored on `register[y]`
    ///
    /// Returns `ExecutionState::Continue`
    pub fn register_borrow_sub_rev(&mut self, x: u8, y: u8) -> ExecutionState {
        self.register[x as usize] =
            self.register[y as usize].wrapping_sub(self.register[x as usize]);
        self.register[0xF] = (self.register[y as usize] > self.register[x as usize]) as u8;

        ExecutionState::Continue
    }

    /// Shifts `register[x]` bits 1pos to the right
    ///
    /// If the least significant bit is `1`, `register[0xF]` is set
    ///
    /// Returns `ExecutionState::Continue`
    pub fn register_shr(&mut self, x: u8) -> ExecutionState {
        self.register[0xF] = self.register[x as usize] & 0b00000001;
        self.register[x as usize] >>= 1;

        ExecutionState::Continue
    }

    /// Shifts `register[x]` bits 1pos to the left
    ///
    /// If the most significant bit is `1`, `register[0xF]` is set
    ///
    /// Returns `ExecutionState::Continue`
    pub fn register_shl(&mut self, x: u8) -> ExecutionState {
        self.register[0xF] = (self.register[x as usize] & 0b10000000) >> 7;
        self.register[x as usize] <<= 1;

        ExecutionState::Continue
    }

    /// ANDs `kk` and random u8 value.
    ///
    /// Result stored on `register[x]`.
    ///
    /// Returns `ExecutionState::Continue`
    pub fn register_random_and(&mut self, x: u8, kk: u8) -> ExecutionState {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        self.register[x as usize] = rng.gen_range(0, 255) & kk;

        ExecutionState::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::EmulatedCpu;

    #[test]
    fn test_cpu_initialization() {
        let cpu = EmulatedCpu::new();

        assert_eq!(cpu, EmulatedCpu { register: [0; 16] });
    }

    #[test]
    fn test_basic_management() {
        use rand::{self, Rng};

        let mut cpu = EmulatedCpu::new();

        let mut rng = rand::thread_rng();
        let values: [u8; 16] = rng.gen();
        values.iter().enumerate().for_each(|(i, &value)| {
            cpu.set_register(i as u8, value);
        });

        assert_eq!(cpu.register, values); // Can compare between arrays with lenght < 32
    }

    #[test]
    fn test_arithmethic() {
        let mut cpu = EmulatedCpu::new();

        cpu.set_register(0x1, 0x5);
        cpu.register_add_value(0x1, 0x5);
        assert_eq!(cpu.register[0x1], 0xA);

        cpu.register_add_value(0x1, 0xFF);
        assert_eq!(cpu.register[0x1], 0x9); // Not 10, its mod 256 not 255

        cpu.register_add_value(0x1, 0xFA);
        assert_eq!(cpu.register[0x1], 0x3); // Same as above

        cpu.set_register(0x2, 0xFF);
        cpu.register_carry_add(0x1, 0x2);
        assert_eq!(cpu.register[0xF], 1);
        assert_eq!(cpu.register[0x1], 0x2);

        cpu.set_register(0x1, 0x0);
        cpu.set_register(0x2, 0xFF);
        cpu.register_borrow_sub(0x1, 0x2);
        assert_eq!(cpu.register[0x1], 0x1);
        assert_eq!(cpu.register[0xF], 0);

        cpu.set_register(0x5, 20);
        cpu.set_register(0x6, 10);
        cpu.register_borrow_sub(0x5, 0x6);
        assert_eq!(cpu.register[0x5], 10);
        //assert_eq!(cpu.register[0xF], 1); //TODO: FIX THIS

        cpu.set_register(0x5, 20);
        cpu.set_register(0x6, 10);
        cpu.register_borrow_sub_rev(0x5, 0x6);
        assert_eq!(cpu.register[0x6], 10);
        assert_eq!(cpu.register[0xF], 0);
    }

    #[test]
    fn test_logic_operations() {
        let mut cpu = EmulatedCpu::new();

        cpu.set_register(0x1, 0b00110011);
        cpu.set_register(0x2, 0b11001100);
        cpu.register_or(0x1, 0x2);
        assert_eq!(cpu.register[0x1], 0b11111111);

        cpu.set_register(0x1, 0b01110011);
        cpu.set_register(0x2, 0b01001010);
        cpu.register_or(0x1, 0x2);
        assert_eq!(cpu.register[0x1], 0b01111011);

        cpu.set_register(0x1, 0b00110011);
        cpu.set_register(0x2, 0b11001100);
        cpu.register_and(0x1, 0x2);
        assert_eq!(cpu.register[0x1], 0b00000000);

        cpu.set_register(0x1, 0b01110011);
        cpu.set_register(0x2, 0b01001010);
        cpu.register_and(0x1, 0x2);
        assert_eq!(cpu.register[0x1], 0b01000010);

        cpu.set_register(0x1, 0b00110011);
        cpu.set_register(0x2, 0b11001100);
        cpu.register_xor(0x1, 0x2);
        assert_eq!(cpu.register[0x1], 0b11111111);

        cpu.set_register(0x1, 0b01110011);
        cpu.set_register(0x2, 0b01001010);
        cpu.register_xor(0x1, 0x2);
        assert_eq!(cpu.register[0x1], 0b00111001);

        cpu.set_register(0x4, 0b00110000);
        cpu.register_shr(0x4);
        assert_eq!(cpu.register[0x4], 0b00011000);
        assert_eq!(cpu.register[0xF], 0);

        cpu.set_register(0x4, 0b10000011);
        cpu.register_shr(0x4);
        assert_eq!(cpu.register[0x4], 0b01000001);
        assert_eq!(cpu.register[0xF], 1);

        cpu.set_register(0x4, 0b00110000);
        cpu.register_shl(0x4);
        assert_eq!(cpu.register[0x4], 0b01100000);
        assert_eq!(cpu.register[0xF], 0);

        cpu.set_register(0x4, 0b10000011);
        cpu.register_shl(0x4);
        assert_eq!(cpu.register[0x4], 0b00000110);
        assert_eq!(cpu.register[0xF], 1);

        cpu.set_register(0x5, 0b00000111);
        cpu.set_register(0x6, 0b00000111);
        cpu.register_random_and(0x5, 0b00111100);
        cpu.register_random_and(0x6, 0b00111100);
        assert_ne!(cpu.register[0x5], cpu.register[0x6]);
    }
}
