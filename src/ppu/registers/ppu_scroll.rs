#[derive(Debug)]
enum Enable {
    X,
    Y,
}

#[derive(Debug)]
pub struct PpuScroll {
    x: u8,
    y: u8,
    enable: Enable,
}

impl PpuScroll {
    pub fn new() -> Self {
        PpuScroll {
            x: 0,
            y: 0,
            enable: Enable::X,
        }
    }
    pub fn enable_x(&mut self) {
        self.enable = Enable::X;
    }
    pub fn get_x(&self) -> u8 {
        self.x
    }
    pub fn get_y(&self) -> u8 {
        self.y
    }
    pub fn write(&mut self, data: u8) {
        match self.enable {
            Enable::X => {
                self.enable = Enable::Y;
                self.x = data;
            }
            Enable::Y => {
                self.enable = Enable::X;
                self.y = data;
            }
        }
    }
}
