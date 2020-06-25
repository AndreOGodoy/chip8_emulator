mod chip8;
use chip8::Chip8;

use std::error::Error;
use std::path;

fn main() -> Result<(), Box<dyn Error>> {
    let mut chip8 = Chip8::new();

    let path = path::Path::new("./roms/pong.rom");
    chip8.load_program(path)?;

    for _ in 0..40 {
        chip8.emulate_cycle();
    }

    Ok(())
}
