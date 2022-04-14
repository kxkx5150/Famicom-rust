use crate::irq;
use crate::rom;
use rom::Mirroring;

pub struct Ppu {
    ppux: usize,
    line: usize,
    regs: Vec<u8>,
    pub imgdata: Vec<u8>,
    imgok: bool,
    imgidx: usize,
    rcount: usize,

    sprite_zero: bool,
    scroll_reg_flg: bool,
    ppu_addr_buffer: usize,
    h_scroll_val: usize,
    ppu_addr_reg_flg: bool,
    ppu_addr: usize,
    ppu_read_buffer: usize,

    screen_mirroring: Mirroring,
    vram: Vec<Vec<u8>>,
    vrams: Vec<Vec<u8>>,

    bg_line_buffer: Vec<u8>,
    sp_line_buffer: Vec<u16>,

    palette: Vec<u8>,
    pub sprite_ram: Vec<u8>,
    spbit_pattern: Vec<Vec<Vec<u8>>>,
}
pub trait Port {
    fn write_scroll_reg(&mut self, value: u8);

    fn write_ppu_ctrl0_reg(&mut self, value: u8);
    fn write_ppu_ctrl1_reg(&mut self, value: u8);

    fn read_ppu_status_reg(&mut self) -> u8;
    fn write_ppu_addr_reg(&mut self, value: u8);

    fn read_ppu_data_reg(&mut self) -> u8;
    fn write_ppu_data_reg(&mut self, value: u8);

