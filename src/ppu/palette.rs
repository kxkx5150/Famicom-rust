#[derive(Debug)]
pub enum PaletteType {
    Sprite,
    Background,
}

pub type PaletteList = Vec<u8>;

#[derive(Debug)]
pub struct Palette(PaletteList);

pub trait PaletteRam {
    fn get(&self, palette_id: u8, palette_type: PaletteType) -> PaletteList;

    fn read(&self, addr: u16) -> u8;

    fn write(&mut self, addr: u16, data: u8);
}

impl Palette {
    pub fn new() -> Self {
        Palette(vec![0; 0x20])
    }

    fn is_sprite_mirror(&self, addr: u16) -> bool {
        (addr == 0x10) || (addr == 0x14) || (addr == 0x18) || (addr == 0x1c)
    }

    fn is_background_mirror(&self, addr: u16) -> bool {
        (addr == 0x04) || (addr == 0x08) || (addr == 0x0c)
    }

    fn get_palette_addr(&self, addr: u16) -> u16 {
        let mirror_downed = (addr & 0xFF) % 0x20;
        if self.is_sprite_mirror(mirror_downed) {
            mirror_downed - 0x10
        } else {
            mirror_downed
        }
    }
}

impl PaletteRam for Palette {
    fn get(&self, palette_id: u8, palette_type: PaletteType) -> PaletteList {
        let offset = match palette_type {
            PaletteType::Sprite => 0x10,
            _ => 0x00,
        };
        let start = (palette_id * 4 + offset) as usize;
        let end = start + 4;
        (start..end).map(|p| self.read(p as u16)).collect()
    }

    fn read(&self, addr: u16) -> u8 {
        if self.is_sprite_mirror(addr) {
            return self.0[(addr - 0x10) as usize];
        }
        if self.is_background_mirror(addr) {
            return self.0[0x00];
        }
        self.0[addr as usize]
    }

    fn write(&mut self, addr: u16, data: u8) {
        let index: usize;
        {
            index = self.get_palette_addr(addr) as usize;
        }
        self.0[index] = data;
    }
}
