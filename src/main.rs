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
    println!("Using SDL renderer: {}", canvas.info().name);
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();
    canvas.present();

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
                Event::KeyUp {
                    keycode: Some(Keycode::Num1),
                    ..
                } => chip8.release_key(1),
                Event::KeyUp {
                    keycode: Some(Keycode::Num2),
                    ..
                } => chip8.release_key(2),
                Event::KeyUp {
                    keycode: Some(Keycode::Num3),
                    ..
                } => chip8.release_key(3),
                Event::KeyUp {
                    keycode: Some(Keycode::Num4),
                    ..
                } => chip8.release_key(0xC),
                Event::KeyUp {
                    keycode: Some(Keycode::Q),
                    ..
                } => chip8.release_key(4),
                Event::KeyUp {
                    keycode: Some(Keycode::W),
                    ..
                } => chip8.release_key(5),
                Event::KeyUp {
                    keycode: Some(Keycode::E),
                    ..
                } => chip8.release_key(6),
                Event::KeyUp {
                    keycode: Some(Keycode::R),
                    ..
                } => chip8.release_key(0xD),
                Event::KeyUp {
                    keycode: Some(Keycode::A),
                    ..
                } => chip8.release_key(7),
                Event::KeyUp {
                    keycode: Some(Keycode::S),
                    ..
                } => chip8.release_key(8),
                Event::KeyUp {
                    keycode: Some(Keycode::D),
                    ..
                } => chip8.release_key(9),
                Event::KeyUp {
                    keycode: Some(Keycode::F),
                    ..
                } => chip8.release_key(0xE),
                Event::KeyUp {
                    keycode: Some(Keycode::Z),
                    ..
                } => chip8.release_key(0xA),
                Event::KeyUp {
                    keycode: Some(Keycode::X),
                    ..
                } => chip8.release_key(0),
                Event::KeyUp {
                    keycode: Some(Keycode::C),
                    ..
                } => chip8.release_key(0xB),
                Event::KeyUp {
                    keycode: Some(Keycode::V),
                    ..
                } => chip8.release_key(0xF),
                Event::KeyDown {
                    keycode: Some(Keycode::Num1),
                    ..
                } => chip8.press_key(1),
                Event::KeyDown {
                    keycode: Some(Keycode::Num2),
                    ..
                } => chip8.press_key(2),
                Event::KeyDown {
                    keycode: Some(Keycode::Num3),
                    ..
                } => chip8.press_key(3),
                Event::KeyDown {
                    keycode: Some(Keycode::Num4),
                    ..
                } => chip8.press_key(0xC),
                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => chip8.press_key(4),
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => chip8.press_key(5),
                Event::KeyDown {
                    keycode: Some(Keycode::E),
                    ..
                } => chip8.press_key(6),
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => chip8.press_key(0xD),
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => chip8.press_key(7),
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => chip8.press_key(8),
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => chip8.press_key(9),
                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    ..
                } => chip8.press_key(0xE),
                Event::KeyDown {
                    keycode: Some(Keycode::Z),
                    ..
                } => chip8.press_key(0xA),
                Event::KeyDown {
                    keycode: Some(Keycode::X),
                    ..
                } => chip8.press_key(0),
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => chip8.press_key(0xB),
                Event::KeyDown {
                    keycode: Some(Keycode::V),
                    ..
                } => chip8.press_key(0xF),
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
