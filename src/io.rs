#[derive(Debug)]
pub struct Io {
    ctrlstat1: u8,
    ctrlstat2: u8,
    latchstate1: u8,
    latchstate2: u8,
    ctrllatched: bool,
}
impl Io {
    pub fn new() -> Self {
        Self {
            ctrlstat1: 0,
            ctrlstat2: 0,
            latchstate1: 0,
            latchstate2: 0,
            ctrllatched: false,
        }
    }
    pub fn init(&mut self) {
        self.reset();
    }
    pub fn reset(&mut self) {
        self.ctrlstat1 = 0;
        self.ctrlstat2 = 0;
        self.latchstate1 = 0;
        self.latchstate2 = 0;
        self.ctrllatched = false;
    }
    pub fn set_ctrllatched(&mut self, state: bool) {
        self.ctrllatched = state;
    }
    pub fn get_ctrllatched(&mut self) -> bool {
        return self.ctrllatched;
    }

    pub fn set_ctrlstat1(&mut self, flg: u8) {
        self.ctrlstat1 = flg;
    }
    pub fn get_ctrlstat1(&mut self) -> u8 {
        return self.ctrlstat1;
    }
    pub fn set_ctrlstat2(&mut self, flg: u8) {
        self.ctrlstat2 = flg;
    }
    pub fn get_ctrlstat2(&mut self) -> u8 {
        return self.ctrlstat2;
    }
    pub fn hdCtrlLatch(&mut self) {
        self.latchstate1 = self.ctrlstat1;
        self.latchstate2 = self.ctrlstat2;
    }

    pub fn set_latched_ctrl_state(&mut self, no: u8) {
        if (no == 1) {
            let mut val = self.latchstate1;
            val >>= 1;
            val |= 0x80;
            self.latchstate1 = val;
        } else {
            let mut val = self.latchstate2;
            val >>= 1;
            val |= 0x80;
            self.latchstate2 = val;
        }
    }
    pub fn get_latched_ctrl_state(&mut self, no: u8) -> u8 {
        if (no == 1) {
            return self.latchstate1;
        } else {
            return self.latchstate2;
        }
    }
}
