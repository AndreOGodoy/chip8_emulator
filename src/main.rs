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
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        self.emulate_cycle();
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
        present(ctx)
    }
}
