use crate::cpu;

#[derive(Debug)]
pub struct Irq {
    nmi: bool,
    irq: bool,
}
impl Irq {
    pub fn new() -> Self {
        Self {
            nmi: false,
            irq: false,
        }
    }
    pub fn init(&mut self) {
        self.clear();
    }
    pub fn set_nmi(&mut self, flg: bool) {
        self.nmi = flg;
    }
    pub fn get_nmi(&mut self) -> bool {
        return self.nmi;
    }
    pub fn set_irq(&mut self, flg: bool) {
        self.irq = flg;
    }
    pub fn get_irq(&mut self) -> bool {
        return self.irq;
    }
    pub fn check_interrupt(&mut self, cpu: &cpu::Cpu) -> String {
        if (self.nmi) {
            return "nmi".to_string();
        } else if cpu.interrupt && self.irq {
            return "irq".to_string();
        } else {
            return "".to_string();
        }
    }
    pub fn clear_irq(&mut self) {
        self.irq = false;
    }
    pub fn clear_nmi(&mut self) {
        self.nmi = false;
    }
    pub fn clear(&mut self) {
        self.nmi = false;
        self.irq = false;
    }
}
