pub struct Base {
    mapper_reg: Vec<u8>,
}
pub trait MapperBase {
    fn init(&mut self) {
        println!("mapperbase init");
    }
    fn read_low(&mut self, addr: u16) -> u8 {
        return 0x00;
    }
    fn write_low(&mut self, addr: u16, data: u8) {}
    fn read_ppudata(&mut self) -> u8 {
        return 0x00;
    }
    fn write_ppudata(&mut self) {}
    fn build_bgline(&mut self) {}
    fn build_spriteline(&mut self) {}
    fn read_sram(&mut self) -> u8 {
        return 0x00;
    }
    fn write_sram(&mut self) {}
    fn write(&mut self, addr: u16, data: u8) {}
    fn hsync(&mut self) {}
    fn cpusync(&mut self) {}
    fn setirq(&mut self) {}
    fn clearirq(&mut self) {}
    fn out_exsound(&mut self) -> u8 {
        return 0x00;
    }
    fn exsound_sync(&mut self) {}
    fn getstate(&mut self) {}
    fn setstate(&mut self) {}
}
