
#[derive(Debug)]
pub struct Ppuu16 {
    addr: u16,
    is_lower_addr: bool,
}
impl Ppuu16 {
    pub fn new() -> Self {
        Ppuu16 {
            addr: 0,
            is_lower_addr: false,
        }
    }
    pub fn get(&self) -> u16 {
        self.addr
    }
    pub fn reser_latch(&mut self) {
        self.is_lower_addr = false;
    }
    pub fn update(&mut self, offset: u16) {
        self.addr += offset;
    }
    pub fn write(&mut self, data: u8) {
        if self.is_lower_addr {
            self.addr += data as u16;
            self.is_lower_addr = false;
        } else {
            self.addr = (data as u16) << 8;
            self.is_lower_addr = true;
        }
    }
}

