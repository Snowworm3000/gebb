use crate::mbc;

const ROM_SIZE: usize = 0x8000;
const RAM_SIZE: usize = 0x2000;

pub struct MMU {
    rom: [u8; ROM_SIZE],
    ram: [u8; RAM_SIZE],
}

impl MMU {
    pub fn new() -> Self {
        let mut mmu = MMU {
            rom: [0; ROM_SIZE],
            ram: [0; RAM_SIZE],
        };
        // mmu.reset();
        mmu
    }

    pub fn reset(&mut self) {
        self.ram = [0; RAM_SIZE];
        self.write_byte(0xFF05, 0);
        self.write_byte(0xFF06, 0);
        self.write_byte(0xFF07, 0);
        self.write_byte(0xFF10, 0x80);
        self.write_byte(0xFF11, 0xBF);
        self.write_byte(0xFF12, 0xF3);
        self.write_byte(0xFF14, 0xBF);
        self.write_byte(0xFF16, 0x3F);
        self.write_byte(0xFF16, 0x3F);
        self.write_byte(0xFF17, 0);
        self.write_byte(0xFF19, 0xBF);
        self.write_byte(0xFF1A, 0x7F);
        self.write_byte(0xFF1B, 0xFF);
        self.write_byte(0xFF1C, 0x9F);
        self.write_byte(0xFF1E, 0xFF);
        self.write_byte(0xFF20, 0xFF);
        self.write_byte(0xFF21, 0);
        self.write_byte(0xFF22, 0);
        self.write_byte(0xFF23, 0xBF);
        self.write_byte(0xFF24, 0x77);
        self.write_byte(0xFF25, 0xF3);
        self.write_byte(0xFF26, 0xF1);
        self.write_byte(0xFF40, 0x91);
        self.write_byte(0xFF42, 0);
        self.write_byte(0xFF43, 0);
        self.write_byte(0xFF45, 0);
        self.write_byte(0xFF47, 0xFC);
        self.write_byte(0xFF48, 0xFF);
        self.write_byte(0xFF49, 0xFF);
        self.write_byte(0xFF4A, 0);
        self.write_byte(0xFF4B, 0);
    }

    pub fn load(&mut self, data: &[u8]) {
        let mbc_type = data[0x147];
        let rom_size = data[0x148];
        let ram_size = data[0x149];
        println!("MBC info, {} {} {}", mbc_type, rom_size, ram_size);
        self.write(0x0, ROM_SIZE -1, data);
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