use std::borrow::Borrow;
use std::borrow::BorrowMut;

use crate::ppu;
use crate::render;
use crate::rom;

pub mod registers;
use render::bg_pallette;
use render::frame::Frame;
use render::palette;
use render::render;
use render::render_tile;

use ppu::registers::addr::AddrRegister;
use ppu::registers::control::ControlRegister;
use ppu::registers::mask::MaskRegister;
use ppu::registers::scroll::ScrollRegister;
use ppu::registers::status::StatusRegister;
use rom::Mirroring;

pub struct NesPPU {
    pub chr_rom: Vec<u8>,
    pub mirroring: Mirroring,
    pub ctrl: ControlRegister,
    pub mask: MaskRegister,
    pub status: StatusRegister,
    pub scroll: ScrollRegister,
    pub addr: AddrRegister,
    pub vram: [u8; 2048],

    pub oam_addr: u8,
    pub oam_data: [u8; 256],
    pub palette_table: [u8; 32],

    internal_data_buf: u8,

    pub scanline: u16,
    cycles: usize,
    pub nmi_interrupt: Option<u8>,
    scroll_info: Vec<(usize, usize)>,
    pub frame: Frame,
}

pub trait PPU {
    fn write_to_ctrl(&mut self, value: u8);
    fn write_to_mask(&mut self, value: u8);
    fn read_status(&mut self) -> u8;
    fn write_to_oam_addr(&mut self, value: u8);
    fn write_to_oam_data(&mut self, value: u8);
    fn read_oam_data(&self) -> u8;
    fn write_to_scroll(&mut self, value: u8);
    fn write_to_ppu_addr(&mut self, value: u8);
    fn write_to_data(&mut self, value: u8);
    fn read_data(&mut self) -> u8;
    fn write_oam_dma(&mut self, value: &[u8; 256]);
}

impl NesPPU {
    pub fn new_empty_rom() -> Self {
        Self::new(vec![0; 2048], Mirroring::HORIZONTAL)
    }
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        let mut s = Frame::new();
        Self {
            chr_rom: chr_rom,
            mirroring: mirroring,
            ctrl: ControlRegister::new(),
            mask: MaskRegister::new(),
            status: StatusRegister::new(),
            oam_addr: 0,
            scroll: ScrollRegister::new(),
            addr: AddrRegister::new(),
            vram: [0; 2048],
            oam_data: [0; 64 * 4],
            palette_table: [0; 32],
            internal_data_buf: 0,

            cycles: 0,
            scanline: 0,
            nmi_interrupt: None,
            scroll_info: (0..240).map(|x| (0, 0)).collect(),
            frame: s,
        }
    }
    pub fn set_rom(&mut self, chr_rom: Vec<u8>, mirroring: Mirroring) {
        self.chr_rom = chr_rom;
        self.mirroring = mirroring;
        println!("load ppu rom");
    }
    pub fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0b10111111111111;
        let vram_index = mirrored_vram - 0x2000;
        let name_table = vram_index / 0x400;
        match (&self.mirroring, name_table) {
            (Mirroring::VERTICAL, 2) | (Mirroring::VERTICAL, 3) => vram_index - 0x800,
            (Mirroring::HORIZONTAL, 2) => vram_index - 0x400,
            (Mirroring::HORIZONTAL, 1) => vram_index - 0x400,
            (Mirroring::HORIZONTAL, 3) => vram_index - 0x800,
            _ => vram_index,
        }
    }

    fn increment_vram_addr(&mut self) {
        self.addr.increment(self.ctrl.vram_addr_increment());
    }

    pub fn tick(&mut self, cycles: u8) -> bool {
        self.cycles += cycles as usize;
        if self.cycles >= 341 {
            if self.is_sprite_0_hit(self.cycles) {
                self.status.set_sprite_zero_hit(true);
            }
            self.cycles = self.cycles - 341;
            self.scanline += 1;

            if self.scanline <= 240 && self.scroll.scroll_y <= 240 {
                self.build_bg_line();
            }
            if self.scanline == 241 {
                self.status.set_vblank_status(true);
                self.status.set_sprite_zero_hit(false);
                if self.ctrl.generate_vblank_nmi() {
                    self.nmi_interrupt = Some(1);
                }
            } else if self.scanline >= 262 {
                self.scanline = 0;
                self.nmi_interrupt = None;
                self.status.set_sprite_zero_hit(false);
                self.status.reset_vblank_status();
            }
        }
        return false;
    }
    pub fn render(&mut self){

    }
    pub fn build_bg_line(&mut self) {
        let scroll_x = (self.scroll.scroll_x) as usize;
        let scroll_y = (self.scroll.scroll_y) as usize;

        let main_nametable: &[u8];
        let second_nametable: &[u8];

        if self.scanline % 8 == 0 {
            (main_nametable, second_nametable) = match (&self.mirroring, self.ctrl.nametable_addr())
            {
                (Mirroring::VERTICAL, 0x2000)
                | (Mirroring::VERTICAL, 0x2800)
                | (Mirroring::HORIZONTAL, 0x2000)
                | (Mirroring::HORIZONTAL, 0x2400) => {
                    (&self.vram[0..0x400], &self.vram[0x400..0x800])
                }
                (Mirroring::VERTICAL, 0x2400)
                | (Mirroring::VERTICAL, 0x2C00)
                | (Mirroring::HORIZONTAL, 0x2800)
                | (Mirroring::HORIZONTAL, 0x2C00) => {
                    (&self.vram[0x400..0x800], &self.vram[0..0x400])
                }
                (_, _) => {
                    panic!("Not supported mirroring type {:?}", self.mirroring);
                }
            };

            let tile_row = (self.scanline / 8) as usize;
            let opts = render_tile(
                &self,
                main_nametable,
                render::Rect::new(scroll_x, scroll_y, 256, 240),
                -(scroll_x as isize),
                -(scroll_y as isize),
                tile_row-1,
            );
            if -1 < opts.0 {
                self.frame.set_pixel(opts.0 as usize, opts.1 as usize, opts.2);
            }
            // if scroll_x > 0 {
            //     render_tile(
            //         ppu,
            //         frame,
            //         second_nametable,
            //         Rect::new(0, 0, scroll_x, 240),
            //         (256 - scroll_x) as isize,
            //         0,
            //     );
            // } else if scroll_y > 0 {
            //     render_tile(
            //         ppu,
            //         frame,
            //         second_nametable,
            //         Rect::new(0, 0, 256, scroll_y),
            //         0,
            //         (240 - scroll_y) as isize,
            //     );
            // }
        }
    }

    pub fn clear_nmi(&mut self) {
        self.nmi_interrupt = None;
    }
    pub fn poll_nmi_interrupt(&mut self) -> Option<u8> {
        self.nmi_interrupt.take()
    }
    fn is_sprite_0_hit(&self, cycle: usize) -> bool {
        let y = self.oam_data[0] as usize;
        let x = self.oam_data[3] as usize;
        (y == self.scanline as usize) && x <= cycle && self.mask.show_sprites()
    }
}

