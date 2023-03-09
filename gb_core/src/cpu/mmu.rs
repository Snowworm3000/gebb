use crate::mbc;
use crate::ppu::PPU;
use std::str;

const ROM_SIZE: usize = 0x4000;
const RAM_SIZE: usize = 0x5000;

pub struct MMU {
    rom: [u8; ROM_SIZE],
    ram: [u8; RAM_SIZE],
    rom1: [u8; ROM_SIZE],
    rom2: [u8; ROM_SIZE],
    rom3: [u8; ROM_SIZE],
    rom4: [u8; ROM_SIZE],
    io: [u8; 0x80],
    hram: [u8; 0x7f],
    IE: u8,
    tac: u8,
    IF: u8,
    wram: [u8; ROM_SIZE],
    wram1: [u8; ROM_SIZE],
    pub ppu: PPU,
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
            io: [0; 0x80],
            hram: [0; 0x7f],
            IE: 0,
            tac: 0,
            IF: 0,
            wram: [0xff; ROM_SIZE],
            wram1: [0xff; ROM_SIZE],
            ppu: PPU::new(),
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

        // TODO: Add more banks.
        // let rom_size_int = match rom_size { 
        //     0 => {let size = 0x8000; self.load_rom(&data[0..(ROM_SIZE -1)], 0); size}
        //     1 => {let size = 0x10000; self.load_rom(&data[0..(ROM_SIZE -1)], 0); self.load_rom(&data[0..(ROM_SIZE-1)], 1); size}
        //     _ => unimplemented!("Size not implemented")
        // };
        self.load_rom(&data[0..(ROM_SIZE -1)], 0);
        self.load_rom(&data[(ROM_SIZE)..((ROM_SIZE * 2) -1)], 1);
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
        // if pointer == 0xff01 {
        // if pointer == 0xc185 {
        //     let v = vec![self.read_byte(0xff01)];
        //     print!("{} ", str::from_utf8(&v).unwrap());
        // }
        if pointer == 0xff01 {
            let v = vec![self.read_byte(0xff01)];
            print!("{} ", str::from_utf8(&v).unwrap());
        }
        match pointer {
            // 0x0000..=0x7fff=> {unimplemented!("Attempt to write rom {:#04x}", data)}
            0x0000..=0x7fff=> {}
            0x8000 ..= 0x9FFF => self.ppu.write_byte(pointer, data),
            0xA000..=0xbfff=> {self.ram[(pointer - 0xA000) as usize] = data;}
            0xc000..=0xcfff=> {self.wram[(pointer - 0xc000) as usize] = data;}
            0xd000..=0xdfff=> {self.wram1[(pointer - 0xd000) as usize] = data;}
            0xff00..=0xff3f => {self.io[(pointer - 0xff00) as usize] = data}
            0xFF40 ..= 0xFF4F => {self.ppu.write_byte(pointer, data)},
            0xff50..=0xff67 => {self.io[(pointer - 0xff00) as usize] = data}
            0xff68 ..= 0xff6b => self.ppu.write_byte(pointer, data),
            0xff6c..=0xff7f => {self.io[(pointer - 0xff00) as usize] = data}
            0xff80..=0xfffe=> {self.hram[(pointer as usize - 0xff80) as usize] = data}
            0xffff => {self.IE = data}
            _ => {}
            // _ => unimplemented!("Undefined write location {:#04x}", pointer)
        };
    }

    pub fn write_word(&mut self, pointer: u16, data: u16){
        let data_h = (data >> 8) as u8;
        let data_l = data as u8;
        self.write_byte(pointer, data_l);
        self.write_byte(pointer + 1, data_h);
    }

    pub fn read_byte(&self, loc: u16) -> u8 {
        // println!("Read {:#04x}", loc);
        if loc == 0xc185 {
            let v = vec![self.read_byte(0xff01)];
            print!("{} ", str::from_utf8(&v).unwrap());
        }
        match loc {
            0x0000..=0x3fff=> {self.rom[loc as usize]}
            0x4000..=0x7fff=> {self.rom1[(loc as usize - ROM_SIZE) as usize]} //TODO : Change depending on current rom bank
            0x8000 ..= 0x9FFF => self.ppu.read_byte(loc),
            0xA000..=0xbfff=> {self.ram[(loc - 0xA000) as usize]}
            0xc000..=0xcfff=> {self.wram[(loc - 0xc000) as usize]}
            0xd000..=0xdfff=> {self.wram1[(loc - 0xd000) as usize]}
            0xfe00 ..= 0xfe9f => {self.ppu.read_byte(loc)},
            0xfea0..=0xfeff=> {0xFF}
            0xff00..=0xff3f => {self.io[(loc - 0xff00) as usize]}
            0xFF40 ..= 0xFF4F => self.ppu.read_byte(loc),
            0xff50..=0xff67 => {self.io[(loc - 0xff00) as usize]}
            0xff68 ..= 0xff6b => self.ppu.read_byte(loc),
            0xff6c..=0xff7f => {self.io[(loc - 0xff00) as usize]}
            0xff80..=0xfffe=> {self.hram[(loc as usize - 0xff80) as usize]}
            0xffff => {self.IE}

            
            
            // 0xFF51 ..= 0xFF55 => self.hdma_read(address),
            
            _ => unimplemented!("Undefined read location {:#04x}", loc)
        }
    }

    pub fn read_word(&self, loc: u16) -> u16 {
        (self.read_byte(loc) as u16) | ((self.read_byte(loc + 1) as u16) << 8 )
    }


}