    fn write_sprite_data(&mut self, value: u8);
    fn write_sprite_addr_reg(&mut self, value: u8);
}
impl Ppu {
    pub fn new() -> Self {
        Self {
            ppux: 341,
            line: 0,
            regs: (0..8).map(|x| 0).collect(),
            imgdata: vec![0; 256 * 240 * 3],
            imgok: false,
            imgidx: 0,
            rcount: 0,

            sprite_zero: false,
            scroll_reg_flg: false,
            ppu_addr_buffer: 0,
            h_scroll_val: 0,
            ppu_addr_reg_flg: false,
            ppu_addr: 0,
            ppu_read_buffer: 0,
            screen_mirroring: Mirroring::HORIZONTAL,

            vram: vec![vec![0; 4096]; 16],
            vrams: vec![vec![0; 1024]; 16],

            bg_line_buffer: (0..264).map(|x| 0).collect(),
            sp_line_buffer: (0..264).map(|x| 0).collect(),

            palette: (0..33).map(|x| 0x0f).collect(),
            sprite_ram: (0..0x100).map(|x| 0).collect(),
            spbit_pattern: vec![vec![vec![0; 8]; 256]; 256],
        }
    }
    pub fn init(&mut self) {
        self.reset();
    }
    pub fn start(&mut self, rom: &mut rom::Rom) {
        println!("ppu start");
        self.crate_spbit_array();
        self.palette = [0x0f; 33].to_vec();
        self.sprite_ram = [0; 0x100].to_vec();
        self.bg_line_buffer = [0; 264].to_vec();
        self.sp_line_buffer = [0; 264].to_vec();

        self.screen_mirroring = rom.screen_mirroring.clone();
        match self.screen_mirroring {
            Mirroring::VERTICAL => {
                self.set_mode_mirror(false, rom);
            }
            Mirroring::HORIZONTAL => {
                self.set_mode_mirror(true, rom);
            }
            Mirroring::FOUR_SCREEN => {
                self.init_mirrors(0, 1, 2, 3, rom);
            }
        }

        self.ppux = 341;
        self.line = 0;
        self.sprite_zero = false;
        self.imgok = false;
    }
    pub fn reset(&mut self) {
        self.imgidx = 0;
        self.scroll_reg_flg = false;
        self.ppu_addr_reg_flg = false;
        self.ppu_addr_buffer = 0;
        self.ppu_read_buffer = 0;
        self.ppu_addr = 0;
        self.h_scroll_val = 0;
        self.ppux = 341;
        self.line = 0;
        self.sprite_zero = false;
        self.imgok = false;
        self.clear_arryas();
    }
    fn crate_spbit_array(&mut self) {
        for i in 0..256 {
            for j in 0..256 {
                for k in 0..8 {
                    let lval = (((i << k) & 0x80) >> 7);
                    let rval = (((j << k) & 0x80) >> 6);
                    let val = (lval | rval);
                    self.spbit_pattern[i][j][k] = val as u8;
                }
            }
        }
    }
    fn clear_arryas(&mut self) {
        // self.regs = (0..8).map(|x| 0).collect();
        // self.vram = vec![vec![0; 4096]; 16];
        // self.vrams = vec![vec![0; 1024]; 16];
        // self.bg_line_buffer = (0..264).map(|x| 0).collect();
        // self.sp_line_buffer = (0..264).map(|x| 0).collect();
        // self.palette = (0..33).map(|x| 0x0f).collect();
        // self.sprite_ram =  (0..0x100).map(|x| 0).collect();
    }
    fn set_mode_mirror(&mut self, value: bool, rom: &mut rom::Rom) {
        if (value) {
            self.init_mirrors(0, 0, 1, 1, rom);
        } else {
            self.init_mirrors(0, 1, 0, 1, rom);
        }
    }
    fn init_mirrors(
        &mut self,
        value0: isize,
        value1: isize,
        value2: isize,
        value3: isize,
        rom: &mut rom::Rom,
    ) {
        self.set_chr_rom_data1k(8, value0 + 8 + 0x0100, rom);
        self.set_chr_rom_data1k(9, value1 + 8 + 0x0100, rom);
        self.set_chr_rom_data1k(10, value2 + 8 + 0x0100, rom);
        self.set_chr_rom_data1k(11, value3 + 8 + 0x0100, rom);
    }
    fn set_chr_rom_data1k(&mut self, mut page: isize, romPage: isize, rom: &mut rom::Rom) {
        if (romPage >= 0x0100) {
            rom.chrrom_state[page as usize] = romPage as u8;
            self.vram[page as usize] = self.vrams[(romPage & 0xff) as usize].to_vec();
        } else {
            if (rom.chr_rom_page_count > 0) {
                let tmp = romPage % (rom.chr_rom_page_count as isize * 8);
                rom.chrrom_state[page as usize] = tmp as u8;
                self.vram[page as usize] =
                    rom.chrrom_pages[rom.chrrom_state[page as usize] as usize].to_vec();
            }
        }
    }
    fn set_chrrom_pages1k(
        &mut self,
        rompage0: isize,
        rompage1: isize,
        rompage2: isize,
        rompage3: isize,
        rompage4: isize,
        rompage5: isize,
        rompage6: isize,
        rompage7: isize,
        rom: &mut rom::Rom,
    ) {
        self.set_chr_rom_data1k(0, rompage0, rom);
        self.set_chr_rom_data1k(1, rompage1, rom);
        self.set_chr_rom_data1k(2, rompage2, rom);
        self.set_chr_rom_data1k(3, rompage3, rom);
        self.set_chr_rom_data1k(4, rompage4, rom);
        self.set_chr_rom_data1k(5, rompage5, rom);
        self.set_chr_rom_data1k(6, rompage6, rom);
        self.set_chr_rom_data1k(7, rompage7, rom);
    }
    pub fn set_chr_rom_page(&mut self, mut num: isize, rom: &mut rom::Rom) {
        num <<= 3;
        for i in 0..8 {
            self.set_chr_rom_data1k(i, num + i, rom);
        }
    }
    pub fn run(&mut self, cpuclock: usize, irq: &mut irq::Irq) {
        let mut tmpx = self.ppux;
        self.ppux += cpuclock * 3;

        while (341 <= self.ppux) {
            self.ppux -= 341;
            self.line += 1;
            tmpx = 0;
            self.sprite_zero = false;

            if self.line < 240 {
                self.render_frame();
            } else if self.line == 240 {
                self.in_vblank(irq);
                continue;
            } else if self.line == 262 {
                self.post_render();
            }
        }

        if (self.sprite_zero && (self.regs[0x02] & 0x40) != 0x40) {
            let i = if self.ppux > 255 { 255 } else { self.ppux };
            while tmpx <= i {
                if (self.sp_line_buffer[tmpx] == 0) {
                    self.regs[0x02] |= 0x40;
                    break;
                }
                tmpx += 1;
            }
        }
    }
    fn render_frame(&mut self) {
        if self.is_screen_enable() || self.is_sprite_enable() {
            self.ppu_addr = (self.ppu_addr & 0xfbe0) | (self.ppu_addr_buffer & 0x041f);

            if (8 <= self.line && self.line < 232) {
                self.build_bg();
                self.build_sp_line();
                for p in (0..256) {
                    let idx = self.palette[self.bg_line_buffer[p] as usize];
                    let pal = PALLETE_TABLE[idx as usize];
                    self.set_img_data(pal);
                }
            } else {
                for p in (0..264) {
                    self.bg_line_buffer[p] = 0x10;
                }
                self.build_sp_line();
            }

            if ((self.ppu_addr & 0x7000) == 0x7000) {
                self.ppu_addr &= 0x8fff;

                if ((self.ppu_addr & 0x03e0) == 0x03a0) {
                    self.ppu_addr = (self.ppu_addr ^ 0x0800) & 0xfc1f;
                } else if ((self.ppu_addr & 0x03e0) == 0x03e0) {
                    self.ppu_addr &= 0xfc1f;
                } else {
                    self.ppu_addr += 0x0020;
                }
            } else {
                self.ppu_addr += 0x1000;
            }
        } else if (8 <= self.line && self.line < 232) {
            let pal = PALLETE_TABLE[self.palette[0x10] as usize];
            for x in (0..256) {
                self.set_img_data(pal);
            }
        }
    }
    fn build_bg(&mut self) {
        if ((self.regs[0x01] & 0x08) != 0x08) {
            for p in 0..264 {
                self.bg_line_buffer[p] = 0x10;
            }
            return;
        }
        self.build_bg_line();
        if ((self.regs[0x01] & 0x02) != 0x02) {
            for x in 0..8 {
                self.bg_line_buffer[x] = 0x10;
            }
        }
    }
    fn build_bg_line(&mut self) {
        let nameaddr = 0x2000 | (self.ppu_addr & 0x0fff);
        let tableaddr =
            ((self.ppu_addr & 0x7000) >> 12) | (((self.regs[0x00] & 0x10) as usize) << 8);
        let mut name_addr_h = nameaddr >> 10;
        let mut name_addr_l = nameaddr & 0x03ff;
        let mut pre_name_addrh = name_addr_h;
        let mut s = self.h_scroll_val;
        let mut q = 0;

        for p in 0..33 {
            let vram = &self.vram[pre_name_addrh];
            let mut ptndist = ((vram[name_addr_l] as usize) << 4) | tableaddr;
            let vvram = &self.vram[(ptndist >> 10) as usize];
            ptndist &= 0x03ff;

            let lval = (name_addr_l & 0x0380) >> 4;
            let rval = ((name_addr_l & 0x001c) >> 2) + 0x03c0;

            let lval2 = (name_addr_l & 0x0040) >> 4;
            let rval2 = name_addr_l & 0x0002;
            let attr = (((vram[lval | rval] as usize) << 2) >> (lval2 | rval2)) & 0x0c;

            let spbidx1 = vvram[ptndist as usize];
            let spbidx2 = vvram[(ptndist + 8) as usize];
            let ptn = &self.spbit_pattern[spbidx1 as usize][spbidx2 as usize];

            while s < 8 {
                let idx = ptn[s] | attr as u8;
                self.bg_line_buffer[q] = PALLETE[idx as usize];
                q += 1;
                s += 1;
            }
            s = 0;

            if ((name_addr_l & 0x001f) == 0x001f) {
                name_addr_l &= 0xffe0;
                name_addr_h ^= 0x01;
                pre_name_addrh = name_addr_h;
            } else {
                name_addr_l += 1;
            }
        }
    }
    fn build_sp_line(&mut self) {
        let spclip = if (self.regs[0x01] & 0x04) == 0x04 {
            0
        } else {
            8
        };

        if ((self.regs[0x01] & 0x10) == 0x10) {
            for p in 0..264 {
                self.sp_line_buffer[p] = 256;
            }

            let spptableaddr = ((self.regs[0x00] & 0x08) as usize) << 9;
            let mut count = 0;
            let bzsize = self.is_bigsize();

            for i in (0..=252).step_by(4) {
                let isy = (self.sprite_ram[i] as usize + 1);
                if isy > self.line || (isy + bzsize <= self.line) {
                    continue;
                }

                if (i == 0) {
                    self.sprite_zero = true;
                }

                count += 1;
                if (count == 9) {
                    break;
                }

                let attr = self.sprite_ram[i + 2];
                let attribute = (((attr & 0x03) as usize) << 2) | 0x10;
                let bgsp = (attr & 0x20) == 0x00;

                let mut x = (self.sprite_ram[i + 3]) as usize;
                let mut ex = x + 8;
                if ex > 256 {
                    ex = 256;
                }

                let iy = if (attr & 0x80) == 0x80 {
                    bzsize - 1 - (self.line - isy)
                } else {
                    self.line - isy
                };

                let lval = ((self.sprite_ram[i + 1] as usize) << 4) + spptableaddr;
                let rval = ((self.sprite_ram[i + 1] as usize & 0xfe) << 4)
                    + ((self.sprite_ram[i + 1] as usize & 0x01) << 12);
                let sval = if bzsize == 8 { lval } else { rval };
                let tilenum = ((iy & 0x08) << 1) + (iy & 0x07) + sval;
                let tlow = tilenum & 0x03ff;

                let mut is: isize;
                let ia: isize;
                if ((attr & 0x40) == 0x00) {
                    is = 0;
                    ia = 1;
                } else {
                    is = 7;
                    ia = -1;
                }

                let ptnidxl = self.vram[tilenum >> 10][tlow];
                let ptnidxr = self.vram[tilenum >> 10][tlow + 8];
                let ptn = &self.spbit_pattern[ptnidxl as usize][ptnidxr as usize];

                while x < ex {
                    let tptn = ptn[is as usize];
                    if (tptn != 0x00 && (self.sp_line_buffer[x as usize] == 256)) {
                        self.sp_line_buffer[x as usize] = i as u16;
                        if (x >= spclip && (bgsp || self.bg_line_buffer[x as usize] == 0x10)) {
                            self.bg_line_buffer[x as usize] = tptn | attribute as u8;
                        }
                    }
                    x += 1;
                    is += ia;
                }
            }

            if (8 <= count) {
                self.regs[0x02] |= 0x20;
            } else {
                self.regs[0x02] &= 0xdf;
            }
        }
    }

