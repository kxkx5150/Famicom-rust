#![allow(warnings, unused, dead_code)]
use std::env;
use std::fs;
pub mod cpu;
pub mod dma;
pub mod io;
pub mod irq;
pub mod mapper;
pub mod mapper0;
pub mod mem;
pub mod nes;
pub mod nestest;
pub mod ppu;
pub mod rom;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Canvas;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::video::Window;
use sdl2::video::WindowContext;
use sdl2::EventPump;

const WIDTH: u32 = 256;
const HEIGHT: u32 = 224;
const SCALE: u32 = 2;

#[macro_use]
extern crate bitflags;

fn main() {
    let (event_pump, canvas) = create_window();

    let mut nes = nes::Nes::new();
    nes.init();

    let cputest = false;
    let mut filename = "";
    if cputest {
        filename = "nestest.nes";
    } else {
        filename = "sm.nes";
    }

    match fs::read(filename) {
        Result::Ok(buf) => {
            nes.set_rom(buf);
        }
        Result::Err(err) => {
            eprintln!("Cannot open .nes file: {}", filename);
            filename = "j.nes";
            match fs::read(filename) {
                Result::Ok(buf) => {
                    nes.set_rom(buf);
                }
                Result::Err(err) => {
                    eprintln!("Cannot open .nes file: {}", filename);
                    panic!("{}", err);
                }
            }
        }
    }
    nes.start(cputest, event_pump, canvas);
}
fn create_window() -> (EventPump, Canvas<Window>) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("", (WIDTH * SCALE) as u32, (HEIGHT * SCALE) as u32)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_scale(SCALE as f32, SCALE as f32).unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    (event_pump, canvas)
}
