pub mod background;
mod palette;
mod registers;
mod sprite;
pub mod tile;

pub use self::background::*;
pub use self::palette::*;
use self::registers::*;
pub use self::sprite::*;
pub use self::tile::*;
use crate::rom::Mirroring;

#[derive(Debug)]
pub struct PpuConfig {
    pub is_hmirror: bool,
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
    pub regs: Registers,
    pub ctx: PpuCtx<Palette>,
    pub sprites: SpritesWithCtx,
    pub background: Background,
    pub mirroring: Mirroring,
    pub nmi: bool,
    pub okimg: bool,
}

impl Ppu {
    pub fn new_empty_rom() -> Self {
        Self::new(vec![0; 2048], Mirroring::HORIZONTAL)
    }
    pub fn new(cram: Vec<u8>, mirroring: Mirroring) -> Ppu {
        Ppu {
            cycle: 0,
            line: 0,
            regs: Registers::new(),
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
            okimg:false,
        }
    }
    pub fn reset(&mut self){
        self.nmi = false;
        self.okimg = false;
        self.cycle = 341;
        self.line = 0;
    }
    pub fn set_rom(&mut self, cram: Vec<u8>, mirroring: Mirroring) {
        self.reset();
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
    pub fn get_img(&self) -> bool{
        self.okimg
    }
    pub fn clear_okimg(&mut self) {
        self.okimg = false;
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        self.regs.read(addr, &mut self.ctx)
    }
    pub fn write(&mut self, addr: u16, data: u8) {
        self.regs.write(addr, data, &mut self.ctx);
    }
    pub fn run(&mut self, cycle: usize) -> bool {
        let cycle = self.cycle + cycle;
        if cycle <= 341 {
            self.cycle = cycle;
            return false;
        }

        if self.line == 0 {
            self.background.clear();
            self.okimg = false;
        }

        if self.has_sprite_hit(cycle) {
            self.regs.set_sprite_hit();
        }

        self.cycle = cycle - 341;
        self.line = self.line + 1;

        let scroll_x = self.regs.get_scroll_x();
        let scroll_y = self.regs.get_scroll_y();
        if self.line <= 240 && self.line % 8 == 0 && scroll_y <= 240 {
            let hmirr = match &self.mirroring {
                VERTICAL => false,
                HORIZONTAL => true,
                FOUR_SCREEN => false,
            };

            let mut config = SpriteConfig {
                offset_addr_ntable: None,
                offset_addr_bgtable: self.regs.get_bg_table_offset(),
                offset_addr_sptable: self.regs.get_sp_table_offset(),
                is_hmirror: hmirr,
                is_bg_enable: self.regs.is_bg_enable(),
            };
            let tile_x = ((scroll_x as usize
                + (self.regs.get_name_table_id() % 2) as usize * 256)
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
        }else if self.line == 241 {
            self.regs.set_vblank();
            self.regs.clear_sprite_hit();
            if self.regs.is_irq_enable() {
                self.nmi = true;
            }
        }else if self.line >= 262 {
            self.regs.clear_vblank();
            self.regs.clear_sprite_hit();
            self.nmi = false;
            self.line = 0;
            self.sprites = build_sprites(
                &self.ctx.cram,
                &self.ctx.sprite_ram,
                &self.ctx.palette,
                self.regs.get_sp_table_offset(),
                self.regs.is_sprite_8x8(),
            );
            self.okimg = true;
            return true;
        }
        false
    }

    pub fn transfer_sprite(&mut self, addr: u16, data: u8) {
        let addr = addr + self.regs.oam.get_addr();
        self.ctx.sprite_ram[(addr % 0x100) as usize] = data;
    }

    fn get_scroll_tile_y(&self) -> u8 {
        ((self.regs.get_scroll_y() as usize
            + self.line
            + ((self.regs.get_name_table_id() / 2) as usize * 240))
            / 8) as u8
    }

    fn has_sprite_hit(&self, cycle: usize) -> bool {
        let y = self.ctx.sprite_ram[0] as usize;
        let x = self.ctx.sprite_ram[3] as usize;
        (y == self.line) && x <= cycle && self.regs.is_sprite_enable()
    }
}
