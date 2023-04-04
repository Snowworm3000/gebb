pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,
}

pub mod flags {
    pub const Z: u8 = 7;
    pub const N: u8 = 6;
    pub const H: u8 = 5;
    pub const C: u8 = 4;
}

impl Registers{
    pub fn new_empty() -> Registers{
        Registers {a: 0, b: 0, c: 0, d: 0, e: 0, f: 0, h:0, l:0}
    }
    pub fn new_default() -> Registers{
        Registers { a: 0x11, b: 0x00, c: 0x13, d: 0x00, e: 0xd8, f: 0xb0, h: 0x01, l: 0x4d}
    }
    pub fn get_af(&self) -> u16{
        (self.a as u16) << 8 | self.f as u16
    }
    pub fn get_bc(&self) -> u16{
        (self.b as u16) << 8 | self.c as u16
    }
    pub fn get_de(&self) -> u16{
        (self.d as u16) << 8 | self.e as u16
    }
    pub fn get_hl(&self) -> u16{
        (self.h as u16) << 8 | self.l as u16
    }
    pub fn set_af(&mut self, v:u16) {
        self.a = (v >> 8) as u8;
        self.f = v as u8;
    }
    pub fn set_bc(&mut self, v:u16) {
        self.b = (v >> 8) as u8;
        self.c = v as u8;
    }
    pub fn set_de(&mut self, v:u16) {
        self.d = (v >> 8) as u8;
        self.e = v as u8;
    }
    pub fn set_hl(&mut self, v:u16) {
        self.h = (v >> 8) as u8;
        self.l = v as u8;
    }

    pub fn get_flag(&self, flag: u8) -> bool {
        let flag = (self.f >> flag) & 0b1;
        if flag == 1 {return true} else {return false};
    }

    pub fn set_flag(&mut self, flag: u8, value: bool) {
        self.f = self.f & !(1 << flag) | (u8::from(value) << flag);
    }
    
    pub fn unset_flags(&mut self) { 
        self.f = 0;
    }

    pub fn hld(&mut self) -> u16 {
        let res = self.get_hl();
        self.set_hl(res - 1);
        res
    }
    pub fn hli(&mut self) -> u16 {
        let res = self.get_hl();
        self.set_hl(res + 1);
        res
    }
    
}

