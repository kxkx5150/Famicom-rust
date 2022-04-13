use crate::base::MapperBase;
use crate::cpu;
use crate::mapper0;
use crate::mem;
use crate::ppu;
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

const WIDTH: u32 = 256;
const HEIGHT: u32 = 224;

pub struct Nes {
    cpu: cpu::Cpu,
}
impl Nes {
    pub fn new() -> Self {
        let rom = rom::Rom::new();
        let ppu = ppu::Ppu::new();
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
        println!("load rom");
        self.cpu.mem.mapper.set_rom(buf);
    }
    pub fn start(&mut self, cputest: bool, mut event_pump: EventPump, mut canvas: Canvas<Window>) {
        let mut count = 0;
        let mut cputest = false;
        if cputest {
            self.cpu.init_nestest();
            count = 8992;
            cputest = true;
        } else {
            self.cpu.start();
        }
        self.main_loop(count, cputest, event_pump, canvas);
    }
    pub fn main_loop(
        &mut self,
        mut count: usize,
        cputest: bool,
        mut event_pump: EventPump,
        mut canvas: Canvas<Window>,
    ) {
        let mut i = 0;

        loop {
            i += 1;
            if count != 0 && count == i {
                break;
            }

            self.cpu.run(cputest);
            if self.cpu.mem.dma.get_status() {
                self.cpu.mem.dma.clear();
                self.cpu.cpuclock += 514;
            }
            self.cpu.mem.mapper.ppu.run(self.cpu.cpuclock as usize);
            self.cpu.clear_cpucycle();

            if !cputest {
                let imgopt = self.cpu.mem.mapper.ppu.get_img_status();
                if imgopt.0 {
                    let buf = imgopt.1;
                    for i in 0..HEIGHT {
                        for j in 0..WIDTH {
                            let base = ((i * WIDTH + j) * 3) as usize;
                            let r = buf[base + 0];
                            let g = buf[base + 1];
                            let b = buf[base + 2];
                            canvas.set_draw_color(Color::RGB(r, g, b));
                            let _ = canvas.draw_point(Point::new(j as i32, i as i32));
                        }
                    }
                    self.cpu.mem.mapper.ppu.clear_img();
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
