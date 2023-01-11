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
        let byte = self.mmu.read_byte(self.pc as usize);
        self.pc += 1;
        byte
    }

    fn fetch_word(&mut self) -> u16 {
        let word = self.mmu.read_word(self.pc as usize);
        self.pc += 2;
        word
    }

    fn execute(&mut self, op: u8) {
        let timing = match op {
            0x01 => {let word = self.fetch_word(); self.reg.set_bc(word); 3}
            0x02 => {self.mmu.write_byte(self.reg.get_bc() as usize, self.reg.a); 2}
            0x03 => {self.reg.set_bc(self.reg.get_bc().wrapping_add(1)); 2}
            0x04 => {self.reg.b = self.inc(self.reg.b); 1}
            0x05 => {self.reg.b = self.dec(self.reg.b); 1}
            0x06 => {self.reg.b = self.fetch_byte(); 2}

            0x0c => {self.reg.c += 1; 1}
            0x0d => {self.reg.c -= 1; 1}
            0x0e => {self.reg.c = self.fetch_byte(); 2}

            0x11 => {let word = self.fetch_word(); self.reg.set_de(word); 3}
            0x12 => {self.mmu.write_byte(self.reg.get_de() as usize, self.reg.a); 2}

            0x1c => {self.reg.e += 1; 1}
            0x1d => {self.reg.e -= 1; 1}
            0x1e => {self.reg.e = self.fetch_byte(); 2}
            
            0x20 => {if !self.reg.get_flag(flags::Z) {self.jr(); 3} else {self.pc += 1; 2}}
            0x21 => {let word = self.fetch_word(); self.reg.set_hl(word); 3}
            0x22 => {self.mmu.write_byte(self.reg.get_hl() as usize, self.reg.a); self.reg.set_hl(self.reg.get_hl() + 1); 2}

            0x2c => {self.reg.l += 1; 1}
            0x2d => {self.reg.l -= 1; 1}
            0x2e => {self.reg.l = self.fetch_byte(); 2}

            0x30 => {if !self.reg.get_flag(flags::C) {self.jr(); 3} else {self.pc += 1; 2}}
            0x31 => {self.sp = self.fetch_word(); 3}
            0x32 => {self.mmu.write_byte(self.reg.get_hl() as usize, self.reg.a); self.reg.set_hl(self.reg.get_hl() - 1); 2}

            0x3c => {self.reg.a += 1; 1}
            0x3d => {self.reg.a -= 1; 1}
            0x3e => {self.reg.a = self.fetch_byte(); 2}

            // 0x77 => {let self.mmu.read_pointer(self.reg.get_hl()); }

            0xaf => {self.xor(self.reg.a); 1}

            0xe2 => {let pointer = self.mmu.read_pointer(0xff00) as usize; self.mmu.write_byte(pointer, self.reg.a); 2}

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

    fn rla(&mut self, val: u8) -> u8 {
        self.reg.set_flag(flags::C, (val >> 7) == 1);
        val.rotate_left(1)
    }

    fn rlca(&mut self, val: u8) -> u8 {
        let right_bit = (if self.reg.get_flag(flags::C) {1 as u8} else {0 as u8}) >> 7;
        self.reg.set_flag(flags::C, (val >> 7) == 1);
        (val << 1) | right_bit
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
    fn rla() {
        let mut cpu = Cpu::new();
        assert_eq!(cpu.rla(0b10101010), 0b01010101);
        
        assert_eq!(cpu.rla(0b01010101), 0b10101010);
    }
}