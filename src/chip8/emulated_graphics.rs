use super::ExecutionState;

const WINDOW_SIZE: (usize, usize) = (64, 32);

pub struct EmulatedGraphics {
    pub display: [u8; WINDOW_SIZE.0 * WINDOW_SIZE.1],
}

impl Default for EmulatedGraphics {
    fn default() -> EmulatedGraphics {
        EmulatedGraphics {
            display: [0; WINDOW_SIZE.0 * WINDOW_SIZE.1],
        }
    }
}

impl EmulatedGraphics {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn clear_display(&mut self) -> ExecutionState {
        self.display.iter_mut().for_each(|byte| *byte = 0);

        ExecutionState::Continue
    }

    pub fn draw_sprite(&mut self, x_pos: u8, y_pos: u8, n: u8, sprite: &[u8]) -> bool {
        let mut colision_flag = false;

        for byte in 0..n {
            let current_line = sprite[byte as usize];
            let y_pos = (y_pos + byte) as usize % 32;

            for bit in 0..8 {
                let current_pixel = (current_line & (0b10000000 >> bit)) >> (7 - bit);
                let x_pos = (x_pos + bit) as usize % 64;

                if current_pixel != 0 {
                    let ref mut actual_display_pixel = self.display[WINDOW_SIZE.0 * y_pos + x_pos];

                    if *actual_display_pixel == 1 {
                        //The actual pixel position beeing set is already set
                        colision_flag = true;
                    }

                    *actual_display_pixel ^= 1;
                }
            }
        }
        colision_flag
    }
}

#[cfg(test)]
mod tests {
    use super::EmulatedGraphics;

    #[test]
    fn test_graphics_initialization() {
        let graphics = EmulatedGraphics::new();

        assert!(graphics.display.iter().all(|&byte| byte == 0));
    }

    #[test]
    fn test_clear_display() {
        let mut graphics = EmulatedGraphics::new();

        graphics.clear_display();
        assert!(graphics.display.iter().all(|&byte| byte == 0));

        graphics
            .display
            .iter_mut()
            .step_by(3)
            .for_each(|byte| *byte = 1);
        assert!(graphics.display.iter().any(|&byte| byte == 1));

        graphics.clear_display();
        assert!(graphics.display.iter().all(|&byte| byte == 0));
    }
}
