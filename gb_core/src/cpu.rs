mod registers;
use registers::*;
mod mmu;
use mmu::*;
const RAM_SIZE: usize = 0x100;
const START_ADDR: usize = 0x0;

pub struct Cpu {
    reg: Registers,
    // ram: [u8; RAM_SIZE],
    pc: u16,
    sp: u16,
    mmu: MMU
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            reg: Registers::new_empty(),
            // ram: [0; RAM_SIZE],
            pc: 0,
            sp: 0,
            mmu: MMU::new()
        }
    }
    pub fn reset(&mut self) {
        self.reg = Registers::new_empty();
        // self.ram = [0; RAM_SIZE];
        self.pc = 0;
        self.sp = 0;
        self.mmu.reset();
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        // self.ram[start..end].copy_from_slice(data);
        self.mmu.write(start, end, data);
    }

    // fn fetch(&mut self) -> u8 {
    //     let op = self.ram[self.pc as usize] as u8;
    //     self.pc += 1;
    //     op
    // }

    pub fn tick(&mut self) {
        let op = self.fetch_byte();

        self.execute(op);
    }

    fn fetch_byte(&mut self) -> u8 {
        let byte = self.mmu.read_byte(self.pc);
        self.pc += 1;
        byte
    }

    fn fetch_word(&mut self) -> u16 {
        let word = self.mmu.read_word(self.pc);
        self.pc += 2;
        word
    }

    fn execute(&mut self, op: u8) {
        let timing = match op {
            // Notation for LD functions:
            // LD(to_set, set_with)
            0x00 => {1}
            0x01 => {let word = self.fetch_word(); self.reg.set_bc(word); 3}
            0x02 => {self.mmu.write_byte(self.reg.get_bc(), self.reg.a); 2}
            0x03 => {self.reg.set_bc(self.reg.get_bc().wrapping_add(1)); 2}
            0x04 => {self.reg.b = self.inc(self.reg.b); 1}
            0x05 => {self.reg.b = self.dec(self.reg.b); 1}
            0x06 => {self.reg.b = self.fetch_byte(); 2}
            0x07 => {self.reg.a = self.rlc(self.reg.a); 1}
            0x08 => {let word = self.fetch_word(); self.mmu.write_word(word, self.sp); 5}
            0x09 => {let res = self.add_word(self.reg.get_hl(), self.reg.get_bc()); self.reg.set_hl(res); 2}
            0x0a => {self.reg.a = self.mmu.read_byte(self.reg.get_bc()); 2}
            0x0b => {self.reg.set_bc(self.reg.get_bc().wrapping_sub(1)); 2}
            0x0c => {self.reg.c = self.inc(self.reg.c); 1}
            0x0d => {self.reg.c = self.dec(self.reg.c); 1}
            0x0e => {self.reg.c = self.fetch_byte(); 2}
            0x0f => {self.reg.a = self.rrc(self.reg.a); 1}

            0x10 => {
                // TODO: Implement functionality in game loop
                todo!("Not implemented")
            }
            0x11 => {let word = self.fetch_word(); self.reg.set_de(word); 3}
            0x12 => {self.mmu.write_byte(self.reg.get_de(), self.reg.a); 2}
            0x13 => {self.reg.set_de(self.reg.get_de().wrapping_add(1)); 2}
            0x14 => {self.reg.d = self.inc(self.reg.d); 1}
            0x15 => {self.reg.d = self.dec(self.reg.d); 1}
            0x16 => {self.reg.d = self.fetch_byte(); 2}
            0x17 => {self.reg.a = self.rl(self.reg.a); 1}

            0x19 => {let res = self.add_word(self.reg.get_hl(), self.reg.get_de()); self.reg.set_hl(res); 2}
            0x1a => {self.reg.a = self.mmu.read_byte(self.reg.get_de()); 2}
            0x1b => {self.reg.set_bc(self.reg.get_de().wrapping_sub(1)); 2}
            0x1c => {self.reg.e = self.inc(self.reg.e); 1}
            0x1d => {self.reg.e = self.dec(self.reg.e); 1}
            0x1e => {self.reg.e = self.fetch_byte(); 2}
            
            0x20 => {if !self.reg.get_flag(flags::Z) {self.jr(); 3} else {self.pc += 1; 2}}
            0x21 => {let word = self.fetch_word(); self.reg.set_hl(word); 3}
            0x22 => {self.mmu.write_byte(self.reg.get_hl(), self.reg.a); self.reg.set_hl(self.reg.get_hl() + 1); 2}
            0x23 => {self.reg.set_hl(self.reg.get_hl().wrapping_add(1)); 2}
            0x24 => {self.reg.h = self.inc(self.reg.h); 1}
            0x25 => {self.reg.h = self.dec(self.reg.h); 1}
            0x26 => {self.reg.h = self.fetch_byte(); 2}
            0x27 => { // DAA - Decimal adjust accumulator to get a correct BCD representation after an arithmetic instruction.
                self.reg.a = self.daa(self.reg.a); 1
            }

            0x2b => {self.reg.set_hl(self.reg.get_hl().wrapping_sub(1)); 2}
            0x2c => {self.reg.l += 1; 1}
            0x2d => {self.reg.l -= 1; 1}
            0x2e => {self.reg.l = self.fetch_byte(); 2}
            0x2f => {self.reg.a = !self.reg.a; self.reg.set_flag(flags::N, true); self.reg.set_flag(flags::H, true); 1}

            0x30 => {if !self.reg.get_flag(flags::C) {self.jr(); 3} else {self.pc += 1; 2}}
            0x31 => {self.sp = self.fetch_word(); 3}
            0x32 => {self.mmu.write_byte(self.reg.get_hl(), self.reg.a); self.reg.set_hl(self.reg.get_hl() - 1); 2}
            0x33 => {self.sp += 1; 2}
            0x34 => {let v = self.inc(self.mmu.read_byte(self.reg.get_hl())); self.mmu.write_byte(self.reg.get_hl(), v); 3}
            0x35 => {let v = self.dec(self.mmu.read_byte(self.reg.get_hl())); self.mmu.write_byte(self.reg.get_hl(), v); 3}
            0x36 => {let v = self.fetch_byte(); self.mmu.write_byte(self.reg.get_hl(), v); 3}

            0x3b => {self.sp -= 1; 2}
            0x3c => {self.reg.a += 1; 1}
            0x3d => {self.reg.a -= 1; 1}
            0x3e => {self.reg.a = self.fetch_byte(); 2}
            0x3f => {self.reg.set_flag(flags::C, !self.reg.get_flag(flags::C)); self.reg.set_flag(flags::N, false); self.reg.set_flag(flags::H, false); 1}

            0xaf => {self.xor(self.reg.a); 1}

            0xe2 => {self.mmu.write_byte(self.mmu.read_byte(0xff00 + (self.reg.c as u16)) as u16, self.reg.a); 2}

            0xcb => {
                let op = self.fetch_byte();
                let timing = match op {
                    0x7c => {self.bit(7, self.reg.h); 2}
                    _ => unimplemented!("Unimplemented CB prefixed opcode: {:#04x}", op)
                };
                timing + 1
            }
            _ => unimplemented!("Unimplemented opcode: {:#04x}", op),
        };
        print!("length of execution {}\n", timing);
    }

    fn daa(&mut self, hex: u8) -> u8 {
        let mut high = hex & 0xF0;
        let mut low = hex & 0x0F;
        if low > 9 {
            high += low - 9;
            low -= 9;
            return high & low;
        } else {
            return hex;
        }
    }

    fn add_byte(&mut self, a: u8, b: u8) -> u8 { // TODO: Write tests
        let (result, carry) = a.overflowing_add(b);
        self.reg.set_flag(flags::C, carry);
        self.reg.set_flag(flags::H ,((self.reg.b & 0xF + self.reg.c & 0xF) & 0xF0) != 0);
        self.reg.set_flag(flags::N, false);
        result
    }

    fn add_word(&mut self, a: u16, b: u16) -> u16 { // TODO: Write tests
        let (result, carry) = a.overflowing_add(b);
        self.reg.set_flag(flags::C, carry);
        self.reg.set_flag(flags::H ,((self.reg.b as u16 + self.reg.c as u16) & 0xFF00) != 0);
        self.reg.set_flag(flags::N, false);
        result
    }

    fn rl(&mut self, val: u8) -> u8 {
        self.reg.set_flag(flags::C, (val >> 7) == 1);
        val.rotate_left(1)
    }

    fn rlc(&mut self, val: u8) -> u8 {
        let right_bit = if self.reg.get_flag(flags::C) {1 as u8} else {0 as u8};
        self.reg.set_flag(flags::C, (val >> 7) == 1);    
        (val << 1) | right_bit
    }

    fn rr(&mut self, val: u8) -> u8 {
        println!("and {}", val & 1);
        self.reg.set_flag(flags::C, (val & 1) == 1);
        val.rotate_right(1)
    }

    fn rrc(&mut self, val: u8) -> u8 {
        let left_bit = (if self.reg.get_flag(flags::C) {1 as u8} else {0 as u8}) << 7;
        self.reg.set_flag(flags::C, (val & 1) == 1);    
        (val >> 1) | left_bit
    }

    fn inc(&mut self, val: u8) -> u8 {
        let (res, carry) = val.overflowing_add(1);
        if res == 0 {self.reg.set_flag(flags::Z, true)} else {self.reg.set_flag(flags::Z, false)}
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::H, carry);
        res
    }

    fn dec(&mut self, val: u8) -> u8 {
        let (res, carry) = val.overflowing_sub(1);
        self.reg.set_flag(flags::Z, res == 0);
        self.reg.set_flag(flags::N, true);
        self.reg.set_flag(flags::H, carry);
        res
    }

    fn xor(&mut self, val: u8) {
        self.reg.a |= val; 
    }

    fn bit(&mut self, pos: u8, reg: u8){ // TODO: Less unnecessary casting could improve performance
        let bit = if (reg >> pos) == 1 {true} else {false};
        self.reg.set_flag(pos, bit);
    }

    fn jr(&mut self) {
        self.pc = self.pc + (self.fetch_byte() as i8) as u16
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rl() {
        let mut cpu = Cpu::new();
        assert_eq!(cpu.rl(0b10101010), 0b01010101);
        assert_eq!(cpu.reg.get_flag(flags::C), true);
        
        assert_eq!(cpu.rl(0b01010101), 0b10101010);
        assert_eq!(cpu.reg.get_flag(flags::C), false);
    }

    #[test]
    fn rlc() {
        let mut cpu = Cpu::new();
        assert_eq!(cpu.rlc(0b10101010), 0b01010100);
        assert_eq!(cpu.reg.get_flag(flags::C), true);
        
        assert_eq!(cpu.rlc(0b01010100), 0b10101001);
        assert_eq!(cpu.reg.get_flag(flags::C), false);
    }

    #[test]
    fn rr() {
        let mut cpu = Cpu::new();
        assert_eq!(cpu.rr(0b10000001), 0b11000000);
        assert_eq!(cpu.reg.get_flag(flags::C), true);
        
        assert_eq!(cpu.rr(0b11000000), 0b01100000);
        assert_eq!(cpu.reg.get_flag(flags::C), false);
    }

    #[test]
    fn rrc() {
        let mut cpu = Cpu::new();
        assert_eq!(cpu.rrc(0b10000001), 0b01000000);
        assert_eq!(cpu.reg.get_flag(flags::C), true);
        
        assert_eq!(cpu.rrc(0b01000000), 0b10100000);
        assert_eq!(cpu.reg.get_flag(flags::C), false);
    }

    #[test]
    fn set_flag() {
        let mut cpu = Cpu::new();
        assert_eq!(cpu.reg.get_flag(flags::C), false);

        cpu.reg.set_flag(flags::C, true);
        assert_eq!(cpu.reg.get_flag(flags::C), true);

        cpu.reg.set_flag(flags::C, false);
        assert_eq!(cpu.reg.get_flag(flags::C), false);
        assert!((0b10101010 >> 7) == 1);
    }
}