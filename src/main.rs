#![allow(warnings, unused, dead_code)]
use std::env;
use std::fs;
pub mod base;
pub mod cpu;
pub mod mapper0;
pub mod mem;
pub mod nes;
pub mod nestest;
pub mod rom;
pub mod ppu;
pub mod palette;

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

#[macro_use]
extern crate bitflags;

fn main() {
    let (event_pump, canvas, creator) = create_window();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 256 * 1, 240)
        .unwrap();

    let mut nes = nes::Nes::new();
    nes.init();
    let cputest = false;
    let mut filename = "";
    if cputest {
        filename = "nestest.nes";
    } else {
        filename = "s.nes";
    }

    match fs::read(filename) {
        Result::Ok(buf) => nes.set_rom(buf),
        Result::Err(err) => {
            eprintln!("Cannot open .nes file: {}", filename);
            panic!("{}", err);
        }
    }
    nes.start(cputest, event_pump, canvas, texture);
}
fn create_window() -> (EventPump, Canvas<Window>, TextureCreator<WindowContext>) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("", (256.0 * 1.0) as u32, (240.0 * 1.0) as u32)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(1.0, 1.0).unwrap();
    let creator = canvas.texture_creator();
    (event_pump, canvas, creator)
}