impl PPU for NesPPU {
    fn write_to_ctrl(&mut self, value: u8) {
        let before_nmi_status = self.ctrl.generate_vblank_nmi();
        self.ctrl.update(value);
        if !before_nmi_status && self.ctrl.generate_vblank_nmi() && self.status.is_in_vblank() {
            self.nmi_interrupt = Some(1);
        }
    }

    fn write_to_mask(&mut self, value: u8) {
        self.mask.update(value);
    }

    fn read_status(&mut self) -> u8 {
        let data = self.status.snapshot();
        self.status.reset_vblank_status();
        self.addr.reset_latch();
        self.scroll.reset_latch();
        data
    }

    fn write_to_oam_addr(&mut self, value: u8) {
        self.oam_addr = value;
    }

    fn write_to_oam_data(&mut self, value: u8) {
        self.oam_data[self.oam_addr as usize] = value;

        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    fn read_oam_data(&self) -> u8 {
        self.oam_data[self.oam_addr as usize]
    }

    fn write_to_scroll(&mut self, value: u8) {
        self.scroll.write(value);
    }

    fn write_to_ppu_addr(&mut self, value: u8) {
        self.addr.update(value);
    }

    fn write_to_data(&mut self, value: u8) {
        let addr = self.addr.get();
        match addr {
            0..=0x1fff => println!("attempt to write to chr rom space {}", addr),
            0x2000..=0x2fff => {
                self.vram[self.mirror_vram_addr(addr) as usize] = value;
            }
            0x3000..=0x3eff => unimplemented!("addr {} shouldn't be used in reallity", addr),

            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => {
                let add_mirror = addr - 0x10;
                self.palette_table[(add_mirror - 0x3f00) as usize] = value;
            }
            0x3f00..=0x3fff => {
                self.palette_table[(addr - 0x3f00) as usize] = value;
            }
            _ => panic!("unexpected access to mirrored space {}", addr),
        }
        self.increment_vram_addr();
    }

    fn read_data(&mut self) -> u8 {
        let addr = self.addr.get();

        self.increment_vram_addr();

        match addr {
            0..=0x1fff => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.chr_rom[addr as usize];
                result
            }
            0x2000..=0x2fff => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            }
            0x3000..=0x3eff => unimplemented!("addr {} shouldn't be used in reallity", addr),

            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => {
                let add_mirror = addr - 0x10;
                self.palette_table[(add_mirror - 0x3f00) as usize]
            }

            0x3f00..=0x3fff => self.palette_table[(addr - 0x3f00) as usize],
            _ => panic!("unexpected access to mirrored space {}", addr),
        }
    }

    fn write_oam_dma(&mut self, data: &[u8; 256]) {
        for x in data.iter() {
            self.oam_data[self.oam_addr as usize] = *x;
            self.oam_addr = self.oam_addr.wrapping_add(1);
        }
    }
}
