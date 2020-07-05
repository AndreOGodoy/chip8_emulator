mod chip8;
use chip8::Chip8;

use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use std::error::Error;
use std::thread::sleep;

fn main() -> Result<(), Box<dyn Error>> {
    let mut chip8 = Chip8::new();
    chip8.load_program("roms/pong.rom")?;

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
        //chip8.clear_keys();
        //std::thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok(())
}
