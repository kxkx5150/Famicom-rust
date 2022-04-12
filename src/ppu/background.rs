use super::palette::*;
use super::sprite::*;
use super::tile::Tile;

#[derive(Debug)]
pub struct BackgroundCtx {
    pub tile: Tile,
    pub scroll_x: u8,
    pub scroll_y: u8,
    pub is_enabled: bool,
}

pub type BackgroundField = Vec<BackgroundCtx>;

#[derive(Debug)]
pub struct Background(pub BackgroundField);
impl Background {
    pub fn new() -> Self {
        Background(Vec::new())
    }

    pub fn clear(&mut self) {
        self.0 = Vec::new();
    }

    pub fn build_line<P: PaletteRam>(
        &mut self,
        vram: &Vec<u8>,
        cram: &Vec<u8>,
        palette: &P,
        tile: (u8, u8),
        scroll: (u8, u8),
        config: &mut SpriteConfig,
    ) {
        let clamped_tile_y = tile.1 % 30;
        let table_id_offset = if (tile.1 / 30) % 2 == 0 { 0 } else { 2 };
        for x in 0..(32 + 1) {
            let tile_x = x + tile.0;
            let clamped_tile_x = tile_x % 32;
            let name_table_id = ((tile_x / 32) % 2) + table_id_offset;
            config.offset_addr_ntable = Some((name_table_id as u16) * 0x400);
            let position: SpritePosition = (clamped_tile_x as u8, clamped_tile_y as u8);
            self.0.push(BackgroundCtx {
                tile: Tile::new(vram, cram, palette, &position, &config),
                scroll_x: scroll.0,
                scroll_y: scroll.1,
                is_enabled: config.is_bg_enable,
            });
        }
    }
}
