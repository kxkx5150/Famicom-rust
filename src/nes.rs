use crate::cpu;
use crate::io;
use crate::irq;
use crate::mapper::MapperBase;
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

const PAD_A: u8 = 0x01;
const PAD_B: u8 = 0x02;
const PAD_SELECT: u8 = 0x04;
const PAD_START: u8 = 0x08;
const PAD_U: u8 = 0x10;
const PAD_D: u8 = 0x20;
const PAD_L: u8 = 0x40;
const PAD_R: u8 = 0x80;

pub struct Nes {
    cpu: cpu::Cpu,
    irq: irq::Irq,
}
impl Nes {
    pub fn new() -> Self {
        let irq = irq::Irq::new();
        let io = io::Io::new();
        let rom = rom::Rom::new();
        let ppu = ppu::Ppu::new();
        let mapper = mapper0::Mapper0::new(rom, ppu, io);
        let mem = mem::Mem::new(mapper);

        Self {
            cpu: cpu::Cpu::new(mem),
            irq,
        }
    }
    pub fn init(&mut self) {
        self.cpu.init();
        self.irq.init();
    }
    pub fn set_rom(&mut self, mut buf: Vec<u8>) {
        println!("load rom");
        self.init();
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
        let mut pad = 0;
        let player = 1;
        let creator = canvas.texture_creator();
        let mut texture = creator
            .create_texture_target(PixelFormatEnum::RGB24, WIDTH, HEIGHT)
            .unwrap();

        while  count == 0 || count != i {
            i += 1;

            if (self.cpu.mem.mapper.io.get_ctrllatched()) {
                self.cpu.mem.mapper.io.hdCtrlLatch();
            }

            self.cpu.run(&mut self.irq, cputest);
            if self.cpu.mem.dma.get_status() {
                self.cpu.mem.dma.clear();
                self.cpu.cpuclock += 514;
            }
            self.cpu
                .mem
                .mapper
                .ppu
                .run(self.cpu.cpuclock as usize, &mut self.irq);
            self.cpu.clear_cpucycle();

            let imgopt = self.cpu.mem.mapper.ppu.get_img_status();
            if imgopt.0 {
                texture.update(None, imgopt.1, 256 * 3).unwrap();
                canvas.copy(&texture, None, None).unwrap();
                canvas.present();
                self.cpu.mem.mapper.ppu.clear_img();
            }
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => std::process::exit(0),
                    Event::KeyDown {
                        keycode: Some(key), ..
                    } => {
                        pad |= self.keycode_to_pad(key);
                        if (player == 1) {
                            self.cpu.mem.mapper.io.set_ctrlstat1(pad);
                        } else if (player == 2) {
                            self.cpu.mem.mapper.io.set_ctrlstat2(pad);
                        }
                    }
                    Event::KeyUp {
                        keycode: Some(key), ..
                    } => {
                        pad &= !self.keycode_to_pad(key);
                        if (player == 1) {
                            self.cpu.mem.mapper.io.set_ctrlstat1(pad);
                        } else if (player == 2) {
                            self.cpu.mem.mapper.io.set_ctrlstat2(pad);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    pub fn keycode_to_pad(&mut self, key: Keycode) -> u8 {
        match key {
            Keycode::X => PAD_A,
            Keycode::Z => PAD_B,
            Keycode::A => PAD_SELECT,
            Keycode::S => PAD_START,
            Keycode::Up => PAD_U,
            Keycode::Down => PAD_D,
            Keycode::Left => PAD_L,
            Keycode::Right => PAD_R,
            _ => 0,
        }
    }
}
