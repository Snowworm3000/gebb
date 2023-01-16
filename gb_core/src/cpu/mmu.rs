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

    pub fn write_byte(&mut self, pointer: u16, data: u8){
        self.ram[pointer as usize] = data;
    }

    pub fn write_word(&mut self, pointer: u16, data: u16){
        let data_h = (data >> 8) as u8;
        let data_l = data as u8;
        self.ram[pointer as usize] = data_h;
        self.ram[(pointer as usize) + 1] = data_l;
    }

    pub fn read_byte(&self, loc: u16) -> u8 {
        self.ram[loc as usize]
    }

    pub fn read_word(&self, loc: u16) -> u16 {
        self.read_byte(loc) as u16 | ((self.read_byte(loc + 1) as u16) << 8 )
    }


}