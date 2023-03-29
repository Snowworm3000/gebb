use crate::mbc;
use crate::ppu::PPU;
use crate::joypad::Joypad;
use crate::timer::Timer;
use std::str;

const ROM_SIZE: usize = 0x16000;
const RAM_SIZE: usize = 0x5000;

#[derive(PartialEq)]
enum DMAType {
    NoDMA,
    GDMA,
    HDMA,
}
pub struct MMU {
    rom: [u8; ROM_SIZE],
    ram: [u8; RAM_SIZE],
    // rom1: [u8; ROM_SIZE],
    // rom2: [u8; ROM_SIZE],
    // rom3: [u8; ROM_SIZE],
    // rom4: [u8; ROM_SIZE],
    io: [u8; 0x80],
    hram: [u8; 0x7f],
    hdma: [u8; 4],
    IE: u8,
    tac: u8,
    IF: u8,
    wram: [u8; ROM_SIZE],
    wram1: [u8; ROM_SIZE],
    pub ppu: PPU,
    pub joypad: Joypad,
    hdma_status: DMAType,
    hdma_src: u16,
    hdma_dst: u16,
    hdma_len: u8,
    pub timer: Timer,
    pub inte: u8,
    pub intf: u8,
    current_bank: u8,
}

impl MMU {
    pub fn new() -> Self {
        let mut mmu = MMU {
            rom: [0; ROM_SIZE],
            ram: [0; RAM_SIZE],
            // rom1: [0; ROM_SIZE],
            // rom2: [0; ROM_SIZE],
            // rom3: [0; ROM_SIZE],
            // rom4: [0; ROM_SIZE],
            io: [0; 0x80],
            hram: [0; 0x7f],
            hdma: [0; 4],
            IE: 0,
            tac: 0,
            IF: 0,
            wram: [0xff; ROM_SIZE],
            wram1: [0xff; ROM_SIZE],
            ppu: PPU::new(),
            joypad: Joypad::new(),
            hdma_src: 0,
            hdma_dst: 0,
            hdma_status: DMAType::NoDMA,
            hdma_len: 0xFF,
            timer: Timer::new(),
            inte: 0,
            intf: 0,
            current_bank: 1,
        };
        // mmu.reset();
        mmu.set_initial();
        mmu
    }

