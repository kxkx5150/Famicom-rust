use crate::base;
use crate::ppu;
use crate::render;
use crate::rom;

pub struct Mapper0 {
    pub rom: rom::Rom,
    pub ppu: ppu::Ppu,
    pub render: render::Renderer,
}
impl base::MapperBase for base::Base {}
impl base::MapperBase for Mapper0 {}
impl Mapper0 {
    pub fn new(rom: rom::Rom, ppu: ppu::Ppu) -> Self {
        let render = render::Renderer::new();
        Self { rom: rom, ppu: ppu, render:render }
    }

    pub fn set_rom(&mut self, mut buf: Vec<u8>) {
        println!("Mapper0 set_rom");
        self.rom.set_rom(buf);
        self.rom.set_prgrom_page(0, 0);
        self.rom.set_prgrom_page(1, self.rom.prg_rom_page_count - 1);
        self.ppu.set_rom(self.rom.chr_rom.clone(), self.rom.screen_mirroring.clone());
    }
    pub fn init(&mut self) {
        println!("mapper init");
        self.rom.init();
    }
    pub fn render(&mut self) {
        self.render.render(&self.ppu.background.0, &self.ppu.sprites);
    }
}
