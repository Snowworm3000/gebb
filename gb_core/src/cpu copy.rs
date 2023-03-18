mod registers;
use core::panic;
use std::fs::File;
use std::io::{Read, BufReader, BufRead, Lines};
use std::ops::Shl;
use std::os::unix::prelude::FileExt;
use std::{str, result};

use registers::*;
mod mmu;
use mmu::*;

use crate::debug_reader;
const RAM_SIZE: usize = 0x100; // I'm not entirely sure how large this should be yet.
const STACK_SIZE: usize = 0xFF; // I'm not sure how large this should be either, just increase the size if anything bad happens.
const START_ADDR: usize = 0x0;

const LOG_LEVEL: usize = 2;

pub struct Cpu {
    reg: Registers,
    // ram: [u8; RAM_SIZE],
    pc: u16,
    sp: u16,
    ime: bool,
    tempIme: bool,
    stack: [u16; STACK_SIZE],
    mmu: MMU,
    depth: u8,
    halt: bool,
    cycle: usize,
    line: usize,
    debug_file: Vec<String>,
    halted: bool,
    setdi: u32,
    setei: u32,
}

impl Cpu {
    pub fn new() -> Self {
        let mut temp = Self {
            reg: Registers::new_default(),
            // ram: [0; RAM_SIZE],
            pc: 0x100,
            sp: 0xfffe,
            ime: false,
            tempIme: false,
            stack: [0; STACK_SIZE],
            mmu: MMU::new(),
            depth: 0,
            halt: false,
            cycle: 0,
            line: 0,
            debug_file: Vec::new(),
            halted: false,
            setdi: 0,
            setei: 0,
        };
        // BufReader::new(File::open("tetris_output.txt").expect("Unable to open file")).read_until(b'\n',&mut temp.debug_file).unwrap();


        // let mut reader = debug_reader::BufReader::open("/home/ethan/Downloads/binjgb/out/Debug/out.txt").expect("test");
        let mut reader = debug_reader::BufReader::open("/home/ethan/code/rust/rboy/out.txt").expect("test");
        let mut buffer = String::new();

        while let Some(line) = reader.read_line(&mut buffer) {
            // println!("{}", line.expect("test").trim());
            temp.debug_file.push(line.expect("test").trim().to_string());
        }

        temp
    }
    pub fn reset(&mut self) {
        self.reg = Registers::new_default();
        // self.ram = [0; RAM_SIZE];
        self.pc = 0x100;
        self.sp = 0xfffe;
        self.ime = false;
        self.stack = [0; STACK_SIZE];
        // self.mmu.reset();
        self.cycle = 0;
    }

    pub fn load(&mut self, data: &[u8]) {
        // let start = START_ADDR as usize;
        // let end = (START_ADDR as usize) + data.len();
        // self.mmu.write(start, end, data);

        self.mmu.load(data);
    }

    // fn fetch(&mut self) -> u8 {
    //     let op = self.ram[self.pc as usize] as u8;
    //     self.pc += 1;
    //     op
    // }
    
    pub fn get_display(&self) -> &Vec<u8> {
        &self.mmu.ppu.data
    }

    pub fn do_cycle(&mut self) -> u32 {
        let ticks = self.docycle() * 4;
        return self.mmu.do_cycle(ticks);
    }

    fn docycle(&mut self) -> u32 {
        self.updateime();
        match self.handleinterrupt() {
            0 => {},
            n => return n,
        };

        if self.halted {
            // Emulate an noop instruction
            1
        } else {
            let op = self.fetch_byte();
            self.execute(op)
        }
    } 

    fn updateime(&mut self) {
        self.setdi = match self.setdi {
            2 => 1,
            1 => { self.ime = false; 0 },
            _ => 0,
        };
        self.setei = match self.setei {
            2 => 1,
            1 => { self.ime = true; 0 },
            _ => 0,
        };
    }

    fn handleinterrupt(&mut self) -> u32 {
        if self.ime == false && self.halted == false { return 0 }

        let triggered = self.mmu.inte & self.mmu.intf;
        if triggered == 0 { return 0 }

        self.halted = false;
        if self.ime == false { return 0 }
        self.ime = false;

        let n = triggered.trailing_zeros();
        if n >= 5 { panic!("Invalid interrupt triggered"); }
        self.mmu.intf &= !(1 << n);
        let pc = self.pc;
        self.push(pc);
        self.pc = 0x0040 | ((n as u16) << 3);

        return 4
    }

    // pub fn tick(&mut self) {
    //     let op = self.fetch_byte();

    //     let ticks = self.execute(op) * 4;

    //     let vramticks = self.mmu.perform_vramdma();
    //     let cputicks = ticks + vramticks;

    //     self.mmu.timer.do_cycle(cputicks);
    //     // self.mmu.intf |= self.timer.interrupt;
    //     self.mmu.timer.interrupt = 0;

    //     self.mmu.ppu.do_cycle(cputicks);
    //     self.mmu.ppu.interrupt = 0;
    //     if self.ime {
    //         if self.tempIme { // TODO: Interrupt here. Timing is 5 machine cycles I think.
    //             // unimplemented!("Interrupt here.")
    //             self.pc -= 1; // because we are not using op
    //             let interrupt_enable = self.rightmost_set_bit(self.mmu.read_byte(0xffff));
    //             let interrupt_flag = self.rightmost_set_bit(self.mmu.read_byte(0xff0f));
    //             if (interrupt_enable == interrupt_flag) & self.ime {
    //                 self.ime = false;
    //                 let original_enable = self.mmu.read_byte(0xffff);
    //                 let original_flag = self.mmu.read_byte(0xffff);
    //                 self.mmu.write_byte(0xffff, self.res(interrupt_enable, original_enable));
    //                 self.mmu.write_byte(0xff0f, self.res(interrupt_flag, original_flag));
    //                 match interrupt_enable {
    //                     0 => { // VBlank interrupt
    //                         self.call(0x40);
    //                     }
    //                     1 => {
    //                         self.call(0x48);
    //                     }
    //                     2 => {
    //                         self.call(0x50);
    //                     }
    //                     3 => {
    //                         self.call(0x58);
    //                     }
    //                     4 => {
    //                         self.call(0x60);
    //                     }
    //                     _ => {unimplemented!("Unimplemented interrupt")}
    //                 }
    //             }

    //         }
    //         self.tempIme = true;
    //     } else {
    //         self.tempIme = false;
    //     }
        
    // }

    fn rightmost_set_bit(&self, val: u8) -> u8 {
        ((val & !(val-1)) as f32).log2() as u8
    }


    fn fetch_byte(&mut self) -> u8 {
        let byte = self.mmu.read_byte(self.pc);
        self.pc += 1;
        byte
    }

    fn fetch_word(&mut self) -> u16 {
        let word = self.mmu.read_word(self.pc);
        // println!("Word {:#04x} at {:#04x}", word, self.pc);
        // println!("Word {:#04x} {:#04x} at {:#04x}", self.fetch_byte(), self.fetch_byte(), self.pc);
        self.pc += 2;
        word
    }

    pub fn ppu_updated(&mut self) -> bool {
        let result = self.mmu.ppu.updated;
        self.mmu.ppu.updated = false;
        result
    }

    fn debug_equal_u8(&self, first: &str, second: u8) -> bool {
        u8::from_str_radix(first, 16).expect("msg") == second
    }

    fn debug_equal(&self, first: &str, second: u16) -> bool {
        u16::from_str_radix(first, 16).expect("msg") == second
    }

    fn to_num(&self, v: bool) -> u8 {
        if v {1} else {0}
    }

