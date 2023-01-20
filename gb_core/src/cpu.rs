mod registers;
use registers::*;
mod mmu;
use mmu::*;
const RAM_SIZE: usize = 0x100; // I'm not entirely sure how large this should be yet.
const STACK_SIZE: usize = 0xFF; // I'm not sure how large this should be either, just increase the size if anything bad happens.
const START_ADDR: usize = 0x0;

pub struct Cpu {
    reg: Registers,
    // ram: [u8; RAM_SIZE],
    pc: u16,
    sp: u16,
    ime: bool,
    stack: [u16; STACK_SIZE],
    mmu: MMU
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            reg: Registers::new_empty(),
            // ram: [0; RAM_SIZE],
            pc: 0,
            sp: 0,
            ime: false,
            stack: [0; STACK_SIZE],
            mmu: MMU::new()
        }
    }
    pub fn reset(&mut self) {
        self.reg = Registers::new_empty();
        // self.ram = [0; RAM_SIZE];
        self.pc = 0;
        self.sp = 0;
        self.ime = false;
        self.stack = [0; STACK_SIZE];
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
            0x18 => {self.jr(); 3}
            0x19 => {let res = self.add_word(self.reg.get_hl(), self.reg.get_de()); self.reg.set_hl(res); 2}
            0x1a => {self.reg.a = self.mmu.read_byte(self.reg.get_de()); 2}
            0x1b => {self.reg.set_bc(self.reg.get_de().wrapping_sub(1)); 2}
            0x1c => {self.reg.e = self.inc(self.reg.e); 1}
            0x1d => {self.reg.e = self.dec(self.reg.e); 1}
            0x1e => {self.reg.e = self.fetch_byte(); 2}
            0x1f => {self.reg.a = self.rr(self.reg.a); 1}
            
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
            0x28 => {if self.reg.get_flag(flags::Z) {self.jr(); 3} else {self.pc += 1; 2}}
            0x29 => {let res = self.add_word(self.reg.get_hl(), self.reg.get_hl()); self.reg.set_hl(res); 2}
            0x2a => {self.reg.a = self.mmu.read_byte(self.reg.get_hl() + 1); 2}
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
            0x37 => {
                self.reg.set_flag(flags::C, true);
                self.reg.set_flag(flags::N, false);
                self.reg.set_flag(flags::H, false);
                1
            }
            0x38 => {if self.reg.get_flag(flags::C) {self.jr(); 3} else {self.pc += 1; 2}}
            0x39 => {let res = self.add_word(self.reg.get_hl(), self.sp); self.reg.set_hl(res); 2}
            0x3a => {self.reg.a = self.mmu.read_byte(self.reg.get_hl() - 1); 2}
            0x3b => {self.sp -= 1; 2}
            0x3c => {self.reg.a += 1; 1}
            0x3d => {self.reg.a -= 1; 1}
            0x3e => {self.reg.a = self.fetch_byte(); 2}
            0x3f => {self.reg.set_flag(flags::C, !self.reg.get_flag(flags::C)); self.reg.set_flag(flags::N, false); self.reg.set_flag(flags::H, false); 1}

            0x40 => {1} // If you ever feel useless, remember this opcode exists.
            0x41 => {self.reg.b = self.reg.c; 1}
            0x42 => {self.reg.b = self.reg.d; 1}
            0x43 => {self.reg.b = self.reg.e; 1}
            0x44 => {self.reg.b = self.reg.h; 1}
            0x45 => {self.reg.b = self.reg.l; 1}
            0x46 => {self.reg.b = self.mmu.read_byte(self.reg.get_hl()); 2}
            0x47 => {self.reg.b = self.reg.a; 1}
            0x48 => {self.reg.c = self.reg.b; 1}
            0x49 => {1}
            0x4a => {self.reg.c = self.reg.d; 1}
            0x4b => {self.reg.c = self.reg.e; 1}
            0x4c => {self.reg.c = self.reg.h; 1}
            0x4d => {self.reg.c = self.reg.l; 1}
            0x4e => {self.reg.c = self.mmu.read_byte(self.reg.get_hl()); 2}
            0x4f => {self.reg.c = self.reg.a; 1}

            0x77 => {self.mmu.write_byte(self.reg.get_hl(), self.reg.a); 1}

            0xaf => {self.xor(self.reg.a); 1}

            0xc0 => {if !self.reg.get_flag(flags::Z) {self.ret(); 5} else {2}}
            0xc1 => {let v = self.pop(); self.reg.set_bc(v); 3}
            0xc2 => {if !self.reg.get_flag(flags::Z) {self.pc = self.fetch_word(); 4} else {3}}
            0xc3 => {self.pc = self.fetch_word(); 4}
            0xc4 => {if !self.reg.get_flag(flags::Z) {let p = self.fetch_word(); self.call(p); 6} else {3}}
            0xc5 => {self.push(self.reg.get_bc()); 4}
            0xc6 => {let v = self.fetch_byte(); self.reg.a = self.add_byte(self.reg.a, v); 2}
            0xc7 => {self.call(0x00); 4}
            0xc8 => {if self.reg.get_flag(flags::Z) {self.ret(); 5} else {2}}
            0xc9 => {self.ret(); 4}
            0xca => {if self.reg.get_flag(flags::Z) {self.jr(); 4} else {3}}
            
            0xcc => {if self.reg.get_flag(flags::Z) {let p = self.fetch_word(); self.call(p); 6} else {3}}
            0xcd => {let p = self.fetch_word(); self.call(p); 6}
            0xce => {let v = self.fetch_byte(); self.adc(v); 2}
            0xcf => {self.call(0x08); 4}

            0xd0 => {if !self.reg.get_flag(flags::C) {self.ret(); 5} else {2}}
            0xd1 => {let v = self.pop(); self.reg.set_de(v); 3}
            0xd2 => {if !self.reg.get_flag(flags::C) {self.pc = self.fetch_word(); 4} else {3}}
            0xd4 => {if !self.reg.get_flag(flags::C) {let p = self.fetch_word(); self.call(p); 6} else {3}}
            0xd5 => {self.push(self.reg.get_de()); 4}
            0xd6 => {let v = self.fetch_byte(); self.reg.a = self.sub_byte(self.reg.a, v); 2}
            0xd7 => {self.call(0x10); 4}
            0xd8 => {if self.reg.get_flag(flags::C) {self.ret(); 5} else {2}}
            0xd9 => {self.reti(); 4}
            0xda => {if self.reg.get_flag(flags::C) {self.jr(); 4} else {3}}

            0xdc => {if self.reg.get_flag(flags::C) {let p = self.fetch_word(); self.call(p); 6} else {3}}

            0xde => {let v = self.fetch_byte(); self.sbc(v); 2}
            0xdf => {self.call(0x18); 4}

            0xe0 => {let v = self.fetch_byte() as u16; self.mmu.write_byte(self.mmu.read_byte(0xff00 + v) as u16, self.reg.a); 3}
            0xe1 => {let v = self.pop(); self.reg.set_hl(v); 3}
            0xe2 => {self.mmu.write_byte(self.mmu.read_byte(0xff00 + (self.reg.c as u16)) as u16, self.reg.a); 2}

            0xe5 => {self.push(self.reg.get_hl()); 4}
            0xe6 => {let v = self.fetch_byte(); self.and(v); 2}
            0xe7 => {self.call(0x20); 4}
            0xe8 => {let v = self.fetch_word(); self.add_word_z(self.sp, v); 4}
            0xe9 => {self.pc = self.reg.get_hl(); 1}
            0xea => {let pointer = self.fetch_word(); self.mmu.write_byte(pointer, self.reg.a); 4}

            0xee => {let v = self.fetch_byte(); self.xor(v); 2}
            0xef => {self.call(0x28); 4}

            0xf0 => {let v = self.fetch_byte() as u16; self.reg.a = self.mmu.read_byte(0xff00 + v); 3}
            0xf1 => { // This pop is slightly different.
                let v = self.pop(); self.reg.set_af(v); 
                self.reg.set_flag(flags::Z, (v >> 6) & 0b1 == 1);
                self.reg.set_flag(flags::N, (v >> 5) & 0b1 == 1);
                self.reg.set_flag(flags::H, (v >> 4) & 0b1 == 1);
                self.reg.set_flag(flags::C, (v >> 3) & 0b1 == 1);
                3
            }
            0xf2 => {let v = self.reg.c as u16; self.reg.a = self.mmu.read_byte(0xff00 + v); 2}
            0xf3 => {self.di(); 1}
            
            0xf5 => {self.push(self.reg.get_af()); 4}
            0xf6 => {let v = self.fetch_byte(); self.or(v); 2}
            0xf7 => {self.call(0x30); 4}
            0xf8 => {
                let v = (self.fetch_word() as i8) as u16; 
                self.reg.set_hl(self.sp + v as u16); 
                self.reg.set_flag(flags::Z, false);
                self.reg.set_flag(flags::N, false);
                self.reg.set_flag(flags::H, (((self.sp >> 3) & 0b1) & (v >> 3) & 0b1) == 1);
                self.reg.set_flag(flags::C, (((self.sp >> 7) & 0b1) & (v >> 7) & 0b1) == 1);
                3
            }
            0xf9 => {self.sp = self.reg.get_hl(); 2}
            0xfa => {let pointer = self.fetch_word(); self.reg.a = self.mmu.read_byte(pointer); 4}
            0xfb => {self.ei(); 1}

            0xfe => {let v = self.fetch_byte(); self.cp(v); 2}
            0xff => {self.call(0x38); 4}

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

    fn adc(&mut self, val: u8) {
        let orig = self.reg.a;
        self.reg.a = self.reg.a + val + if self.reg.get_flag(flags::C) {1} else {0};
        self.reg.set_flag(flags::Z, self.reg.a == 0);
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::H, ((orig >> 3) & 0b1) != ((self.reg.a >> 3) & 0b1));
        self.reg.set_flag(flags::C, ((orig >> 7) & 0b1) != ((self.reg.a >> 7) & 0b1));

    }

    fn sbc(&mut self, val: u8) {
        let orig = self.reg.a;
        let carry = if self.reg.get_flag(flags::C) {1} else {0};
        self.reg.a = self.reg.a - val - carry;
        self.reg.set_flag(flags::Z, self.reg.a == 0);
        self.reg.set_flag(flags::N, true);
        self.reg.set_flag(flags::H, ((orig >> 3) & 0b1) != ((self.reg.a >> 3) & 0b1));
        self.reg.set_flag(flags::C, (val + carry) > orig);
    }

    fn daa(&mut self, hex: u8) -> u8 {
        let mut high = hex & 0xF0;
        let mut low = hex & 0x0F;
        self.reg.set_flag(flags::H, false);
        if low > 9 {
            high += low - 9;
            low -= 9;
            self.reg.set_flag(flags::C, true);
            self.reg.set_flag(flags::Z, (high & low) == 0);
            return high & low;
        } else {
            self.reg.set_flag(flags::C, false);
            self.reg.set_flag(flags::Z, hex == 0);
            return hex;
        }
    }

    fn cp(&mut self, val: u8) {
        self.reg.a = self.sub_byte(self.reg.a, val);
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

    fn add_word_z(&mut self, a: u16, b: u16) -> u16 { // It would be good if rust had an easy way to provide optional parameters for this case https://stackoverflow.com/questions/24047686/default-function-arguments-in-rust
        let (result, carry) = a.overflowing_add(b);
        self.reg.set_flag(flags::C, carry);
        self.reg.set_flag(flags::H ,((self.reg.b as u16 + self.reg.c as u16) & 0xFF00) != 0);
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::Z, false);
        result
    }

    fn and(&mut self, val: u8) {
        let res = self.reg.a & val;
        self.reg.set_flag(flags::Z, res == 0);
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::H, true);
        self.reg.set_flag(flags::C, false);
        self.reg.a = res;
    }

    fn or(&mut self, val: u8) {
        let res = self.reg.a | val;
        self.reg.set_flag(flags::Z, res == 0);
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::H, false);
        self.reg.set_flag(flags::C, false);
        self.reg.a = res; 
    }

    fn xor(&mut self, val: u8) {
        let res = self.reg.a ^ val;
        self.reg.set_flag(flags::Z, res == 0);
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::H, false);
        self.reg.set_flag(flags::C, false);
        self.reg.a = res;
    }

    fn sub_byte(&mut self, a: u8, b: u8) -> u8 { // TODO: Write tests
        let (result, carry) = a.overflowing_sub(b);
        self.reg.set_flag(flags::C, carry);
        self.reg.set_flag(flags::H ,((self.reg.b & 0xF + self.reg.c & 0xF) & 0xF0) != 0);
        self.reg.set_flag(flags::N, true);
        result
    }

    fn sub_word(&mut self, a: u16, b: u16) -> u16 { // TODO: Write tests
        let (result, carry) = a.overflowing_add(b);
        self.reg.set_flag(flags::C, carry);
        self.reg.set_flag(flags::H ,((self.reg.b as u16 + self.reg.c as u16) & 0xFF00) != 0);
        self.reg.set_flag(flags::N, true);
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

    fn bit(&mut self, pos: u8, reg: u8){ // TODO: Less unnecessary casting could improve performance
        let bit = if (reg >> pos) == 1 {true} else {false};
        self.reg.set_flag(pos, bit);
    }

    fn jr(&mut self) {
        self.pc = self.pc + (self.fetch_byte() as i8) as u16
    }

    fn ret(&mut self) {
        self.pc = self.stack[self.sp as usize];
        self.sp -= 1;
    }

    fn ei(&mut self) {
        self.ime = true;
    }
    fn di(&mut self) {
        self.ime = false;
    }

    fn reti(&mut self) {
        self.ei();
        self.ret();
    }

    fn call(&mut self, pointer: u16) {
        self.push(self.pc);
        self.pc = pointer;
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 { // TODO: Checks might need to be made here.
        self.sp -= 1;
        self.stack[self.sp as usize]
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