const RAM_SIZE: usize = 0x100;
pub struct MMU {
    ram: [u8; RAM_SIZE],
}

impl MMU {
    pub fn new() -> Self {
        MMU {
            ram: [0; RAM_SIZE],
        }
    }

    pub fn reset(&self) {
        self.ram = [0; RAM_SIZE];
    }

    pub fn write(&self, start: usize, end: usize, data: &[u8]) {
        self.ram[start..end].copy_from_slice(data);
    }

    pub fn read_byte(&self, loc: usize) -> u8 {
        self.ram[loc]
    }

    pub fn read_word(&self, loc: usize) -> u16 {
        let upper = self.ram[loc + 1] as u16;
        let lower = self.ram[loc] as u16;
        (upper & lower)
    }


}