    fn in_vblank(&mut self, irq: &mut irq::Irq) {
        self.scroll_reg_flg = false;
        self.regs[0x02] &= 0x1f;
        self.regs[0x02] |= 0x80;
        if ((self.regs[0x00] & 0x80) == 0x80) {
            irq.set_nmi(true);
        }
    }
    fn post_render(&mut self) {
        self.line = 0;
        if (self.is_screen_enable() || self.is_sprite_enable()) {
            self.ppu_addr = self.ppu_addr_buffer;
        }
        self.regs[0x02] &= 0x7f;
        self.imgok = true;
    }
    fn set_img_data(&mut self, plt: (u8, u8, u8)) {
        self.imgdata[self.imgidx] = plt.0;
        self.imgdata[self.imgidx + 1] = plt.1;
        self.imgdata[self.imgidx + 2] = plt.2;
        self.imgidx += 3;
    }
    pub fn clear_img(&mut self) {
        self.imgidx = 0;
        self.imgok = false;
    }
    pub fn get_img_status(&mut self) -> (bool, &Vec<u8>) {
        if self.imgok {
            return (true, &self.imgdata);
        } else {
            return (false, &self.imgdata);
        }
    }
    fn is_screen_enable(&mut self) -> bool {
        return (self.regs[0x01] & 0x08) == 0x08;
    }
    fn is_sprite_enable(&mut self) -> bool {
        return (self.regs[0x01] & 0x10) == 0x10;
    }
    fn is_bigsize(&mut self) -> usize {
        let val = (if (self.regs[0x00] & 0x20) == 0x20 {
            return 16;
        } else {
            return 8;
        });
    }
}
impl Port for Ppu {
    fn write_scroll_reg(&mut self, value: u8) {
        self.regs[0x05] = value;
        if (self.scroll_reg_flg) {
            self.ppu_addr_buffer = (self.ppu_addr_buffer & 0x8c1f)
                | ((value as usize & 0xf8) << 2)
                | ((value as usize & 0x07) << 12);
        } else {
            self.ppu_addr_buffer = (self.ppu_addr_buffer & 0xffe0) | ((value as usize & 0xf8) >> 3);
            self.h_scroll_val = value as usize & 7;
        }
        self.scroll_reg_flg = !self.scroll_reg_flg;
    }
    fn write_ppu_ctrl0_reg(&mut self, value: u8) {
        self.regs[0x00] = value;
        self.ppu_addr_buffer = (self.ppu_addr_buffer & 0xf3ff) | (((value & 0x03) as usize) << 10);
    }
    fn write_ppu_ctrl1_reg(&mut self, value: u8) {
        self.regs[0x01] = value;
    }
    fn read_ppu_status_reg(&mut self) -> u8 {
        let result = self.regs[0x02];
        self.regs[0x02] &= 0x1f;
        self.scroll_reg_flg = false;
        self.ppu_addr_reg_flg = false;
        return result;
    }
    fn write_ppu_addr_reg(&mut self, value: u8) {
        self.regs[0x06] = value;
        if (self.ppu_addr_reg_flg) {
            self.ppu_addr_buffer = (self.ppu_addr_buffer & 0xff00) | value as usize;
            self.ppu_addr = self.ppu_addr_buffer;
        } else {
            self.ppu_addr_buffer = (self.ppu_addr_buffer & 0x00ff) | ((value as usize & 0x3f) << 8);
        }
        self.ppu_addr_reg_flg = !self.ppu_addr_reg_flg;
    }
    fn read_ppu_data_reg(&mut self) -> u8 {
        let tmp = self.ppu_read_buffer;
        let addr = self.ppu_addr & 0x3fff;
        self.ppu_read_buffer = self.vram[(addr >> 10) as usize][addr & 0x03ff] as usize;

        let val = (if (self.regs[0x00] & 0x04) == 0x04 {
            32
        } else {
            1
        });
        self.ppu_addr = (self.ppu_addr + val) & 0xffff;
        return tmp as u8;
    }
    fn write_ppu_data_reg(&mut self, value: u8) {
        self.regs[0x07] = value;
        let tmpppu_addr = self.ppu_addr & 0x3fff;
        self.vram[tmpppu_addr >> 10][tmpppu_addr & 0x03ff] = value;

        if (tmpppu_addr < 0x3000) {
            let val = if (self.regs[0x00] & 0x04) == 0x04 {
                32
            } else {
                1
            };

            self.ppu_addr = (self.ppu_addr + val) & 0xffff;
            return;
        }

        if (tmpppu_addr < 0x3eff) {
            self.vram[(tmpppu_addr - 0x1000) >> 10][(tmpppu_addr - 0x1000) & 0x03ff] = value;
            let val = if (self.regs[0x00] & 0x04) == 0x04 {
                32
            } else {
                1
            };

            self.ppu_addr = (self.ppu_addr + val) & 0xffff;
            return;
        }

        let palNo = tmpppu_addr & 0x001f;
        if (palNo == 0x00 || palNo == 0x10) {
            self.palette[0x10] = (value & 0x3f);
            self.palette[0x00] = self.palette[0x10];
        } else {
            self.palette[palNo] = value & 0x3f;
        }
        let val = if (self.regs[0x00] & 0x04) == 0x04 {
            32
        } else {
            1
        };
        self.ppu_addr = (self.ppu_addr + val) & 0xffff;
    }
    fn write_sprite_data(&mut self, value: u8) {
        let idx = self.regs[0x03];
        self.sprite_ram[idx as usize] = value;
        self.regs[0x03] = (self.regs[0x03] + 1) & 0xff;
    }
    fn write_sprite_addr_reg(&mut self, value: u8) {
        self.regs[0x03] = value;
    }
}

