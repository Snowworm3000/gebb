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

// pub mod reg_code {
//     pub const A: u8 = 0;
//     pub const B: u8 = 1;
//     pub const C: u8 = 2;
//     pub const D: u8 = 3;
//     pub const E: u8 = 4;
//     pub const H: u8 = 5;
//     pub const L: u8 = 6;
//     pub const AF: u8 = 7;
//     pub const BC: u8 = 8;
//     pub const DE: u8 = 9;
//     pub const HL: u8 = 10;
// }

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

    // pub fn increment(&mut self, register: u8) {
    //     self.plus_or_minus(register, true);
    // }

    // pub fn decrement(&mut self, register: u8){
    //     self.plus_or_minus(register, false);
    // }

    // fn plus_or_minus(&mut self, register: u8, increment: bool){
    //     let add: i16 = if increment {1} else {-1};
    //     let add2: i32 = if increment {1} else {-1};

    //     match register {
    //         0 => {let mut temp = self.a as i16; temp += add; self.a = temp as u8;}
    //         1 => {let mut temp = self.b as i16; temp += add; self.b = temp as u8;}
    //         2 => {let mut temp = self.c as i16; temp += add; self.c = temp as u8;}
    //         3 => {let mut temp = self.d as i16; temp += add; self.d = temp as u8;}
    //         4 => {let mut temp = self.e as i16; temp += add; self.e = temp as u8;}
    //         5 => {let mut temp = self.h as i16; temp += add; self.h = temp as u8;}
    //         6 => {let mut temp = self.l as i16; temp += add; self.l = temp as u8;}
    //         7 => {let mut temp = self.get_af() as i32; temp += add2; self.set_af(temp as u16);}
    //         8 => {let mut temp = self.get_bc() as i32; temp += add2; self.set_bc(temp as u16);}
    //         9 => {let mut temp = self.get_de() as i32; temp += add2; self.set_de(temp as u16);}
    //         10 => {let mut temp = self.get_hl() as i32; temp += add2; self.set_hl(temp as u16);}
    //         _ => {unimplemented!("Unimplemented register");}
    //     }
    // }

    pub fn get_flag(&self, flag: u8) -> bool {
        let flag = (self.f >> flag) & 0b1;
        if flag == 1 {return true} else {return false};
    }

    pub fn set_flag(&mut self, flag: u8, value: bool) {
        self.f = self.f & !(1 << flag) | (u8::from(value) << flag);
    }
    
    pub fn unset_flags(&mut self) { // TODO: Refactor code to use this when possible.
        self.f = 0;
    }

    
}

