#![allow(warnings, unused, dead_code)]
use crate::mem;
use crate::nestest;
use std::process;

struct Opcode {
    int: u64,
    hex: String,
    op: String,
    adm: String,
    cycle: u64,
}

pub struct Cpu {
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,

    negative: bool,
    overflow: bool,
    decimal: bool,
    interrupt: bool,
    zero: bool,
    carry: bool,

    toirq: u8,
    pub cpuclock: u64,
    cycles: usize,
    total: u64,

    pub mem: mem::Mem,
    opcodes: Vec<Opcode>,
    steps: u64,
    totalcycle: u64,
}
impl Cpu {
    pub fn new(mem: mem::Mem) -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0,
            pc: 0,

            negative: false,
            overflow: false,
            decimal: false,
            interrupt: true,
            zero: false,
            carry: false,

            toirq: 0x00,
            cpuclock: 7,
            cycles: 0,
            total: 0,

            mem: mem,
            opcodes: Vec::new(),
            steps: 0,
            totalcycle: 0,
        }
    }
    pub fn init(&mut self) {
        println!("cpu init");
        self.mem.init();
        self.reset();
        self.create_opcodes();
    }
    pub fn start(&mut self) {
        self.reset();
        let data = self.mem.get16(0xfffc);
        self.pc = data;
    }
    pub fn init_nestest(&mut self) {
        self.reset();
        self.pc = 0xc000;
    }
    pub fn exec_nmi(&mut self) {
        self.mem.mapper.ppu.clear_nmi();
        let opc = Opcode {
            int: 256,
            hex: "100".to_string(),
            op: "NMI".to_string(),
            adm: "IMP".to_string(),
            cycle: 0,
        };
        self.cpuclock += 7;
        self.exe_instruction(opc.op.as_str(), 0);
    }
    pub fn run(&mut self, test: bool) {
        let nmi = self.mem.mapper.ppu.get_nmi_status();
        if (nmi) {
            self.exec_nmi();
        } else if (false) { //irq
        }

        let oldpc = self.pc;
        let pc = self.post_incpc();
        let instr = self.mem.get(pc);
        let opobj = self.opcodes.get(instr as usize).unwrap();
        let optcycle = opobj.cycle;
        self.cpuclock += optcycle;

        let admstr = opobj.adm.to_string();
        let op = opobj.op.to_string();
        let adrm = self.get_addr(admstr.as_str());

        if test {
            self.show_test_state(pc, &op, &admstr);
        }
        if false {
            self.show_state(pc, &op, &admstr);
        }

        self.exe_instruction(op.as_str(), adrm);
        let cpucycle = self.cpuclock;
        self.totalcycle += cpucycle;
        self.steps += 1;
    }
    pub fn clear_cpucycle(&mut self) {
        self.cpuclock = 0;
    }
    fn show_state(&mut self, pc: u16, op: &String, admstr: &String) {
        let p = self.getp(false);
        println!("");
        println!("pc         : {:#04X}", pc);
        println!("opcode     : {}", op);
        println!("adrmode    : {}", admstr);
        println!(
            "regs       : A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
            self.a, self.x, self.y, p, self.sp
        );
    }
    fn show_test_state(&mut self, pc: u16, op: &String, admstr: &String) {
        let p = self.getp(false);
        let teststr = format!("{:#04X}", pc);
        let teststr = format!("CYC:{}", self.totalcycle);
        let teststr = format!(
            "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
            self.a, self.x, self.y, p, self.sp
        );

        let okstr = nestest::NESTEST_ADDR[self.steps as usize].to_string();
        let okstr = nestest::NESTEST_CYCLES[self.steps as usize].to_string();
        let okstr = nestest::NESTEST_REGS[self.steps as usize].to_string();

        println!("");
        println!("OK : {}", okstr);
        println!("   : {}", teststr);
        println!("pc        : {:#04X}\n", pc);
        println!("opcode    : {}\n", op);
        // println!("admstr    : {}\n", admstr);

        if !okstr.starts_with(&teststr) {
            if teststr.starts_with("0x3") {
                //bug? 0x300 ??? {:#04X}
            } else {
                println!("\n\n----------- error -----------");
                println!("pc    : {:#04X}\n", pc);
                println!("op    : {}\n", op);
                println!("steps : {}\n", self.steps);
                process::exit(0x0100);
            }
        }
    }
    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xfd;
        self.pc = 0;
        self.negative = false;
        self.overflow = false;
        self.decimal = false;
        self.interrupt = true;
        self.zero = false;
        self.carry = false;
        self.toirq = 0x00;
        self.cpuclock = 7;
        self.totalcycle = 7;
        self.steps = 0;
    }
    fn get_addr(&mut self, mode: &str) -> u16 {
        match mode {
            "IMP" => {
                return 0;
            }
            "IMM" => {
                return self.post_incpc();
            }
            "ZP" => {
                let adr = self.post_incpc();
                return self.mem.get(adr) as u16;
            }
            "ZPX" => {
                let adr = self.post_incpc();
                let adr = self.mem.get(adr);
                return ((adr as u16 + self.x as u16) & 0xff) as u16;
            }
            "ZPY" => {
                let adr = self.post_incpc();
                let adr = self.mem.get(adr);
                return ((adr as u16 + self.y as u16) & 0xff) as u16;
            }
            "IZX" => {
                let adr = self.post_incpc();
                let mdata = self.mem.get(adr) as u16;
                let adr = ((mdata + self.x as u16) & 0xff) as u16;
                let val = self.mem.get((adr + 1) & 0xff) as u16;
                return (self.mem.get(adr) as u16 | (val << 8));
            }
            "IZY" => {
                let adr = self.post_incpc();
                let adr = self.mem.get(adr) as u16;
                let val = self.mem.get((adr + 1) & 0xff) as u16;
                let radr = self.mem.get(adr) as u16 | (val << 8);
                return (radr + self.y as u16) & 0xffff;
            }
            "IZYr" => {
                // let adr = this.mem.Get(this.PC[0]++);
                // let radr = this.mem.Get(adr) | (this.mem.Get((adr + 1) & 0xff) << 8);
                // if (radr >> 8 < (radr + this.Y[0]) >> 8) {
                //   this.CPUClock++;
                // }
                // return (radr + this.Y[0]) & 0xffff;

                let adr = self.post_incpc();
                let adr = self.mem.get(adr) as u16;

                let hval = self.mem.get((adr + 1) & 0xff) as u16;
                let radr = (self.mem.get(adr) as u32) | ((hval as u32) << 8);

                let aaa = radr + (self.y as u32);
                let bbb = (aaa >> 8) as u32;
                if ((radr >> 8) < (bbb) >> 8) {
                    self.cpuclock += 1;
                }
                return ((radr + self.y as u32) & 0xffff) as u16;

                // let pcadr = self.post_incpc();
                // let mut adr = self.mem.get(pcadr);
                // let ladr = self.mem.get(adr as u16);
                // let radr = self.mem.get((adr as u16 + 1));
                // let data = ladr as u16 | ((radr & 0xff) as u16) << 8;
                // let datay32 = (data as u32 + self.y as u32);

                // if (data >> 8) < (datay32 >> 8) as u16{
                //     self.cpuclock += 1;
                // }
                // let rval = (data as u32 + self.y as u32);
                // return (rval & 0xffff) as u16;
            }
            "ABS" => {
                let adr = self.post_incpc();
                let mut adr = self.mem.get(adr) as u16;
                let madr = self.post_incpc();
                let val = self.mem.get(madr) as u16;
                adr |= val << 8;
                return adr;
            }
            "ABX" => {
                let adr = self.post_incpc();
                let mut adr = self.mem.get(adr) as u16;
                let madr = self.post_incpc();
                let val = self.mem.get(madr) as u16;
                adr |= val << 8;
                return (adr + self.x as u16) & 0xffff;
            }
            "ABXr" => {
                let adr = self.post_incpc();
                let mut adr = self.mem.get(adr) as u16;
                let madr = self.post_incpc();
                let val = self.mem.get(madr) as u16;
                adr |= val << 8;
                if (adr >> 8 < (adr + self.x as u16) >> 8) {
                    self.cpuclock += 1;
                }
                return (adr + self.x as u16) & 0xffff;
            }
            "ABY" => {
                let adr = self.post_incpc();
                let mut adr = self.mem.get(adr) as u16;
                let madr = self.post_incpc();
                let val = self.mem.get(madr) as u16;
                adr |= val << 8;
                return (adr + self.y as u16) & 0xffff;
            }
            "ABYr" => {
                let adr = self.post_incpc();
                let mut adr = self.mem.get(adr) as u16;
                let madr = self.post_incpc();
                let val = self.mem.get(madr) as u16;
                adr |= val << 8;
                if (adr >> 8 < (adr as u32 + self.y as u32) as u16 >> 8) {
                    self.cpuclock += 1;
                }
                return (adr as u32 + self.y as u32) as u16 & 0xffff;
            }
            "IND" => {
                let adr = self.post_incpc();
                let adrl = self.mem.get(adr) as u16;
                let adr = self.post_incpc();
                let adrh = self.mem.get(adr) as u16;
                let mut radr = self.mem.get(adrl | (adrh << 8)) as u16;
                let val = self.mem.get(((adrl + 1) & 0xff) | (adrh << 8)) as u16;
                radr |= val << 8;
                return radr;
            }
            "REL" => {
                let adr = self.post_incpc();
                return self.mem.get(adr) as u16;
            }
            _ => {
                println!("unimplemented addr");
                return 0;
            }
        }
        return 0;
    }
    fn exe_instruction(&mut self, opcode: &str, addr: u16) {
        match (opcode) {
            "UNI" => {
                println!("unimplemented instruction");
                return;
            }
            "ORA" => {
                self.a |= self.mem.get(addr);
                self.set_zero_and_ng(self.a);
            }
            "AND" => {
                self.a &= self.mem.get(addr);
                self.set_zero_and_ng(self.a);
            }
            "EOR" => {
                self.a ^= self.mem.get(addr);
                self.set_zero_and_ng(self.a);
            }
            "ADC" => {
                let value = self.mem.get(addr);
                let result: u16 = self.a as u16 + value as u16 + (if self.carry { 1 } else { 0 });
                self.carry = result > 0xff;

                self.overflow =
                    (self.a & 0x80) == (value & 0x80) && (value & 0x80) != (result as u8 & 0x80);
                self.a = result as u8;
                self.set_zero_and_ng(self.a);
            }
            "SBC" => {
                let value = self.mem.get(addr) ^ 0xff;
                let result: u16 = self.a as u16 + value as u16 + (if self.carry { 1 } else { 0 });
                self.carry = result > 0xff;
                self.overflow =
                    (self.a & 0x80) == (value & 0x80) && (value & 0x80) != (result as u8 & 0x80);
                self.a = result as u8;
                self.set_zero_and_ng(self.a);
            }
            "CMP" => {
                let value = self.mem.get(addr) ^ 0xff;
                let result: u16 = self.a as u16 + value as u16 + 1;
                self.carry = result > 0xff;
                self.set_zero_and_ng(result as u8);
            }
            "CPX" => {
                let value = self.mem.get(addr) ^ 0xff;
                let result: u16 = self.x as u16 + value as u16 + 1;
                self.carry = result > 0xff;
                self.set_zero_and_ng(result as u8);
            }
            "CPY" => {
                let value = self.mem.get(addr) ^ 0xff;
                let result: u16 = self.y as u16 + value as u16 + 1;
                self.carry = result > 0xff;
                self.set_zero_and_ng(result as u8);
            }
            "DEC" => {
                let mut data = self.mem.get(addr);
                if data == 0 {
                    data = 0xff;
                } else {
                    data -= 1;
                }

                let result = data & 0xff;
                self.set_zero_and_ng(result);
                self.mem.set(addr, result);
            }
            "DEX" => {
                if self.x == 0 {
                    self.x = 0xff;
                } else {
                    self.x -= 1;
                }
                self.set_zero_and_ng(self.x);
            }
            "DEY" => {
                if self.y == 0 {
                    self.y = 0xff;
                } else {
                    self.y -= 1;
                }
                self.set_zero_and_ng(self.y);
            }
            "INC" => {
                let result = ((self.mem.get(addr) as u16) + 1) & 0xff;
                self.set_zero_and_ng(result as u8);
                self.mem.set(addr, result as u8);
            }
            "INX" => {
                if self.x == 255 {
                    self.x = 0;
                } else {
                    self.x += 1;
                }
                self.set_zero_and_ng(self.x);
            }
            "INY" => {
                if self.y == 255 {
                    self.y = 0;
                } else {
                    self.y += 1;
                }
                self.set_zero_and_ng(self.y);
            }
            "ASLA" => {
                let result: u16 = (self.a as u16) << 1;
                self.carry = result > 0xff;
                self.set_zero_and_ng(result as u8);
                self.a = result as u8;
            }
            "ASL" => {
                let result: u16 = (self.mem.get(addr) as u16) << 1;
                self.carry = result > 0xff;
                self.set_zero_and_ng(result as u8);
                self.mem.set(addr, result as u8);
            }
            "ROLA" => {
                let result: u16 = ((self.a as u16) << 1) | (if self.carry { 1 } else { 0 });
                self.carry = result > 0xff;
                self.set_zero_and_ng(result as u8);
                self.a = result as u8;
            }
            "ROL" => {
                let result: u16 =
                    ((self.mem.get(addr) as u16) << 1) | (if self.carry { 1 } else { 0 });
                self.carry = result > 0xff;
                self.set_zero_and_ng(result as u8);
                self.mem.set(addr, result as u8);
            }
            "LSRA" => {
                let carry = self.a & 0x1;
                let result: u16 = (self.a as u16) >> 1;
                self.carry = carry > 0;
                self.set_zero_and_ng(result as u8);
                self.a = result as u8;
            }
            "LSR" => {
                let value = self.mem.get(addr);
                let carry = value & 0x1;
                let result: u16 = (value as u16) >> 1;
                self.carry = carry > 0;
                self.set_zero_and_ng(result as u8);
                self.mem.set(addr, result as u8);
            }
            "RORA" => {
                let carry = self.a & 0x1;
                let result: u16 = (self.a >> 1) as u16 | ((if self.carry { 1 } else { 0 }) << 7);
                self.carry = carry > 0;
                self.set_zero_and_ng(result as u8);
                self.a = result as u8;
            }
            "ROR" => {
                let value = self.mem.get(addr);
                let carry = value & 0x1;
                let result: u16 = (value >> 1) as u16 | ((if self.carry { 1 } else { 0 }) << 7);
                self.carry = carry > 0;
                self.set_zero_and_ng(result as u8);
                self.mem.set(addr, result as u8);
            }
            "LDA" => {
                self.a = self.mem.get(addr);
                self.set_zero_and_ng(self.a);
            }
            "STA" => {
                self.mem.set(addr, self.a);
            }
            "LDX" => {
                self.x = self.mem.get(addr);
                self.set_zero_and_ng(self.x);
            }
            "STX" => {
                self.mem.set(addr, self.x);
            }
            "LDY" => {
                let y = self.mem.get(addr);
                self.y = y;
                self.set_zero_and_ng(y);
            }
            "STY" => {
                self.mem.set(addr, self.y);
            }
            "TAX" => {
                self.x = self.a;
                self.set_zero_and_ng(self.x);
            }
            "TXA" => {
                self.a = self.x;
                self.set_zero_and_ng(self.a);
            }
            "TAY" => {
                self.y = self.a;
                self.set_zero_and_ng(self.y);
            }
            "TYA" => {
                self.a = self.y;
                self.set_zero_and_ng(self.a);
            }
            "TSX" => {
                self.x = self.sp;
                self.set_zero_and_ng(self.x);
            }
            "TXS" => {
                self.sp = self.x;
            }
            "PLA" => {
                self.pre_incsp();
                let adr = 0x100 + (self.sp & 0xff) as u16;
                self.a = self.mem.get(adr);
                self.set_zero_and_ng(self.a);
            }
            "PHA" => {
                let sp = self.post_decsp();
                let adr = 0x100 + (sp & 0xff) as u16;
                self.mem.set(adr, self.a);
            }
            "PLP" => {
                self.pre_incsp();
                let adr = 0x100 + (self.sp & 0xff) as u16;
                let val = self.mem.get(adr);
                self.setp(val);
            }
            "PHP" => {
                let sp = self.post_decsp();
                let adr = 0x100 + (sp & 0xff) as u16;
                let data = self.getp(true);
                self.mem.set(adr, data);
            }
            "BPL" => {
                self.doBranch(!self.negative, addr);
            }
            "BMI" => {
                self.doBranch(self.negative, addr);
            }
            "BVC" => {
                self.doBranch(!self.overflow, addr);
            }
            "BVS" => {
                self.doBranch(self.overflow, addr);
            }
            "BCC" => {
                self.doBranch(!self.carry, addr);
            }
            "BCS" => {
                self.doBranch(self.carry, addr);
            }
            "BNE" => {
                self.doBranch(!self.zero, addr);
            }
            "BEQ" => {
                self.doBranch(self.zero, addr);
            }
            "BRK" => {
                let pushpc = (self.pc + 1) & 0xffff;

                let sp = self.post_decsp();
                let adr = 0x100 + sp as u16;
                self.mem.set(adr, (pushpc >> 8) as u8);

                let sp = self.post_decsp();
                let adr = 0x100 + sp as u16;
                self.mem.set(adr, (pushpc & 0xff) as u8);

                let sp = self.post_decsp();
                let adr = 0x100 + sp as u16;
                let data = self.getp(true);
                self.mem.set(adr, data);

                self.interrupt = true;
                self.pc = self.mem.get(0xfffe) as u16 | ((self.mem.get(0xffff) as u16) << 8);
            }
            "RTI" => {
                self.pre_incsp();
                let adr = 0x100 + (self.sp & 0xff) as u16;
                let val = self.mem.get(adr);
                self.setp(val);

                self.pre_incsp();
                let adr = 0x100 + (self.sp & 0xff) as u16;
                let mut data = self.mem.get(adr) as u16;

                self.pre_incsp();
                let adr = 0x100 + (self.sp & 0xff) as u16;
                let val = self.mem.get(adr) as u16;
                let tmp = val << 8;
                data |= tmp;
                self.pc = data;
            }
            "JSR" => {
                let pushpc = (self.pc - 1) & 0xffff;

                let sp = self.post_decsp();
                let adr = 0x100 + sp as u16;
                let data = pushpc >> 8;
                self.mem.set(adr, data as u8);

                let sp = self.post_decsp();
                let adr = 0x100 + sp as u16;
                let data = pushpc & 0xff;
                self.mem.set(adr, data as u8);
                self.pc = addr;
            }
            "RTS" => {
                self.pre_incsp();
                let adr = 0x100 + (self.sp as u16 & 0xff);
                let pullPc = self.mem.get(adr) as u16;

                self.pre_incsp();
                let adr = 0x100 + (self.sp & 0xff) as u16;
                let data = self.mem.get(adr) as u16;
                let pp = pullPc | (data << 8);
                self.pc = pp + 1;
            }
            "JMP" => {
                self.pc = addr;
            }
            "BIT" => {
                let value = self.mem.get(addr);
                self.negative = (value & 0x80) > 0;
                self.overflow = (value & 0x40) > 0;
                let res = self.a & value;
                self.zero = res == 0;
            }
            "CLC" => {
                self.carry = false;
            }
            "SEC" => {
                self.carry = true;
            }
            "CLD" => {
                self.decimal = false;
            }
            "SED" => {
                self.decimal = true;
            }
            "CLI" => {
                self.interrupt = false;
            }
            "SEI" => {
                self.interrupt = true;
            }
            "CLV" => {
                self.overflow = false;
            }
            "NOP" => {}
            "IRQ" => {
                let pushpc = self.pc;

                let sp = self.post_incsp();
                let adr = 0x100 + sp as u16;
                let data = pushpc >> 8;
                self.mem.set(adr, data as u8);

                let sp = self.post_incsp();
                let adr = 0x100 + sp as u16;
                let data = pushpc & 0xff;
                self.mem.set(adr, data as u8);

                let sp = self.post_incsp();
                let adr = 0x100 + sp as u16;
                let data = self.getp(false);
                self.mem.set(adr, data);

                self.interrupt = true;
                self.pc = self.mem.get(0xfffe) as u16 | ((self.mem.get(0xffff) as u16) << 8);
            }
            "NMI" => {
                let pushpc = self.pc;

                let sp = self.post_decsp();
                let adr = 0x100 + sp as u16;
                let data = pushpc >> 8;
                self.mem.set(adr, data as u8);

                let sp = self.post_decsp();
                let adr = 0x100 + sp as u16;
                let data = pushpc & 0xff;
                self.mem.set(adr, data as u8);

                let sp = self.post_decsp();
                let adr = 0x100 + sp as u16;
                let data = self.getp(false);
                self.mem.set(adr, data as u8);

                self.interrupt = true;
                self.pc = self.mem.get(0xfffa) as u16 | ((self.mem.get(0xfffb) as u16) << 8);
            }
            // undocumented opcodes
            "KIL" => {
                self.decpc();
            }
            "SLO" => {
                let data = self.mem.get(addr) as u16;
                let result: u16 = data << 1;

                self.carry = result > 0xff;
                self.mem.set(addr, result as u8);
                self.a |= result as u8;
                self.set_zero_and_ng(self.a);
            }
            "RLA" => {
                let data = self.mem.get(addr) as u16;
                let result: u16 = (data << 1) | (if self.carry { 1 } else { 0 });
                self.carry = result > 0xff;
                self.mem.set(addr, result as u8);
                self.a &= result as u8;
                self.set_zero_and_ng(self.a);
            }
            "SRE" => {
                let value = self.mem.get(addr);
                let carry = value & 0x1;
                let result = value >> 1;
                self.carry = carry > 0;
                self.mem.set(addr, result);
                self.a ^= result;
                self.set_zero_and_ng(self.a);
            }
            "RRA" => {
                let value = self.mem.get(addr);
                let carry = value & 0x1;
                let result = (value >> 1) | ((if self.carry { 1 } else { 0 }) << 7);
                self.mem.set(addr, result);
                let data: u16 = self.a as u16 + result as u16 + carry as u16;
                self.carry = data > 0xff;
                self.overflow =
                    (self.a & 0x80) == (result & 0x80) && (result & 0x80) != (data as u8 & 0x80);
                self.a = data as u8;
                self.set_zero_and_ng(self.a);
            }
            "SAX" => {
                self.mem.set(addr, self.a & self.x);
            }
            "LAX" => {
                self.a = self.mem.get(addr);
                self.x = self.a;
                self.set_zero_and_ng(self.x);
            }
            "DCP" => {
                let mut dat = self.mem.get(addr);
                if dat == 0 {
                    dat = 0xff;
                } else {
                    dat -= 1;
                }
                let mut value = dat & 0xff;
                self.mem.set(addr, value);
                value ^= 0xff;
                let result: u16 = self.a as u16 + value as u16 + 1;
                self.carry = result > 0xff;
                self.set_zero_and_ng(result as u8 & 0xff);
            }
            "ISC" => {
                let mut dat = self.mem.get(addr);
                if dat == 0xff {
                    dat = 0;
                } else {
                    dat += 1;
                }
                let mut value = dat & 0xff;
                self.mem.set(addr, value);
                value ^= 0xff;

                let result: u16 = self.a as u16 + value as u16 + (if self.carry { 1 } else { 0 });

                self.carry = result > 0xff;
                self.overflow =
                    (self.a & 0x80) == (value & 0x80) && (value & 0x80) != ((result & 0x80) as u8);

                self.a = result as u8;
                self.set_zero_and_ng(self.a);
            }
            "ANC" => {
                self.a &= self.mem.get(addr);
                self.set_zero_and_ng(self.a);
                self.carry = self.negative;
            }
            "ALR" => {
                self.a &= self.mem.get(addr);
                let carry = self.a & 0x1;
                let result = self.a >> 1;
                self.carry = carry > 0;
                self.set_zero_and_ng(result);
                self.a = result;
            }
            "ARR" => {
                self.a &= self.mem.get(addr);
                let result = (self.a >> 1) | ((if self.carry { 1 } else { 0 }) << 7);
                self.set_zero_and_ng(result);
                self.carry = (result & 0x40) > 0;
                self.overflow = ((result & 0x40) ^ ((result & 0x20) << 1)) > 0;
                self.a = result;
            }
            "AXS" => {
                let value = self.mem.get(addr) ^ 0xff;
                let data = self.a & self.x;
                let result: u16 = data as u16 + value as u16 + 1;
                self.carry = result > 0xff;
                self.x = result as u8;
                self.set_zero_and_ng(self.x);
            }
            _ => {
                println!("opcode error")
            }
        }
    }
    fn set_zero_and_ng(&mut self, rval: u8) {
        let val = rval & 0xff;
        self.zero = val == 0;
        self.negative = val > 0x7f;
    }

    fn post_incpc(&mut self) -> u16 {
        let pc = self.pc;
        self.pc += 1;
        pc
    }
    fn decpc(&mut self) -> u16 {
        self.pc -= 1;
        self.pc
    }

    fn pre_incsp(&mut self) -> u8 {
        self.sp += 1;
        self.sp
    }
    fn post_incsp(&mut self) -> u8 {
        let sp = self.sp;
        self.sp += 1;
        sp
    }
    fn post_decsp(&mut self) -> u8 {
        let sp = self.sp;
        self.sp -= 1;
        sp
    }

    fn setp(&mut self, value: u8) {
        self.negative = (value & 0x80) > 0;
        self.overflow = (value & 0x40) > 0;
        self.decimal = (value & 0x08) > 0;
        self.interrupt = (value & 0x04) > 0;
        self.zero = (value & 0x02) > 0;
        self.carry = (value & 0x01) > 0;
    }
    fn getp(&mut self, bFlag: bool) -> u8 {
        let mut value = 0;
        value |= (if self.negative { 0x80 } else { 0 });
        value |= (if self.overflow { 0x40 } else { 0 });
        value |= (if self.decimal { 0x08 } else { 0 });
        value |= (if self.interrupt { 0x04 } else { 0 });
        value |= (if self.zero { 0x02 } else { 0 });
        value |= (if self.carry { 0x01 } else { 0 });
        value |= 0x20;
        value |= (if bFlag { 0x10 } else { 0 });
        return value;
    }
    fn doBranch(&mut self, test: bool, reladr: u16) {
        // if (test) {
        //     this.CPUClock++;
        //     if (this.PC[0] >> 8 !== (this.PC[0] + rel) >> 8) {
        //       this.CPUClock++;
        //     }
        //     this.PC[0] += rel;
        //   }
        if (test) {
            self.cpuclock += 1;

            let u8val = reladr as u8;
            let mut relval: i32 = 0;
            let mut adr: u16 = 0;

            if u8val > 127 {
                relval = (256 - u8val as i32);
                adr = (self.pc as i32 - relval) as u16;
            } else {
                adr = (self.pc as i32 + u8val as i32) as u16;
            }

            if (self.pc >> 8 != adr >> 8) {
                self.cpuclock += 1;
            }
            self.pc = adr;
        }
    }
    fn create_opcode(&mut self, int: u64, hex: &str, op: &str, adm: &str, cycle: u64) {
        let opc = Opcode {
            int,
            hex: hex.to_string(),
            op: op.to_string(),
            adm: adm.to_string(),
            cycle,
        };
        self.opcodes.push(opc);
    }
    fn create_opcodes(&mut self) {
        self.create_opcode(0, "0", "BRK", "IMP", 7);
        self.create_opcode(1, "1", "ORA", "IZX", 6);
        self.create_opcode(2, "2", "KIL", "IMP", 2);
        self.create_opcode(3, "3", "SLO", "IZX", 8);
        self.create_opcode(4, "4", "NOP", "ZP", 3);
        self.create_opcode(5, "5", "ORA", "ZP", 3);
        self.create_opcode(6, "6", "ASL", "ZP", 5);
        self.create_opcode(7, "7", "SLO", "ZP", 5);
        self.create_opcode(8, "8", "PHP", "IMP", 3);
        self.create_opcode(9, "9", "ORA", "IMM", 2);
        self.create_opcode(10, "a", "ASLA", "IMP", 2);
        self.create_opcode(11, "b", "ANC", "IMM", 2);
        self.create_opcode(12, "c", "NOP", "ABS", 4);
        self.create_opcode(13, "d", "ORA", "ABS", 4);
        self.create_opcode(14, "e", "ASL", "ABS", 6);
        self.create_opcode(15, "f", "SLO", "ABS", 6);
        self.create_opcode(16, "10", "BPL", "REL", 2);
        self.create_opcode(17, "11", "ORA", "IZYr", 5);
        self.create_opcode(18, "12", "KIL", "IMP", 2);
        self.create_opcode(19, "13", "SLO", "IZY", 8);
        self.create_opcode(20, "14", "NOP", "ZPX", 4);
        self.create_opcode(21, "15", "ORA", "ZPX", 4);
        self.create_opcode(22, "16", "ASL", "ZPX", 6);
        self.create_opcode(23, "17", "SLO", "ZPX", 6);
        self.create_opcode(24, "18", "CLC", "IMP", 2);
        self.create_opcode(25, "19", "ORA", "ABYr", 4);
        self.create_opcode(26, "1a", "NOP", "IMP", 2);
        self.create_opcode(27, "1b", "SLO", "ABY", 7);
        self.create_opcode(28, "1c", "NOP", "ABXr", 4);
        self.create_opcode(29, "1d", "ORA", "ABXr", 4);
        self.create_opcode(30, "1e", "ASL", "ABX", 7);
        self.create_opcode(31, "1f", "SLO", "ABX", 7);
        self.create_opcode(32, "20", "JSR", "ABS", 6);
        self.create_opcode(33, "21", "AND", "IZX", 6);
        self.create_opcode(34, "22", "KIL", "IMP", 2);
        self.create_opcode(35, "23", "RLA", "IZX", 8);
        self.create_opcode(36, "24", "BIT", "ZP", 3);
        self.create_opcode(37, "25", "AND", "ZP", 3);
        self.create_opcode(38, "26", "ROL", "ZP", 5);
        self.create_opcode(39, "27", "RLA", "ZP", 5);
        self.create_opcode(40, "28", "PLP", "IMP", 4);
        self.create_opcode(41, "29", "AND", "IMM", 2);
        self.create_opcode(42, "2a", "ROLA", "IMP", 2);
        self.create_opcode(43, "2b", "ANC", "IMM", 2);
        self.create_opcode(44, "2c", "BIT", "ABS", 4);
        self.create_opcode(45, "2d", "AND", "ABS", 4);
        self.create_opcode(46, "2e", "ROL", "ABS", 6);
        self.create_opcode(47, "2f", "RLA", "ABS", 6);
        self.create_opcode(48, "30", "BMI", "REL", 2);
        self.create_opcode(49, "31", "AND", "IZYr", 5);
        self.create_opcode(50, "32", "KIL", "IMP", 2);
        self.create_opcode(51, "33", "RLA", "IZY", 8);
        self.create_opcode(52, "34", "NOP", "ZPX", 4);
        self.create_opcode(53, "35", "AND", "ZPX", 4);
        self.create_opcode(54, "36", "ROL", "ZPX", 6);
        self.create_opcode(55, "37", "RLA", "ZPX", 6);
        self.create_opcode(56, "38", "SEC", "IMP", 2);
        self.create_opcode(57, "39", "AND", "ABYr", 4);
        self.create_opcode(58, "3a", "NOP", "IMP", 2);
        self.create_opcode(59, "3b", "RLA", "ABY", 7);
        self.create_opcode(60, "3c", "NOP", "ABXr", 4);
        self.create_opcode(61, "3d", "AND", "ABXr", 4);
        self.create_opcode(62, "3e", "ROL", "ABX", 7);
        self.create_opcode(63, "3f", "RLA", "ABX", 7);
        self.create_opcode(64, "40", "RTI", "IMP", 6);
        self.create_opcode(65, "41", "EOR", "IZX", 6);
        self.create_opcode(66, "42", "KIL", "IMP", 2);
        self.create_opcode(67, "43", "SRE", "IZX", 8);
        self.create_opcode(68, "44", "NOP", "ZP", 3);
        self.create_opcode(69, "45", "EOR", "ZP", 3);
        self.create_opcode(70, "46", "LSR", "ZP", 5);
        self.create_opcode(71, "47", "SRE", "ZP", 5);
        self.create_opcode(72, "48", "PHA", "IMP", 3);
        self.create_opcode(73, "49", "EOR", "IMM", 2);
        self.create_opcode(74, "4a", "LSRA", "IMP", 2);
        self.create_opcode(75, "4b", "ALR", "IMM", 2);
        self.create_opcode(76, "4c", "JMP", "ABS", 3);
        self.create_opcode(77, "4d", "EOR", "ABS", 4);
        self.create_opcode(78, "4e", "LSR", "ABS", 6);
        self.create_opcode(79, "4f", "SRE", "ABS", 6);
        self.create_opcode(80, "50", "BVC", "REL", 2);
        self.create_opcode(81, "51", "EOR", "IZYr", 5);
        self.create_opcode(82, "52", "KIL", "IMP", 2);
        self.create_opcode(83, "53", "SRE", "IZY", 8);
        self.create_opcode(84, "54", "NOP", "ZPX", 4);
        self.create_opcode(85, "55", "EOR", "ZPX", 4);
        self.create_opcode(86, "56", "LSR", "ZPX", 6);
        self.create_opcode(87, "57", "SRE", "ZPX", 6);
        self.create_opcode(88, "58", "CLI", "IMP", 2);
        self.create_opcode(89, "59", "EOR", "ABYr", 4);
        self.create_opcode(90, "5a", "NOP", "IMP", 2);
        self.create_opcode(91, "5b", "SRE", "ABY", 7);
        self.create_opcode(92, "5c", "NOP", "ABXr", 4);
        self.create_opcode(93, "5d", "EOR", "ABXr", 4);
        self.create_opcode(94, "5e", "LSR", "ABX", 7);
        self.create_opcode(95, "5f", "SRE", "ABX", 7);
        self.create_opcode(96, "60", "RTS", "IMP", 6);
        self.create_opcode(97, "61", "ADC", "IZX", 6);
        self.create_opcode(98, "62", "KIL", "IMP", 2);
        self.create_opcode(99, "63", "RRA", "IZX", 8);
        self.create_opcode(100, "64", "NOP", "ZP", 3);
        self.create_opcode(101, "65", "ADC", "ZP", 3);
        self.create_opcode(102, "66", "ROR", "ZP", 5);
        self.create_opcode(103, "67", "RRA", "ZP", 5);
        self.create_opcode(104, "68", "PLA", "IMP", 4);
        self.create_opcode(105, "69", "ADC", "IMM", 2);
        self.create_opcode(106, "6a", "RORA", "IMP", 2);
        self.create_opcode(107, "6b", "ARR", "IMM", 2);
        self.create_opcode(108, "6c", "JMP", "IND", 5);
        self.create_opcode(109, "6d", "ADC", "ABS", 4);
        self.create_opcode(110, "6e", "ROR", "ABS", 6);
        self.create_opcode(111, "6f", "RRA", "ABS", 6);
        self.create_opcode(112, "70", "BVS", "REL", 2);
        self.create_opcode(113, "71", "ADC", "IZYr", 5);
        self.create_opcode(114, "72", "KIL", "IMP", 2);
        self.create_opcode(115, "73", "RRA", "IZY", 8);
        self.create_opcode(116, "74", "NOP", "ZPX", 4);
        self.create_opcode(117, "75", "ADC", "ZPX", 4);
        self.create_opcode(118, "76", "ROR", "ZPX", 6);
        self.create_opcode(119, "77", "RRA", "ZPX", 6);
        self.create_opcode(120, "78", "SEI", "IMP", 2);
        self.create_opcode(121, "79", "ADC", "ABYr", 4);
        self.create_opcode(122, "7a", "NOP", "IMP", 2);
        self.create_opcode(123, "7b", "RRA", "ABY", 7);
        self.create_opcode(124, "7c", "NOP", "ABXr", 4);
        self.create_opcode(125, "7d", "ADC", "ABXr", 4);
        self.create_opcode(126, "7e", "ROR", "ABX", 7);
        self.create_opcode(127, "7f", "RRA", "ABX", 7);
        self.create_opcode(128, "80", "NOP", "IMM", 2);
        self.create_opcode(129, "81", "STA", "IZX", 6);
        self.create_opcode(130, "82", "NOP", "IMM", 2);
        self.create_opcode(131, "83", "SAX", "IZX", 6);
        self.create_opcode(132, "84", "STY", "ZP", 3);
        self.create_opcode(133, "85", "STA", "ZP", 3);
        self.create_opcode(134, "86", "STX", "ZP", 3);
        self.create_opcode(135, "87", "SAX", "ZP", 3);
        self.create_opcode(136, "88", "DEY", "IMP", 2);
        self.create_opcode(137, "89", "NOP", "IMM", 2);
        self.create_opcode(138, "8a", "TXA", "IMP", 2);
        self.create_opcode(139, "8b", "UNI", "IMM", 2);
        self.create_opcode(140, "8c", "STY", "ABS", 4);
        self.create_opcode(141, "8d", "STA", "ABS", 4);
        self.create_opcode(142, "8e", "STX", "ABS", 4);
        self.create_opcode(143, "8f", "SAX", "ABS", 4);
        self.create_opcode(144, "90", "BCC", "REL", 2);
        self.create_opcode(145, "91", "STA", "IZY", 6);
        self.create_opcode(146, "92", "KIL", "IMP", 2);
        self.create_opcode(147, "93", "UNI", "IZY", 6);
        self.create_opcode(148, "94", "STY", "ZPX", 4);
        self.create_opcode(149, "95", "STA", "ZPX", 4);
        self.create_opcode(150, "96", "STX", "ZPY", 4);
        self.create_opcode(151, "97", "SAX", "ZPY", 4);
        self.create_opcode(152, "98", "TYA", "IMP", 2);
        self.create_opcode(153, "99", "STA", "ABY", 5);
        self.create_opcode(154, "9a", "TXS", "IMP", 2);
        self.create_opcode(155, "9b", "UNI", "ABY", 5);
        self.create_opcode(156, "9c", "UNI", "ABX", 5);
        self.create_opcode(157, "9d", "STA", "ABX", 5);
        self.create_opcode(158, "9e", "UNI", "ABY", 5);
        self.create_opcode(159, "9f", "UNI", "ABY", 5);
        self.create_opcode(160, "a0", "LDY", "IMM", 2);
        self.create_opcode(161, "a1", "LDA", "IZX", 6);
        self.create_opcode(162, "a2", "LDX", "IMM", 2);
        self.create_opcode(163, "a3", "LAX", "IZX", 6);
        self.create_opcode(164, "a4", "LDY", "ZP", 3);
        self.create_opcode(165, "a5", "LDA", "ZP", 3);
        self.create_opcode(166, "a6", "LDX", "ZP", 3);
        self.create_opcode(167, "a7", "LAX", "ZP", 3);
        self.create_opcode(168, "a8", "TAY", "IMP", 2);
        self.create_opcode(169, "a9", "LDA", "IMM", 2);
        self.create_opcode(170, "aa", "TAX", "IMP", 2);
        self.create_opcode(171, "ab", "UNI", "IMM", 2);
        self.create_opcode(172, "ac", "LDY", "ABS", 4);
        self.create_opcode(173, "ad", "LDA", "ABS", 4);
        self.create_opcode(174, "ae", "LDX", "ABS", 4);
        self.create_opcode(175, "af", "LAX", "ABS", 4);
        self.create_opcode(176, "b0", "BCS", "REL", 2);
        self.create_opcode(177, "b1", "LDA", "IZYr", 5);
        self.create_opcode(178, "b2", "KIL", "IMP", 2);
        self.create_opcode(179, "b3", "LAX", "IZYr", 5);
        self.create_opcode(180, "b4", "LDY", "ZPX", 4);
        self.create_opcode(181, "b5", "LDA", "ZPX", 4);
        self.create_opcode(182, "b6", "LDX", "ZPY", 4);
        self.create_opcode(183, "b7", "LAX", "ZPY", 4);
        self.create_opcode(184, "b8", "CLV", "IMP", 2);
        self.create_opcode(185, "b9", "LDA", "ABYr", 4);
        self.create_opcode(186, "ba", "TSX", "IMP", 2);
        self.create_opcode(187, "bb", "UNI", "ABYr", 4);
        self.create_opcode(188, "bc", "LDY", "ABXr", 4);
        self.create_opcode(189, "bd", "LDA", "ABXr", 4);
        self.create_opcode(190, "be", "LDX", "ABYr", 4);
        self.create_opcode(191, "bf", "LAX", "ABYr", 4);
        self.create_opcode(192, "c0", "CPY", "IMM", 2);
        self.create_opcode(193, "c1", "CMP", "IZX", 6);
        self.create_opcode(194, "c2", "NOP", "IMM", 2);
        self.create_opcode(195, "c3", "DCP", "IZX", 8);
        self.create_opcode(196, "c4", "CPY", "ZP", 3);
        self.create_opcode(197, "c5", "CMP", "ZP", 3);
        self.create_opcode(198, "c6", "DEC", "ZP", 5);
        self.create_opcode(199, "c7", "DCP", "ZP", 5);
        self.create_opcode(200, "c8", "INY", "IMP", 2);
        self.create_opcode(201, "c9", "CMP", "IMM", 2);
        self.create_opcode(202, "ca", "DEX", "IMP", 2);
        self.create_opcode(203, "cb", "AXS", "IMM", 2);
        self.create_opcode(204, "cc", "CPY", "ABS", 4);
        self.create_opcode(205, "cd", "CMP", "ABS", 4);
        self.create_opcode(206, "ce", "DEC", "ABS", 6);
        self.create_opcode(207, "cf", "DCP", "ABS", 6);
        self.create_opcode(208, "d0", "BNE", "REL", 2);
        self.create_opcode(209, "d1", "CMP", "IZYr", 5);
        self.create_opcode(210, "d2", "KIL", "IMP", 2);
        self.create_opcode(211, "d3", "DCP", "IZY", 8);
        self.create_opcode(212, "d4", "NOP", "ZPX", 4);
        self.create_opcode(213, "d5", "CMP", "ZPX", 4);
        self.create_opcode(214, "d6", "DEC", "ZPX", 6);
        self.create_opcode(215, "d7", "DCP", "ZPX", 6);
        self.create_opcode(216, "d8", "CLD", "IMP", 2);
        self.create_opcode(217, "d9", "CMP", "ABYr", 4);
        self.create_opcode(218, "da", "NOP", "IMP", 2);
        self.create_opcode(219, "db", "DCP", "ABY", 7);
        self.create_opcode(220, "dc", "NOP", "ABXr", 4);
        self.create_opcode(221, "dd", "CMP", "ABXr", 4);
        self.create_opcode(222, "de", "DEC", "ABX", 7);
        self.create_opcode(223, "df", "DCP", "ABX", 7);
        self.create_opcode(224, "e0", "CPX", "IMM", 2);
        self.create_opcode(225, "e1", "SBC", "IZX", 6);
        self.create_opcode(226, "e2", "NOP", "IMM", 2);
        self.create_opcode(227, "e3", "ISC", "IZX", 8);
        self.create_opcode(228, "e4", "CPX", "ZP", 3);
        self.create_opcode(229, "e5", "SBC", "ZP", 3);
        self.create_opcode(230, "e6", "INC", "ZP", 5);
        self.create_opcode(231, "e7", "ISC", "ZP", 5);
        self.create_opcode(232, "e8", "INX", "IMP", 2);
        self.create_opcode(233, "e9", "SBC", "IMM", 2);
        self.create_opcode(234, "ea", "NOP", "IMP", 2);
        self.create_opcode(235, "eb", "SBC", "IMM", 2);
        self.create_opcode(236, "ec", "CPX", "ABS", 4);
        self.create_opcode(237, "ed", "SBC", "ABS", 4);
        self.create_opcode(238, "ee", "INC", "ABS", 6);
        self.create_opcode(239, "ef", "ISC", "ABS", 6);
        self.create_opcode(240, "f0", "BEQ", "REL", 2);
        self.create_opcode(241, "f1", "SBC", "IZYr", 5);
        self.create_opcode(242, "f2", "KIL", "IMP", 2);
        self.create_opcode(243, "f3", "ISC", "IZY", 8);
        self.create_opcode(244, "f4", "NOP", "ZPX", 4);
        self.create_opcode(245, "f5", "SBC", "ZPX", 4);
        self.create_opcode(246, "f6", "INC", "ZPX", 6);
        self.create_opcode(247, "f7", "ISC", "ZPX", 6);
        self.create_opcode(248, "f8", "SED", "IMP", 2);
        self.create_opcode(249, "f9", "SBC", "ABYr", 4);
        self.create_opcode(250, "fa", "NOP", "IMP", 2);
        self.create_opcode(251, "fb", "ISC", "ABY", 7);
        self.create_opcode(252, "fc", "NOP", "ABXr", 4);
        self.create_opcode(253, "fd", "SBC", "ABXr", 4);
        self.create_opcode(254, "fe", "INC", "ABX", 7);
        self.create_opcode(255, "ff", "ISC", "ABX", 7);
        self.create_opcode(256, "100", "NMI", "NMI", 0);
        self.create_opcode(257, "101", "IRQ", "IRQ", 0);
    }
}