const PALLETE: &'static [u8] = &[
    0x10, 0x01, 0x02, 0x03, 0x10, 0x05, 0x06, 0x07, 0x10, 0x09, 0x0a, 0x0b, 0x10, 0x0d, 0x0e, 0x0f,
];

const PALLETE_TABLE: &'static [(u8, u8, u8); 64] = &[
    (101, 101, 101),
    (0, 45, 105),
    (19, 31, 127),
    (60, 19, 124),
    (96, 11, 98),
    (115, 10, 55),
    (113, 15, 7),
    (90, 26, 0),
    (52, 40, 0),
    (11, 52, 0),
    (0, 60, 0),
    (0, 61, 16),
    (0, 56, 64),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (174, 174, 174),
    (15, 99, 179),
    (64, 81, 208),
    (120, 65, 204),
    (167, 54, 169),
    (192, 52, 112),
    (189, 60, 48),
    (159, 74, 0),
    (109, 92, 0),
    (54, 109, 0),
    (7, 119, 4),
    (0, 121, 61),
    (0, 114, 125),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (254, 254, 255),
    (93, 179, 255),
    (143, 161, 255),
    (200, 144, 255),
    (247, 133, 250),
    (255, 131, 192),
    (255, 139, 127),
    (239, 154, 73),
    (189, 172, 44),
    (133, 188, 47),
    (85, 199, 83),
    (60, 201, 140),
    (62, 194, 205),
    (78, 78, 78),
    (0, 0, 0),
    (0, 0, 0),
    (254, 254, 255),
    (188, 223, 255),
    (209, 216, 255),
    (232, 209, 255),
    (251, 205, 253),
    (255, 204, 229),
    (255, 207, 202),
    (248, 213, 180),
    (228, 220, 168),
    (204, 227, 169),
    (185, 232, 184),
    (174, 232, 208),
    (175, 229, 234),
    (182, 182, 182),
    (0, 0, 0),
    (0, 0, 0),
];
