mod chip8;
use chip8::Chip8;

use std::error::Error;
use std::path;

use minifb::{Key, Window, WindowOptions};

fn main() -> Result<(), Box<dyn Error>> {
    let mut chip8 = Chip8::new();
    
    let path = path::Path::new("./roms/pong.rom");
    chip8.load_program(path)?;

    let mut window = Window::new("Chip-8 Simulator", 640, 320, WindowOptions::default())
        .unwrap_or_else(|e| panic!("{}", e));

    window.limit_update_rate(Some(std::time::Duration::from_micros(66400)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        chip8.emulate_cycle();


        let rgb_buffer: Vec<u32> = chip8
            .display
            .iter()
            .map(|bw_pixel| match bw_pixel {
                1 => 0x00FFFFFF,
                0 => 0x00000000,
                _ => 0x00222222,
            })
            .collect();

        if chip8.draw_flag {
            window
                .update_with_buffer(&rgb_buffer, 64, 32)
                .unwrap_or_else(|e| panic!("Error updating: {}", e));
        } else {
            window.update();
        }
    }

    Ok(())
}
