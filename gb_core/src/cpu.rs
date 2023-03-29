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
    pub mmu: MMU,
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
            0x00 => {1}
            0x01 => {let word = self.fetch_word(); self.reg.set_bc(word); 3}
            0x02 => {self.mmu.write_byte(self.reg.get_bc(), self.reg.a); 2}
            0x03 => {self.reg.set_bc(self.reg.get_bc().wrapping_add(1)); 2}
            0x04 => {self.reg.b = self.inc(self.reg.b); 1}
            0x05 => {self.reg.b = self.dec(self.reg.b); 1}
            0x06 => {self.reg.b = self.fetch_byte(); 2}
            // 0x07 => {self.reg.a = self.rlc(self.reg.a); 1}
            0x07 => { self.reg.a = self.rlc(self.reg.a); self.reg.set_flag(flags::Z, false); 1 },
            0x08 => {let word = self.fetch_word(); self.mmu.write_word(word, self.sp); 5}
            0x09 => {let res = self.add_word(self.reg.get_hl(), self.reg.get_bc()); self.reg.set_hl(res); 2}
            0x0a => {self.reg.a = self.mmu.read_byte(self.reg.get_bc()); 2}
            0x0b => {self.reg.set_bc(self.reg.get_bc().wrapping_sub(1)); 2}
            0x0c => {self.reg.c = self.inc(self.reg.c); 1}
            0x0d => {self.reg.c = self.dec(self.reg.c); 1}
            0x0e => {self.reg.c = self.fetch_byte(); 2}
            // 0x0f => {self.reg.a = self.rrc(self.reg.a); 1}
            0x0f => { self.reg.a = self.rrc(self.reg.a); self.reg.set_flag(flags::Z, false); 1 },
            // 0x10 => { self.mmu.switch_speed(); 1 }, // STOP
            0x10 => { 1 }
            0x11 => {let word = self.fetch_word(); self.reg.set_de(word); 3}
            0x12 => {self.mmu.write_byte(self.reg.get_de(), self.reg.a); 2}
            0x13 => {self.reg.set_de(self.reg.get_de().wrapping_add(1)); 2}
            0x14 => {self.reg.d = self.inc(self.reg.d); 1}
            0x15 => {self.reg.d = self.dec(self.reg.d); 1}
            0x16 => {self.reg.d = self.fetch_byte(); 2}
            // 0x17 => {self.reg.a = self.rl(self.reg.a); 1}
            0x17 => { self.reg.a = self.rl(self.reg.a); self.reg.set_flag(flags::Z, false); 1 },
            0x18 => {self.jr(); 3}
            0x19 => {let res = self.add_word(self.reg.get_hl(), self.reg.get_de()); self.reg.set_hl(res); 2}
            0x1a => {self.reg.a = self.mmu.read_byte(self.reg.get_de()); 2}
            // 0x1b => {self.reg.set_bc(self.reg.get_de().wrapping_sub(1)); 2}
            0x1b => { self.reg.set_de(self.reg.get_de().wrapping_sub(1)); 2 },
            0x1c => {self.reg.e = self.inc(self.reg.e); 1}
            0x1d => {self.reg.e = self.dec(self.reg.e); 1}
            0x1e => {self.reg.e = self.fetch_byte(); 2}
            // 0x1f => {self.reg.a = self.rr(self.reg.a); 1}
            0x1F => { self.reg.a = self.rr(self.reg.a); self.reg.set_flag(flags::Z, false); 1 },
            0x20 => {if !self.reg.get_flag(flags::Z) {self.jr(); 3} else {self.pc += 1; 2}}
            0x21 => {let word = self.fetch_word(); self.reg.set_hl(word); 3}
            0x22 => {let p = self.reg.get_hl(); self.reg.set_hl(p + 1); self.mmu.write_byte(p, self.reg.a); 2}
            0x23 => {self.reg.set_hl(self.reg.get_hl().wrapping_add(1)); 2}
            0x24 => {self.reg.h = self.inc(self.reg.h); 1}
            0x25 => {self.reg.h = self.dec(self.reg.h); 1}
            0x26 => {self.reg.h = self.fetch_byte(); 2}
            0x27 => { // DAA - Decimal adjust accumulator to get a correct BCD representation after an arithmetic instruction.
                self.daa(); 1
            }
            0x28 => {if self.reg.get_flag(flags::Z) {self.jr(); 3} else {self.pc += 1; 2}}
            0x29 => {let res = self.add_word(self.reg.get_hl(), self.reg.get_hl()); self.reg.set_hl(res); 2}
            0x2a => {let p = self.reg.get_hl(); self.reg.set_hl(p + 1); self.reg.a = self.mmu.read_byte(p); 2}
            0x2b => {self.reg.set_hl(self.reg.get_hl().wrapping_sub(1)); 2}
            0x2c => {self.reg.l = self.inc(self.reg.l); 1}
            0x2d => {self.reg.l = self.dec(self.reg.l); 1}
            0x2e => {self.reg.l = self.fetch_byte(); 2}
            0x2f => {self.reg.a = !self.reg.a; self.reg.set_flag(flags::N, true); self.reg.set_flag(flags::H, true); 1}

            0x30 => {if !self.reg.get_flag(flags::C) {self.jr(); 3} else {self.pc += 1; 2}}
            0x31 => {self.sp = self.fetch_word(); 3}
            // 0x32 => {let p = self.reg.get_hl(); self.reg.set_hl(p - 1); self.mmu.write_byte(p - 1, self.reg.a); 2}
            0x32 => { self.mmu.write_byte(self.reg.hld(), self.reg.a); 2 },
            0x33 => {self.sp = self.sp.wrapping_add(1); 2}
            0x34 => {let v = self.inc(self.mmu.read_byte(self.reg.get_hl())); self.mmu.write_byte(self.reg.get_hl(), v); 3}
            0x35 => {let v = self.dec(self.mmu.read_byte(self.reg.get_hl())); self.mmu.write_byte(self.reg.get_hl(), v); 3}
            0x36 => {let v = self.fetch_byte(); self.mmu.write_byte(self.reg.get_hl(), v); 3}
            0x37 => {
                self.reg.set_flag(flags::C, true);
                self.reg.set_flag(flags::N, false);
                self.reg.set_flag(flags::H, false);
                1
            }
            0x38 => {if self.reg.get_flag(flags::C) {self.jr(); 3} else {self.pc += 1; 2}}
            0x39 => {let res = self.add_word(self.reg.get_hl(), self.sp); self.reg.set_hl(res); 2}
            // 0x3a => {let p = self.reg.get_hl(); self.reg.set_hl(p - 1); self.reg.a = self.mmu.read_byte(p - 1); 2}
            0x3a => { self.reg.a = self.mmu.read_byte(self.reg.hld()); 2 },
            0x3b => { self.sp = self.sp.wrapping_sub(1); 2 },
            0x3c => {self.reg.a = self.inc(self.reg.a); 1}
            0x3d => {self.reg.a = self.dec(self.reg.a); 1}
            0x3e => {self.reg.a = self.fetch_byte(); 2}
            0x3f => {self.reg.set_flag(flags::C, !self.reg.get_flag(flags::C)); self.reg.set_flag(flags::N, false); self.reg.set_flag(flags::H, false); 1}

            0x40..=0x7f => {
                let params = op - 0x40;
                let first_param = (params / 8) as usize;
                let position = (params % 8) as usize;
                // println!("Important {} {}", params, position);
                if position == 6 || position == 0xe {
                    let value = self.mmu.read_byte(self.reg.get_hl());
                    if first_param == 6 {
                        // println!("Halt at {:#04x} {} {} {} {}", op, self.cycle, position, params, first_param);
                        // unimplemented!("Halt!");
                        self.halted = true
                    } else {
                        let second_param_mut = [&mut self.reg.b, &mut self.reg.c, &mut self.reg.d, &mut self.reg.e, &mut self.reg.h, &mut self.reg.l, &mut 0, &mut self.reg.a];
                        *second_param_mut[first_param as usize] = value;
                    }
                    2
                } else {
                    let second_param = [&self.reg.b, &self.reg.c, &self.reg.d, &self.reg.e, &self.reg.h, &self.reg.l, &0, &self.reg.a]; 
                    let second_param_final = *second_param[position];
                    if first_param == 6 {
                        self.mmu.write_byte(self.reg.get_hl(), second_param_final);
                        2
                    } else {
                        let second_param_mut = [&mut self.reg.b, &mut self.reg.c, &mut self.reg.d, &mut self.reg.e, &mut self.reg.h, &mut self.reg.l, &mut 0, &mut self.reg.a];
                        *second_param_mut[first_param as usize] = second_param_final; 
                        // println!("Important {} {} {} {}", position, first_param, second_param_mut[first_param as usize], second_param_final);
                        1
                    }
                }
            }

            0x80..=0x87 => {
                let params = op - 0x80;
                let position = (params % 8) as usize;
                if position == 6 {
                    let value = self.mmu.read_byte(self.reg.get_hl());
                    self.add_byte(value, false);
                    2
                } else {
                    let second_param = [&self.reg.b, &self.reg.c, &self.reg.d, &self.reg.e, &self.reg.h, &self.reg.l, &0, &self.reg.a]; 
                    let second_param_final = *second_param[position];
                    self.add_byte(second_param_final, false); 
                    1
                }
            }

            0x88..=0x8f => {
                let params = op - 0x88;
                let position = (params % 8) as usize;
                if position == 6 {
                    let value = self.mmu.read_byte(self.reg.get_hl());
                    self.adc(value);
                    2
                } else {
                    let second_param = [&self.reg.b, &self.reg.c, &self.reg.d, &self.reg.e, &self.reg.h, &self.reg.l, &0, &self.reg.a]; 
                    let second_param_final = *second_param[position];
                    self.adc(second_param_final); 
                    1
                }
            }

            0x90..=0x97 => {
                let params = op - 0x90;
                let position = (params % 8) as usize;
                if position == 6 {
                    let value = self.mmu.read_byte(self.reg.get_hl());
                    // self.reg.a = self.sub_byte(self.reg.a, value);
                    self.sub(value, false);
                    2
                } else {
                    let second_param = [&self.reg.b, &self.reg.c, &self.reg.d, &self.reg.e, &self.reg.h, &self.reg.l, &0, &self.reg.a]; 
                    let second_param_final = *second_param[position];
                    // self.reg.a = self.sub_byte(self.reg.a, second_param_final); 
                    self.sub(second_param_final, false);
                    1
                }
            }

            0x98..=0x9f => {
                let params = op - 0x98;
                let position = (params % 8) as usize;
                if position == 6 {
                    let value = self.mmu.read_byte(self.reg.get_hl());
                    // self.sbc(value);
                    self.sub(value, true);
                    2
                } else {
                    let second_param = [&self.reg.b, &self.reg.c, &self.reg.d, &self.reg.e, &self.reg.h, &self.reg.l, &0, &self.reg.a]; 
                    let second_param_final = *second_param[position];
                    // self.sbc(second_param_final); 
                    self.sub(second_param_final, true);
                    1
                }
            }

            0xa0..=0xa7 => {
                let params = op - 0xa0;
                let position = (params % 8) as usize;
                if position == 6 {
                    let value = self.mmu.read_byte(self.reg.get_hl());
                    self.and(value);
                    2
                } else {
                    let second_param = [&self.reg.b, &self.reg.c, &self.reg.d, &self.reg.e, &self.reg.h, &self.reg.l, &0, &self.reg.a]; 
                    let second_param_final = *second_param[position];
                    self.and(second_param_final); 
                    1
                }
            }

            0xa8..=0xaf => {
                let params = op - 0xa8;
                let position = (params % 8) as usize;
                if position == 6 {
                    let value = self.mmu.read_byte(self.reg.get_hl());
                    self.xor(value);
                    2
                } else {
                    let second_param = [&self.reg.b, &self.reg.c, &self.reg.d, &self.reg.e, &self.reg.h, &self.reg.l, &0, &self.reg.a]; 
                    let second_param_final = *second_param[position];
                    self.xor(second_param_final); 
                    1
                }
            }

            0xb0..=0xb7 => {
                let params = op - 0xb0;
                let position = (params % 8) as usize;
                if position == 6 {
                    let value = self.mmu.read_byte(self.reg.get_hl());
                    self.or(value);
                    2
                } else {
                    let second_param = [&self.reg.b, &self.reg.c, &self.reg.d, &self.reg.e, &self.reg.h, &self.reg.l, &0, &self.reg.a]; 
                    let second_param_final = *second_param[position];
                    self.or(second_param_final); 
                    1
                }
            }

            0xb8..=0xbf => {
                let params = op - 0xb8;
                let position = (params % 8) as usize;
                if position == 6 {
                    let value = self.mmu.read_byte(self.reg.get_hl());
                    self.cp(value);
                    2
                } else {
                    let second_param = [&self.reg.b, &self.reg.c, &self.reg.d, &self.reg.e, &self.reg.h, &self.reg.l, &0, &self.reg.a]; 
                    let second_param_final = *second_param[position];
                    self.cp(second_param_final); 
                    1
                }
            }
            0xc0 => {if !self.reg.get_flag(flags::Z) {self.ret(); 5} else {2}}
            0xc1 => { let v = self.pop(); self.reg.set_bc(v); 3 },
            // 0xc2 => {if !self.reg.get_flag(flags::Z) {self.pc = self.fetch_word(); 4} else {3}}
            0xc2 => { if !self.reg.get_flag(flags::Z) { self.pc = self.fetch_word(); 4 } else { self.pc += 2; 3 } },
            0xc3 => {self.pc = self.fetch_word(); 4}
            0xc4 => {if !self.reg.get_flag(flags::Z) { self.push(self.pc + 2); self.pc = self.fetch_word(); 6} else {self.pc += 2; 3}}
            0xc5 => {self.push(self.reg.get_bc()); 4}
            0xc6 => {let v = self.fetch_byte(); self.add_byte(v, false); 2}
            0xc7 => {self.call(0x00); 4}
            0xc8 => {if self.reg.get_flag(flags::Z) {self.ret(); 5} else {2}}
            0xc9 => {self.ret(); 4}
            0xca => { if self.reg.get_flag(flags::Z) { self.pc = self.fetch_word(); 4 } else { self.pc += 2; 3 } },
            
            0xcc => {if self.reg.get_flag(flags::Z) { self.push(self.pc + 2); self.pc = self.fetch_word(); 6} else {self.pc += 2; 3}}
            0xcd => {self.push(self.pc + 2); self.pc = self.fetch_word(); 6}
            0xce => {let v = self.fetch_byte(); self.adc(v); 2}
            0xcf => {self.call(0x08); 4}
            0xd0 => {if !self.reg.get_flag(flags::C) {self.ret(); 5} else {2}}
            0xd1 => { let v = self.pop(); self.reg.set_de(v); 3 },
            0xd2 => { if !self.reg.get_flag(flags::C) { self.pc = self.fetch_word(); 4 } else { self.pc += 2; 3 } },
            0xd4 => {if !self.reg.get_flag(flags::C) { self.push(self.pc + 2); self.pc = self.fetch_word(); 6} else {self.pc += 2; 3}}
            0xd5 => {self.push(self.reg.get_de()); 4}
            // 0xd6 => {let v = self.fetch_byte(); self.reg.a = self.sub_byte(self.reg.a, v); 2}
            0xd6 => { let v = self.fetch_byte(); self.sub(v, false); 2 },
            0xd7 => {self.call(0x10); 4}
            0xd8 => {if self.reg.get_flag(flags::C) {self.ret(); 5} else {2}}
            0xd9 => {self.reti(); 4}
            0xda => { if self.reg.get_flag(flags::C) { self.pc = self.fetch_word(); 4 } else { self.pc += 2; 3 } },

            0xdc => {if self.reg.get_flag(flags::C) { self.push(self.pc + 2); self.pc = self.fetch_word(); 6} else {self.pc += 2; 3}}

            // 0xde => {let v = self.fetch_byte(); self.sbc(v); 2}
            0xde => { let v = self.fetch_byte(); self.sub(v, true); 2 },
            0xdf => {self.call(0x18); 4}

            0xe0 => {let v =  0xff00 | self.fetch_byte() as u16; self.mmu.write_byte(v, self.reg.a); 3}
            0xe1 => { let v = self.pop(); self.reg.set_hl(v); 3 },
            0xe2 => {self.mmu.write_byte((0xff00 + (self.reg.c as u16)) as u16, self.reg.a); 2}

            0xe5 => {self.push(self.reg.get_hl()); 4}
            0xe6 => {let v = self.fetch_byte(); self.and(v); 2}
            0xe7 => {self.call(0x20); 4}
            0xe8 => { self.sp = self.add16imm(self.sp); 4 },
            0xe9 => {self.pc = self.reg.get_hl(); 1}
            0xea => {let pointer = self.fetch_word(); self.mmu.write_byte(pointer, self.reg.a); 4}

            0xee => {let v = self.fetch_byte(); self.xor(v); 2}
            0xef => {self.call(0x28); 4}

            // 0xf0 => {let v = self.fetch_byte() as u16; self.reg.a = self.mmu.read_byte(0xff00 + v); 3}
            0xf0 => {let v = 0xFF00 | self.fetch_byte() as u16; self.reg.a = self.mmu.read_byte(v); 3 }
            // 0xf1 => { // This pop is slightly different.
            //     let v = self.pop(); self.reg.set_af(v); 
            //     // self.reg.set_flag(flags::Z, (v >> 6) & 0b1 == 1);
            //     // self.reg.set_flag(flags::N, (v >> 5) & 0b1 == 1);
            //     // self.reg.set_flag(flags::H, (v >> 4) & 0b1 == 1);
            //     // self.reg.set_flag(flags::C, (v >> 3) & 0b1 == 1);
            //     3
            // }
            0xf1 => { let v = self.pop() & 0xFFF0; self.reg.set_af(v); 3 },
            0xf2 => {let v = self.reg.c as u16; self.reg.a = self.mmu.read_byte(0xff00 + v); 2}
            0xf3 => {self.di(); 1}
            
            0xf5 => {self.push(self.reg.get_af()); 4}
            0xf6 => {let v = self.fetch_byte(); self.or(v); 2}
            0xf7 => { self.push(self.pc); self.pc = 0x30; 4 },
            0xf8 => { let r = self.add16imm(self.sp); self.reg.set_hl(r); 3 },
            0xf9 => {self.sp = self.reg.get_hl(); 2}
            0xfa => {let pointer = self.fetch_word(); self.reg.a = self.mmu.read_byte(pointer); 4}
            0xfb => {self.ei(); 1}

            0xfe => {let v = self.fetch_byte(); self.cp(v); 2}
            0xff => {self.call(0x38); 4}

            0xcb => {
                let op = self.fetch_byte();
                let timing = match op { 
                    0x00..=0x0f => {
                        let params = op;
                        if (params % 8) == 6 || (params % 8) == 0xe {
                            let value;
                            if params == 6 {
                                value = self.rlc(self.mmu.read_byte(self.reg.get_hl()));
                            } else {
                                value = self.rrc(self.mmu.read_byte(self.reg.get_hl()));
                            }
                            self.mmu.write_byte(self.reg.get_hl(), value);
                            4
                        } else {
                            let second_param = [&self.reg.b, &self.reg.c, &self.reg.d, &self.reg.e, &self.reg.h, &self.reg.l, &0, &self.reg.a]; 
                            let position = (params % 8) as usize;
    
                            let second_param_final = *second_param[position];
                            let value;
                            if params < 8 {
                                value = self.rlc(second_param_final);
                            } else {
                                value = self.rrc(second_param_final);
                            }
                            // println!("{} {} {}", second_param_final, value, position);

                            let second_param_mut = [&mut self.reg.b, &mut self.reg.c, &mut self.reg.d, &mut self.reg.e, &mut self.reg.h, &mut self.reg.l, &mut 0, &mut self.reg.a];
                            *second_param_mut[position] = value; 
                            2
                        }
                    }
                    0x10..=0x1f => {
                        let params = op - 0x10;
                        if (params % 8) == 6 || (params % 8) == 0xe {
                            let value;
                            if params == 6 {
                                value = self.rl(self.mmu.read_byte(self.reg.get_hl()));
                            } else {
                                value = self.rr(self.mmu.read_byte(self.reg.get_hl()));
                            }
                            self.mmu.write_byte(self.reg.get_hl(), value);
                            4
                        } else {
                            let second_param = [&self.reg.b, &self.reg.c, &self.reg.d, &self.reg.e, &self.reg.h, &self.reg.l, &0, &self.reg.a]; 
                            let position = (params % 8) as usize;
    
                            let second_param_final = *second_param[position];
                            
                            let value;
                            if params < 8 {
                                value = self.rl(second_param_final);
                            } else {
                                value = self.rr(second_param_final);
                            }
                            // let d = self.reg.d;
                            // println!("Set set, {:#04x} {:#04x} {:#04x} {:#04x} {:#04x}", params, position, d, second_param_final, self.rr(second_param_final));
    
                            let second_param_mut = [&mut self.reg.b, &mut self.reg.c, &mut self.reg.d, &mut self.reg.e, &mut self.reg.h, &mut self.reg.l, &mut 0, &mut self.reg.a];
                            *second_param_mut[position] = value; 
                            2
                        }
                    }
                    0x20..=0x2f => {
                        let params = op - 0x20;
                        if (params % 8) == 6 || (params % 8) == 0xe {
                            let value;
                            if params == 6 {
                                value = self.sla(self.mmu.read_byte(self.reg.get_hl()));
                            } else {
                                value = self.sra(self.mmu.read_byte(self.reg.get_hl()));
                            }
                            self.mmu.write_byte(self.reg.get_hl(), value);
                            4
                        } else {
                            let second_param = [&self.reg.b, &self.reg.c, &self.reg.d, &self.reg.e, &self.reg.h, &self.reg.l, &0, &self.reg.a]; 
                            let position = (params % 8) as usize;
    
                            let second_param_final = *second_param[position];
                            let value;
                            if params < 8 {
                                value = self.sla(second_param_final);
                            } else {
                                value = self.sra(second_param_final);
                            }
    
                            let second_param_mut = [&mut self.reg.b, &mut self.reg.c, &mut self.reg.d, &mut self.reg.e, &mut self.reg.h, &mut self.reg.l, &mut 0, &mut self.reg.a];
                            *second_param_mut[position] = value; 
                            2
                        }
                    }
                    0x30..=0x3f => {
                        let params = op - 0x30;
                        if (params % 8) == 6 || (params % 8) == 0xe {
                            let value;
                            if params == 6 {
                                value = self.swap(self.mmu.read_byte(self.reg.get_hl()));
                            } else {
                                value = self.srl(self.mmu.read_byte(self.reg.get_hl()));
                            }
                            self.mmu.write_byte(self.reg.get_hl(), value);
                            4
                        } else {
                            let second_param = [&self.reg.b, &self.reg.c, &self.reg.d, &self.reg.e, &self.reg.h, &self.reg.l, &0, &self.reg.a]; 
                            let position = (params % 8) as usize;
    
                            let second_param_final = *second_param[position];
                            let value;
                            if params < 8 {
                                value = self.swap(second_param_final);
                            } else {
                                value = self.srl(second_param_final);
                            }
    
                            let second_param_mut = [&mut self.reg.b, &mut self.reg.c, &mut self.reg.d, &mut self.reg.e, &mut self.reg.h, &mut self.reg.l, &mut 0, &mut self.reg.a];
                            *second_param_mut[position] = value; 
                            2
                        }
                    }
                    0x40..=0x7f => { // TODO: All of this code is copied straight from the block below, remember to change this when fixes are made.
                        let params = op - 0x40;
                        let first_param = params / 8;
                       
                        // println!("{:#04x} to set {} ", params, first_param);
                        if (params % 8) == 6 {
                            // println!("{} {}", first_param, op);
                            self.bit(self.mmu.read_byte(self.reg.get_hl()), first_param);
                            3
                        } else {
                            let second_param = [&self.reg.b, &self.reg.c, &self.reg.d, &self.reg.e, &self.reg.h, &self.reg.l, &0, &self.reg.a]; 
                            let position = (params % 8) as usize;
    
                            let second_param_final = *second_param[position];
                            // println!("{} {} {}", second_param_final, first_param, op);
                            self.bit(second_param_final, first_param);
                            2
                        }
                    }

                    0x80..=0xbf => { // TODO: All of this code is copied straight from the block below, remember to change this when fixes are made.
                        let params = op - 0x80;
                        let first_param = params / 8;
                       
                        // println!("{:#04x} to set {} ", params, first_param);
                        if (params % 8) == 6 {
                            let value = self.res(first_param, self.mmu.read_byte(self.reg.get_hl()));
                            // println!("val: {:#04x} orig: {:#04x} {}", value, self.mmu.read_byte(self.reg.get_hl()), first_param);
                            self.mmu.write_byte(self.reg.get_hl(), value);
                            4
                        } else {
                            let second_param = [&self.reg.b, &self.reg.c, &self.reg.d, &self.reg.e, &self.reg.h, &self.reg.l, &0, &self.reg.a]; 
                            let position = (params % 8) as usize;
    
                            let second_param_final = *second_param[position];
                            let value = self.res(first_param, second_param_final);
    
                            let second_param_mut = [&mut self.reg.b, &mut self.reg.c, &mut self.reg.d, &mut self.reg.e, &mut self.reg.h, &mut self.reg.l, &mut 0, &mut self.reg.a];
                            *second_param_mut[position] = value; 
                            2
                        }
                    }

                    0xc0..=0xff => { // Implement range of opcodes from c0 to ff (they're all set instructions)
                        // TODO: This might be really messy code, see if there is a way to improve it.

                        let params = op - 0xc0;
                        let first_param = params / 8;
                       
                        // println!("{:#04x} to set {} ", params, first_param);
                        if (params % 8) == 6 || (params % 8) == 0xe {
                            let value = self.set(first_param, self.mmu.read_byte(self.reg.get_hl()));
                            // println!("val: {:#04x} orig: {:#04x} {}", value, self.mmu.read_byte(self.reg.get_hl()), first_param);
                            self.mmu.write_byte(self.reg.get_hl(), value);
                            4
                        } else {
                            let second_param = [&self.reg.b, &self.reg.c, &self.reg.d, &self.reg.e, &self.reg.h, &self.reg.l, &0, &self.reg.a]; 
                            let position = (params % 8) as usize;
    
                            let second_param_final = *second_param[position];
                            let value = self.set(first_param, second_param_final);
    
                            let second_param_mut = [&mut self.reg.b, &mut self.reg.c, &mut self.reg.d, &mut self.reg.e, &mut self.reg.h, &mut self.reg.l, &mut 0, &mut self.reg.a];
                            *second_param_mut[position] = value; 
                            2
                        }
                    }
                    _ => unimplemented!("Unimplemented CB prefixed opcode: {:#04x}", op)
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
        self.alu_add(val, true);
    }

    fn alu_add(&mut self, b: u8, usec: bool) {
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

    // fn daa(&mut self, hex: u8) -> u8 {
    //     let mut high = hex & 0xF0;
    //     let mut low = hex & 0x0F;
    //     self.reg.set_flag(flags::H, false);
    //     if low > 9 {
    //         high += low - 9;
    //         low -= 9;
    //         self.reg.set_flag(flags::C, true);
    //         self.reg.set_flag(flags::Z, (high & low) == 0);
    //         return high & low;
    //     } else {
    //         self.reg.set_flag(flags::C, false);
    //         self.reg.set_flag(flags::Z, hex == 0);
    //         return hex;
    //     }
    // }
    fn daa(&mut self) {
        let mut a = self.reg.a;
        let mut adjust = if self.reg.get_flag(flags::C) { 0x60 } else { 0x00 };
        if self.reg.get_flag(flags::H) { adjust |= 0x06; };
        if !self.reg.get_flag(flags::N) {
            if a & 0x0F > 0x09 { adjust |= 0x06; };
            if a > 0x99 { adjust |= 0x60; };
            a = a.wrapping_add(adjust);
        } else {
            a = a.wrapping_sub(adjust);
        }

        self.reg.set_flag(flags::C, adjust >= 0x60);
        self.reg.set_flag(flags::H, false);
        self.reg.set_flag(flags::Z, a == 0);
        self.reg.a = a;
    }

    fn cp(&mut self, val: u8) {
        let r = self.reg.a;
        self.alu_sub(val, false);
        self.reg.a = r;
    }
    fn alu_sub(&mut self, b: u8, usec: bool) {
        let c = if usec && self.reg.get_flag(flags::C) { 1 } else { 0 };
        let a = self.reg.a;
        let r = a.wrapping_sub(b).wrapping_sub(c);
        self.reg.set_flag(flags::Z, r == 0);
        self.reg.set_flag(flags::H, (a & 0x0F) < (b & 0x0F) + c);
        self.reg.set_flag(flags::N, true);
        self.reg.set_flag(flags::C, (a as u16) < (b as u16) + (c as u16));
        self.reg.a = r;
    }
    fn add16imm(&mut self, a: u16) -> u16 { 
        let b = self.fetch_byte() as i8 as i16 as u16;
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::Z, false);
        self.reg.set_flag(flags::H, (a & 0x000F) + (b & 0x000F) > 0x000F);
        self.reg.set_flag(flags::C, (a & 0x00FF) + (b & 0x00FF) > 0x00FF);
        return a.wrapping_add(b)
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

    fn alu_srflagupdate(&mut self, r: u8, c: bool) {
        self.reg.set_flag(flags::H, false);
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::Z, r == 0);
        self.reg.set_flag(flags::C, c);
    }

    fn rlc(&mut self, a: u8) -> u8 {
        let c = a & 0x80 == 0x80;
        let r = (a << 1) | (if c { 1 } else { 0 });
        self.alu_srflagupdate(r, c);
        return r
    }

    fn rl(&mut self, a: u8) -> u8 {
        let c = a & 0x80 == 0x80;
        let r = (a << 1) | (if self.reg.get_flag(flags::C) { 1 } else { 0 });
        self.alu_srflagupdate(r, c);
        return r
    }

    fn rr(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = (a >> 1) | (if self.reg.get_flag(flags::C) { 0x80 } else { 0 });
        self.alu_srflagupdate(r, c);
        return r
    }

    fn rrc(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = (a >> 1) | (if c { 0x80 } else { 0 });
        self.alu_srflagupdate(r, c);
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


    fn srl(&mut self, val: u8) -> u8 { // Shift right logically
        let res = val >> 1;
        let carry = (val & 0b1) == 1;
        self.reg.unset_flags();
        self.reg.set_flag(flags::Z, res == 0);
        self.reg.set_flag(flags::C, carry);
        res
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

    fn add16(&mut self, b: u16) {
        let a = self.reg.get_hl();
        let r = a.wrapping_add(b);
        self.reg.set_flag(flags::H, (a & 0x07FF) + (b & 0x07FF) > 0x07FF);
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::C, a > 0xFFFF - b);
        self.reg.set_hl(r);
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