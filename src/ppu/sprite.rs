use self::super::palette::*;
use self::super::sprite_utils::*;

const SPRITES_NUMBER: u16 = 0x100;

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
    for i in 0..(SPRITES_NUMBER / 4) {
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
