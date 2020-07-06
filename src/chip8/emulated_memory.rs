use super::ExecutionState;

use std::convert::TryInto;

const MEM_SIZE: usize = 4096;

const FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0,
    0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80,
    0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80,
    0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];

const FONT_SET_START: usize = 0x50;

pub struct EmulatedMemory {
    pub mem_array: [u8; 4096],
    pub index: usize,
    pub stack: [u16; 16],
    pub stack_pointer: usize,
}

impl EmulatedMemory {
    /// Creates new EmulatedMemory with default parameters
    pub fn new() -> Self {
        Self::default()
    }

    /// Initializes the EmulatedMemory with the initial values, loading the FONT_SET to the corret pos
    pub fn default() -> Self {
        let mut mem_array: [u8; MEM_SIZE] = [0; MEM_SIZE];
        mem_array[FONT_SET_START..FONT_SET_START + 80].clone_from_slice(&FONT_SET[..]);

        EmulatedMemory {
            mem_array,
            index: 0,
            stack: [0; 16],
            stack_pointer: 0,
        }
    }

    /// Set index to nnn
    ///
    /// Returns ExecutionState::Continue
    pub fn set_index(&mut self, nnn: usize) -> ExecutionState {
        self.index = nnn;

        ExecutionState::Continue
    }

    /// Stores the current address at the stack and points one position above it
    ///
    /// Returns ExecutionState::JumpTo(nnn)
    pub fn call_subroutine(&mut self, nnn: usize, pc: usize) -> ExecutionState {
        self.stack[self.stack_pointer] = pc.try_into().expect("Fail to convert pc to u8");
        self.stack_pointer += 1;

        ExecutionState::JumpTo(nnn)
    }

    /// Decrements the stack pointer, acessing the previous address stored in it
    ///
    /// Returns ExecutionState::ReturnTo(pc)
    pub fn return_from_subroutine(&mut self) -> ExecutionState {
        self.stack_pointer -= 1;
        let pc = self.stack[self.stack_pointer];

        ExecutionState::ReturnTo(pc as usize)
    }

    /// Returns ExecutionState::JumpTo(nnn)
    pub fn jump_to_address(&mut self, nnn: usize) -> ExecutionState {
        ExecutionState::JumpTo(nnn)
    }

    /// Adds vx to index
    ///
    /// The result is computated modulo u16
    ///
    /// Returns ExecutionState::Continue
    pub fn index_add(&mut self, vx: u8) -> ExecutionState {
        self.index = (self.index as u16).wrapping_add(vx as u16) as usize; // Avoids invalid index values

        ExecutionState::Continue
    }

    /// Breaks x into Hundreds / Tens / Units
    ///
    /// Stores at positions I, I+1, I+2 of the memory respectively
    pub fn memory_store_bcd(&mut self, x: u8) -> ExecutionState {
        self.mem_array[self.index as usize] = x / 100;
        self.mem_array[self.index as usize + 1] = (x / 10) % 10;
        self.mem_array[self.index as usize + 2] = x % 10;

        ExecutionState::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_memory_initialization() {
        let memory = EmulatedMemory::new();

        memory.mem_array.iter().enumerate().for_each(|(i, &val)| {
            if i >= FONT_SET_START && i < FONT_SET_START + 80 {
                assert_eq!(val, FONT_SET[i - FONT_SET_START]);
            } else {
                assert_eq!(val, 0);
            }
        });
        assert_eq!(memory.stack_pointer, 0);
        assert_eq!(memory.stack, [0; 16]);
        assert_eq!(memory.index, 0);
    }

    #[test]
    fn test_index_operations() {
        let mut memory = EmulatedMemory::new();

        memory.set_index(0xABC);
        assert_eq!(memory.index, 0xABC);

        memory.set_index(0xFFF);
        assert_eq!(memory.index, 0xFFF);

        memory.set_index(0xA);
        memory.index_add(0xF);
        assert_eq!(memory.index, 0x19);

        memory.set_index(0xFFFF);
        memory.index_add(0xF);
        assert_eq!(memory.index, 0xE);
    }

    #[test]
    fn test_memory_operations() {
        let mut memory = EmulatedMemory::new();

        if let ExecutionState::JumpTo(addr) = memory.jump_to_address(0x0FFF) {
            assert_eq!(addr, 0xFFF);
        }

        let mut pc = 0x200;
        let state = memory.call_subroutine(0xABC, pc);

        assert_eq!(memory.stack[0], 0x200);
        assert_eq!(memory.stack_pointer, 1);
        if let ExecutionState::JumpTo(addr) = state {
            pc = addr;
            assert_eq!(pc, 0xABC);
        }

        let state = memory.call_subroutine(0xFCE, pc);

        assert_eq!(memory.stack[0], 0x200);
        assert_eq!(memory.stack[1], 0xABC);
        assert_eq!(memory.stack_pointer, 2);
        if let ExecutionState::JumpTo(addr) = state {
            pc = addr;
            assert_eq!(pc, 0xFCE);
        }

        let state = memory.return_from_subroutine();
        assert_eq!(memory.stack[0], 0x200);
        assert_eq!(memory.stack[1], 0xABC);
        assert_eq!(memory.stack_pointer, 1);
        if let ExecutionState::ReturnTo(addr) = state {
            pc = addr;
            assert_eq!(pc, 0xABC);
        }

        let state = memory.return_from_subroutine();
        assert_eq!(memory.stack[0], 0x200);
        assert_eq!(memory.stack_pointer, 0);
        if let ExecutionState::ReturnTo(addr) = state {
            pc = addr;
            assert_eq!(pc, 0x200);
        }

        let number = 159;
        memory.memory_store_bcd(number);
        assert_eq!(memory.mem_array[memory.index], 1);
        assert_eq!(memory.mem_array[memory.index + 1], 5);
        assert_eq!(memory.mem_array[memory.index + 2], 9);

        let number = 255;
        memory.memory_store_bcd(number);
        assert_eq!(memory.mem_array[memory.index], 2);
        assert_eq!(memory.mem_array[memory.index + 1], 5);
        assert_eq!(memory.mem_array[memory.index + 2], 5);
    }
}
