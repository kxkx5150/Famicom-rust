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
use sdl2::render::Canvas;
use sdl2::render::Texture;
use sdl2::video::Window;
use sdl2::EventPump;

pub struct Nes {
    cpu: cpu::Cpu,
}
impl Nes {
    pub fn new() -> Self {
        let rom = rom::Rom::new();
        let ppu = ppu::Pppu::new();
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
        let chr_rom = self.cpu.mem.mapper.rom.chr_rom.clone();
        let mirroring = self.cpu.mem.mapper.rom.screen_mirroring.clone();
        println!("load rom");
    }
    pub fn start(
        &mut self,
        test: bool,
        mut event_pump: EventPump,
        mut canvas: Canvas<Window>,
        mut texture: Texture,
    ) {
        let mut count = 0;
        let mut testflg = false;
        if test {
            self.cpu.init_nestest();
            count = 8992;
            testflg = true;
        } else {
            self.cpu.start();
        }
        self.main_loop(count, testflg, event_pump, canvas, texture);
    }
    pub fn main_loop(
        &mut self,
        mut count: usize,
        test: bool,
        mut event_pump: EventPump,
        mut canvas: Canvas<Window>,
        mut texture: Texture,
    ) {
        let mut i = 0;
        loop {
            i += 1;
            if count != 0 && count == i {
                break;
            }
            let cycles = self.cpu.run(test);
            self.cpu.mem.mapper.ppu.PpuRun(cycles as usize);

            if !test {
                let nmi = self.cpu.mem.mapper.ppu.get_nmi_status();
                if nmi {
                    // println!("");
                    // self.cpu.mem.mapper.render();
                    texture
                        .update(None, &self.cpu.mem.mapper.ppu.imgdata, 256 * 1 * 3)
                        .unwrap();
                    canvas.copy(&texture, None, None).unwrap();
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
