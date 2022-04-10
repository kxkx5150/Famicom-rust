#![allow(warnings, unused, dead_code)]
use std::env;
use std::fs;
pub mod base;
pub mod cpu;
pub mod mapper0;
pub mod mem;
pub mod nes;
pub mod nestest;
pub mod ppu;
pub mod rom;
pub mod render;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::EventPump;
const WIDTH: u32 = 256;
const HEIGHT: u32 = 224;

fn main() {
    //sdl2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut nes = nes::Nes::new();
    nes.init();

    let cputest = false;
    let mut filename = "";
    if cputest {
        filename = "nestest.nes";
    } else {
        filename = "sm.nes";
        // filename = "nestest.nes";
    }

    match fs::read(filename) {
        Result::Ok(buf) => {
            nes.set_rom(buf);
        }
        Result::Err(err) => {
            eprintln!("Cannot open .nes file: {}", filename);
            panic!("{}", err);
        }
    }
    nes.start(cputest, event_pump, canvas);
}
