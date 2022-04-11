use crate::base;
use crate::rom;
use crate::ppu;

pub struct Mapper0 {
    pub rom: rom::Rom,
    pub ppu: ppu::Pppu,
}
impl base::MapperBase for base::Base {}
impl base::MapperBase for Mapper0 {}
impl Mapper0 {
    pub fn new(rom: rom::Rom, ppu: ppu::Pppu) -> Self {
        Self {
            rom: rom,
            ppu: ppu,
        }
    }

    pub fn set_rom(&mut self, mut buf: Vec<u8>) {
        println!("Mapper0 set_rom");
        self.rom.set_rom(buf);
        self.rom.set_prgrom_page(0, 0);
        self.rom.set_prgrom_page(1, self.rom.prg_rom_page_count - 1);
        self.ppu.set_chr_rom_page(0, &mut self.rom);
        self.ppu.start(&mut self.rom);
    }
    pub fn init(&mut self) {
        println!("mapper init");
        // self.rom.init();
    }
    pub fn render(&mut self) {

    }
}
