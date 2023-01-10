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
        let length = match op {
            0x01 => {let word = self.fetch_word(); self.reg.set_bc(word); 3}

            0x11 => {let word = self.fetch_word(); self.reg.set_de(word); 3}
            
            0x21 => {let word = self.fetch_word(); self.reg.set_hl(word); 3}

            0x31 => {self.sp = self.fetch_word(); 3}
            0x32 => {self.reg.a -= self.reg.get_hl(); 1}

            0xaf => {self.xor(self.reg.a); 4}
            _ => unimplemented!("Unimplemented opcode: {:#04x}", op),
        };
        print!("length of execution {}\n", length);
    }

    fn xor(&mut self, val: u8) {
        self.reg.a |= val; 
    }
}