    fn execute(&mut self, op: u8) -> u32{
        
        // if (self.mmu.read_byte(0xff02) == 0x81) {
        //     let c = self.mmu.read_byte(0xff01);
        //     println!("{}", c);
        //     if let Ok(s) = str::from_utf8(&[c]) {
        //         println!("{}", s);
        //     }
        //     self.mmu.write_byte(0xff02, 0x0);
        // }
        // println!("Flags: {:#04x} Opcode: {:#04x} PC: {:#04x} Registers: {:#04x} {:#04x} {:#04x} {:#04x} {:#04x} {:#04x} {:#04x} {:#04x}", self.reg.f, op, self.pc, self.reg.a, self.reg.b, self.reg.c, self.reg.d, self.reg.e, self.reg.f, self.reg.h, self.reg.l);
        // println!("Flags: {:#04x} Opcode: {:#04x} PC: {:#04x} SP: {:#04x} Registers: af {:#04x} bc {:#04x} de {:#04x} hl {:#04x}", self.reg.f, op, self.pc, self.sp, self.reg.get_af(), self.reg.get_bc(), self.reg.get_de(), self.reg.get_hl());
        let flz = if self.reg.get_flag(flags::Z) {"Z"} else {"-"};
        let fln = if self.reg.get_flag(flags::N) {"N"} else {"-"};
        let flh = if self.reg.get_flag(flags::H) {"H"} else {"-"};
        let flc = if self.reg.get_flag(flags::C) {"C"} else {"-"};
        // println!("{}", self.mmu.read_word(self.sp));

        if LOG_LEVEL >= 3 {
            let this = format!("{} A:{:#04x} F:{flz}{fln}{flh}{flc} BC:{:#04x} DE:{:#04x} HL:{:#04x} SP:{:#04x} PC:{:#04x} Opcode:{:#04x} Flags:{:#04x} Next:{:#04x}", self.cycle + 1, self.reg.a , self.reg.get_bc(), self.reg.get_de(), self.reg.get_hl(), self.sp, self.pc - 1, op, self.reg.f, self.mmu.read_word(self.pc));
            println!("{}", this);
            println!("{}", self.debug_file[self.line]);
            if this != self.debug_file[self.line]{
                unimplemented!("Not matching original");
            }
            self.line += 1;

            let this = format!("{}", self.mmu.ppu.modeclock);
            println!("{}", this);
            println!("{}", self.debug_file[(self.line)]);
            if this != self.debug_file[self.line ]{
                unimplemented!("Not matching original");
            }
            self.line += 1;

            // let this = format!("{}", self.mmu.ppu.lcds.ly);
            let this = format!("{}", self.mmu.ppu.line);
            println!("{}", this);

            println!("{}", self.debug_file[(self.line)]);
            if this != self.debug_file[self.line ]{
                unimplemented!("Not matching original");
            }

            if self.cycle == 0{
                print!("Here")
            }

            self.cycle += 1;
            self.line += 1;


        }
        // if LOG_LEVEL >= 3 && false{
        //     println!("{} A:{:#04x} F:{flz}{fln}{flh}{flc} BC:{:#04x} DE:{:#04x} HL:{:#04x} SP:{:#04x} PC:{:#04x} Opcode:{:#04x} Flags:{:#04x} ", self.cycle, self.reg.a , self.reg.get_bc(), self.reg.get_de(), self.reg.get_hl(), self.sp, self.pc - 1, op, self.reg.f);
        //     // println!("{} A: {:#04x} BC: {:#04x} DE: {:#04x} HL: {:#04x} SP: {:#04x} PC: {:#04x} Opcode: {:#04x} Flags: {:#04x} ", self.cycle, self.reg.a , self.reg.get_bc(), self.reg.get_de(), self.reg.get_hl(), self.sp, self.pc - 1, op, self.reg.f);
        //     println!("{}", self.debug_file[self.cycle]);
        //     let line_vec: Vec<char> = self.debug_file[self.cycle].chars().collect();
        //     let line = &self.debug_file[self.cycle];
        //     let pos = [line.find("A:").expect("msg"), line.find("F:").expect("msg"), line.find("BC:").expect("msg"), line.find("DE:").expect("msg"), line.find("HL:").expect("msg"), line.find("SP:").expect("msg"), line.find("PC:").expect("msg")];
        //     // println!("{} {} {}", line_vec[pos[0] + 2], line_vec[pos[0] + 3], pos[0].to_string());
            
        //     let mut A = String::new();
        //     A.push(line_vec[pos[0] + 2]);
        //     A.push(line_vec[pos[0] + 3]);


        //     // println!("{} {} ", line_vec[pos[1] + 2], line_vec[pos[1] + 3]);
        //     let mut F: u8 = 0;
        //     F |= self.to_num(line_vec[pos[1] + 2] != '-') << 7;
        //     F |= self.to_num(line_vec[pos[1] + 3] != '-')  << 6;
        //     F |= self.to_num(line_vec[pos[1] + 4] != '-')  << 5;
        //     F |= self.to_num(line_vec[pos[1] + 5] != '-')  << 4;

        //     // println!("{:#08b} {:#08b}", F, self.reg.f);

        //     let mut B = String::new();
        //     B.push(line_vec[pos[2] + 3]);
        //     B.push(line_vec[pos[2] + 4]);
        //     B.push(line_vec[pos[2] + 5]);
        //     B.push(line_vec[pos[2] + 6]);

        //     let mut D = String::new();
        //     D.push(line_vec[pos[3] + 3]);
        //     D.push(line_vec[pos[3] + 4]);
        //     D.push(line_vec[pos[3] + 5]);
        //     D.push(line_vec[pos[3] + 6]);

        //     let mut H = String::new();
        //     H.push(line_vec[pos[4] + 3]);
        //     H.push(line_vec[pos[4] + 4]);
        //     H.push(line_vec[pos[4] + 5]);
        //     H.push(line_vec[pos[4] + 6]);

        //     let mut SP = String::new();
        //     SP.push(line_vec[pos[5] + 3]);
        //     SP.push(line_vec[pos[5] + 4]);
        //     SP.push(line_vec[pos[5] + 5]);
        //     SP.push(line_vec[pos[5] + 6]);

        //     let mut PC = String::new();
        //     PC.push(line_vec[pos[6] + 3]);
        //     PC.push(line_vec[pos[6] + 4]);
        //     PC.push(line_vec[pos[6] + 5]);
        //     PC.push(line_vec[pos[6] + 6]);

        //     // println!("{:#08b} {:#08b} {:#08b} {:#08b} {:#08b} {:#08b} {:#08b}", A, F, B, D, H, SP, PC);
        //     println!("{}", line_vec[2]);

        //     println!("{} {} {} {} {} {} {}", self.debug_equal_u8(&A, self.reg.a), F == self.reg.f, self.debug_equal(&B, self.reg.get_bc()), self.debug_equal(&D, self.reg.get_de()), self.debug_equal(&H, self.reg.get_hl()), self.debug_equal(&SP, self.sp), self.debug_equal(&PC, self.pc -1));

        //     // println!("{} {} {}", PC, self.pc -1 , u16::from_str_radix(&PC, 16).expect("msg"));
            
        //     let conditions = (self.debug_equal_u8(&A, self.reg.a) && F == self.reg.f && self.debug_equal(&B, self.reg.get_bc()) && self.debug_equal(&D, self.reg.get_de()) && self.debug_equal(&H, self.reg.get_hl()) && self.debug_equal(&SP, self.sp) && self.debug_equal(&PC, self.pc -1));
        //     // let conditions = (self.debug_equal(&PC, self.pc -1));
        //     // println!("{} {}", self.debug_equal_u8(&A, self.reg.a), self.reg.a);
        //     if !conditions {
        //         unimplemented!("Not matching original");
        //     }
        // }
        let timing = match op {
            // Notation for LD functions:
            // LD(to_set, set_with)
            // 0x00 => {if self.depth > 100 {unimplemented!("Stop")} else {self.depth += 1;1}}
            0x00 => { 1 },
            0x01 => { let v = self.fetch_word(); self.reg.set_bc(v); 3 },
            0x02 => { self.mmu.write_byte(self.reg.get_bc(), self.reg.a); 2 },
            0x03 => { self.reg.set_bc(self.reg.get_bc().wrapping_add(1)); 2 },
            0x04 => { self.reg.b = self.inc(self.reg.b); 1 },
            0x05 => { self.reg.b = self.dec(self.reg.b); 1 },
            0x06 => { self.reg.b = self.fetch_byte(); 2 },
            0x07 => { self.reg.a = self.rlc(self.reg.a); self.reg.set_flag(flags::Z, false); 1 },
            0x08 => { let a = self.fetch_word(); self.mmu.write_word(a, self.sp); 5 },
            0x09 => { self.add16(self.reg.get_bc()); 2 },
            0x0A => { self.reg.a = self.mmu.read_byte(self.reg.get_bc()); 2 },
            0x0B => { self.reg.set_bc(self.reg.get_bc().wrapping_sub(1)); 2 },
            0x0C => { self.reg.c = self.inc(self.reg.c); 1 },
            0x0D => { self.reg.c = self.dec(self.reg.c); 1 },
            0x0E => { self.reg.c = self.fetch_byte(); 2 },
            0x0F => { self.reg.a = self.rrc(self.reg.a); self.reg.set_flag(flags::Z, false); 1 },
            // 0x10 => { self.mmu.switch_speed(); 1 }, // STOP
            0x10 => {  1 }, // STOP
            0x11 => { let v = self.fetch_word(); self.reg.set_de(v); 3 },
            0x12 => { self.mmu.write_byte(self.reg.get_de(), self.reg.a); 2 },
            0x13 => { self.reg.set_de(self.reg.get_de().wrapping_add(1)); 2 },
            0x14 => { self.reg.d = self.inc(self.reg.d); 1 },
            0x15 => { self.reg.d = self.dec(self.reg.d); 1 },
            0x16 => { self.reg.d = self.fetch_byte(); 2 },
            0x17 => { self.reg.a = self.rl(self.reg.a); self.reg.set_flag(flags::Z, false); 1 },
            0x18 => { self.cpu_jr(); 3 },
            0x19 => { self.add16(self.reg.get_de()); 2 },
            0x1A => { self.reg.a = self.mmu.read_byte(self.reg.get_de()); 2 },
            0x1B => { self.reg.set_de(self.reg.get_de().wrapping_sub(1)); 2 },
            0x1C => { self.reg.e = self.inc(self.reg.e); 1 },
            0x1D => { self.reg.e = self.dec(self.reg.e); 1 },
            0x1E => { self.reg.e = self.fetch_byte(); 2 },
            0x1F => { self.reg.a = self.rr(self.reg.a); self.reg.set_flag(flags::Z, false); 1 },
            0x20 => { if !self.reg.get_flag(flags::Z) { self.cpu_jr(); 3 } else { self.pc += 1; 2 } },
            0x21 => { let v = self.fetch_word(); self.reg.set_hl(v); 3 },
            0x22 => { self.mmu.write_byte(self.reg.hli(), self.reg.a); 2 },
            0x23 => { let v = self.reg.get_hl().wrapping_add(1); self.reg.set_hl(v); 2 },
            0x24 => { self.reg.h = self.inc(self.reg.h); 1 },
            0x25 => { self.reg.h = self.dec(self.reg.h); 1 },
            0x26 => { self.reg.h = self.fetch_byte(); 2 },
            0x27 => { self.reg.a = self.daa(self.reg.a); 1 },
            0x28 => { if self.reg.get_flag(flags::Z) { self.cpu_jr(); 3 } else { self.pc += 1; 2  } },
            0x29 => { let v = self.reg.get_hl(); self.add16(v); 2 },
            0x2A => { self.reg.a = self.mmu.read_byte(self.reg.hli()); 2 },
            0x2B => { let v = self.reg.get_hl().wrapping_sub(1); self.reg.set_hl(v); 2 },
            0x2C => { self.reg.l = self.inc(self.reg.l); 1 },
            0x2D => { self.reg.l = self.dec(self.reg.l); 1 },
            0x2E => { self.reg.l = self.fetch_byte(); 2 },
            0x2F => { self.reg.a = !self.reg.a; self.reg.set_flag(flags::H, true); self.reg.set_flag(flags::N, true); 1 },
            0x30 => { if !self.reg.get_flag(flags::C) { self.cpu_jr(); 3 } else { self.pc += 1; 2 } },
            0x31 => { self.sp = self.fetch_word(); 3 },
            0x32 => { self.mmu.write_byte(self.reg.hld(), self.reg.a); 2 },
            0x33 => { self.sp = self.sp.wrapping_add(1); 2 },
            0x34 => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a); let v2 = self.inc(v); self.mmu.write_byte(a, v2); 3 },
            0x35 => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a); let v2 = self.dec(v); self.mmu.write_byte(a, v2); 3 },
            0x36 => { let v = self.fetch_byte(); self.mmu.write_byte(self.reg.get_hl(), v); 3 },
            0x37 => { self.reg.set_flag(flags::C, true); self.reg.set_flag(flags::H, false); self.reg.set_flag(flags::N, false); 1 },
            0x38 => { if self.reg.get_flag(flags::C) { self.cpu_jr(); 3 } else { self.pc += 1; 2  } },
            0x39 => { self.add16(self.sp); 2 },
            0x3A => { self.reg.a = self.mmu.read_byte(self.reg.hld()); 2 },
            0x3B => { self.sp = self.sp.wrapping_sub(1); 2 },
            0x3C => { self.reg.a = self.inc(self.reg.a); 1 },
            0x3D => { self.reg.a = self.dec(self.reg.a); 1 },
            0x3E => { self.reg.a = self.fetch_byte(); 2 },
            0x3F => { let v = !self.reg.get_flag(flags::C); self.reg.set_flag(flags::C, v); self.reg.set_flag(flags::H, false); self.reg.set_flag(flags::N, false); 1 },
            0x40 => { 1 },
            0x41 => { self.reg.b = self.reg.c; 1 },
            0x42 => { self.reg.b = self.reg.d; 1 },
            0x43 => { self.reg.b = self.reg.e; 1 },
            0x44 => { self.reg.b = self.reg.h; 1 },
            0x45 => { self.reg.b = self.reg.l; 1 },
            0x46 => { self.reg.b = self.mmu.read_byte(self.reg.get_hl()); 2 },
            0x47 => { self.reg.b = self.reg.a; 1 },
            0x48 => { self.reg.c = self.reg.b; 1 },
            0x49 => { 1 },
            0x4A => { self.reg.c = self.reg.d; 1 },
            0x4B => { self.reg.c = self.reg.e; 1 },
            0x4C => { self.reg.c = self.reg.h; 1 },
            0x4D => { self.reg.c = self.reg.l; 1 },
            0x4E => { self.reg.c = self.mmu.read_byte(self.reg.get_hl()); 2 },
            0x4F => { self.reg.c = self.reg.a; 1 },
            0x50 => { self.reg.d = self.reg.b; 1 },
            0x51 => { self.reg.d = self.reg.c; 1 },
            0x52 => { 1 },
            0x53 => { self.reg.d = self.reg.e; 1 },
            0x54 => { self.reg.d = self.reg.h; 1 },
            0x55 => { self.reg.d = self.reg.l; 1 },
            0x56 => { self.reg.d = self.mmu.read_byte(self.reg.get_hl()); 2 },
            0x57 => { self.reg.d = self.reg.a; 1 },
            0x58 => { self.reg.e = self.reg.b; 1 },
            0x59 => { self.reg.e = self.reg.c; 1 },
            0x5A => { self.reg.e = self.reg.d; 1 },
            0x5B => { 1 },
            0x5C => { self.reg.e = self.reg.h; 1 },
            0x5D => { self.reg.e = self.reg.l; 1 },
            0x5E => { self.reg.e = self.mmu.read_byte(self.reg.get_hl()); 2 },
            0x5F => { self.reg.e = self.reg.a; 1 },
            0x60 => { self.reg.h = self.reg.b; 1 },
            0x61 => { self.reg.h = self.reg.c; 1 },
            0x62 => { self.reg.h = self.reg.d; 1 },
            0x63 => { self.reg.h = self.reg.e; 1 },
            0x64 => { 1 },
            0x65 => { self.reg.h = self.reg.l; 1 },
            0x66 => { self.reg.h = self.mmu.read_byte(self.reg.get_hl()); 2 },
            0x67 => { self.reg.h = self.reg.a; 1 },
            0x68 => { self.reg.l = self.reg.b; 1 },
            0x69 => { self.reg.l = self.reg.c; 1 },
            0x6A => { self.reg.l = self.reg.d; 1 },
            0x6B => { self.reg.l = self.reg.e; 1 },
            0x6C => { self.reg.l = self.reg.h; 1 },
            0x6D => { 1 },
            0x6E => { self.reg.l = self.mmu.read_byte(self.reg.get_hl()); 2 },
            0x6F => { self.reg.l = self.reg.a; 1 },
            0x70 => { self.mmu.write_byte(self.reg.get_hl(), self.reg.b); 2 },
            0x71 => { self.mmu.write_byte(self.reg.get_hl(), self.reg.c); 2 },
            0x72 => { self.mmu.write_byte(self.reg.get_hl(), self.reg.d); 2 },
            0x73 => { self.mmu.write_byte(self.reg.get_hl(), self.reg.e); 2 },
            0x74 => { self.mmu.write_byte(self.reg.get_hl(), self.reg.h); 2 },
            0x75 => { self.mmu.write_byte(self.reg.get_hl(), self.reg.l); 2 },
            0x76 => { self.halted = true; 1 },
            0x77 => { self.mmu.write_byte(self.reg.get_hl(), self.reg.a); 2 },
            0x78 => { self.reg.a = self.reg.b; 1 },
            0x79 => { self.reg.a = self.reg.c; 1 },
            0x7A => { self.reg.a = self.reg.d; 1 },
            0x7B => { self.reg.a = self.reg.e; 1 },
            0x7C => { self.reg.a = self.reg.h; 1 },
            0x7D => { self.reg.a = self.reg.l; 1 },
            0x7E => { self.reg.a = self.mmu.read_byte(self.reg.get_hl()); 2 },
            0x7F => { 1 },
            0x80 => { self.add(self.reg.b, false); 1 },
            0x81 => { self.add(self.reg.c, false); 1 },
            0x82 => { self.add(self.reg.d, false); 1 },
            0x83 => { self.add(self.reg.e, false); 1 },
            0x84 => { self.add(self.reg.h, false); 1 },
            0x85 => { self.add(self.reg.l, false); 1 },
            0x86 => { let v = self.mmu.read_byte(self.reg.get_hl()); self.add(v, false); 2 },
            0x87 => { self.add(self.reg.a, false); 1 },
            0x88 => { self.add(self.reg.b, true); 1 },
            0x89 => { self.add(self.reg.c, true); 1 },
            0x8A => { self.add(self.reg.d, true); 1 },
            0x8B => { self.add(self.reg.e, true); 1 },
            0x8C => { self.add(self.reg.h, true); 1 },
            0x8D => { self.add(self.reg.l, true); 1 },
            0x8E => { let v = self.mmu.read_byte(self.reg.get_hl()); self.add(v, true); 2 },
            0x8F => { self.add(self.reg.a, true); 1 },
            0x90 => { self.sub(self.reg.b, false); 1 },
            0x91 => { self.sub(self.reg.c, false); 1 },
            0x92 => { self.sub(self.reg.d, false); 1 },
            0x93 => { self.sub(self.reg.e, false); 1 },
            0x94 => { self.sub(self.reg.h, false); 1 },
            0x95 => { self.sub(self.reg.l, false); 1 },
            0x96 => { let v = self.mmu.read_byte(self.reg.get_hl()); self.sub(v, false); 2 },
            0x97 => { self.sub(self.reg.a, false); 1 },
            0x98 => { self.sub(self.reg.b, true); 1 },
            0x99 => { self.sub(self.reg.c, true); 1 },
            0x9A => { self.sub(self.reg.d, true); 1 },
            0x9B => { self.sub(self.reg.e, true); 1 },
            0x9C => { self.sub(self.reg.h, true); 1 },
            0x9D => { self.sub(self.reg.l, true); 1 },
            0x9E => { let v = self.mmu.read_byte(self.reg.get_hl()); self.sub(v, true); 2 },
            0x9F => { self.sub(self.reg.a, true); 1 },
            0xA0 => { self.and(self.reg.b); 1 },
            0xA1 => { self.and(self.reg.c); 1 },
            0xA2 => { self.and(self.reg.d); 1 },
            0xA3 => { self.and(self.reg.e); 1 },
            0xA4 => { self.and(self.reg.h); 1 },
            0xA5 => { self.and(self.reg.l); 1 },
            0xA6 => { let v = self.mmu.read_byte(self.reg.get_hl()); self.and(v); 2 },
            0xA7 => { self.and(self.reg.a); 1 },
            0xA8 => { self.xor(self.reg.b); 1 },
            0xA9 => { self.xor(self.reg.c); 1 },
            0xAA => { self.xor(self.reg.d); 1 },
            0xAB => { self.xor(self.reg.e); 1 },
            0xAC => { self.xor(self.reg.h); 1 },
            0xAD => { self.xor(self.reg.l); 1 },
            0xAE => { let v = self.mmu.read_byte(self.reg.get_hl()); self.xor(v); 2 },
            0xAF => { self.xor(self.reg.a); 1 },
            0xB0 => { self.or(self.reg.b); 1 },
            0xB1 => { self.or(self.reg.c); 1 },
            0xB2 => { self.or(self.reg.d); 1 },
            0xB3 => { self.or(self.reg.e); 1 },
            0xB4 => { self.or(self.reg.h); 1 },
            0xB5 => { self.or(self.reg.l); 1 },
            0xB6 => { let v = self.mmu.read_byte(self.reg.get_hl()); self.or(v); 2 },
            0xB7 => { self.or(self.reg.a); 1 },
            0xB8 => { self.cp(self.reg.b); 1 },
            0xB9 => { self.cp(self.reg.c); 1 },
            0xBA => { self.cp(self.reg.d); 1 },
            0xBB => { self.cp(self.reg.e); 1 },
            0xBC => { self.cp(self.reg.h); 1 },
            0xBD => { self.cp(self.reg.l); 1 },
            0xBE => { let v = self.mmu.read_byte(self.reg.get_hl()); self.cp(v); 2 },
            0xBF => { self.cp(self.reg.a); 1 },
            0xC0 => { if !self.reg.get_flag(flags::Z) { self.pc = self.pop(); 5 } else { 2 } },
            0xC1 => { let v = self.pop(); self.reg.set_bc(v); 3 },
            0xC2 => { if !self.reg.get_flag(flags::Z) { self.pc = self.fetch_word(); 4 } else { self.pc += 2; 3 } },
            0xC3 => { self.pc = self.fetch_word(); 4 },
            0xC4 => { if !self.reg.get_flag(flags::Z) { self.push(self.pc + 2); self.pc = self.fetch_word(); 6 } else { self.pc += 2; 3 } },
            0xC5 => { self.push(self.reg.get_bc()); 4 },
            0xC6 => { let v = self.fetch_byte(); self.add(v, false); 2 },
            0xC7 => { self.push(self.pc); self.pc = 0x00; 4 },
            0xC8 => { if self.reg.get_flag(flags::Z) { self.pc = self.pop(); 5 } else { 2 } },
            0xC9 => { self.pc = self.pop(); 4 },
            0xCA => { if self.reg.get_flag(flags::Z) { self.pc = self.fetch_word(); 4 } else { self.pc += 2; 3 } },
            
            0xCC => { if self.reg.get_flag(flags::Z) { self.push(self.pc + 2); self.pc = self.fetch_word(); 6 } else { self.pc += 2; 3 } },
            0xCD => { self.push(self.pc + 2); self.pc = self.fetch_word(); 6 },
            0xCE => { let v = self.fetch_byte(); self.add(v, true); 2 },
            0xCF => { self.push(self.pc); self.pc = 0x08; 4 },
            0xD0 => { if !self.reg.get_flag(flags::C) { self.pc = self.pop(); 5 } else { 2 } },
            0xD1 => { let v = self.pop(); self.reg.set_de(v); 3 },
            0xD2 => { if !self.reg.get_flag(flags::C) { self.pc = self.fetch_word(); 4 } else { self.pc += 2; 3 } },
            0xD4 => { if !self.reg.get_flag(flags::C) { self.push(self.pc + 2); self.pc = self.fetch_word(); 6 } else { self.pc += 2; 3 } },
            0xD5 => { self.push(self.reg.get_de()); 4 },
            0xD6 => { let v = self.fetch_byte(); self.sub(v, false); 2 },
            0xD7 => { self.push(self.pc); self.pc = 0x10; 4 },
            0xD8 => { if self.reg.get_flag(flags::C) { self.pc = self.pop(); 5 } else { 2 } },
            0xD9 => { self.pc = self.pop(); self.setei = 1; 4 },
            0xDA => { if self.reg.get_flag(flags::C) { self.pc = self.fetch_word(); 4 } else { self.pc += 2; 3 } },
            0xDC => { if self.reg.get_flag(flags::C) { self.push(self.pc + 2); self.pc = self.fetch_word(); 6 } else { self.pc += 2; 3 } },
            0xDE => { let v = self.fetch_byte(); self.sub(v, true); 2 },
            0xDF => { self.push(self.pc); self.pc = 0x18; 4 },
            0xE0 => { let a = 0xFF00 | self.fetch_byte() as u16; self.mmu.write_byte(a, self.reg.a); 3 },
            0xE1 => { let v = self.pop(); self.reg.set_hl(v); 3 },
            0xE2 => { self.mmu.write_byte(0xFF00 | self.reg.c as u16, self.reg.a); 2 },
            0xE5 => { self.push(self.reg.get_hl()); 4 },
            0xE6 => { let v = self.fetch_byte(); self.and(v); 2 },
            0xE7 => { self.push(self.pc); self.pc = 0x20; 4 },
            0xE8 => { self.sp = self.add16imm(self.sp); 4 },
            0xE9 => { self.pc = self.reg.get_hl(); 1 },
            0xEA => { let a = self.fetch_word(); self.mmu.write_byte(a, self.reg.a); 4 },
            0xEE => { let v = self.fetch_byte(); self.xor(v); 2 },
            0xEF => { self.push(self.pc); self.pc = 0x28; 4 },
            0xF0 => { let a = 0xFF00 | self.fetch_byte() as u16; self.reg.a = self.mmu.read_byte(a); 3 },
            0xF1 => { let v = self.pop() & 0xFFF0; self.reg.set_af(v); 3 },
            0xF2 => { self.reg.a = self.mmu.read_byte(0xFF00 | self.reg.c as u16); 2 },
            0xF3 => { self.setdi = 2; 1 },
            0xF5 => { self.push(self.reg.get_af()); 4 },
            0xF6 => { let v = self.fetch_byte(); self.or(v); 2 },
            0xF7 => { self.push(self.pc); self.pc = 0x30; 4 },
            0xF8 => { let r = self.add16imm(self.sp); self.reg.set_hl(r); 3 },
            0xF9 => { self.sp = self.reg.get_hl(); 2 },
            0xFA => { let a = self.fetch_word(); self.reg.a = self.mmu.read_byte(a); 4 },
            0xFB => { self.setei = 2; 1 },
            0xFE => { let v = self.fetch_byte(); self.cp(v); 2 },
            0xFF => { self.push(self.pc); self.pc = 0x38; 4 },
            // other=> panic!("Instruction {:2X} is not implemented", other),

            0xcb => {
                let op = self.fetch_byte();
                let timing = match op {
                    0x00 => { self.reg.b = self.rlc(self.reg.b); 2 },
                    0x01 => { self.reg.c = self.rlc(self.reg.c); 2 },
                    0x02 => { self.reg.d = self.rlc(self.reg.d); 2 },
                    0x03 => { self.reg.e = self.rlc(self.reg.e); 2 },
                    0x04 => { self.reg.h = self.rlc(self.reg.h); 2 },
                    0x05 => { self.reg.l = self.rlc(self.reg.l); 2 },
                    0x06 => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a); let v2 = self.rlc(v); self.mmu.write_byte(a, v2); 4 },
                    0x07 => { self.reg.a = self.rlc(self.reg.a); 2 },
                    0x08 => { self.reg.b = self.rrc(self.reg.b); 2 },
                    0x09 => { self.reg.c = self.rrc(self.reg.c); 2 },
                    0x0A => { self.reg.d = self.rrc(self.reg.d); 2 },
                    0x0B => { self.reg.e = self.rrc(self.reg.e); 2 },
                    0x0C => { self.reg.h = self.rrc(self.reg.h); 2 },
                    0x0D => { self.reg.l = self.rrc(self.reg.l); 2 },
                    0x0E => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a); let v2 = self.rrc(v); self.mmu.write_byte(a, v2); 4 },
                    0x0F => { self.reg.a = self.rrc(self.reg.a); 2 },
                    0x10 => { self.reg.b = self.rl(self.reg.b); 2 },
                    0x11 => { self.reg.c = self.rl(self.reg.c); 2 },
                    0x12 => { self.reg.d = self.rl(self.reg.d); 2 },
                    0x13 => { self.reg.e = self.rl(self.reg.e); 2 },
                    0x14 => { self.reg.h = self.rl(self.reg.h); 2 },
                    0x15 => { self.reg.l = self.rl(self.reg.l); 2 },
                    0x16 => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a); let v2 = self.rl(v); self.mmu.write_byte(a, v2); 4 },
                    0x17 => { self.reg.a = self.rl(self.reg.a); 2 },
                    0x18 => { self.reg.b = self.rr(self.reg.b); 2 },
                    0x19 => { self.reg.c = self.rr(self.reg.c); 2 },
                    0x1A => { self.reg.d = self.rr(self.reg.d); 2 },
                    0x1B => { self.reg.e = self.rr(self.reg.e); 2 },
                    0x1C => { self.reg.h = self.rr(self.reg.h); 2 },
                    0x1D => { self.reg.l = self.rr(self.reg.l); 2 },
                    0x1E => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a); let v2 = self.rr(v); self.mmu.write_byte(a, v2); 4 },
                    0x1F => { self.reg.a = self.rr(self.reg.a); 2 },
                    0x20 => { self.reg.b = self.sla(self.reg.b); 2 },
                    0x21 => { self.reg.c = self.sla(self.reg.c); 2 },
                    0x22 => { self.reg.d = self.sla(self.reg.d); 2 },
                    0x23 => { self.reg.e = self.sla(self.reg.e); 2 },
                    0x24 => { self.reg.h = self.sla(self.reg.h); 2 },
                    0x25 => { self.reg.l = self.sla(self.reg.l); 2 },
                    0x26 => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a); let v2 = self.sla(v); self.mmu.write_byte(a, v2); 4 },
                    0x27 => { self.reg.a = self.sla(self.reg.a); 2 },
                    0x28 => { self.reg.b = self.sra(self.reg.b); 2 },
                    0x29 => { self.reg.c = self.sra(self.reg.c); 2 },
                    0x2A => { self.reg.d = self.sra(self.reg.d); 2 },
                    0x2B => { self.reg.e = self.sra(self.reg.e); 2 },
                    0x2C => { self.reg.h = self.sra(self.reg.h); 2 },
                    0x2D => { self.reg.l = self.sra(self.reg.l); 2 },
                    0x2E => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a); let v2 = self.sra(v); self.mmu.write_byte(a, v2); 4 },
                    0x2F => { self.reg.a = self.sra(self.reg.a); 2 },
                    0x30 => { self.reg.b = self.swap(self.reg.b); 2 },
                    0x31 => { self.reg.c = self.swap(self.reg.c); 2 },
                    0x32 => { self.reg.d = self.swap(self.reg.d); 2 },
                    0x33 => { self.reg.e = self.swap(self.reg.e); 2 },
                    0x34 => { self.reg.h = self.swap(self.reg.h); 2 },
                    0x35 => { self.reg.l = self.swap(self.reg.l); 2 },
                    0x36 => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a); let v2 = self.swap(v); self.mmu.write_byte(a, v2); 4 },
                    0x37 => { self.reg.a = self.swap(self.reg.a); 2 },
                    0x38 => { self.reg.b = self.srl(self.reg.b); 2 },
                    0x39 => { self.reg.c = self.srl(self.reg.c); 2 },
                    0x3A => { self.reg.d = self.srl(self.reg.d); 2 },
                    0x3B => { self.reg.e = self.srl(self.reg.e); 2 },
                    0x3C => { self.reg.h = self.srl(self.reg.h); 2 },
                    0x3D => { self.reg.l = self.srl(self.reg.l); 2 },
                    0x3E => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a); let v2 = self.srl(v); self.mmu.write_byte(a, v2); 4 },
                    0x3F => { self.reg.a = self.srl(self.reg.a); 2 },
                    0x40 => { self.bit(self.reg.b, 0); 2 },
                    0x41 => { self.bit(self.reg.c, 0); 2 },
                    0x42 => { self.bit(self.reg.d, 0); 2 },
                    0x43 => { self.bit(self.reg.e, 0); 2 },
                    0x44 => { self.bit(self.reg.h, 0); 2 },
                    0x45 => { self.bit(self.reg.l, 0); 2 },
                    0x46 => { let v = self.mmu.read_byte(self.reg.get_hl()); self.bit(v, 0); 3 },
                    0x47 => { self.bit(self.reg.a, 0); 2 },
                    0x48 => { self.bit(self.reg.b, 1); 2 },
                    0x49 => { self.bit(self.reg.c, 1); 2 },
                    0x4A => { self.bit(self.reg.d, 1); 2 },
                    0x4B => { self.bit(self.reg.e, 1); 2 },
                    0x4C => { self.bit(self.reg.h, 1); 2 },
                    0x4D => { self.bit(self.reg.l, 1); 2 },
                    0x4E => { let v = self.mmu.read_byte(self.reg.get_hl()); self.bit(v, 1); 3 },
                    0x4F => { self.bit(self.reg.a, 1); 2 },
                    0x50 => { self.bit(self.reg.b, 2); 2 },
                    0x51 => { self.bit(self.reg.c, 2); 2 },
                    0x52 => { self.bit(self.reg.d, 2); 2 },
                    0x53 => { self.bit(self.reg.e, 2); 2 },
                    0x54 => { self.bit(self.reg.h, 2); 2 },
                    0x55 => { self.bit(self.reg.l, 2); 2 },
                    0x56 => { let v = self.mmu.read_byte(self.reg.get_hl()); self.bit(v, 2); 3 },
                    0x57 => { self.bit(self.reg.a, 2); 2 },
                    0x58 => { self.bit(self.reg.b, 3); 2 },
                    0x59 => { self.bit(self.reg.c, 3); 2 },
                    0x5A => { self.bit(self.reg.d, 3); 2 },
                    0x5B => { self.bit(self.reg.e, 3); 2 },
                    0x5C => { self.bit(self.reg.h, 3); 2 },
                    0x5D => { self.bit(self.reg.l, 3); 2 },
                    0x5E => { let v = self.mmu.read_byte(self.reg.get_hl()); self.bit(v, 3); 3 },
                    0x5F => { self.bit(self.reg.a, 3); 2 },
                    0x60 => { self.bit(self.reg.b, 4); 2 },
                    0x61 => { self.bit(self.reg.c, 4); 2 },
                    0x62 => { self.bit(self.reg.d, 4); 2 },
                    0x63 => { self.bit(self.reg.e, 4); 2 },
                    0x64 => { self.bit(self.reg.h, 4); 2 },
                    0x65 => { self.bit(self.reg.l, 4); 2 },
                    0x66 => { let v = self.mmu.read_byte(self.reg.get_hl()); self.bit(v, 4); 3 },
                    0x67 => { self.bit(self.reg.a, 4); 2 },
                    0x68 => { self.bit(self.reg.b, 5); 2 },
                    0x69 => { self.bit(self.reg.c, 5); 2 },
                    0x6A => { self.bit(self.reg.d, 5); 2 },
                    0x6B => { self.bit(self.reg.e, 5); 2 },
                    0x6C => { self.bit(self.reg.h, 5); 2 },
                    0x6D => { self.bit(self.reg.l, 5); 2 },
                    0x6E => { let v = self.mmu.read_byte(self.reg.get_hl()); self.bit(v, 5); 3 },
                    0x6F => { self.bit(self.reg.a, 5); 2 },
                    0x70 => { self.bit(self.reg.b, 6); 2 },
                    0x71 => { self.bit(self.reg.c, 6); 2 },
                    0x72 => { self.bit(self.reg.d, 6); 2 },
                    0x73 => { self.bit(self.reg.e, 6); 2 },
                    0x74 => { self.bit(self.reg.h, 6); 2 },
                    0x75 => { self.bit(self.reg.l, 6); 2 },
                    0x76 => { let v = self.mmu.read_byte(self.reg.get_hl()); self.bit(v, 6); 3 },
                    0x77 => { self.bit(self.reg.a, 6); 2 },
                    0x78 => { self.bit(self.reg.b, 7); 2 },
                    0x79 => { self.bit(self.reg.c, 7); 2 },
                    0x7A => { self.bit(self.reg.d, 7); 2 },
                    0x7B => { self.bit(self.reg.e, 7); 2 },
                    0x7C => { self.bit(self.reg.h, 7); 2 },
                    0x7D => { self.bit(self.reg.l, 7); 2 },
                    0x7E => { let v = self.mmu.read_byte(self.reg.get_hl()); self.bit(v, 7); 3 },
                    0x7F => { self.bit(self.reg.a, 7); 2 },
                    0x80 => { self.reg.b = self.reg.b & !(1 << 0); 2 },
                    0x81 => { self.reg.c = self.reg.c & !(1 << 0); 2 },
                    0x82 => { self.reg.d = self.reg.d & !(1 << 0); 2 },
                    0x83 => { self.reg.e = self.reg.e & !(1 << 0); 2 },
                    0x84 => { self.reg.h = self.reg.h & !(1 << 0); 2 },
                    0x85 => { self.reg.l = self.reg.l & !(1 << 0); 2 },
                    0x86 => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a) & !(1 << 0); self.mmu.write_byte(a, v); 4 },
                    0x87 => { self.reg.a = self.reg.a & !(1 << 0); 2 },
                    0x88 => { self.reg.b = self.reg.b & !(1 << 1); 2 },
                    0x89 => { self.reg.c = self.reg.c & !(1 << 1); 2 },
                    0x8A => { self.reg.d = self.reg.d & !(1 << 1); 2 },
                    0x8B => { self.reg.e = self.reg.e & !(1 << 1); 2 },
                    0x8C => { self.reg.h = self.reg.h & !(1 << 1); 2 },
                    0x8D => { self.reg.l = self.reg.l & !(1 << 1); 2 },
                    0x8E => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a) & !(1 << 1); self.mmu.write_byte(a, v); 4 },
                    0x8F => { self.reg.a = self.reg.a & !(1 << 1); 2 },
                    0x90 => { self.reg.b = self.reg.b & !(1 << 2); 2 },
                    0x91 => { self.reg.c = self.reg.c & !(1 << 2); 2 },
                    0x92 => { self.reg.d = self.reg.d & !(1 << 2); 2 },
                    0x93 => { self.reg.e = self.reg.e & !(1 << 2); 2 },
                    0x94 => { self.reg.h = self.reg.h & !(1 << 2); 2 },
                    0x95 => { self.reg.l = self.reg.l & !(1 << 2); 2 },
                    0x96 => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a) & !(1 << 2); self.mmu.write_byte(a, v); 4 },
                    0x97 => { self.reg.a = self.reg.a & !(1 << 2); 2 },
                    0x98 => { self.reg.b = self.reg.b & !(1 << 3); 2 },
                    0x99 => { self.reg.c = self.reg.c & !(1 << 3); 2 },
                    0x9A => { self.reg.d = self.reg.d & !(1 << 3); 2 },
                    0x9B => { self.reg.e = self.reg.e & !(1 << 3); 2 },
                    0x9C => { self.reg.h = self.reg.h & !(1 << 3); 2 },
                    0x9D => { self.reg.l = self.reg.l & !(1 << 3); 2 },
                    0x9E => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a) & !(1 << 3); self.mmu.write_byte(a, v); 4 },
                    0x9F => { self.reg.a = self.reg.a & !(1 << 3); 2 },
                    0xA0 => { self.reg.b = self.reg.b & !(1 << 4); 2 },
                    0xA1 => { self.reg.c = self.reg.c & !(1 << 4); 2 },
                    0xA2 => { self.reg.d = self.reg.d & !(1 << 4); 2 },
                    0xA3 => { self.reg.e = self.reg.e & !(1 << 4); 2 },
                    0xA4 => { self.reg.h = self.reg.h & !(1 << 4); 2 },
                    0xA5 => { self.reg.l = self.reg.l & !(1 << 4); 2 },
                    0xA6 => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a) & !(1 << 4); self.mmu.write_byte(a, v); 4 },
                    0xA7 => { self.reg.a = self.reg.a & !(1 << 4); 2 },
                    0xA8 => { self.reg.b = self.reg.b & !(1 << 5); 2 },
                    0xA9 => { self.reg.c = self.reg.c & !(1 << 5); 2 },
                    0xAA => { self.reg.d = self.reg.d & !(1 << 5); 2 },
                    0xAB => { self.reg.e = self.reg.e & !(1 << 5); 2 },
                    0xAC => { self.reg.h = self.reg.h & !(1 << 5); 2 },
                    0xAD => { self.reg.l = self.reg.l & !(1 << 5); 2 },
                    0xAE => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a) & !(1 << 5); self.mmu.write_byte(a, v); 4 },
                    0xAF => { self.reg.a = self.reg.a & !(1 << 5); 2 },
                    0xB0 => { self.reg.b = self.reg.b & !(1 << 6); 2 },
                    0xB1 => { self.reg.c = self.reg.c & !(1 << 6); 2 },
                    0xB2 => { self.reg.d = self.reg.d & !(1 << 6); 2 },
                    0xB3 => { self.reg.e = self.reg.e & !(1 << 6); 2 },
                    0xB4 => { self.reg.h = self.reg.h & !(1 << 6); 2 },
                    0xB5 => { self.reg.l = self.reg.l & !(1 << 6); 2 },
                    0xB6 => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a) & !(1 << 6); self.mmu.write_byte(a, v); 4 },
                    0xB7 => { self.reg.a = self.reg.a & !(1 << 6); 2 },
                    0xB8 => { self.reg.b = self.reg.b & !(1 << 7); 2 },
                    0xB9 => { self.reg.c = self.reg.c & !(1 << 7); 2 },
                    0xBA => { self.reg.d = self.reg.d & !(1 << 7); 2 },
                    0xBB => { self.reg.e = self.reg.e & !(1 << 7); 2 },
                    0xBC => { self.reg.h = self.reg.h & !(1 << 7); 2 },
                    0xBD => { self.reg.l = self.reg.l & !(1 << 7); 2 },
                    0xBE => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a) & !(1 << 7); self.mmu.write_byte(a, v); 4 },
                    0xBF => { self.reg.a = self.reg.a & !(1 << 7); 2 },
                    0xC0 => { self.reg.b = self.reg.b | (1 << 0); 2 },
                    0xC1 => { self.reg.c = self.reg.c | (1 << 0); 2 },
                    0xC2 => { self.reg.d = self.reg.d | (1 << 0); 2 },
                    0xC3 => { self.reg.e = self.reg.e | (1 << 0); 2 },
                    0xC4 => { self.reg.h = self.reg.h | (1 << 0); 2 },
                    0xC5 => { self.reg.l = self.reg.l | (1 << 0); 2 },
                    0xC6 => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a) | (1 << 0); self.mmu.write_byte(a, v); 4 },
                    0xC7 => { self.reg.a = self.reg.a | (1 << 0); 2 },
                    0xC8 => { self.reg.b = self.reg.b | (1 << 1); 2 },
                    0xC9 => { self.reg.c = self.reg.c | (1 << 1); 2 },
                    0xCA => { self.reg.d = self.reg.d | (1 << 1); 2 },
                    0xCB => { self.reg.e = self.reg.e | (1 << 1); 2 },
                    0xCC => { self.reg.h = self.reg.h | (1 << 1); 2 },
                    0xCD => { self.reg.l = self.reg.l | (1 << 1); 2 },
                    0xCE => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a) | (1 << 1); self.mmu.write_byte(a, v); 4 },
                    0xCF => { self.reg.a = self.reg.a | (1 << 1); 2 },
                    0xD0 => { self.reg.b = self.reg.b | (1 << 2); 2 },
                    0xD1 => { self.reg.c = self.reg.c | (1 << 2); 2 },
                    0xD2 => { self.reg.d = self.reg.d | (1 << 2); 2 },
                    0xD3 => { self.reg.e = self.reg.e | (1 << 2); 2 },
                    0xD4 => { self.reg.h = self.reg.h | (1 << 2); 2 },
                    0xD5 => { self.reg.l = self.reg.l | (1 << 2); 2 },
                    0xD6 => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a) | (1 << 2); self.mmu.write_byte(a, v); 4 },
                    0xD7 => { self.reg.a = self.reg.a | (1 << 2); 2 },
                    0xD8 => { self.reg.b = self.reg.b | (1 << 3); 2 },
                    0xD9 => { self.reg.c = self.reg.c | (1 << 3); 2 },
                    0xDA => { self.reg.d = self.reg.d | (1 << 3); 2 },
                    0xDB => { self.reg.e = self.reg.e | (1 << 3); 2 },
                    0xDC => { self.reg.h = self.reg.h | (1 << 3); 2 },
                    0xDD => { self.reg.l = self.reg.l | (1 << 3); 2 },
                    0xDE => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a) | (1 << 3); self.mmu.write_byte(a, v); 4 },
                    0xDF => { self.reg.a = self.reg.a | (1 << 3); 2 },
                    0xE0 => { self.reg.b = self.reg.b | (1 << 4); 2 },
                    0xE1 => { self.reg.c = self.reg.c | (1 << 4); 2 },
                    0xE2 => { self.reg.d = self.reg.d | (1 << 4); 2 },
                    0xE3 => { self.reg.e = self.reg.e | (1 << 4); 2 },
                    0xE4 => { self.reg.h = self.reg.h | (1 << 4); 2 },
                    0xE5 => { self.reg.l = self.reg.l | (1 << 4); 2 },
                    0xE6 => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a) | (1 << 4); self.mmu.write_byte(a, v); 4 },
                    0xE7 => { self.reg.a = self.reg.a | (1 << 4); 2 },
                    0xE8 => { self.reg.b = self.reg.b | (1 << 5); 2 },
                    0xE9 => { self.reg.c = self.reg.c | (1 << 5); 2 },
                    0xEA => { self.reg.d = self.reg.d | (1 << 5); 2 },
                    0xEB => { self.reg.e = self.reg.e | (1 << 5); 2 },
                    0xEC => { self.reg.h = self.reg.h | (1 << 5); 2 },
                    0xED => { self.reg.l = self.reg.l | (1 << 5); 2 },
                    0xEE => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a) | (1 << 5); self.mmu.write_byte(a, v); 4 },
                    0xEF => { self.reg.a = self.reg.a | (1 << 5); 2 },
                    0xF0 => { self.reg.b = self.reg.b | (1 << 6); 2 },
                    0xF1 => { self.reg.c = self.reg.c | (1 << 6); 2 },
                    0xF2 => { self.reg.d = self.reg.d | (1 << 6); 2 },
                    0xF3 => { self.reg.e = self.reg.e | (1 << 6); 2 },
                    0xF4 => { self.reg.h = self.reg.h | (1 << 6); 2 },
                    0xF5 => { self.reg.l = self.reg.l | (1 << 6); 2 },
                    0xF6 => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a) | (1 << 6); self.mmu.write_byte(a, v); 4 },
                    0xF7 => { self.reg.a = self.reg.a | (1 << 6); 2 },
                    0xF8 => { self.reg.b = self.reg.b | (1 << 7); 2 },
                    0xF9 => { self.reg.c = self.reg.c | (1 << 7); 2 },
                    0xFA => { self.reg.d = self.reg.d | (1 << 7); 2 },
                    0xFB => { self.reg.e = self.reg.e | (1 << 7); 2 },
                    0xFC => { self.reg.h = self.reg.h | (1 << 7); 2 },
                    0xFD => { self.reg.l = self.reg.l | (1 << 7); 2 },
                    0xFE => { let a = self.reg.get_hl(); let v = self.mmu.read_byte(a) | (1 << 7); self.mmu.write_byte(a, v); 4 },
                    0xFF => { self.reg.a = self.reg.a | (1 << 7); 2 },
                };
                //timing + 1 // TODO: It turns out the opcode table already includes the timing for the extra cycle to get to the next opcode table, so adding one isn't necessary.
                timing
            }
            _ => unimplemented!("Unimplemented opcode: {:#04x}", op),
        };
        if LOG_LEVEL >= 4 {
            print!("length of execution {}\n", timing);
        }
        if LOG_LEVEL >= 3 {
            let this = format!("{}", timing);
            println!("{}", this);
            println!("{}", self.debug_file[(self.line)]);
            if this != self.debug_file[self.line ]{
                unimplemented!("Not matching original");
            }
            self.line += 1;

        }
        timing
    }

    // fn adc(&mut self, val: u8) {
    //     let orig = self.reg.a;
    //     self.reg.a = self.reg.a + val + if self.reg.get_flag(flags::C) {1} else {0};
    //     self.reg.set_flag(flags::Z, self.reg.a == 0);
    //     self.reg.set_flag(flags::N, false);
    //     self.reg.set_flag(flags::H, ((orig >> 3) & 0b1) != ((self.reg.a >> 3) & 0b1));
    //     self.reg.set_flag(flags::C, ((orig >> 7) & 0b1) != ((self.reg.a >> 7) & 0b1));

    // }

    fn adc(&mut self, val: u8){
        self.add(val, true);
    }

    fn add(&mut self, b: u8, usec: bool) {
        let c = if usec && self.reg.get_flag(flags::C) { 1 } else { 0 };
        let a = self.reg.a;
        let r = a.wrapping_add(b).wrapping_add(c);
        self.reg.set_flag(flags::Z, r == 0);
        self.reg.set_flag(flags::H, (a & 0xF) + (b & 0xF) + c > 0xF);
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::C, (a as u16) + (b as u16) + (c as u16) > 0xFF);
        self.reg.a = r;
    }

    fn sbc(&mut self, val: u8) {
        let orig = self.reg.a;
        let carry = if self.reg.get_flag(flags::C) {1} else {0};
        self.reg.a = self.reg.a - val - carry;
        self.reg.set_flag(flags::Z, self.reg.a == 0);
        self.reg.set_flag(flags::N, true);
        self.reg.set_flag(flags::H, ((orig >> 3) & 0b1) != ((self.reg.a >> 3) & 0b1));
        self.reg.set_flag(flags::C, (val + carry) > orig);
    }

    fn daa(&mut self, hex: u8) -> u8 {
        let mut high = hex & 0xF0;
        let mut low = hex & 0x0F;
        self.reg.set_flag(flags::H, false);
        if low > 9 {
            high += low - 9;
            low -= 9;
            self.reg.set_flag(flags::C, true);
            self.reg.set_flag(flags::Z, (high & low) == 0);
            return high & low;
        } else {
            self.reg.set_flag(flags::C, false);
            self.reg.set_flag(flags::Z, hex == 0);
            return hex;
        }
    }

    fn add16(&mut self, b: u16) {
        let a = self.reg.get_hl();
        let r = a.wrapping_add(b);
        self.reg.set_flag(flags::H, (a & 0x07FF) + (b & 0x07FF) > 0x07FF);
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::C, a > 0xFFFF - b);
        self.reg.set_hl(r);
    }

    fn add16imm(&mut self, a: u16) -> u16 { 
        let b = self.fetch_byte() as i8 as i16 as u16;
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::Z, false);
        self.reg.set_flag(flags::H, (a & 0x000F) + (b & 0x000F) > 0x000F);
        self.reg.set_flag(flags::C, (a & 0x00FF) + (b & 0x00FF) > 0x00FF);
        return a.wrapping_add(b)
    }

    fn cp(&mut self, val: u8) {
        let r = self.reg.a;
        self.sub(val, false);
        self.reg.a = r;
    }
    fn sub(&mut self, b: u8, usec: bool) {
        let c = if usec && self.reg.get_flag(flags::C) { 1 } else { 0 };
        let a = self.reg.a;
        let r = a.wrapping_sub(b).wrapping_sub(c);
        self.reg.set_flag(flags::Z, r == 0);
        self.reg.set_flag(flags::H, (a & 0x0F) < (b & 0x0F) + c);
        self.reg.set_flag(flags::N, true);
        self.reg.set_flag(flags::C, (a as u16) < (b as u16) + (c as u16));
        self.reg.a = r;
    }

    fn cpu_jr(&mut self) {
        let n = self.fetch_byte() as i8;
        self.pc = ((self.pc as u32 as i32) + (n as i32)) as u16;
    }

    fn add_byte(&mut self, b: u8, usec: bool) { // TODO: Rewrite function
        let c = if usec && self.reg.get_flag(flags::C) { 1 } else { 0 };
        let a = self.reg.a;
        let r = a.wrapping_add(b).wrapping_add(c);
        self.reg.set_flag(flags::Z, r == 0);
        self.reg.set_flag(flags::H, (a & 0xF) + (b & 0xF) + c > 0xF);
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::C, (a as u16) + (b as u16) + (c as u16) > 0xFF);
        self.reg.a = r;
    }

    fn add_word(&mut self, a: u16, b: u16) -> u16 { // TODO: Write tests
        let (result, carry) = a.overflowing_add(b);
        self.reg.set_flag(flags::C, carry);
        // self.reg.set_flag(flags::H ,((self.reg.b as u16 + self.reg.c as u16) & 0xFF00) != 0);
        self.reg.set_flag(flags::H ,(a & 0x07FF) + (b & 0x07FF) > 0x07FF);
        self.reg.set_flag(flags::N, false);
        result
    }

    fn add_word_z(&mut self, a: u16, b: u16) -> u16 { // It would be good if rust had an easy way to provide optional parameters for this case https://stackoverflow.com/questions/24047686/default-function-arguments-in-rust
        let (result, carry) = a.overflowing_add(b);
        self.reg.set_flag(flags::C, carry);
        // self.reg.set_flag(flags::H ,((self.reg.b as u16 + self.reg.c as u16) & 0xFF00) != 0);
        self.reg.set_flag(flags::H ,(a & 0x07FF) + (b & 0x07FF) > 0x07FF);
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::Z, false);
        result
    }

    fn and(&mut self, val: u8) {
        let res = self.reg.a & val;
        self.reg.set_flag(flags::Z, res == 0);
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::H, true);
        self.reg.set_flag(flags::C, false);
        self.reg.a = res;
    }

    fn or(&mut self, val: u8) {
        let res = self.reg.a | val;
        self.reg.set_flag(flags::Z, res == 0);
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::H, false);
        self.reg.set_flag(flags::C, false);
        self.reg.a = res; 
    }

    fn xor(&mut self, val: u8) {
        let res = self.reg.a ^ val;
        self.reg.set_flag(flags::Z, res == 0);
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::H, false);
        self.reg.set_flag(flags::C, false);
        self.reg.a = res;
    }

    fn sub_byte(&mut self, a: u8, b: u8) -> u8 { // TODO: Write tests
        let c = if self.reg.get_flag(flags::C) {1} else {0} ;
        let result = a.wrapping_sub(b);
        self.reg.set_flag(flags::Z, result == 0);
        self.reg.set_flag(flags::H, (a & 0x0F) < (b & 0x0F) + c);
        self.reg.set_flag(flags::N, true);
        self.reg.set_flag(flags::C, (a as u16) < (b as u16) + (c as u16));
        result
    }

    fn sub_word(&mut self, a: u16, b: u16) -> u16 { // TODO: Write tests
        let (result, carry) = a.overflowing_add(b);
        self.reg.set_flag(flags::C, carry);
        self.reg.set_flag(flags::H ,((self.reg.b as u16 + self.reg.c as u16) & 0xFF00) != 0);
        self.reg.set_flag(flags::N, true);
        result
    }

    // fn rl(&mut self, val: u8) -> u8 {
    //     self.reg.set_flag(flags::C, (val >> 7) == 1);
    //     val.rotate_left(1)
    // }

    // fn rlc(&mut self, val: u8) -> u8 {
    //     let right_bit = if self.reg.get_flag(flags::C) {1 as u8} else {0 as u8};
    //     self.reg.set_flag(flags::C, (val >> 7) == 1);    
    //     (val << 1) | right_bit
    // }

    // fn rr(&mut self, val: u8) -> u8 {
    //     // println!("and {}", val & 1);
    //     self.reg.set_flag(flags::C, (val & 1) == 1);
    //     val.rotate_right(1)
    // }

    fn srflagupdate(&mut self, r: u8, c: bool) {
        self.reg.set_flag(flags::H, false);
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::Z, r == 0);
        self.reg.set_flag(flags::C, c);
    }

    fn rlc(&mut self, a: u8) -> u8 {
        let c = a & 0x80 == 0x80;
        let r = (a << 1) | (if c { 1 } else { 0 });
        self.srflagupdate(r, c);
        return r
    }

    fn rl(&mut self, a: u8) -> u8 {
        let c = a & 0x80 == 0x80;
        let r = (a << 1) | (if self.reg.get_flag(flags::C) { 1 } else { 0 });
        self.srflagupdate(r, c);
        return r
    }

    fn rr(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = (a >> 1) | (if self.reg.get_flag(flags::C) { 0x80 } else { 0 });
        self.srflagupdate(r, c);
        return r
    }

    fn rrc(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = (a >> 1) | (if c { 0x80 } else { 0 });
        self.srflagupdate(r, c);
        return r
    }

    // fn rrc(&mut self, val: u8) -> u8 {
    //     let left_bit = (if self.reg.get_flag(flags::C) {1 as u8} else {0 as u8}) << 7;
    //     self.reg.set_flag(flags::C, (val & 1) == 1);    
    //     (val >> 1) | left_bit
    // }

    fn inc(&mut self, val: u8) -> u8 {
        let (res, carry) = val.overflowing_add(1);
        if res == 0 {self.reg.set_flag(flags::Z, true)} else {self.reg.set_flag(flags::Z, false)}
        self.reg.set_flag(flags::Z, res == 0);
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::H, (val & 0x0F) + 1 > 0x0F);
        res
    }

    fn dec(&mut self, val: u8) -> u8 {
        let (res, carry) = val.overflowing_sub(1);
        self.reg.set_flag(flags::Z, res == 0);
        self.reg.set_flag(flags::N, true);
        self.reg.set_flag(flags::H, (val & 0x0F) == 0);
        res
    }

    fn jr(&mut self) {
        let offset = self.fetch_byte() as i8;
        self.pc = ((self.pc as u32 as i32) + (offset as i32)) as u16;
    }

    fn ret(&mut self) {
        // self.pc = self.mmu.read_word(self.sp);
        // println!("{} {} {}", self.mmu.read_word(self.sp -1), self.mmu.read_word(self.sp), self.mmu.read_word(self.sp + 1));
        // self.sp += 2;
        self.pc = self.pop();
    }

    fn ei(&mut self) {
        self.ime = true;
    }
    fn di(&mut self) {
        self.ime = false;
        self.tempIme = false;
    }

    fn reti(&mut self) {
        self.ei();
        self.ret();
    }

    fn call(&mut self, pointer: u16) {
        self.push(self.pc);
        self.pc = pointer;
    }

    fn push(&mut self, val: u16) {
        self.sp = self.sp - 2;
        self.mmu.write_word(self.sp, val);
    }

    fn pop(&mut self) -> u16 { // TODO: Checks might need to be made here.
        self.sp = self.sp + 2;
        self.mmu.read_word(self.sp - 2) // rr = popped value
    }

    // fn sla(&mut self, val: u8) -> u8 { // Shift left arithmetically
    //     let res = val << 1;
    //     let carry = (val >> 7) == 1;
    //     self.reg.unset_flags();
    //     self.reg.set_flag(flags::Z, res == 0);
    //     self.reg.set_flag(flags::C, carry);
    //     res
    // }

    // fn sra(&mut self, val: u8) -> u8 { // Shift right arithmetically
    //     let msb = val >> 7; // Most significant bit
    //     let res = (val >> 1) & msb;
    //     let carry = (val & 0b1) == 1;
    //     self.reg.unset_flags();
    //     self.reg.set_flag(flags::Z, res == 0);
    //     self.reg.set_flag(flags::C, carry);
    //     res
    // }

    // fn srl(&mut self, val: u8) -> u8 { // Shift right logically
    //     let res = val >> 1;
    //     let carry = (val & 0b1) == 1;
    //     self.reg.unset_flags();
    //     self.reg.set_flag(flags::Z, res == 0);
    //     self.reg.set_flag(flags::C, carry);
    //     res
    // }

    fn sla(&mut self, a: u8) -> u8 {
        let c = a & 0x80 == 0x80;
        let r = a << 1;
        self.alu_srflagupdate(r, c);
        return r
    }

    fn sra(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = (a >> 1) | (a & 0x80);
        self.alu_srflagupdate(r, c);
        return r
    }

    fn srl(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = a >> 1;
        self.alu_srflagupdate(r, c);
        return r
    }

    fn alu_srflagupdate(&mut self, r: u8, c: bool) {
        self.reg.set_flag(flags::H, false);
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::Z, r == 0);
        self.reg.set_flag(flags::C, c);
    }

    fn swap(&mut self, val: u8) -> u8 {
        let lth = (val & 0x0F) << 4; // Lower bit to higher bit
        let htl = val >> 4; 
        self.reg.unset_flags();
        self.reg.set_flag(flags::Z, (lth | htl) == 0);
        lth | htl
    }

    // fn bit(&mut self, position: u8, val: u8) {
    //     let bit = (val >> position) & 0b1;
    //     self.reg.set_flag(flags::Z, bit == 1);
    //     self.reg.set_flag(flags::N, false);
    //     self.reg.set_flag(flags::H, true);
    // }
    fn bit(&mut self, a: u8, b: u8) {
        let r = a & (1 << (b as u32)) == 0;
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::H, true);
        self.reg.set_flag(flags::Z, r);
    }

    fn res(&self, position: u8, val: u8) -> u8 { // TODO: Write unit test for this
        val & !(1 << position) | (u8::from(0) << position)
    }

    fn set (&self, position: u8, val: u8) -> u8 {
        val & !(1 << position) | (u8::from(1) << position)
    }

}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rl() {
        let mut cpu = Cpu::new();
        assert_eq!(cpu.rl(0b10101010), 0b01010101);
        assert_eq!(cpu.reg.get_flag(flags::C), true);
        
        assert_eq!(cpu.rl(0b01010101), 0b10101010);
        assert_eq!(cpu.reg.get_flag(flags::C), false);
    }

    // #[test]
    // fn rlc() {
    //     let mut cpu = Cpu::new();
    //     assert_eq!(cpu.rlc(0b10101010), 0b01010100);
    //     assert_eq!(cpu.reg.get_flag(flags::C), true);
        
    //     assert_eq!(cpu.rlc(0b01010100), 0b10101001);
    //     assert_eq!(cpu.reg.get_flag(flags::C), false);
    // }

    #[test]
    fn rr() {
        let mut cpu = Cpu::new();
        assert_eq!(cpu.rr(0b10000001), 0b11000000);
        assert_eq!(cpu.reg.get_flag(flags::C), true);
        
        assert_eq!(cpu.rr(0b11000000), 0b01100000);
        assert_eq!(cpu.reg.get_flag(flags::C), false);
    }

    // #[test]
    // fn rrc() {
    //     let mut cpu = Cpu::new();
    //     assert_eq!(cpu.rrc(0b10000001), 0b01000000);
    //     assert_eq!(cpu.reg.get_flag(flags::C), true);
        
    //     assert_eq!(cpu.rrc(0b01000000), 0b10100000);
    //     assert_eq!(cpu.reg.get_flag(flags::C), false);
    // }

    // #[test]
    // fn set_flag() {
    //     let mut cpu = Cpu::new();
    //     assert_eq!(cpu.reg.get_flag(flags::C), false);

    //     cpu.reg.set_flag(flags::C, true);
    //     assert_eq!(cpu.reg.get_flag(flags::C), true);

    //     cpu.reg.set_flag(flags::C, false);
    //     assert_eq!(cpu.reg.get_flag(flags::C), false);
    //     assert!((0b10101010 >> 7) == 1);
    // }

    // #[test]
    // fn set() {
    //     let mut cpu = Cpu::new();
    //     cpu.mmu.write_byte(cpu.pc, 0xc0);
    //     cpu.execute(0xcb);
    //     assert_eq!(cpu.reg.b, 0b1);
    // }

    // #[test]
    // fn set2() {
    //     let mut cpu = Cpu::new();
    //     cpu.mmu.write_byte(cpu.pc, 0xe1);
    //     cpu.execute(0xcb);
    //     assert_eq!(cpu.reg.c, 0b10000);
    // }

    // #[test]
    // fn set3() {
    //     let mut cpu = Cpu::new();
    //     cpu.reg.set_hl(0xff);
    //     cpu.mmu.write_byte(cpu.pc, 0xc6);
        
    //     cpu.execute(0xcb);
    //     assert_eq!(cpu.mmu.read_byte(cpu.reg.get_hl()), 0b1);
    // }

    #[test]
    fn xor_a() {
        let mut cpu = Cpu::new();
        cpu.reg.a = 0x01;
        cpu.execute(0xaf);
        assert_eq!(cpu.reg.a, 0b0);
    }

    #[test]
    fn rightmost_set_bit() {
        let mut cpu = Cpu::new();

        assert_eq!(cpu.rightmost_set_bit(0b00000001), 0);
        assert_eq!(cpu.rightmost_set_bit(0b00010000), 4);
    }

}