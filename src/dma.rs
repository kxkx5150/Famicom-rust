use crate::ppu::Ppu;

#[derive(Debug)]
pub struct Dma {
    base_addr: u8,
    should_run: bool,
}

impl Dma {
    pub fn new() -> Self {
        Dma {
            base_addr: 0,
            should_run: false,
        }
    }

    pub fn set(&mut self, data: u8) {
        self.base_addr = data;
        self.should_run = true;
    }

    pub fn should_run(&self) -> bool {
        self.should_run
    }    

    pub fn run(&mut self, ram: &Vec<u8>, ppu: &mut Ppu) {
        let offset = (self.base_addr as u16) << 8;
        for i in 0..0x100 {
            ppu.transfer_sprite(i, ram[(offset + i) as usize]);
        }
        self.should_run = false;
    }
}
