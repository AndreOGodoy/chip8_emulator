mod chip8;
use chip8::Chip8;

use sdl2;
use sdl2::event::*;
use sdl2::keyboard::*;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use std::error::Error;
use std::thread::sleep;

use std::collections::HashMap;

fn main() -> Result<(), Box<dyn Error>> {
    let mut chip8 = Chip8::new();
    chip8.load_program("roms/pong.rom")?;

    let key_map: HashMap<Keycode, u8> = [
        (Keycode::Num1, 1),
        (Keycode::Num2, 2),
        (Keycode::Num3, 3),
        (Keycode::Num4, 0xC),
        (Keycode::Q, 0x4),
        (Keycode::W, 0x5),
        (Keycode::E, 0x6),
        (Keycode::R, 0xD),
        (Keycode::A, 0x7),
        (Keycode::S, 0x8),
        (Keycode::D, 0x9),
        (Keycode::F, 0xE),
        (Keycode::Z, 0xA),
        (Keycode::X, 0x0),
        (Keycode::C, 0xB),
        (Keycode::V, 0xF),
    ]
    .iter()
    .cloned()
    .collect();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Chip8 - Emulator", 640, 320)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().present_vsync().build()?;
    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        chip8.emulate_cycle();
        canvas.clear();

        if chip8.draw_flag {
            chip8.display.iter().enumerate().for_each(|(index, &byte)| {
                let color = match byte {
                    1 => Color::RGB(255, 255, 255),
                    0 => Color::RGB(0, 0, 0),
                    _ => panic!("Unknown byte value on display"),
                };
                canvas.set_draw_color(color);
                canvas
                    .fill_rect(Rect::new(
                        (index as i32 % 64) * 10,
                        (index as i32 / 64) * 10,
                        10,
                        10,
                    ))
                    .expect("Fail drawing");
            });
        }

        canvas.present();
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::Quit { .. } => break 'running,
                _ => (),
            }
        }

        event_pump
            .keyboard_state()
            .scancodes()
            .for_each(|(scancode, is_pressed)| {
                if let Some(keycode) = Keycode::from_scancode(scancode) {
                    if key_map.contains_key(&keycode) {
                        let key_as_u8 = key_map[&keycode];

                        if is_pressed {
                            chip8.press_key(key_as_u8)
                        } else {
                            chip8.release_key(key_as_u8)
                        }
                    }
                }
            });
    }
    Ok(())
}