    fn set_initial(&mut self) {
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

    pub fn do_cycle(&mut self, ticks: u32) -> u32 {
        let cpudivider = 1;
        let vramticks = self.perform_vramdma();
        let gputicks = ticks / cpudivider + vramticks;
        let cputicks = ticks + vramticks * cpudivider;

        self.timer.do_cycle(cputicks);
        self.intf |= self.timer.interrupt;
        self.timer.interrupt = 0;

        // self.intf |= self.keypad.interrupt;
        // self.keypad.interrupt = 0;

        self.ppu.do_cycle(gputicks);
        self.intf |= self.ppu.interrupt;
        self.ppu.interrupt = 0;

        // self.sound.as_mut().map_or((), |s| s.do_cycle(gputicks));

        // self.intf |= self.serial.interrupt;
        // self.serial.interrupt = 0;

        return gputicks;
    }

    pub fn load(&mut self, data: &[u8]) {
        let mbc_type = data[0x147];
        let rom_size = data[0x148] as usize;

        
        // self.load_rom(&data[0..(ROM_SIZE -1)], 0);
        // self.load_rom(&data[(ROM_SIZE)..((ROM_SIZE * 2) -1)], 1);
        self.rom[0..data.len()].copy_from_slice(data);
        let ram_size = data[0x149];
        println!("MBC info, {} {} {}", mbc_type, rom_size, ram_size);
        // self.write(0x0, ROM_SIZE -1, data);
        
    }

    // fn load_rom(&mut self, data: &[u8], bank: u8) {
    //     match bank {
    //         0 => {self.rom[0x0000..(ROM_SIZE -1)].copy_from_slice(data);}
    //         1 => {self.rom1[0x0000..(ROM_SIZE -1)].copy_from_slice(data);}
    //         2 => {self.rom2[0x0000..(ROM_SIZE -1)].copy_from_slice(data);}
    //         3 => {self.rom3[0x0000..(ROM_SIZE -1)].copy_from_slice(data);}
    //         4 => {self.rom4[0x0000..(ROM_SIZE -1)].copy_from_slice(data);}
    //         _ => {unimplemented!("Not enough banks")}
    //     }
    // }

    // pub fn write(&mut self, start: usize, end: usize, data: &[u8]) {
    //     self.ram[start..end].copy_from_slice(data);
    // }

    
    fn oamdma(&mut self, value: u8) {
        let base = (value as u16) << 8;
        for i in 0 .. 0xA0 {
            let b = self.read_byte(base + i);
            self.write_byte(0xFE00 + i, b);
        }
    }

    pub fn perform_vramdma(&mut self) -> u32 {
        match self.hdma_status {
            DMAType::NoDMA => 0,
            DMAType::GDMA => self.perform_gdma(),
            DMAType::HDMA => self.perform_hdma(),
        }
    }

    fn perform_hdma(&mut self) -> u32 {

        self.perform_vramdma_row();
        if self.hdma_len == 0x7F { self.hdma_status = DMAType::NoDMA; }

        return 8;
    }

    fn perform_gdma(&mut self) -> u32 {
        let len = self.hdma_len as u32 + 1;
        for _i in 0 .. len {
            self.perform_vramdma_row();
        }

        self.hdma_status = DMAType::NoDMA;
        return len * 8;
    }
    fn perform_vramdma_row(&mut self) {
        let mmu_src = self.hdma_src;
        for j in 0 .. 0x10 {
            let b: u8 = self.read_byte(mmu_src + j);
            self.ppu.write_byte(self.hdma_dst + j, b);
        }
        self.hdma_src += 0x10;
        self.hdma_dst += 0x10;

        if self.hdma_len == 0 {
            self.hdma_len = 0x7F;
        }
        else {
            self.hdma_len -= 1;
        }
    }


    fn hdma_read(&self, a: u16) -> u8 {
        match a {
            0xFF51 ..= 0xFF54 => { self.hdma[(a - 0xFF51) as usize] },
            0xFF55 => self.hdma_len | if self.hdma_status == DMAType::NoDMA { 0x80 } else { 0 },
            _ => panic!("The address {:04X} should not be handled by hdma_read", a),
        }
    }

    fn hdma_write(&mut self, a: u16, v: u8) {
        match a {
            0xFF51 => self.hdma[0] = v,
            0xFF52 => self.hdma[1] = v & 0xF0,
            0xFF53 => self.hdma[2] = v & 0x1F,
            0xFF54 => self.hdma[3] = v & 0xF0,
            0xFF55 => {
                if self.hdma_status == DMAType::HDMA {
                    if v & 0x80 == 0 { self.hdma_status = DMAType::NoDMA; };
                    return;
                }
                let src = ((self.hdma[0] as u16) << 8) | (self.hdma[1] as u16);
                let dst = ((self.hdma[2] as u16) << 8) | (self.hdma[3] as u16) | 0x8000;
                if !(src <= 0x7FF0 || (src >= 0xA000 && src <= 0xDFF0)) { panic!("HDMA transfer with illegal start address {:04X}", src); }

                self.hdma_src = src;
                self.hdma_dst = dst;
                self.hdma_len = v & 0x7F;

                self.hdma_status =
                    if v & 0x80 == 0x80 { DMAType::HDMA }
                    else { DMAType::GDMA };
            },
            _ => panic!("The address {:04X} should not be handled by hdma_write", a),
        };
    }
    
    pub fn write_byte(&mut self, loc: u16, data: u8){
        
        if loc == 0xff01 {
            let v = vec![self.read_byte(0xff01)];
            print!("{} ", str::from_utf8(&v).unwrap());
        }
        if loc == 0xc7f6 {
            let v = vec![self.read_byte(0xff01)];
            print!("{} ", str::from_utf8(&v).unwrap());
        }
        match loc {
            // 0x0000..=0x7fff=> {unimplemented!("Attempt to write rom {:#04x}", data)}
            // 0x0000..=0x7fff=> {}
            // 0x0000..=0x1fff=> {unimplemented!("Enable/Disable ram")}
            0x0000..=0x1fff=> {}
            // 0x2000..=0x3fff=> {unimplemented!("Bank switch {} {}", loc, data)} // Higher nibble is discarded
            0x2000..=0x3fff=>{self.current_bank = (data & 0x0F);}
            0x4000..=0x5fff=> {unimplemented!("RAM bank or additional rom bank switch, {} {} ", loc, data)}
            0x8000..= 0x9FFF => self.ppu.write_byte(loc, data),
            0xA000..=0xbfff=> {self.ram[(loc - 0xA000) as usize] = data;}
            0xc000..=0xcfff=> {self.wram[(loc - 0xc000) as usize] = data;}
            0xd000..=0xdfff=> {self.wram1[(loc - 0xd000) as usize] = data;}
            0xfe00 ..= 0xfe9f => {self.ppu.write_byte(loc, data)},
            0xFF00 => {self.joypad.write(data)}
            0xFF04 ..= 0xFF07 => self.timer.wb(loc, data),
            0xFF0F => self.intf = data,
            0xff00..=0xff3f => {self.io[(loc - 0xff00) as usize] = data}
            0xff46 => self.oamdma(data),
            0xFF4D => {}//if data & 0x1 == 0x1 { self.speed_switch_req = true; }, // TODO Speed switch
            0xFF40 ..= 0xFF4F => {self.ppu.write_byte(loc, data)},
            // 0xff50..=0xff67 => {self.io[(loc - 0xff00) as usize] = data}
            0xFF51 ..= 0xFF55 => self.hdma_write(loc, data),
            0xff68 ..= 0xff6b => self.ppu.write_byte(loc, data),
            0xff6c..=0xff7f => {self.io[(loc - 0xff00) as usize] = data}
            0xff80..=0xfffe=> {self.hram[(loc as usize - 0xff80) as usize] = data}
            0xffff => {self.inte = data}
            _ => {}
            // _ => unimplemented!("Undefined write location {:#04x}", loc)
        };
    }

    pub fn write_word(&mut self, loc: u16, data: u16){
        let data_h = (data >> 8) as u8;
        let data_l = data as u8;
        self.write_byte(loc, data_l);
        self.write_byte(loc + 1, data_h);
    }

    pub fn read_byte(&self, loc: u16) -> u8 {
        // println!("Read {:#04x}", loc);
        // if loc == 0xc185 {
        //     let v = vec![self.read_byte(0xff01)];
        //     print!("{} ", str::from_utf8(&v).unwrap());
        // }
        // if loc == 0xc7f5 {
        //     let v = vec![self.read_byte(0xff01)];
        //     print!("{} ", str::from_utf8(&v).unwrap());
        // }
        match loc {
            0x0000..=0x3fff=> {self.rom[loc as usize]}
            // 0x4000..=0x7fff=> {self.rom1[(loc as usize - ROM_SIZE) as usize]} 
            0x4000..=0x7fff=> {let offset = loc - 0x4000; self.rom[(((self.current_bank as u16) * 0x4000) + offset) as usize]} // Offsets read location using rom bank number
            0x8000 ..= 0x9FFF => self.ppu.read_byte(loc),
            0xA000..=0xbfff=> {self.ram[(loc - 0xA000) as usize]}
            0xc000..=0xcfff=> {self.wram[(loc - 0xc000) as usize]}
            0xd000..=0xdfff=> {self.wram1[(loc - 0xd000) as usize]}
            0xfe00 ..= 0xfe9f => {self.ppu.read_byte(loc)},
            0xfea0..=0xfeff=> {0xFF}
            0xFF00 => {self.joypad.read()}
            0xFF04 ..= 0xFF07 => self.timer.rb(loc),
            0xFF0F => self.intf,
            // 0xff00=> {0xff}
            0xff00..=0xff3f => {self.io[(loc - 0xff00) as usize]}
            0xFF40 ..= 0xFF4F => self.ppu.read_byte(loc),
            // 0xff50..=0xff67 => {self.io[(loc - 0xff00) as usize]}
            0xFF51 ..= 0xFF55 => self.hdma_read(loc),
            0xff68 ..= 0xff6b => self.ppu.read_byte(loc),
            0xff6c..=0xff7f => {self.io[(loc - 0xff00) as usize]}
            0xff80..=0xfffe=> {self.hram[(loc as usize - 0xff80) as usize]}
            0xffff => {self.inte}

            
            
            // 0xFF51 ..= 0xFF55 => self.hdma_read(address),
            
            _ => unimplemented!("Undefined read location {:#04x}", loc)
        }
    }

    pub fn read_word(&self, loc: u16) -> u16 {
        (self.read_byte(loc) as u16) | ((self.read_byte(loc + 1) as u16) << 8 )
    }


}