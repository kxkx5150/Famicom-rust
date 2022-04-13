use crate::ppu::Ppu;

#[derive(Debug)]
pub struct Dma {
    on: bool,
}
impl Dma {
    pub fn new() -> Self {
        Dma {
            on: false,
        }
    }
    pub fn run(&mut self, data: u8, ram: &Vec<u8>, ppu: &mut Ppu) {
        let mut offset = (data as usize) << 8;
        for i in (0..0x100) {
            ppu.sprite_ram[i] = ram[offset as usize];
            offset+=1;
        }
        self.on = true;
    }
    pub fn get_status(&mut self) -> bool{
        self.on
    }
    pub fn clear(&mut self){
        self.on = false;
    }
}
