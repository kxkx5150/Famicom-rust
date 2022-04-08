use crate::palette::PALLETE_TABLE;
use crate::rom;
use rom::Mirroring;
use std::borrow::BorrowMut;
use std::isize;

pub struct Pppu {
    PpuX: usize,
    PpuY: usize,
    regs: Vec<u8>,
    nmi: bool,
    pub imgdata: Vec<u8>,
    
    Sprite0Line: bool,
    ScrollRegisterFlag: bool,
    PPUAddressBuffer: usize,
    HScrollTmp: usize,
    PPUAddressRegisterFlag: bool,
    PPUAddress: usize,
    PPUReadBuffer: usize,

    pub screen_mirroring: Mirroring,
    pub vram: Vec<Vec<u8>>,
    pub vrams: Vec<Vec<u8>>,
    BgLineBuffer: Vec<u8>,
    Palette: Vec<u8>,
    sprite_ram: Vec<u8>,
    SPBitArray: Vec<Vec<Vec<u8>>>,
}
impl Pppu {
    pub fn new() -> Self {
        Self {
            PpuX: 341,
            PpuY: 0,
            regs: (0..8).map(|x| 0).collect(),
            nmi: false,
            imgdata: vec![0; 256 * 2 * 240 * 3],

            Sprite0Line: false,
            ScrollRegisterFlag: false,
            PPUAddressBuffer: 0,
            HScrollTmp: 0,
            PPUAddressRegisterFlag: false,
            PPUAddress: 0,
            PPUReadBuffer: 0,
            screen_mirroring: Mirroring::HORIZONTAL,

            vram: vec![vec![0; 4096]; 16],
            vrams: vec![vec![0; 1024]; 16],
            BgLineBuffer: (0..264).map(|x| 0).collect(),
            Palette: (0..33).map(|x| 0x0f).collect(),
            sprite_ram: (0..0x100).map(|x| 0).collect(),

            SPBitArray: vec![vec![vec![0; 8]; 256]; 256],
        }
    }
    pub fn init(&mut self, rom: &mut rom::Rom) {
        self.reset();
        self.Palette = [0x0f; 33].to_vec();
        self.sprite_ram = [0; 0x100].to_vec();
        self.BgLineBuffer = [0; 264].to_vec();

        self.screen_mirroring = rom.screen_mirroring.clone();
        match self.screen_mirroring {
            Mirroring::VERTICAL => {
                self.SetMirror(false, rom);
            }
            Mirroring::HORIZONTAL => {
                self.SetMirror(true, rom);
            }
            Mirroring::FOUR_SCREEN => {
                self.SetMirrors(0, 1, 2, 3, rom);
            }
        }
        println!("ppu init");
    }
    pub fn reset(&mut self) {
        self.ScrollRegisterFlag = false;
        self.PPUAddressRegisterFlag = false;
        self.PPUAddressBuffer = 0;
        self.PPUReadBuffer = 0;
        self.PPUAddress = 0;
        self.HScrollTmp = 0;
        self.PpuX = 341;
        self.PpuY = 0;
        self.Sprite0Line = false;
        self.nmi = false;
        self.clear_arryas();
    }
    pub fn clear_arryas(&mut self) {
        // self.regs = (0..8).map(|x| 0).collect();
        // self.vram = vec![vec![0; 4096]; 16];
        // self.vrams = vec![vec![0; 1024]; 16];
        // self.BgLineBuffer = (0..264).map(|x| 0).collect();
        // self.Palette = (0..33).map(|x| 0x0f).collect();
        // self.sprite_ram =  (0..0x100).map(|x| 0).collect();

        for i in 0..256 {
            for j in 0..256 {
                for k in 0..8 {
                    let mut val = ((((i << k) & 0x80) as usize >> 7)
                        | (((j << k) & 0x80) as usize >> 6))
                        as u8;
                    self.SPBitArray[i][j][k] = val;
                }
            }
        }
    }
    pub fn set_chr_rom_page(&mut self, mut num: isize, rom: &mut rom::Rom) {
        num <<= 3;
        for i in 0..8 {
            self.set_chr_rom_page1k(i, num + i, rom);
        }
    }
    fn set_chr_rom_page1k(&mut self, mut page: isize, romPage: isize, rom: &mut rom::Rom) {
        if (romPage >= 0x0100) {
            rom.chrrom_state[page as usize] = romPage as u8;
            let vrm = self.vrams[(romPage & 0xff) as usize].clone();
            self.vram[page as usize] = vrm;
        } else {
            if (rom.chr_rom_page_count > 0) {
                let tmp = romPage % (rom.chr_rom_page_count as isize * 8);
                rom.chrrom_state[page as usize] = tmp as u8;
                let vrm = rom.chrrom_pages[rom.chrrom_state[page as usize] as usize].clone();
                self.vram[page as usize] = vrm;
            }
        }
    }
    pub fn SetMirror(&mut self, value: bool, rom: &mut rom::Rom) {
        if (value) {
            self.SetMirrors(0, 0, 1, 1, rom);
        } else {
            self.SetMirrors(0, 1, 0, 1, rom);
        }
    }
    pub fn SetMirrors(
        &mut self,
        value0: isize,
        value1: isize,
        value2: isize,
        value3: isize,
        rom: &mut rom::Rom,
    ) {
        self.set_chr_rom_page1k(8, value0 + 8 + 0x0100, rom);
        self.set_chr_rom_page1k(9, value1 + 8 + 0x0100, rom);
        self.set_chr_rom_page1k(10, value2 + 8 + 0x0100, rom);
        self.set_chr_rom_page1k(11, value3 + 8 + 0x0100, rom);
    }
    pub fn SetChrRomPages1K(
        &mut self,
        romPage0: isize,
        romPage1: isize,
        romPage2: isize,
        romPage3: isize,
        romPage4: isize,
        romPage5: isize,
        romPage6: isize,
        romPage7: isize,
        rom: &mut rom::Rom,
    ) {
        self.set_chr_rom_page1k(0, romPage0, rom);
        self.set_chr_rom_page1k(1, romPage1, rom);
        self.set_chr_rom_page1k(2, romPage2, rom);
        self.set_chr_rom_page1k(3, romPage3, rom);
        self.set_chr_rom_page1k(4, romPage4, rom);
        self.set_chr_rom_page1k(5, romPage5, rom);
        self.set_chr_rom_page1k(6, romPage6, rom);
        self.set_chr_rom_page1k(7, romPage7, rom);
    }

