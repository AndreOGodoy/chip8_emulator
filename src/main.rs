mod chip8;
use chip8::Chip8;

use std::error::Error;
use std::path;

use ggez::graphics::*;
use ggez::*;

fn main() -> ggez::GameResult<()> {
    let mut chip8 = Chip8::new();

    let path = path::Path::new("./roms/test_opcode.rom");
    chip8.load_program(path).expect("Fail do load program");

    let cb = ContextBuilder::new("Chip8", "Andre")
        .window_setup(ggez::conf::WindowSetup::default().title("Chip8"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(640.0, 320.0));

    let (ctx, event_loop) = &mut cb.build()?;

    event::run(ctx, event_loop, &mut chip8)
}

impl ggez::event::EventHandler for Chip8 {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.emulate_cycle();
        match_keys_pressed(ctx)
            .iter()
            .for_each(|key_option| match key_option {
                Some(key) => {
                    println!("Pressionou aqui");
                    self.press_key(*key)
                }
                None => (),
            });
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if self.draw_flag == true {
            let mut index = 0;
            for j in 0..32 {
                for i in 0..64 {
                    let rect = Rect::new(i as f32 * 10.0, j as f32 * 10.0, 10.0, 10.0);
                    let color = match self.display[index] {
                        0 => BLACK,
                        1 => WHITE,
                        _ => panic!("Deu pau na cor"),
                    };

                    Mesh::new_rectangle(ctx, DrawMode::Fill(FillOptions::default()), rect, color)
                        .expect("Deu pau na mesh")
                        .draw(ctx, DrawParam::default())
                        .expect("Deu pau no draw");

                    index += 1;
                }
            }
        }
        self.draw_flag = false;
        present(ctx)
    }
}

fn match_keys_pressed(ctx: &Context) -> Vec<Option<u8>> {
    use ggez::event::KeyCode;

    input::keyboard::pressed_keys(ctx)
        .iter()
        .map(|&key| match key {
            KeyCode::Key1 => Some(0x1),
            KeyCode::Key2 => Some(0x2),
            KeyCode::Key3 => Some(0x3),
            KeyCode::Key4 => Some(0xC),
            KeyCode::Q => Some(0x4),
            KeyCode::W => Some(0x5),
            KeyCode::E => Some(0x6),
            KeyCode::R => Some(0xD),
            KeyCode::A => Some(0x7),
            KeyCode::S => Some(0x8),
            KeyCode::D => Some(0x9),
            KeyCode::F => Some(0xE),
            KeyCode::Z => Some(0xA),
            KeyCode::X => Some(0x0),
            KeyCode::C => Some(0xB),
            KeyCode::V => Some(0xF),
            _ => None,
        })
        .collect()
}
