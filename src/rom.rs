use std::str;

const PRG_ROM_PAGE_SIZE: usize = 16384;
const CHR_ROM_PAGE_SIZE: usize = 8192;

#[derive(Clone, Debug)]
pub enum Mirroring {
    VERTICAL,
    HORIZONTAL,
    FOUR_SCREEN,
}

pub struct Rom {
    pub rom: Vec<u8>,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub prg_rom_page_count: usize,
    pub chr_rom_page_count: usize,
    pub screen_mirroring: Mirroring,
    pub sram_enable: bool,
    pub trainer_Enable: bool,
    pub four_screen: bool,
    pub mapper_number: u8,

    pub srams: Vec<u8>,
    pub roms: Vec<Vec<u8>>,
    pub prgrom_state: Vec<i8>,
    pub chrrom_state: Vec<u8>,
    pub prgrom_pages: Vec<Vec<u8>>,
    pub chrrom_pages: Vec<Vec<u8>>,
}
impl Rom {
    pub fn new() -> Self {
        Self {
            rom: (0..1).map(|x| 0).collect(),
            prg_rom: (0..1).map(|x| 0).collect(),
            chr_rom: (0..1).map(|x| 0).collect(),
            prg_rom_page_count: 0,
            chr_rom_page_count: 0,
            screen_mirroring: Mirroring::HORIZONTAL,
            sram_enable: false,
            trainer_Enable: false,
            four_screen: false,
            mapper_number: 0,
            srams: (0..0x2000).map(|x| 0).collect(),
            roms: vec![vec![0; 4]; 4],
            prgrom_state: (0..8).map(|x| 0).collect(),
            chrrom_state: (0..16).map(|x| 0).collect(),
            prgrom_pages: vec![vec![0; 1]; 1],
            chrrom_pages: vec![vec![0; 1]; 1],
        }
    }
    pub fn init(&mut self) {
        println!("rom init");
        let hlen = 0x0010;
        let prg_psize = 0x4000;
        let chr_psize = 0x2000;

        if (self.prg_rom_page_count > 0) {
            self.prgrom_pages = vec![vec![0; 1]; self.prg_rom_page_count * 2];
            for i in 0..(self.prg_rom_page_count * 2) {
                let offset = hlen + (prg_psize / 2) * i;
                let v = &self.rom[offset..(offset + prg_psize / 2)];
                self.prgrom_pages[i] = v.to_vec();
            }
        }
        if (self.chr_rom_page_count > 0) {
            self.chrrom_pages = vec![vec![0; 1]; self.chr_rom_page_count * 8];
            let romlen = self.rom.len();

            for i in 0..(self.chr_rom_page_count * 8) {
                let offset = hlen + prg_psize * self.chr_rom_page_count + (chr_psize / 8) * i;
                let mut h = offset + chr_psize / 2;
                if h > romlen {
                    h = romlen;
                }
                let v = &self.rom[offset..(h)];
                self.chrrom_pages[i] = v.to_vec();
            }
        }
    }
    pub fn set_rom(&mut self, mut buf: Vec<u8>) {
        if (!(buf[0] == 0x4e && buf[1] == 0x45 && buf[2] == 0x53 && buf[3] == 0x1a)) {
            panic!("Invalid *.nes file.");
        }

        self.rom = buf;
        self.prg_rom_page_count = self.rom[4] as usize;
        self.chr_rom_page_count = self.rom[5] as usize;

        let four_screen = self.rom[6] & 0b1000 != 0;
        let vertical_mirroring = self.rom[6] & 0b1 != 0;
        self.screen_mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FOUR_SCREEN,
            (false, true) => Mirroring::VERTICAL,
            (false, false) => Mirroring::HORIZONTAL,
        };

        self.sram_enable = (self.rom[6] & 0x02) != 0;
        self.trainer_Enable = (self.rom[6] & 0x04) != 0;
        self.four_screen = four_screen;
        self.mapper_number = (self.rom[6] >> 4) | (self.rom[7] & 0xf0) as u8;

        let prg_rom_size = self.rom[4] as usize * PRG_ROM_PAGE_SIZE;
        let chr_rom_size = self.rom[5] as usize * CHR_ROM_PAGE_SIZE;

        let skip_trainer = self.rom[6] & 0b100 != 0;
        let prg_rom_start = 16 + if skip_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        self.prg_rom = self.rom[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec();
        self.chr_rom = self.rom[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec();

        println!("program rom size is {}", self.prg_rom_page_count);
        println!("character rom size is {}", self.chr_rom_page_count);
        println!("mapper type is {}", self.mapper_number);
        self.init();
    }
    pub fn clear_roms(&mut self) {
        self.srams.iter().map(|x| 0);
        self.prgrom_state.iter().map(|x| 0);
        self.chrrom_state.iter().map(|x| 0);

        for i in 0..4 {
            self.set_prgrom_page_8k(i, -(i + 1));
        }
    }
    pub fn set_prgrom_page_8k(&mut self, page: isize, rompage: isize) {
        if (rompage < 0) {
            self.prgrom_state[page as usize] = rompage as i8;
            self.roms[page as usize] = (0..0x2000).map(|x| 0).collect();
        } else {
            self.prgrom_state[page as usize] =
                rompage as i8 % (self.prg_rom_page_count as usize * 2) as i8;
            let idx = self.prgrom_state[page as usize];
            let v = &self.prgrom_pages[idx as usize];
            self.roms[page as usize] = v.to_vec();
        }
    }
    pub fn set_prgrom_page(&mut self, no: usize, num: usize) {
        self.set_prgrom_page_8k((no * 2) as isize, (num * 2) as isize);
        self.set_prgrom_page_8k((no * 2 + 1) as isize, (num * 2 + 1) as isize);
    }
    pub fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= 0x8000;
        if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            addr = addr % 0x4000;
        }
        self.prg_rom[addr as usize]
    } 

}