    pub fn PpuRun(&mut self, cpuclock: usize) {
        let mut tmpx = self.PpuX;
        self.PpuX += cpuclock * 3;

        while (self.PpuX >= 341) {
            self.PpuX -= 341;
            self.PpuY += 1;
            tmpx = 0;
            self.Sprite0Line = false;

            match self.PpuY {
                0..=239 => {
                    self.renderFrame();
                }
                240 => {
                    self.inVblank();
                    continue;
                }
                262 => {
                    self.postRender();
                }
                _ => {}
            }
        }
    }
    fn renderFrame(&mut self) {
        if self.IsScreenEnable() || self.IsSpriteEnable() {
            println!("");
            self.PPUAddress = (self.PPUAddress & 0xfbe0) | (self.PPUAddressBuffer & 0x041f);

            if (8 <= self.PpuY && self.PpuY < 232) {
                self.build_bg();
                self.BuildSpriteLine();
                let mut tmpDist = (self.PpuY - 8) << 10;
                for p in (0..256).step_by(3) {
                    let idx = self.Palette[self.BgLineBuffer[p] as usize];
                    let tmpPal = PALLETE_TABLE[idx as usize];
                    // self.setImageData(tmpDist, tmpPal);
                    tmpDist+=3;
                }
            } else {
                for x in (0..264).step_by(3) {
                    self.BgLineBuffer[x] = 0x10;
                }
                self.BuildSpriteLine();
            }

            if ((self.PPUAddress & 0x7000) == 0x7000) {
                self.PPUAddress &= 0x8fff;

                if ((self.PPUAddress & 0x03e0) == 0x03a0) {
                    self.PPUAddress = (self.PPUAddress ^ 0x0800) & 0xfc1f;
                } else if ((self.PPUAddress & 0x03e0) == 0x03e0) {
                    self.PPUAddress &= 0xfc1f;
                } else {
                    self.PPUAddress += 0x0020;
                }
            } else {
                self.PPUAddress += 0x1000;
            }
        } else if (8 <= self.PpuY && self.PpuY < 232) {
            let mut tmpDist = (self.PpuY - 8) << 10;
            let tmpPal = PALLETE_TABLE[self.Palette[0x10] as usize];
            for x in (0..256).step_by(3) {
                self.setImageData(tmpDist, tmpPal);
                tmpDist+=3;
            }
        }
    }
    fn inVblank(&mut self) {
        self.ScrollRegisterFlag = false;
        self.regs[0x02] &= 0x1f;
        self.regs[0x02] |= 0x80;
        if ((self.regs[0x00] & 0x80) == 0x80) {
            self.nmi = true;
        }
    }
    pub fn clear_nmi(&mut self) {
        self.nmi = false;
    }
    pub fn get_nmi_status(&mut self) -> bool {
        self.nmi
    }
    fn postRender(&mut self) {
        self.PpuY = 0;
        if (self.IsScreenEnable() || self.IsSpriteEnable()) {
            self.PPUAddress = self.PPUAddressBuffer;
        }
        self.regs[0x02] &= 0x7f;
    }
    fn setImageData(&mut self, dist: usize, plt: (u8, u8, u8)) {
        self.imgdata[dist] = plt.0;
        self.imgdata[dist+1] = plt.1;
        self.imgdata[dist+2] = plt.2;
    }
    fn build_bg(&mut self) {
        if ((self.regs[0x01] & 0x08) != 0x08) {
            for x in 0..264 {
                self.BgLineBuffer[x] = 0x10;
            }
            return;
        }

        self.build_bg_line();

        if ((self.regs[0x01] & 0x02) != 0x02) {
            for x in 0..8 {
                self.BgLineBuffer[x] = 0x10;
            }
        }
    }
    fn build_bg_line(&mut self) {
        let nameAddr = 0x2000 | (self.PPUAddress & 0x0fff);
        let lval = (self.PPUAddress & 0x7000) >> 12;
        let rval = ((self.regs[0x00] & 0x10) as usize) << 8;

        let mut nameAddrHigh = nameAddr >> 10;
        let mut nameAddrLow = nameAddr & 0x03ff;
        let s = self.HScrollTmp;

        for p in 0..33 {
            let val = (self.vram[nameAddrHigh][nameAddrLow] as usize) << 4;
            let ptnDist = val | lval | rval;
            let idx = ptnDist >> 10;
            let sptdst = ptnDist & 0x03ff;

            let attr = ((self.vram[nameAddrHigh]
                [((nameAddrLow & 0x0380) >> 4) | (((nameAddrLow & 0x001c) >> 2) + 0x03c0)]
                << 2)
                >> (((nameAddrLow & 0x0040) >> 4) | (nameAddrLow & 0x0002)))
                & 0x0c;

            let sidx = self.vram[idx][sptdst];
            let sidx2 = self.vram[idx][sptdst + 8];
            let ptn = self.SPBitArray[sidx as usize][sidx2 as usize].clone();
            let pletary = vec![
                0x10, 0x01, 0x02, 0x03, 0x10, 0x05, 0x06, 0x07, 0x10, 0x09, 0x0a, 0x0b, 0x10, 0x0d,
                0x0e, 0x0f,
            ];

            let mut q = 0;
            for s in s..8 {
                q += 1;
                let idx = ptn[s] | attr;
                self.BgLineBuffer[q] = pletary[idx as usize];
            }

            if ((nameAddrLow & 0x001f) == 0x001f) {
                nameAddrLow &= 0xffe0;
                nameAddrHigh ^= 0x01;
                let idx = nameAddrHigh;
                self.vram[nameAddrHigh] = self.vram[idx].clone();
            } else {
                nameAddrLow += 1;
            }
        }
    }
    fn BuildSpriteLine(&mut self) {
        //     let SpriteClipping = (self.regs[0x01] & 0x04) === 0x04 ? 0 : 8;

        //     if ((self.regs[0x01] & 0x10) === 0x10) {
        //       let tmpSpLine = self.SpriteLineBuffer;tmpSpLine.fill(256)
        //       let tmpSpRAM = self.sprite_ram;
        //       let spptableaddr = (self.regs[0x00] & 0x08) << 9;
        //       let lineY = self.PpuY;
        //       let count = 0;

        //       for (let i = 0; i <= 252; i += 4) {
        //         let isy = tmpSpRAM[i] + 1;
        //         if (isy > lineY || isy + self.isBigSize <= lineY) continue;
        //         if (i === 0) self.Sprite0Line = true;
        //         if (++count === 9) break;

        //         let attr = tmpSpRAM[i + 2];
        //         let attribute = ((attr & 0x03) << 2) | 0x10;
        //         let bgsp = (attr & 0x20) === 0x00;

        //         let x = tmpSpRAM[i + 3];
        //         let ex = x + 8;
        //         if (ex > 256) ex = 256;
        //         let iy = (attr & 0x80) === 0x80 ? self.isBigSize - 1 - (lineY - isy) : lineY - isy;
        //         let tileNum =
        //           ((iy & 0x08) << 1) +
        //           (iy & 0x07) +
        //           (self.isBigSize === 8
        //             ? (tmpSpRAM[i + 1] << 4) + spptableaddr
        //             : ((tmpSpRAM[i + 1] & 0xfe) << 4) + ((tmpSpRAM[i + 1] & 0x01) << 12));
        //         let tmpHigh = self.vram[tileNum >> 10];
        //         let tmpLow = tileNum & 0x03ff;
        //         if ((attr & 0x40) === 0x00) {
        //           let is = 0;
        //           let ia = 1;
        //         } else {
        //           let is = 7;
        //           let ia = -1;
        //         }

        //         let ptn = self.SPBitArray[tmpHigh[tmpLow]][tmpHigh[tmpLow + 8]];
        //         for (; x < ex; x++, is += ia) {
        //           let tmpPtn = ptn[is];
        //           if (tmpPtn !== 0x00 && tmpSpLine[x] === 256) {
        //             tmpSpLine[x] = i;
        //             if (x >= SpriteClipping && (bgsp || self.BgLineBuffer[x] === 0x10))
        //             self.BgLineBuffer[x] = tmpPtn | attribute;
        //           }
        //         }
        //       }

        //       if (count >= 8) self.regs[0x02] |= 0x20;
        //       else self.regs[0x02] &= 0xdf;
        //     }
    }
    fn isBigSize(&mut self) -> usize {
        let val = (if (self.regs[0x00] & 0x20) == 0x20 {
            16
        } else {
            8
        });
        return 8;
    }

