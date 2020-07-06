use super::ExecutionState;

#[derive(Debug, PartialEq, Default)]
pub struct EmulatedKeypad {
    pub keypad: [u8; 16],
}

impl EmulatedKeypad {
    pub fn new() -> Self {
        Default::default()
    }

    /// Set the corresponding key value to 1
    pub fn press_key(&mut self, key: u8) {
        if key >= 16 {
            panic!(
                "Called press_key() with key = {} when value must be between 0 and 15",
                key
            );
        }

        let key = key as usize;
        self.keypad[key] = 1;
    }

    /// Set the corresponding key value to 0
    pub fn release_key(&mut self, key: u8) {
        if key >= 16 {
            panic!(
                "Called release_key() with key = {} when value must be between 0 and 15",
                key
            );
        }
        let key = key as usize;
        self.keypad[key] = 0;
    }

    pub fn skip_if_pressed(&self, key: u8) -> ExecutionState {
        if key >= 16 {
            panic!(
                "Called skip_if_pressed() with key = {} when value must be between 0 and 15",
                key
            );
        }
        let key = key as usize;
        if self.keypad[key] == 1 { ExecutionState::Skip } else { ExecutionState::Continue }
    }

    pub fn skip_if_released(&self, key: u8) -> ExecutionState {
        if key >= 16 {
            panic!(
                "Called skip_if_released() with key = {} when value must be between 0 and 15",
                key
            );
        }
        let key = key as usize;
        if self.keypad[key] == 0 { ExecutionState::Skip } else { ExecutionState::Continue }
    }

    pub fn wait_for_key(&self, vx: &mut u8) -> ExecutionState {
        if let Some((index, _)) = self.keypad.iter().enumerate().find(|(_, key)| **key == 1) {
            *vx = index as u8;

            ExecutionState::Continue
        } else {
            ExecutionState::Hold
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_keypad_initialization() {
        let mut keypad = EmulatedKeypad::new();

        assert_eq!(keypad, EmulatedKeypad { keypad: [0; 16] });
    }

    #[test]
    fn test_press_release() {
        let mut keypad = EmulatedKeypad::new();

        (0..=15).into_iter().for_each(|key| keypad.press_key(key));
        keypad.keypad.iter().for_each(|&key| assert_eq!(key, 1));

        (0..=15).into_iter().for_each(|key| keypad.release_key(key));
        keypad.keypad.iter().for_each(|&key| assert_eq!(key, 0));
    }

    #[test]
    #[should_panic]
    fn test_panic_on_press() {
        let mut keypad = EmulatedKeypad::new();

        keypad.press_key(16);
    }

    #[test]
    #[should_panic]
    fn test_panic_on_release() {
        let mut keypad = EmulatedKeypad::new();

        keypad.release_key(16);
    }
}
