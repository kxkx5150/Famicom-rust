use crate::base;
use crate::ppu;
use crate::render;
use crate::rom;
use render::frame::Frame;
use render::palette;
use render::render;

pub struct Mapper0 {
    pub rom: rom::Rom,
    pub ppu: ppu::NesPPU,
    pub frame: Frame,
}
impl base::MapperBase for base::Base {}
impl base::MapperBase for Mapper0 {}
impl Mapper0 {
    pub fn new(rom: rom::Rom, ppu: ppu::NesPPU) -> Self {
        let frame = Frame::new();

        Self {
            rom: rom,
            ppu: ppu,
            frame: frame,
        }
    }

    pub fn set_rom(&mut self, mut buf: Vec<u8>) {
        println!("Mapper0 set_rom");
        self.rom.set_rom(buf);
        self.rom.set_prgrom_page(0, 0);
        self.rom.set_prgrom_page(1, self.rom.prg_rom_page_count - 1);
    }
    pub fn init(&mut self) {
        println!("mapper init");
        self.rom.init();
    }
    pub fn render(&mut self) {
        render(&self.ppu, &mut self.frame);
    }
}