    pub fn WriteScrollRegister(&mut self, value: u8) {
        self.regs[0x05] = value;

        if (self.ScrollRegisterFlag) {
            self.PPUAddressBuffer = (self.PPUAddressBuffer & 0x8c1f)
                | ((value as usize & 0xf8) << 2)
                | ((value as usize & 0x07) << 12);
        } else {
            self.PPUAddressBuffer =
                (self.PPUAddressBuffer & 0xffe0) | ((value as usize & 0xf8) >> 3);
            self.HScrollTmp = value as usize & 7;
        }
        self.ScrollRegisterFlag = !self.ScrollRegisterFlag;
    }
    pub fn WritePPUControlRegister0(&mut self, value: u8) {
        self.regs[0x00] = value;
        self.PPUAddressBuffer = (self.PPUAddressBuffer & 0xf3ff) | ((value as usize & 0x03) << 10);
    }
    pub fn WritePPUControlRegister1(&mut self, value: u8) {
        self.regs[0x01] = value;
    }
    pub fn WritePPUAddressRegister(&mut self, value: u8) {
        self.regs[0x06] = value;
        if (self.PPUAddressRegisterFlag) {
            self.PPUAddressBuffer = (self.PPUAddressBuffer & 0xff00) | value as usize;
            self.PPUAddress = self.PPUAddressBuffer;
        } else {
            self.PPUAddressBuffer =
                (self.PPUAddressBuffer & 0x00ff) | ((value as usize & 0x3f) << 8);
        }
        self.PPUAddressRegisterFlag = !self.PPUAddressRegisterFlag;
    }

