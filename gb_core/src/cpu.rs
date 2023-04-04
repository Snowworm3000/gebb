mod registers;
use core::panic;

use registers::*;
mod mmu;
use mmu::*;

const LOG_LEVEL: usize = 2;

pub struct Cpu {
    reg: Registers,
    pc: u16,
    sp: u16,
    ime: bool,
    tempIme: bool,
    pub mmu: MMU,
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
            pc: 0x100,
            sp: 0xfffe,
            ime: false,
            tempIme: false,
            mmu: MMU::new(),
            cycle: 0,
            line: 0,
            debug_file: Vec::new(),
            halted: false,
            setdi: 0,
            setei: 0,
        };

        temp
    }
    pub fn reset(&mut self) {
        self.reg = Registers::new_default();
        self.pc = 0x100;
        self.sp = 0xfffe;
        self.ime = false;
        self.cycle = 0;
    }

    pub fn load(&mut self, data: &[u8]) {
        self.mmu.load(data);
    }
    
    pub fn get_display(&self) -> &[u8] {
        &self.mmu.ppu.screen_buffer.as_ref()
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

    fn fetch_byte(&mut self) -> u8 {
        let byte = self.mmu.read_byte(self.pc);
        self.pc += 1;
        byte
    }

    fn fetch_word(&mut self) -> u16 {
        let word = self.mmu.read_word(self.pc);
        self.pc += 2;
        word
    }

    pub fn ppu_updated(&mut self) -> bool {
        let result = self.mmu.ppu.updated;
        self.mmu.ppu.updated = false;
        result
    }

    fn execute(&mut self, op: u8) -> u32{
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
            0x07 => { self.reg.a = self.rlc(self.reg.a); self.reg.set_flag(flags::Z, false); 1 },
            0x08 => {let word = self.fetch_word(); self.mmu.write_word(word, self.sp); 5}
            0x09 => {let res = self.add_word(self.reg.get_hl(), self.reg.get_bc()); self.reg.set_hl(res); 2}
            0x0a => {self.reg.a = self.mmu.read_byte(self.reg.get_bc()); 2}
            0x0b => {self.reg.set_bc(self.reg.get_bc().wrapping_sub(1)); 2}
            0x0c => {self.reg.c = self.inc(self.reg.c); 1}
            0x0d => {self.reg.c = self.dec(self.reg.c); 1}
            0x0e => {self.reg.c = self.fetch_byte(); 2}
            0x0f => { self.reg.a = self.rrc(self.reg.a); self.reg.set_flag(flags::Z, false); 1 },
            0x10 => { 1 }
            0x11 => {let word = self.fetch_word(); self.reg.set_de(word); 3}
            0x12 => {self.mmu.write_byte(self.reg.get_de(), self.reg.a); 2}
            0x13 => {self.reg.set_de(self.reg.get_de().wrapping_add(1)); 2}
            0x14 => {self.reg.d = self.inc(self.reg.d); 1}
            0x15 => {self.reg.d = self.dec(self.reg.d); 1}
            0x16 => {self.reg.d = self.fetch_byte(); 2}
            0x17 => { self.reg.a = self.rl(self.reg.a); self.reg.set_flag(flags::Z, false); 1 },
            0x18 => {self.jr(); 3}
            0x19 => {let res = self.add_word(self.reg.get_hl(), self.reg.get_de()); self.reg.set_hl(res); 2}
            0x1a => {self.reg.a = self.mmu.read_byte(self.reg.get_de()); 2}
            0x1b => { self.reg.set_de(self.reg.get_de().wrapping_sub(1)); 2 },
            0x1c => {self.reg.e = self.inc(self.reg.e); 1}
            0x1d => {self.reg.e = self.dec(self.reg.e); 1}
            0x1e => {self.reg.e = self.fetch_byte(); 2}
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
                if position == 6 || position == 0xe {
                    let value = self.mmu.read_byte(self.reg.get_hl());
                    if first_param == 6 {
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
            0xd6 => { let v = self.fetch_byte(); self.sub(v, false); 2 },
            0xd7 => {self.call(0x10); 4}
            0xd8 => {if self.reg.get_flag(flags::C) {self.ret(); 5} else {2}}
            0xd9 => {self.reti(); 4}
            0xda => { if self.reg.get_flag(flags::C) { self.pc = self.fetch_word(); 4 } else { self.pc += 2; 3 } },

            0xdc => {if self.reg.get_flag(flags::C) { self.push(self.pc + 2); self.pc = self.fetch_word(); 6} else {self.pc += 2; 3}}
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
            0xf0 => {let v = 0xFF00 | self.fetch_byte() as u16; self.reg.a = self.mmu.read_byte(v); 3 }
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
                       
                        if (params % 8) == 6 {
                            self.bit(self.mmu.read_byte(self.reg.get_hl()), first_param);
                            3
                        } else {
                            let second_param = [&self.reg.b, &self.reg.c, &self.reg.d, &self.reg.e, &self.reg.h, &self.reg.l, &0, &self.reg.a]; 
                            let position = (params % 8) as usize;
    
                            let second_param_final = *second_param[position];
                            self.bit(second_param_final, first_param);
                            2
                        }
                    }

                    0x80..=0xbf => { // TODO: All of this code is copied straight from the block below, remember to change this when fixes are made.
                        let params = op - 0x80;
                        let first_param = params / 8;
                       
                        if (params % 8) == 6 {
                            let value = self.res(first_param, self.mmu.read_byte(self.reg.get_hl()));
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
                        let params = op - 0xc0;
                        let first_param = params / 8;
                        if (params % 8) == 6 || (params % 8) == 0xe {
                            let value = self.set(first_param, self.mmu.read_byte(self.reg.get_hl()));
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

    fn adc(&mut self, val: u8){
        self.alu_add(val, true);
    }

    fn alu_add(&mut self, b: u8, usec: bool) {
        let carry = if usec && self.reg.get_flag(flags::C) { 1 } else { 0 };
        let a = self.reg.a;
        let res = a.wrapping_add(b).wrapping_add(carry);
        self.reg.set_flag(flags::Z, res == 0);
        self.reg.set_flag(flags::H, (a & 0xF) + (b & 0xF) + carry > 0xF);
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::C, (a as u16) + (b as u16) + (carry as u16) > 0xFF);
        self.reg.a = res;
    }

    fn daa(&mut self) {
        let carry = self.reg.get_flag(flags::C);
        let halfcarry = self.reg.get_flag(flags::H);

        if !self.reg.get_flag(flags::N) {
            let mut correction = 0;
            if halfcarry || (self.reg.a & 0xf > 0x9) {
                correction |= 0x6;
            }

            if carry || (self.reg.a > 0x99) {
                correction |= 0x60;
                self.reg.set_flag(flags::C, true);
            }

            self.reg.a = self.reg.a.wrapping_add(correction);
        } else if carry {
            self.reg.set_flag(flags::C, true);
            self.reg.a = self.reg.a.wrapping_add(if halfcarry { 0x9a } else { 0xa0 });
        } else if halfcarry {
            self.reg.a = self.reg.a.wrapping_add(0xfa);
        }

        self.reg.set_flag(flags::Z, self.reg.a == 0);
        self.reg.set_flag(flags::H, false);
    }

    fn cp(&mut self, val: u8) {
        let temp = self.reg.a;
        self.alu_sub(val, false);
        self.reg.a = temp;
    }
    fn alu_sub(&mut self, b: u8, usec: bool) {
        let carry = if usec && self.reg.get_flag(flags::C) { 1 } else { 0 };
        let a = self.reg.a;
        let res = a.wrapping_sub(b).wrapping_sub(carry);
        self.reg.set_flag(flags::Z, res == 0);
        self.reg.set_flag(flags::H, (a & 0x0F) < (b & 0x0F) + carry);
        self.reg.set_flag(flags::N, true);
        self.reg.set_flag(flags::C, (a as u16) < (b as u16) + (carry as u16));
        self.reg.a = res;
    }
    fn add16imm(&mut self, a: u16) -> u16 { 
        let b = self.fetch_byte() as i8 as i16 as u16;
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::Z, false);
        self.reg.set_flag(flags::H, (a & 0x000F) + (b & 0x000F) > 0x000F);
        self.reg.set_flag(flags::C, (a & 0x00FF) + (b & 0x00FF) > 0x00FF);
        return a.wrapping_add(b)
    }

    fn add_byte(&mut self, b: u8, usec: bool) { 
        let carry = if usec && self.reg.get_flag(flags::C) { 1 } else { 0 };
        let a = self.reg.a;
        let res = a.wrapping_add(b).wrapping_add(carry);
        self.reg.set_flag(flags::Z, res == 0);
        self.reg.set_flag(flags::H, (a & 0xF) + (b & 0xF) + carry > 0xF);
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::C, (a as u16) + (b as u16) + (carry as u16) > 0xFF);
        self.reg.a = res;
    }

    fn add_word(&mut self, a: u16, b: u16) -> u16 { 
        let (result, carry) = a.overflowing_add(b);
        self.reg.set_flag(flags::C, carry);
        // self.reg.set_flag(flags::H ,((self.reg.b as u16 + self.reg.c as u16) & 0xFF00) != 0);
        self.reg.set_flag(flags::H ,(a & 0x07FF) + (b & 0x07FF) > 0x07FF);
        self.reg.set_flag(flags::N, false);
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

    fn alu_srflagupdate(&mut self, r: u8, c: bool) {
        self.reg.set_flag(flags::H, false);
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::Z, r == 0);
        self.reg.set_flag(flags::C, c);
    }

    fn rlc(&mut self, a: u8) -> u8 {
        let carry = a & 0x80 == 0x80;
        let res = (a << 1) | (if carry { 1 } else { 0 });
        self.alu_srflagupdate(res, carry);
        return res
    }

    fn rl(&mut self, a: u8) -> u8 {
        let carry = a & 0x80 == 0x80;
        let res = (a << 1) | (if self.reg.get_flag(flags::C) { 1 } else { 0 });
        self.alu_srflagupdate(res, carry);
        return res
    }

    fn rr(&mut self, a: u8) -> u8 {
        let carry = a & 0x01 == 0x01;
        let res = (a >> 1) | (if self.reg.get_flag(flags::C) { 0x80 } else { 0 });
        self.alu_srflagupdate(res, carry);
        return res
    }

    fn rrc(&mut self, a: u8) -> u8 {
        let carry = a & 0x01 == 0x01;
        let res = (a >> 1) | (if carry { 0x80 } else { 0 });
        self.alu_srflagupdate(res, carry);
        return res
    }

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
        let carry = if usec && self.reg.get_flag(flags::C) { 1 } else { 0 };
        let a = self.reg.a;
        let res = a.wrapping_sub(b).wrapping_sub(carry);
        self.reg.set_flag(flags::Z, res == 0);
        self.reg.set_flag(flags::H, (a & 0x0F) < (b & 0x0F) + carry);
        self.reg.set_flag(flags::N, true);
        self.reg.set_flag(flags::C, (a as u16) < (b as u16) + (carry as u16));
        self.reg.a = res;
    }


    fn jr(&mut self) {
        let offset = self.fetch_byte() as i8;
        self.pc = ((self.pc as u32 as i32) + (offset as i32)) as u16;
    }

    fn ret(&mut self) {
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

    fn pop(&mut self) -> u16 { 
        self.sp = self.sp + 2;
        self.mmu.read_word(self.sp - 2) // rr = popped value
    }

    fn sla(&mut self, a: u8) -> u8 {
        let carry = a & 0x80 == 0x80;
        let res = a << 1;
        self.alu_srflagupdate(res, carry);
        return res
    }

    fn sra(&mut self, a: u8) -> u8 {
        let carry = a & 0x01 == 0x01;
        let res = (a >> 1) | (a & 0x80);
        self.alu_srflagupdate(res, carry);
        return res
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

    fn bit(&mut self, a: u8, b: u8) {
        let res = a & (1 << (b as u32)) == 0; 
        self.reg.set_flag(flags::N, false);
        self.reg.set_flag(flags::H, true);
        self.reg.set_flag(flags::Z, res);
    }

    fn res(&self, position: u8, val: u8) -> u8 { 
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


}