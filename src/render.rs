use crate::ppu::*;
// use super::PaletteList;
// use super::{BackgroundCtx, BackgroundField};
// use super::{Sprite, SpritePosition, SpritesWithCtx};

#[derive(Debug)]
pub struct Renderer {
    buf: Vec<u8>,
}
impl Renderer {
    pub fn new() -> Self {
        Renderer {
            buf: vec![0x00; 256 * 224 * 4],
        }
    }

    pub fn render(&mut self, background: &BackgroundField, sprites: &SpritesWithCtx) {
        self.render_background(background);
        self.render_sprites(sprites, background);
        unsafe {}
    }

    pub fn get_buf(&self) -> Vec<u8> {
        self.buf.clone()
    }

    fn should_pixel_hide(&self, x: usize, y: usize, background: &BackgroundField) -> bool {
        let tile_x = x / 8;
        let tile_y = y / 8;
        let background_index = tile_y * 33 + tile_x;
        let sprite = &background[background_index];
        (sprite.tile.sprite[y % 8][x % 8] % 4) != 0
    }

    fn render_background(&mut self, background: &BackgroundField) {
        for (i, bg) in background.into_iter().enumerate() {
            if bg.is_enabled {
                let x = (i % 33) * 8;
                let y = (i / 33) * 8;
                self.render_tile(bg, x, y);
            }
        }
    }

    fn render_sprites(&mut self, sprites: &SpritesWithCtx, background: &BackgroundField) {
        for sprite in sprites {
            self.render_sprite(
                &sprite.sprite,
                &sprite.position,
                &sprite.palette,
                sprite.attr,
                &background,
            );
        }
    }

    fn render_sprite(
        &mut self,
        sprite: &Sprite,
        position: &SpritePosition,
        palette: &PaletteList,
        attr: u8,
        background: &BackgroundField,
    ) {
        let is_vertical_reverse = (attr & 0x80) == 0x80;
        let is_horizontal_reverse = (attr & 0x40) == 0x40;
        let is_low_priority = (attr & 0x20) == 0x20;
        let h = sprite.len();
        for i in 0..h {
            let y = position.1 as usize + if is_vertical_reverse { h - 1 - i } else { i };
            if y >= 224 {
                continue;
            }
            for j in 0..8 {
                let x = position.0 as usize + if is_horizontal_reverse { 7 - j } else { j };
                if x >= 256 {
                    continue;
                }
                if is_low_priority && self.should_pixel_hide(x, y, background) {
                    continue;
                }
                if sprite[i][j] != 0 {
                    let color_id = palette[sprite[i][j] as usize];
                    let color = COLORS[color_id as usize];
                    let index = (x + (y * 0x100)) * 4;
                    self.buf[index] = color.0;
                    self.buf[index + 1] = color.1;
                    self.buf[index + 2] = color.2;
                    if x < 8 {
                        self.buf[index + 3] = 0;
                    }
                }
            }
        }
    }

    fn render_tile(&mut self, bg: &BackgroundCtx, x: usize, y: usize) {
        let offset_x = (bg.scroll_x % 8) as i32;
        let offset_y = (bg.scroll_y % 8) as i32;
        for i in 0..8 {
            for j in 0..8 {
                let x = (x + j) as i32 - offset_x;
                let y = (y + i) as i32 - offset_y;
                if x >= 0 as i32 && 0xFF >= x && y >= 0 as i32 && y < 224 {
                    let color_id = bg.tile.palette[bg.tile.sprite[i][j] as usize];
                    let color = COLORS[color_id as usize];
                    let index = ((x + (y * 0x100)) * 4) as usize;
                    self.buf[index] = color.0;
                    self.buf[index + 1] = color.1;
                    self.buf[index + 2] = color.2;
                    if x < 8 {
                        self.buf[index + 3] = 0;
                    }
                }
            }
        }
    }
}
pub static COLORS: &'static [(u8, u8, u8)] = &[
    (0x80, 0x80, 0x80),
    (0x00, 0x3D, 0xA6),
    (0x00, 0x12, 0xB0),
    (0x44, 0x00, 0x96),
    (0xA1, 0x00, 0x5E),
    (0xC7, 0x00, 0x28),
    (0xBA, 0x06, 0x00),
    (0x8C, 0x17, 0x00),
    (0x5C, 0x2F, 0x00),
    (0x10, 0x45, 0x00),
    (0x05, 0x4A, 0x00),
    (0x00, 0x47, 0x2E),
    (0x00, 0x41, 0x66),
    (0x00, 0x00, 0x00),
    (0x05, 0x05, 0x05),
    (0x05, 0x05, 0x05),
    (0xC7, 0xC7, 0xC7),
    (0x00, 0x77, 0xFF),
    (0x21, 0x55, 0xFF),
    (0x82, 0x37, 0xFA),
    (0xEB, 0x2F, 0xB5),
    (0xFF, 0x29, 0x50),
    (0xFF, 0x22, 0x00),
    (0xD6, 0x32, 0x00),
    (0xC4, 0x62, 0x00),
    (0x35, 0x80, 0x00),
    (0x05, 0x8F, 0x00),
    (0x00, 0x8A, 0x55),
    (0x00, 0x99, 0xCC),
    (0x21, 0x21, 0x21),
    (0x09, 0x09, 0x09),
    (0x09, 0x09, 0x09),
    (0xFF, 0xFF, 0xFF),
    (0x0F, 0xD7, 0xFF),
    (0x69, 0xA2, 0xFF),
    (0xD4, 0x80, 0xFF),
    (0xFF, 0x45, 0xF3),
    (0xFF, 0x61, 0x8B),
    (0xFF, 0x88, 0x33),
    (0xFF, 0x9C, 0x12),
    (0xFA, 0xBC, 0x20),
    (0x9F, 0xE3, 0x0E),
    (0x2B, 0xF0, 0x35),
    (0x0C, 0xF0, 0xA4),
    (0x05, 0xFB, 0xFF),
    (0x5E, 0x5E, 0x5E),
    (0x0D, 0x0D, 0x0D),
    (0x0D, 0x0D, 0x0D),
    (0xFF, 0xFF, 0xFF),
    (0xA6, 0xFC, 0xFF),
    (0xB3, 0xEC, 0xFF),
    (0xDA, 0xAB, 0xEB),
    (0xFF, 0xA8, 0xF9),
    (0xFF, 0xAB, 0xB3),
    (0xFF, 0xD2, 0xB0),
    (0xFF, 0xEF, 0xA6),
    (0xFF, 0xF7, 0x9C),
    (0xD7, 0xE8, 0x95),
    (0xA6, 0xED, 0xAF),
    (0xA2, 0xF2, 0xDA),
    (0x99, 0xFF, 0xFC),
    (0xDD, 0xDD, 0xDD),
    (0x11, 0x11, 0x11),
    (0x11, 0x11, 0x11),
];