    pub fn ReadPPUStatus(&mut self) -> u8 {
        let result = self.regs[0x02];
        self.regs[0x02] &= 0x1f;
        self.ScrollRegisterFlag = false;
        self.PPUAddressRegisterFlag = false;
        return result;
    }
    pub fn ReadPPUData(&mut self) -> u8 {
        let tmp = self.PPUReadBuffer;
        let addr = self.PPUAddress & 0x3fff;
        self.PPUReadBuffer = self.vram[(addr >> 10) as usize][addr & 0x03ff] as usize;

        let flg = (if (self.regs[0x00] & 0x04) == 0x04 {
            32
        } else {
            1
        });
        self.PPUAddress = (self.PPUAddress + flg) & 0xffff;
        return tmp as u8;
    }
    pub fn WritePPUData(&mut self, value: u8) {
        self.regs[0x07] = value;
        self.vram[self.PPUAddress >> 10][self.PPUAddress & 0x03ff] = value;

        if (self.PPUAddress < 0x3000) {
            let val = if (self.regs[0x00] & 0x04) == 0x04 {
                32
            } else {
                1
            };
            let adr = (self.PPUAddress + val) & 0xffff;
            self.PPUAddress = adr;
            return;
        }

        if (self.PPUAddress < 0x3eff) {
            self.vram[(self.PPUAddress - 0x1000) >> 10][(self.PPUAddress - 0x1000) & 0x03ff] =
                value;

            let val = if (self.regs[0x00] & 0x04) == 0x04 {
                32
            } else {
                1
            };

            self.PPUAddress = (self.PPUAddress + val) & 0xffff;
            return;
        }

        let pln = self.PPUAddress & 0x001f;

        if (pln == 0x00 || pln == 0x10) {
            self.Palette[0x10] = (value & 0x3f);
            let plt = self.Palette[0x10];
            self.Palette[0x00] = plt;
        } else {
            self.Palette[pln] = value & 0x3f;
        }
        let val = if (self.regs[0x00] & 0x04) == 0x04 {
            32
        } else {
            1
        };
        self.PPUAddress = (self.PPUAddress + val) & 0xffff;
    }
    pub fn WriteSpriteData(&mut self, value: u8) {
        let idx = self.regs[0x03];
        self.sprite_ram[idx as usize] = value;
        self.regs[0x03] = (self.regs[0x03] + 1) & 0xff;
    }
    pub fn WriteSpriteAddressRegister(&mut self, value: u8) {
        self.regs[0x03] = value;
    }
    fn IsScreenEnable(&mut self) -> bool {
        return (self.regs[0x01] & 0x08) == 0x08;
    }
    fn IsSpriteEnable(&mut self) -> bool {
        return (self.regs[0x01] & 0x10) == 0x10;
    }
}
