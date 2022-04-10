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

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::EventPump;

#[macro_use]
extern crate bitflags;

fn main() {
    //sdl2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("", (256.0 * 1.0) as u32, (224.0 * 1.0) as u32)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let creator = canvas.texture_creator();

    let mut nes = nes::Nes::new();
    nes.init();

    let cputest = false;
    let mut filename = "";
    if cputest {
        filename = "nestest.nes";
    } else {
        filename = "sm.nes";
        filename = "nestest.nes";

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
