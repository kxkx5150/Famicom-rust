pub mod background;
mod palette;
mod registers;
pub mod render;
mod sprite;
mod sprite_utils;
pub mod tile;

pub use self::background::*;
pub use self::palette::*;
use self::registers::*;
pub use self::sprite::*;
pub use self::sprite_utils::*;
pub use self::tile::*;
use crate::rom::Mirroring;

#[derive(Debug)]
pub struct PpuConfig {
    pub is_horizontal_mirror: bool,
}

#[derive(Debug)]
pub struct PpuCtx<P: PaletteRam> {
    pub palette: P,
    pub vram: Box<Vec<u8>>,
    pub cram: Box<Vec<u8>>,
    pub sprite_ram: Box<Vec<u8>>,
}

#[derive(Debug)]
pub struct Ppu {
    pub cycle: usize,
    pub line: usize,
    pub registers: Registers,
    pub ctx: PpuCtx<Palette>,
    pub sprites: SpritesWithCtx,
    pub background: Background,
    pub mirroring: Mirroring,
    pub nmi: bool,
}

impl Ppu {
    pub fn new_empty_rom() -> Self {
        Self::new(vec![0; 2048], Mirroring::HORIZONTAL)
    }
    pub fn new(cram: Vec<u8>, mirroring: Mirroring) -> Ppu {
        Ppu {
            cycle: 0,
            line: 0,
            registers: Registers::new(),
            ctx: PpuCtx {
                palette: Palette::new(),
                vram: Box::new(vec![0; 0x2000]),
                cram: Box::new(cram),
                sprite_ram: Box::new(vec![0; 0x0100]),
            },
            sprites: Vec::new(),
            background: Background::new(),
            mirroring,
            nmi: false,
        }
    }
    pub fn set_rom(&mut self, cram: Vec<u8>, mirroring: Mirroring) {
        self.ctx.cram = Box::new(cram);
        self.mirroring = mirroring;
        println!("load ppu rom");
    }
    pub fn get_nmi_status(&self) -> bool {
        self.nmi
    }
    pub fn clear_nmi(&mut self) {
        self.nmi = false;
    }
    pub fn read(&mut self, addr: u16) -> u8 {
        self.registers.read(addr, &mut self.ctx)
    }
    pub fn write(&mut self, addr: u16, data: u8) {
        self.registers.write(addr, data, &mut self.ctx);
    }
    pub fn run(&mut self, cycle: usize) -> bool {
        let cycle = self.cycle + cycle;
        if cycle < 341 {
            self.cycle = cycle;
            return false;
        }

        if self.line == 0 {
            self.background.clear();
        }

        if self.has_sprite_hit(cycle) {
            self.registers.set_sprite_hit();
        }

        self.cycle = cycle - 341;
        self.line = self.line + 1;

        let scroll_x = self.registers.get_scroll_x();
        let scroll_y = self.registers.get_scroll_y();
        if self.line <= 240 && self.line % 8 == 0 && scroll_y <= 240 {
            let hmirr = match &self.mirroring {
                VERTICAL => false,
                HORIZONTAL => true,
                FOUR_SCREEN => false,
            };
            let mut config = SpriteConfig {
                offset_addr_by_name_table: None,
                offset_addr_by_background_table: self.registers.get_background_table_offset(),
                offset_addr_by_sprite_table: self.registers.get_sprite_table_offset(),
                is_horizontal_mirror: hmirr,
                is_background_enable: self.registers.is_background_enable(),
            };
            let tile_x = ((scroll_x as usize
                + (self.registers.get_name_table_id() % 2) as usize * 256)
                / 8) as u8;
            let tile_y = self.get_scroll_tile_y();
            self.background.build_line(
                &self.ctx.vram,
                &self.ctx.cram,
                &self.ctx.palette,
                (tile_x, tile_y),
                (scroll_x, scroll_y),
                &mut config,
            );
        }

        if self.line == 241 {
            self.nmi = true;
            self.registers.set_vblank();
            self.registers.clear_sprite_hit();
            if self.registers.is_irq_enable() {}
        }

        if self.line >= 262 {
            self.registers.clear_vblank();
            self.registers.clear_sprite_hit();
            // self.nmi = false;
            self.line = 0;
            self.sprites = build_sprites(
                &self.ctx.cram,
                &self.ctx.sprite_ram,
                &self.ctx.palette,
                self.registers.get_sprite_table_offset(),
                self.registers.is_sprite_8x8(),
            );
            return true;
        }
        false
    }

    pub fn transfer_sprite(&mut self, addr: u16, data: u8) {
        let addr = addr + self.registers.oam.get_addr();
        self.ctx.sprite_ram[(addr % 0x100) as usize] = data;
    }

    fn get_scroll_tile_y(&self) -> u8 {
        ((self.registers.get_scroll_y() as usize
            + self.line
            + ((self.registers.get_name_table_id() / 2) as usize * 240))
            / 8) as u8
    }

    fn has_sprite_hit(&self, cycle: usize) -> bool {
        let y = self.ctx.sprite_ram[0] as usize;
        let x = self.ctx.sprite_ram[3] as usize;
        (y == self.line) && x <= cycle && self.registers.is_sprite_enable()
    }
}
