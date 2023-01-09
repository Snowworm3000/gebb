mod registers;
use registers::*;
const RAM_SIZE: usize = 0x100;
const START_ADDR: usize = 0x0;

pub struct Cpu {
    reg: Registers,
    ram: [u8; RAM_SIZE],
    pc: u16,
    sp: u16,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            reg: Registers::new_empty(),
            ram: [0; RAM_SIZE],
            pc: 0,
            sp: 0,
        }
    }
    pub fn reset(&mut self) {
        self.reg = Registers::new_empty();
        self.ram = [0; RAM_SIZE];
        self.pc = 0;
        self.sp = 0;
    }

    fn execute(&mut self, op: u8) {
        let digit1 = (op & 0xF0) >> 4;
        let digit2 = op & 0x0F;

        match (digit1, digit2) {
            (_, _) => unimplemented!("Unimplemented opcode: {:#04x}", op),
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    fn fetch(&mut self) -> u8 {
        let op = self.ram[self.pc as usize] as u8;
        self.pc += 2;
        op
    }

    pub fn tick(&mut self) {
        let op = self.fetch();

        self.execute(op);
    }
}
