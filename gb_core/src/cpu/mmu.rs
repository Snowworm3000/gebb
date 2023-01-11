const RAM_SIZE: usize = 0x10000;
pub struct MMU {
    ram: [u8; RAM_SIZE],
}

impl MMU {
    pub fn new() -> Self {
        MMU {
            ram: [0; RAM_SIZE],
        }
    }

    pub fn reset(&mut self) {
        self.ram = [0; RAM_SIZE];
    }

    pub fn write(&mut self, start: usize, end: usize, data: &[u8]) {
        self.ram[start..end].copy_from_slice(data);
    }

    pub fn write_byte(&mut self, pointer: usize, data: u8){
        self.ram[pointer] = data;
    }

    pub fn read_pointer(&mut self, pointer: usize) -> u8{ // Read data at pointer location
        self.ram[pointer]
    }

    pub fn read_byte(&self, loc: usize) -> u8 {
        self.ram[loc]
    }

    pub fn read_word(&self, loc: usize) -> u16 {
        (self.read_byte(loc) as u16 | ((self.read_byte(loc + 1) as u16) << 8 ))
    }


}