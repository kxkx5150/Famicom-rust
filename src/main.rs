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
pub mod render;
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
        .window("", (256.0 * 2.0) as u32, (240.0 * 2.0) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(2.0, 2.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 256 * 1, 240)
        .unwrap();

    let mut nes = nes::Nes::new();
    nes.init();

    let filename = "nestest.nes";
    let filename = "s.nes";

    match fs::read(filename) {
        Result::Ok(buf) => {
            nes.set_rom(buf);
        }
        Result::Err(err) => {
            eprintln!("Cannot open .nes file: {}", filename);
            panic!("{}", err);
        }
    }
    // cpu test
    // nes.start("test", event_pump, canvas,texture);
    nes.start("", event_pump, canvas,texture);
}
