use crate::base::MapperBase;
use crate::cpu;
use crate::mapper0;
use crate::mem;
use crate::ppu;
use crate::ppu::Ppu;
use crate::rom;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::render::Texture;
use sdl2::video::Window;
use sdl2::EventPump;

pub struct Nes {
    cpu: cpu::Cpu,
}
impl Nes {
    pub fn new() -> Self {
        let ppu = ppu::Ppu::new_empty_rom();
        let rom = rom::Rom::new();
        let mapper = mapper0::Mapper0::new(rom, ppu);
        let mem = mem::Mem::new(mapper);

        Self {
            cpu: cpu::Cpu::new(mem),
        }
    }
    pub fn init(&mut self) {
        self.cpu.init();
    }
    pub fn set_rom(&mut self, mut buf: Vec<u8>) {
        self.cpu.mem.mapper.set_rom(buf);
        println!("load rom");
    }
    pub fn start(&mut self, test: bool, mut event_pump: EventPump, mut canvas: Canvas<Window>) {
        let mut count = 0;
        let mut testflg = false;
        if test {
            self.cpu.init_nestest();
            count = 8992;
            testflg = true;
        } else {
            self.cpu.start();
        }
        self.main_loop(count, testflg, event_pump, canvas);
    }
    pub fn main_loop(
        &mut self,
        mut count: usize,
        test: bool,
        mut event_pump: EventPump,
        mut canvas: Canvas<Window>,
    ) {
        let mut i = 0;
        loop {
            i += 1;
            if count != 0 && count == i {
                break;
            }

            let cycles: usize = if self.cpu.mem.dma.should_run() {
                self.cpu.mem.dma.run(&self.cpu.mem.ram, &mut self.cpu.mem.mapper.ppu);
                514
            } else {
                self.cpu.run(test) as usize
            };

            self.cpu.mem.mapper.ppu.run(cycles * 3);

            
            let nmi = self.cpu.mem.mapper.ppu.get_nmi_status();
            if nmi {
                if self.cpu.mem.mapper.ppu.background.0.len() != 0 {
                    self.cpu.mem.mapper.render();
                    let buf = &self.cpu.mem.mapper.render.get_buf();
                    for i in 0..224 {
                        for j in 0..256 {
                            let base = ((i * 256 + j) * 4) as usize;
                            let r = buf[base + 0];
                            let g = buf[base + 1];
                            let b = buf[base + 2];
                            canvas.set_draw_color(Color::RGB(r, g, b));
                            let _ = canvas.draw_point(Point::new(j as i32, i as i32));
                        }
                    }
                    canvas.present();
                }
            }

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => std::process::exit(0),
                    _ => { /* do nothing */ }
                }
            }
        }
    }
}
