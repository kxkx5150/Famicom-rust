use self::super::palette::*;

pub type SpritesWithCtx = Vec<SpriteWithCtx>;

#[derive(Debug)]
pub struct SpriteWithCtx {
    pub sprite: Sprite,
    pub position: SpritePosition,
    pub attr: u8,
    pub palette: PaletteList,
}

pub fn build_sprites<P: PaletteRam>(
    cram: &Vec<u8>,
    sprite_ram: &Vec<u8>,
    palette: &P,
    offset: u16,
    is_8x8: bool,
) -> SpritesWithCtx {
    let mut buf: SpritesWithCtx = vec![];
    for i in 0..(0x100 / 4) {
        let base = i * 4;
        let y = sprite_ram[base as usize];
        if y >= 8 && y < 224 {
            let sprite_id = sprite_ram[(base + 1) as usize];
            let attr = sprite_ram[(base + 2) as usize];
            let (offset, sprite_id) = if is_8x8 {
                (offset, sprite_id)
            } else {
                let offset = 0x1000u16 * (sprite_id & 0x01) as u16;
                let sprite_id = sprite_id & 0xFE;
                (offset, sprite_id)
            };
            let x = sprite_ram[(base + 3) as usize];
            let sprite = build(&cram, sprite_id as u8, offset, is_8x8);
            let position: SpritePosition = (x, y - 8);
            let palette_id = attr & 0x03;
            buf.push(SpriteWithCtx {
                sprite,
                position,
                attr,
                palette: palette.get(palette_id, PaletteType::Sprite),
            });
        }
    }
    buf
}
pub type Sprite = Vec<Vec<u8>>;
pub type SpritePosition = (u8, u8);

#[derive(Debug)]
pub struct SpriteConfig {
    pub offset_addr_ntable: Option<u16>,
    pub offset_addr_bgtable: u16,
    pub offset_addr_sptable: u16,
    pub is_hmirror: bool,
    pub is_bg_enable: bool,
}

pub fn mirror_down_sprite_addr(addr: u16, is_hmirror: bool) -> u16 {
    if !is_hmirror {
        return addr;
    }
    if (addr >= 0x0400 && addr < 0x0800) || addr >= 0x0C00 {
        return addr - 0x400 as u16;
    }
    addr
}

pub fn get_block_id(position: &SpritePosition) -> u8 {
    ((position.0 % 4) / 2) + (((position.1 % 4) / 2) * 2)
}

pub fn get_sprite_id(vram: &Vec<u8>, position: &SpritePosition, config: &SpriteConfig) -> u8 {
    let tile_number = position.1 as u16 * 32 + position.0 as u16;
    let addr = tile_number + config.offset_addr_ntable.unwrap();
    let addr = mirror_down_sprite_addr(addr, config.is_hmirror);
    let data = vram[addr as usize];
    data
}

pub fn get_attribute(vram: &Vec<u8>, position: &SpritePosition, config: &SpriteConfig) -> u8 {
    let addr = 0x03C0
        + ((position.0 / 4) + ((position.1 / 4) * 8)) as u16
        + config.offset_addr_ntable.unwrap();
    vram[mirror_down_sprite_addr(addr, config.is_hmirror) as usize] as u8
}

pub fn build(cram: &Vec<u8>, sprite_id: u8, offset: u16, is_8x8: bool) -> Sprite {
    let h = if is_8x8 { 1 } else { 2 };
    let mut sprite: Sprite = (0..8 * h).into_iter().map(|_| vec![0; 8 * h]).collect();
    for k in 0..h {
        for i in 0..16 {
            for j in 0..8 {
                let addr = ((sprite_id + (k as u8)) as u16) * 16 + i + offset;
                let ram = cram[addr as usize];
                if ram & (0x80 >> j) as u8 != 0 {
                    sprite[((k as u16) * 8 + i % 8) as usize][j] += (0x01 << (i / 8)) as u8;
                }
            }
        }
    }
    sprite
}
