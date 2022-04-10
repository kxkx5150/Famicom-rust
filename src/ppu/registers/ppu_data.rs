use super::super::palette::*;

#[derive(Debug)]
pub struct Ppuu8 {
    buf: u8,
}
impl Ppuu8 {
    pub fn new() -> Self {
        Ppuu8 { buf: 0 }
    }
    pub fn read<P: PaletteRam>(
        &mut self,
        vram: &Vec<u8>,
        cram: &Vec<u8>,
        addr: u16,
        palette: &P,
    ) -> u8 {
        let buf = self.buf;
        if addr >= 0x2000 {
            let addr = self.calc_addr(addr);
            if addr >= 0x3F00 {
                self.buf = vram[addr as usize];
                return palette.read(addr - 0x3f00);
            }
            self.buf = vram[addr as usize];
        } else {
            self.buf = cram[addr as usize];
        }
        buf
    }
    pub fn write<P: PaletteRam>(
        &mut self,
        vram: &mut Vec<u8>,
        cram: &mut Vec<u8>,
        addr: u16,
        data: u8,
        palette: &mut P,
    ) {
        if addr >= 0x2000 {
            if addr >= 0x3f00 && addr < 0x4000 {
                palette.write(addr - 0x3f00, data);
            } else {
                let addr = self.calc_addr(addr);
                vram[addr as usize] = data;
            }
        } else {
            cram[addr as usize] = data;
        }
    }
    fn calc_addr(&self, addr: u16) -> u16 {
        if addr >= 0x3000 && addr < 0x3f00 {
            addr - 0x3000
        } else {
            addr - 0x2000
        }
    }
}
