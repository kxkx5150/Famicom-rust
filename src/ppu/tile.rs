use self::super::palette::*;
use self::super::sprite_utils::*;

#[derive(Debug)]
pub struct Tile {
    pub sprite: Sprite,
    pub palette: PaletteList,
}

impl Tile {
    pub fn new<P: PaletteRam>(
        vram: &Vec<u8>,
        cram: &Vec<u8>,
        palette: &P,
        position: &SpritePosition,
        config: &SpriteConfig,
    ) -> Self {
        let block_id = get_block_id(position);
        let sprite_id = get_sprite_id(&vram, position, config);
        let attr = get_attribute(&vram, position, config);
        let palette_id = (attr >> (block_id * 2)) & 0x03;
        let sprite = build(
            &cram,
            sprite_id,
            config.offset_addr_by_background_table,
            true,
        );
        Tile {
            sprite,
            palette: palette.get(palette_id, PaletteType::Background),
        }
    }
}
