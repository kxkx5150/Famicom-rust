#[derive(Debug)]
pub struct Oam {
    addr: u16,
}

impl Oam {
    pub fn new() -> Self {
        Oam { addr: 0 }
    }
    pub fn reset_addr(&mut self) {
        self.addr = 0;
    }
    pub fn get_addr(&self) -> u16 {
        self.addr
    }
    pub fn write_addr(&mut self, data: u8) {
        self.addr = data as u16;
    }
    pub fn write_data(&mut self, ram: &mut Vec<u8>, data: u8) {
        ram[self.addr as usize] = data;
        self.addr += 1;
    }
    pub fn read_data(&self, ram: &Vec<u8>) -> u8 {
        ram[self.addr as usize]
    }
}
