mod oam;
mod ppu_addr;
mod ppu_data;
mod ppu_scroll;

use self::oam::Oam;
use self::ppu_addr::Ppuu16;
use self::ppu_data::Ppuu8;
use self::ppu_scroll::PpuScroll;
use super::palette::*;
use super::PpuCtx;

#[derive(Debug)]
pub struct Registers {
    pub ppu_ctrl1: u8,
    pub ppu_ctrl2: u8,
    pub ppu_status: u8,
    pub oam: Oam,
    pub ppu_addr: Ppuu16,
    pub ppu_data: Ppuu8,
    pub ppu_scroll: PpuScroll,
}

pub trait PpuRegisters {
    fn read<P: PaletteRam>(&mut self, addr: u16, ctx: &mut PpuCtx<P>) -> u8;
    fn write<P: PaletteRam>(&mut self, addr: u16, data: u8, ctx: &mut PpuCtx<P>);
    fn is_sprite_8x8(&self) -> bool;
    fn clear_vblank(&mut self);
    fn set_vblank(&mut self);
    fn set_sprite_hit(&mut self);
    fn clear_sprite_hit(&mut self);
    fn get_sprite_table_offset(&self) -> u16;
    fn get_background_table_offset(&self) -> u16;
    fn get_ppu_addr_increment_value(&self) -> usize;
    fn get_name_table_id(&self) -> u8;
    fn get_scroll_x(&self) -> u8;
    fn get_scroll_y(&self) -> u8;
    fn is_irq_enable(&self) -> bool;
    fn is_background_enable(&self) -> bool;
    fn is_sprite_enable(&self) -> bool;
    fn is_background_masked(&self) -> bool;
    fn is_sprite_masked(&self) -> bool;
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            ppu_ctrl1: 0,
            ppu_ctrl2: 0,
            ppu_status: 0,
            oam: Oam::new(),
            ppu_addr: Ppuu16::new(),
            ppu_data: Ppuu8::new(),
            ppu_scroll: PpuScroll::new(),
        }
    }

    fn read_status(&mut self) -> u8 {
        let data = self.ppu_status;
        self.ppu_scroll.enable_x();
        self.clear_vblank();
        self.clear_sprite_hit();
        self.ppu_addr.reser_latch();
        data
    }

    fn write_oam_addr(&mut self, data: u8) {
        self.oam.write_addr(data);
    }

    fn write_oam_data(&mut self, data: u8, sprite_ram: &mut Vec<u8>) {
        self.oam.write_data(sprite_ram, data);
    }

    fn write_ppu_addr(&mut self, data: u8) {
        self.ppu_addr.write(data);
    }

    fn read_ppu_data<P: PaletteRam>(&mut self, vram: &Vec<u8>, cram: &Vec<u8>, palette: &P) -> u8 {
        let addr = self.ppu_addr.get();
        let data = self.ppu_data.read(vram, cram, addr, palette);
        let v = self.get_ppu_addr_increment_value() as u16;
        self.ppu_addr.update(v);
        data
    }

    fn write_ppu_data<P: PaletteRam>(
        &mut self,
        data: u8,
        vram: &mut Vec<u8>,
        cram: &mut Vec<u8>,
        palette: &mut P,
    ) {
        let addr = self.ppu_addr.get();
        self.ppu_data.write(vram, cram, addr, data, palette);
        let v = self.get_ppu_addr_increment_value() as u16;
        self.ppu_addr.update(v);
    }
}

impl PpuRegisters for Registers {
    fn clear_vblank(&mut self) {
        self.ppu_status &= 0x7F;
    }

    fn set_vblank(&mut self) {
        self.ppu_status |= 0x80;
    }

    fn is_sprite_8x8(&self) -> bool {
        self.ppu_ctrl1 & 0x20 != 0x20
    }

    fn clear_sprite_hit(&mut self) {
        self.ppu_status &= 0xbF;
    }

    fn set_sprite_hit(&mut self) {
        self.ppu_status |= 0x40;
    }

    fn get_ppu_addr_increment_value(&self) -> usize {
        if self.ppu_ctrl1 & 0x04 == 0x04 {
            32
        } else {
            1
        }
    }

    fn is_irq_enable(&self) -> bool {
        self.ppu_ctrl1 & 0x80 == 0x80
    }

    fn get_sprite_table_offset(&self) -> u16 {
        if self.ppu_ctrl1 & 0x08 == 0x08 {
            0x1000
        } else {
            0x0000
        }
    }

    fn get_background_table_offset(&self) -> u16 {
        if self.ppu_ctrl1 & 0x10 == 0x10 {
            0x1000
        } else {
            0x0000
        }
    }

    fn get_name_table_id(&self) -> u8 {
        self.ppu_ctrl1 & 0x03
    }

    fn get_scroll_x(&self) -> u8 {
        self.ppu_scroll.get_x()
    }

    fn get_scroll_y(&self) -> u8 {
        self.ppu_scroll.get_y()
    }

    fn is_background_enable(&self) -> bool {
        self.ppu_ctrl2 & 0x08 == 0x08
    }

    fn is_sprite_enable(&self) -> bool {
        self.ppu_ctrl2 & 0x10 == 0x10
    }

    fn is_background_masked(&self) -> bool {
        self.ppu_ctrl2 & 0x02 == 0x02
    }

    fn is_sprite_masked(&self) -> bool {
        self.ppu_ctrl2 & 0x04 == 0x04
    }

    fn read<P: PaletteRam>(&mut self, addr: u16, ctx: &mut PpuCtx<P>) -> u8 {
        match addr {
            0x0002 => self.read_status(),
            0x0004 => self.oam.read_data(&ctx.sprite_ram),
            0x0007 => self.read_ppu_data(&ctx.vram, &ctx.cram, &ctx.palette),
            _ => 0,
        }
    }
    fn write<P: PaletteRam>(&mut self, addr: u16, data: u8, ctx: &mut PpuCtx<P>) {
        match addr {
            0x0000 => self.ppu_ctrl1 = data,
            0x0001 => self.ppu_ctrl2 = data,
            0x0003 => self.write_oam_addr(data),
            0x0004 => self.write_oam_data(data, &mut ctx.sprite_ram),
            0x0005 => self.ppu_scroll.write(data),
            0x0006 => self.write_ppu_addr(data),
            0x0007 => self.write_ppu_data(data, &mut ctx.vram, &mut ctx.cram, &mut ctx.palette),
            _ => (),
        }
    }
}
