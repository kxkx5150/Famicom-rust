use crate::palette::PALLETE_TABLE;
use crate::rom;
use rom::Mirroring;
use std::isize;

pub struct Pppu {
    PpuX: usize,
    PpuY: usize,
    IO1: Vec<u8>,
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
    Palette: Vec<u8>,
    SPRITE_RAM: Vec<u8>,
}
impl Pppu {
    pub fn new() -> Self {
        Self {
            PpuX: 341,
            PpuY: 0,
            IO1: (0..8).map(|x| 0).collect(),
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
            Palette: (0..33).map(|x| 0x0f).collect(),
            SPRITE_RAM: (0..0x100).map(|x| 0).collect(),
        }
    }
    pub fn init(&mut self, rom: &mut rom::Rom) {
        self.reset();
        self.Palette = [0x0f; 33].to_vec();
        self.SPRITE_RAM = [0; 0x100].to_vec();
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
    }
    pub fn set_chr_rom_page(&mut self, mut num: isize, rom: &mut rom::Rom) {
        num <<= 3;
        for i in 0..8 {
            println!("{}", i);
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
                    // self.inVblank();
                    continue;
                }
                262 => {
                    // self.postRender();
                }
                _ => {}
            }
        }
    }
    fn renderFrame(&mut self) {
        if self.IsScreenEnable() || self.IsSpriteEnable() {
            println!("");
            // self.PPUAddress = (self.PPUAddress & 0xfbe0) | (self.PPUAddressBuffer & 0x041f);

            // if (8 <= self.PpuY && self.PpuY < 232) {
            // self.BuildBGLine();
            // self.BuildSpriteLine();
            // let tmpDist = (self.PpuY - 8) << 10;
            // const fb = self.framebuffer_u32;
            // for (let p = 0; p < 256; p++, tmpDist += 4) {
            //     let tmpPal = self.PaletteTable[self.Palette[self.BgLineBuffer[p]]];
            //     self.setImageData(fb, tmpDist, tmpPal);
            // }
            // } else {
            // for (let p = 0; p < 264; p++) self.BgLineBuffer[p] = 0x10;
            // self.BuildSpriteLine();
            // }

            // if ((self.PPUAddress & 0x7000) === 0x7000) {
            // self.PPUAddress &= 0x8fff;
            // if ((self.PPUAddress & 0x03e0) === 0x03a0) self.PPUAddress = (self.PPUAddress ^ 0x0800) & 0xfc1f;
            // else if ((self.PPUAddress & 0x03e0) === 0x03e0) self.PPUAddress &= 0xfc1f;
            // else self.PPUAddress += 0x0020;
            // } else self.PPUAddress += 0x1000;
        } else if (8 <= self.PpuY && self.PpuY < 232) {
            let tmpDist = (self.PpuY - 8) << 10;
            let tmpPal = PALLETE_TABLE[self.Palette[0x10] as usize];

            for x in (1..10).step_by(4) {}
        }
    }
    //   inVblank() {
    //     self.nes.DrawFlag = true;
    //     if (self.nes.speedCount <= 1) self.ctx.putImageData(self.ImageData, 0, 0);
    //     self.ScrollRegisterFlag = false;
    //     self.IO1[0x02] &= 0x1f;
    //     self.IO1[0x02] |= 0x80;
    //     if ((self.IO1[0x00] & 0x80) === 0x80) self.nes.irq.nmiWanted = true;
    //   }
    //   postRender() {
    //     self.PpuY = 0;
    //     if (self.IsScreenEnable || self.IsSpriteEnable) {
    //       self.PPUAddress = self.PPUAddressBuffer;
    //     }
    //     self.IO1[0x02] &= 0x7f;
    //   }
    //   setImageData(fb, dist, plt) {
    //     fb[dist / 4] = (255 << 24) | (plt[2] << 16) | (plt[1] << 8) | plt[0];
    //   }
    //   BuildBGLine() {
    //     if ((self.IO1[0x01] & 0x08) !== 0x08) {
    //       for (let p = 0; p < 264; p++) self.BgLineBuffer[p] = 0x10;
    //       return;
    //     }

    //     self.BuildBGLine_SUB();
    //     if ((self.IO1[0x01] & 0x02) !== 0x02) {
    //       for (let p = 0; p < 8; p++) self.BgLineBuffer[p] = 0x10;
    //     }
    //   }
    //   BuildBGLine_SUB() {
    //     let tmpvram = self.vram;
    //     let nameAddr = 0x2000 | (self.PPUAddress & 0x0fff);
    //     let tableAddr = ((self.PPUAddress & 0x7000) >> 12) | ((self.IO1[0x00] & 0x10) << 8);
    //     let nameAddrHigh = nameAddr >> 10;
    //     let nameAddrLow = nameAddr & 0x03ff;
    //     let tmpvramHigh = tmpvram[nameAddrHigh];
    //     let s = self.HScrollTmp;
    //     let q = 0;

    //     for (let p = 0; p < 33; p++) {
    //       let ptnDist = (tmpvramHigh[nameAddrLow] << 4) | tableAddr;
    //       let tmpSrcV = tmpvram[ptnDist >> 10];
    //       ptnDist &= 0x03ff;
    //       let attr =
    //         ((tmpvramHigh[((nameAddrLow & 0x0380) >> 4) | (((nameAddrLow & 0x001c) >> 2) + 0x03c0)] << 2) >>
    //           (((nameAddrLow & 0x0040) >> 4) | (nameAddrLow & 0x0002))) &
    //         0x0c;
    //       let ptn = self.SPBitArray[tmpSrcV[ptnDist]][tmpSrcV[ptnDist + 8]];

    //       for (; s < 8; s++, q++) self.BgLineBuffer[q] = self.PaletteArray[ptn[s] | attr];
    //       s = 0;

    //       if ((nameAddrLow & 0x001f) === 0x001f) {
    //         nameAddrLow &= 0xffe0;
    //         tmpvramHigh = tmpvram[(nameAddrHigh ^= 0x01)];
    //       } else nameAddrLow++;
    //     }
    //   }
    //   get isBigSize(){
    //     return (self.IO1[0x00] & 0x20) === 0x20 ? 16 : 8;
    //   }
    //   BuildSpriteLine() {
    //     let SpriteClipping = (self.IO1[0x01] & 0x04) === 0x04 ? 0 : 8;

    //     if ((self.IO1[0x01] & 0x10) === 0x10) {
    //       let tmpSpLine = self.SpriteLineBuffer;tmpSpLine.fill(256)
    //       let tmpSpRAM = self.SPRITE_RAM;
    //       let spptableaddr = (self.IO1[0x00] & 0x08) << 9;
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

    //       if (count >= 8) self.IO1[0x02] |= 0x20;
    //       else self.IO1[0x02] &= 0xdf;
    //     }
    //   }

    pub fn WriteScrollRegister(&mut self, value: u8) {
        self.IO1[0x05] = value;

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
        self.IO1[0x00] = value;
        self.PPUAddressBuffer = (self.PPUAddressBuffer & 0xf3ff) | ((value as usize & 0x03) << 10);
    }
    pub fn WritePPUControlRegister1(&mut self, value: u8) {
        self.IO1[0x01] = value;
    }
    pub fn WritePPUAddressRegister(&mut self, value: u8) {
        self.IO1[0x06] = value;
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
        let result = self.IO1[0x02];
        self.IO1[0x02] &= 0x1f;
        self.ScrollRegisterFlag = false;
        self.PPUAddressRegisterFlag = false;
        return result;
    }
    pub fn ReadPPUData(&mut self) -> u8 {
        let tmp = self.PPUReadBuffer;
        let addr = self.PPUAddress & 0x3fff;
        let vrm = self.vram[(addr >> 10) as usize].clone();
        self.PPUReadBuffer = vrm[addr & 0x03ff] as usize;

        let flg = (if (self.IO1[0x00] & 0x04) == 0x04 {
            32
        } else {
            1
        });
        self.PPUAddress = (self.PPUAddress + flg) & 0xffff;
        return tmp as u8;
    }
    pub fn WritePPUData(&mut self, value: u8) {
        self.IO1[0x07] = value;
        self.vram[self.PPUAddress >> 10][self.PPUAddress & 0x03ff] = value;

        if (self.PPUAddress < 0x3000) {
            let val = if (self.IO1[0x00] & 0x04) == 0x04 {
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

            let val = if (self.IO1[0x00] & 0x04) == 0x04 {
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
        let val = if (self.IO1[0x00] & 0x04) == 0x04 {
            32
        } else {
            1
        };
        self.PPUAddress = (self.PPUAddress + val) & 0xffff;
    }
    pub fn WriteSpriteData(&mut self, value: u8) {
        let idx = self.IO1[0x03];
        self.SPRITE_RAM[idx as usize] = value;
        self.IO1[0x03] = (self.IO1[0x03] + 1) & 0xff;
    }
    pub fn WriteSpriteAddressRegister(&mut self, value: u8) {
        self.IO1[0x03] = value;
    }
    fn IsScreenEnable(&mut self) -> bool {
        return (self.IO1[0x01] & 0x08) == 0x08;
    }
    fn IsSpriteEnable(&mut self) -> bool {
        return (self.IO1[0x01] & 0x10) == 0x10;
    }
}
