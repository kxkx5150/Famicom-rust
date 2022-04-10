use crate::base;
use crate::ppu;
use crate::rom;

pub struct Mapper0 {
    pub rom: rom::Rom,
    pub ppu: ppu::Ppu,
}
impl base::MapperBase for base::Base {}
impl base::MapperBase for Mapper0 {}
impl Mapper0 {
    pub fn new(rom: rom::Rom, ppu: ppu::Ppu) -> Self {
        Self { rom: rom, ppu: ppu }
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
}

