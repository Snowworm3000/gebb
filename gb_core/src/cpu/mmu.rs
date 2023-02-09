use crate::mbc;

const ROM_SIZE: usize = 0x4000;
const RAM_SIZE: usize = 0x5000;

pub struct MMU {
    rom: [u8; ROM_SIZE],
    ram: [u8; RAM_SIZE],
    rom1: [u8; ROM_SIZE],
    rom2: [u8; ROM_SIZE],
    rom3: [u8; ROM_SIZE],
    rom4: [u8; ROM_SIZE],
    serial: u8,
    serial2: u8,
    hram: [u8; 0x7f],
    IE: u8,
    tac: u8,
    IF: u8,
}

impl MMU {
    pub fn new() -> Self {
        let mut mmu = MMU {
            rom: [0; ROM_SIZE],
            ram: [0; RAM_SIZE],
            rom1: [0; ROM_SIZE],
            rom2: [0; ROM_SIZE],
            rom3: [0; ROM_SIZE],
            rom4: [0; ROM_SIZE],
            serial: 0,
            serial2: 0,
            hram: [0; 0x7f],
            IE: 0,
            tac: 0,
            IF: 0,
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
        let rom_size = data[0x148] as usize;
        let rom_size_int = match rom_size {
            0 => {let size = 0x8000; self.load_rom(&data[0..(ROM_SIZE -1)], 0); size}
            1 => {let size = 0x10000; self.load_rom(&data[0..(ROM_SIZE -1)], 0); self.load_rom(&data[0..(ROM_SIZE-1)], 1); size}
            _ => unimplemented!("Size not implemented")
        };
        let ram_size = data[0x149];
        println!("MBC info, {} {} {}", mbc_type, rom_size, ram_size);
        // self.write(0x0, ROM_SIZE -1, data);
        
    }

    fn load_rom(&mut self, data: &[u8], bank: u8) {
        match bank {
            0 => {self.rom[0x0000..(ROM_SIZE -1)].copy_from_slice(data);}
            1 => {self.rom1[0x0000..(ROM_SIZE -1)].copy_from_slice(data);}
            2 => {self.rom2[0x0000..(ROM_SIZE -1)].copy_from_slice(data);}
            3 => {self.rom3[0x0000..(ROM_SIZE -1)].copy_from_slice(data);}
            4 => {self.rom4[0x0000..(ROM_SIZE -1)].copy_from_slice(data);}
            _ => {unimplemented!("Not enough banks")}
        }
    }

    // pub fn write(&mut self, start: usize, end: usize, data: &[u8]) {
    //     self.ram[start..end].copy_from_slice(data);
    // }

    pub fn write_byte(&mut self, pointer: u16, data: u8){
        // self.ram[pointer as usize] = data;
        match pointer {
            0x0000..=0x7fff=> {unimplemented!("Attempt to write rom {:#04x}", data)}
            0xA000..=0xdfff=> {self.ram[(pointer - 0xA000) as usize] = data;}
            0xff01 => {self.serial = data;}
            0xff02 => {self.serial2 = data;}
            0xff07 => {self.tac = data}
            0xff0f => {self.IF = data}
            0xff10..=0xff3f => {println!("Audio handling skipped")}
            0xff80..=0xfffe=> {self.hram[(pointer as usize - 0xff80) as usize] = data}
            0xffff => {self.IE = data}
            _ => unimplemented!("Undefined write location {:#04x}", pointer)
        };
    }

    pub fn write_word(&mut self, pointer: u16, data: u16){
        let data_h = (data >> 8) as u8;
        let data_l = data as u8;
        self.write_byte(pointer, data_h);
        self.write_byte(pointer + 1, data_l);
    }

    pub fn read_byte(&self, loc: u16) -> u8 {
        match loc {
            0x0000..=0x3fff=> {self.rom[loc as usize]}
            0x4000..=0x7fff=> {self.rom1[(loc as usize - ROM_SIZE) as usize]} //TODO : Change depending on current rom bank
            0xA000..=0xdfff=> {self.ram[(loc - 0xA000) as usize]}
            0xff01 => {self.serial}
            0xff02 => {self.serial2}
            0xff07 => {self.tac}
            0xff0f => {self.IF}
            0xff10..=0xff3f => {println!("Audio handling skipped"); 0}
            0xfea0..=0xfeff=> {0xFF}
            0xff80..=0xfffe=> {self.hram[(loc as usize - 0xff80) as usize]}
            0xffff => {self.IE}
            _ => unimplemented!("Undefined read location {:#04x}", loc)
        }
    }

    pub fn read_word(&self, loc: u16) -> u16 {
        (self.read_byte(loc) as u16) | ((self.read_byte(loc + 1) as u16) << 8 )
    }


}