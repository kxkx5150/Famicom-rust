use crate::dma::Dma;
use crate::ppu::Port;
use crate::{mapper::MapperBase, mapper0};

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

pub struct Mem {
    pub ram: Vec<u8>,
    pub mapper: mapper0::Mapper0,
    pub dma: Dma,
}
impl Mem {
    pub fn new(mapper: mapper0::Mapper0) -> Self {
        Self {
            ram: (0..0x800).map(|x| 0).collect(),
            mapper: mapper,
            dma: Dma::new(),
        }
    }
    pub fn init(&mut self) {
        println!("mem init");
        self.reset();
        self.mapper.init();
    }

    pub fn get16(&mut self, addr: u16) -> u16 {
        let l = self.get(addr);
        let h = self.get(addr + 1);
        let r: u16 = (l as u16 | ((h as u16) << 8));
        return r;
    }

    pub fn get(&mut self, addr: u16) -> u8 {
        match (addr & 0xe000) {
            0x0000 => {
                return self.ram[(addr & 0x7ff) as usize] as u8;
            }
            0x2000 => {
                match (addr & 0x0007) {
                    0x0000 => {}
                    0x0001 => {}
                    0x0002 => {
                        return self.mapper.ppu.read_ppu_status_reg();
                    }
                    0x0003 => {}
                    0x0004 => {}
                    0x0005 => {}
                    0x0006 => {}
                    0x0007 => {
                        return self.mapper.ppu.read_ppu_data_reg();
                    }
                    0x0008..=PPU_REGISTERS_MIRRORS_END => {
                        let mirror_down_addr = addr & 0b00100000_00000111;
                        return self.get(mirror_down_addr);
                    }
                    _ => {}
                }
                return 0;
            }
            0x3000 => match (addr) {
                _ => {
                    let mirror_down_addr = addr & 0b00100000_00000111;
                    return self.get(mirror_down_addr);
                }
            },
            0x4000 => match (addr) {
                0x4000 => {}
                0x4001 => {}
                0x4002 => {}
                0x4003 => {}
                0x4004 => {}
                0x4005 => {}
                0x4006 => {}
                0x4007 => {}
                0x4008 => {}
                0x4009 => {}
                0x400a => {}
                0x400b => {}
                0x400c => {}
                0x400d => {}
                0x400e => {}
                0x400f => {}
                0x4010 => {}
                0x4011 => {}
                0x4012 => {}
                0x4013 => {}
                0x4014 => {}
                0x4015 => {}
                0x4016 => {
                    let ret = self.mapper.io.get_latched_ctrl_state(1) & 1;
                    self.mapper.io.set_latched_ctrl_state(1);
                    return ret | 0x40;
                }
                0x4017 => {
                    let ret = self.mapper.io.get_latched_ctrl_state(2) & 1;
                    self.mapper.io.set_latched_ctrl_state(2);
                    return ret | 0x40;
                }
                0x4018 => {}
                0x4019 => {}
                0x401a => {}
                0x401b => {}
                0x401c => {}
                0x401d => {}
                0x401e => {}
                0x401f => {}
                _ => {
                    return self.mapper.read_low(addr);
                }
            },
            0x6000 => {}
            // 0x8000..=0xFFFF => {
            //     return self.mapper.rom.read_prg_rom(addr);
            // }
            0x8000 => {
                return self.mapper.rom.roms[0][(addr & 0x1fff) as usize];
            }
            0xa000 => {
                return self.mapper.rom.roms[1][(addr & 0x1fff) as usize];
            }
            0xc000 => {
                return self.mapper.rom.roms[2][(addr & 0x1fff) as usize];
            }
            0xe000 => {
                return self.mapper.rom.roms[3][(addr & 0x1fff) as usize];
            }
            _ => {}
        }
        return 0;
    }
    pub fn set(&mut self, addr: u16, data: u8) {
        match (addr & 0xe000) {
            0x0000 => {
                self.ram[(addr & 0x7ff) as usize] = data;
            }
            0x2000 => match (addr & 0x07) {
                0x00 => {
                    self.mapper.ppu.write_ppu_ctrl0_reg(data);
                }
                0x01 => {
                    self.mapper.ppu.write_ppu_ctrl1_reg(data);
                }
                0x02 => {}
                0x03 => {
                    self.mapper.ppu.write_sprite_addr_reg(data);
                }
                0x04 => {
                    self.mapper.ppu.write_sprite_data(data);
                }
                0x05 => {
                    self.mapper.ppu.write_scroll_reg(data);
                }
                0x06 => {
                    self.mapper.ppu.write_ppu_addr_reg(data);
                }
                0x07 => {
                    self.mapper.ppu.write_ppu_data_reg(data);
                }
                0x0008..=PPU_REGISTERS_MIRRORS_END => {
                    let mirror_down_addr = addr & 0b00100000_00000111;
                    self.set(mirror_down_addr, data);
                }
                _ => {}
            },
            0x3000..=0x3fff => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.set(mirror_down_addr, data);
            }
            0x4000 => match (addr) {
                0x4000 => {}
                0x4001 => {}
                0x4002 => {}
                0x4003 => {}
                0x4004 => {}
                0x4005 => {}
                0x4006 => {}
                0x4007 => {}
                0x4008 => {}
                0x4009 => {}
                0x4010 => {}
                0x400a => {}
                0x400b => {}
                0x400c => {}
                0x400d => {}
                0x400e => {}
                0x400f => {}
                0x4010 => {}
                0x4011 => {}
                0x4012 => {}
                0x4013 => {}
                0x4014 => {
                    self.dma.run(data, &self.ram, &mut self.mapper.ppu);
                }
                0x4015 => {}
                0x4016 => {
                    if ((data & 0x01) > 0) {
                        self.mapper.io.set_ctrllatched(true)
                    } else {
                        self.mapper.io.set_ctrllatched(false)
                    }
                    return;
                }
                0x4017 => {}
                0x4018 => {}
                0x4019 => {}
                0x401a => {}
                0x401b => {}
                0x401c => {}
                0x401d => {}
                0x401e => {}
                0x401f => {}
                _ => {
                    self.mapper.write_low(addr, data);
                }
            },
            0x6000 => {}
            0x8000 => {
                self.mapper.write(addr, data);
            }
            0xa000 => {
                self.mapper.write(addr, data);
            }
            0xc000 => {
                self.mapper.write(addr, data);
            }
            0xe000 => {
                self.mapper.write(addr, data);
            }
            _ => {}
        }
    }
    pub fn reset(&mut self) {
        self.ram.iter().map(|x| 0);
    }
}
