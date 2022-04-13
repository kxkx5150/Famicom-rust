use crate::cpu;

#[derive(Debug)]
pub struct Irq {
    nmiWanted: bool,
    irqWanted: bool,
}
impl Irq {
    pub fn new() -> Self {
        Self {
            nmiWanted: false,
            irqWanted: false,
        }
    }
    pub fn set_nmiWanted(&mut self, flg: bool) {
        self.nmiWanted = flg;
    }
    pub fn get_nmiWanted(&mut self) -> bool {
        return self.nmiWanted;
    }
    pub fn set_irqWanted(&mut self, flg: bool) {
        self.irqWanted = flg;
    }
    pub fn get_irqWanted(&mut self) -> bool {
        return self.irqWanted;
    }
    pub fn checkCpuIrqWanted(&mut self) {
        // if (self.nmiWanted){
        //   return "nmi";
        // }else if(!self.nes.cpu.interrupt && (self.irqWanted || self.nes.cpu.toIRQ !== 0x00)) {
        //   return "irq";
        // } else {
        //   return false;
        // }
    }
    pub fn reset(&mut self) {
        self.nmiWanted = false;
        self.irqWanted = false;
    }
}
