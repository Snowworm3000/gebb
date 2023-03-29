const VRAM_SIZE: usize = 0x4000;
const VOAM_SIZE: usize = 0xA0;

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

struct LCDC { //LCD Control registers
    original: u8,
    lcd_enable: bool,
    w_tile_map_area: bool,
    window_enable: bool,
    bg_and_win_tile_data_area: bool,
    bg_tile_map_area: bool,
    obj_size: bool,
    obj_enable: bool,
    bg_and_window_display: bool,
}

fn rbit(data: u8, pos: u8) -> bool { //Read bit
    if ((data >> pos) & 0b1) == 1 {true} else {false} 
}

fn wbit (data: u8, pos: u8, bit: bool) -> u8 { //Write bit
    data & !(1 << pos) | (u8::from(bit) << pos)
}

fn bn(bool: bool) -> u8 { // Bool to number
    if bool {1} else {0}
}

impl LCDC { //LCD Control registers
    
    fn new(lcdc: u8) -> Self {
        Self {
            original: lcdc,
            lcd_enable: rbit(lcdc, 7),
            w_tile_map_area: rbit(lcdc, 6),
            window_enable: rbit(lcdc, 5),
            bg_and_win_tile_data_area: rbit(lcdc, 4),
            bg_tile_map_area: rbit(lcdc, 3),
            obj_size: rbit(lcdc, 2),
            obj_enable: rbit(lcdc, 1),
            bg_and_window_display: rbit(lcdc, 0),
        }
    }
    
    fn raw(&self) -> u8 {
        (bn(self.lcd_enable) << 7) 
        & (bn(self.w_tile_map_area) << 6)
        & (bn(self.window_enable) << 5)
        & (bn(self.bg_and_win_tile_data_area) << 4) 
        & (bn(self.bg_tile_map_area) << 3)
        & (bn(self.obj_size) << 2)
        & (bn(self.obj_enable) << 1)
        & (bn(self.bg_and_window_display))
    }
}

pub struct LCDS { //LCD Status registers
    pub ly: u8,
    lyc: u8,
    stat: STAT,

}

impl LCDS {
    fn new() -> Self {
        Self {
            ly: 0,
            lyc: 0,
            stat: STAT::new(0),
        }
    }
}

struct STAT {
    original: u8,
    lcy_eq_ly_interrupt: bool,
    mode2: bool,
    mode1: bool,
    mode0: bool,
    lcy_eq_ly_flag: bool,
    mode_flag: u8,
}

impl STAT {
    fn new(stat: u8) -> Self {
        Self {
            original: stat,
            lcy_eq_ly_interrupt: rbit(stat, 6),
            mode2: rbit(stat, 5),
            mode1: rbit(stat, 4),
            mode0: rbit(stat, 3),
            lcy_eq_ly_flag: rbit(stat, 2),
            mode_flag: (stat & 0b11)
        }
    }

    fn raw(&self) -> u8 {
        (bn(self.lcy_eq_ly_interrupt) << 6)
        & (bn(self.mode2) << 5)
        & (bn(self.mode1) << 4)
        & (bn(self.mode0) << 3)
        & (bn(self.lcy_eq_ly_flag) << 2)
        & (self.mode_flag)
    }
}

pub struct PPU {
    vram: [u8; VRAM_SIZE],
    voam: [u8; VOAM_SIZE],
    tile_map: [u8; 32 * 32 * 8],
    lcdc: LCDC,
    pub lcds: LCDS,
    scy: u8,
    scx: u8,
    wy: u8,
    wx: u8,
    bgp: u8, // BG palette data
    x: u8, // Number of pixels along the scanline.

    pub updated: bool,
    pub interrupt: u8,
    pub data: Vec<u8>,
    pub modeclock: u32,
    // pub line: u8,
    hblank: bool,
    mode: Mode,

    framebuffer: [u8; 160 * 144],
    framebufferW: [u8; 160 * 144],
    framebufferO: [u8; 160 * 144],
    


    palbr: u8,
    pal0r: u8,
    pal1r: u8,
    palb: [u8; 4],
    pal0: [u8; 4],
    pal1: [u8; 4],
    tilebase: u16,
}

#[derive(PartialEq, Eq)]
enum Mode {
    HBlank, //0
    VBlank, //1
    OAMSearch, //2
    PixelTransfer, //3
}

impl PPU {
    pub fn new() -> Self {
        Self {
            vram: [0; VRAM_SIZE],
            voam: [0; VOAM_SIZE],
            tile_map: [0; 32 * 32 * 8],
            lcdc: LCDC::new(0),
            lcds: LCDS::new(),
            scy: 0,
            scx: 0,
            wy: 0,
            wx: 0,
            bgp: 0,
            x: 0,

            updated: false,
            interrupt: 0,
            data: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT * 3],
            modeclock: 0,
            // line: 0,
            hblank: false,
            mode: Mode::HBlank,

            framebuffer: [0; 160 * 144],
            framebufferW: [0; 160 * 144],
            framebufferO: [0; 160 * 144],



            palbr: 0,
            pal0r: 0,
            pal1r: 1,
            palb: [0; 4],
            pal0: [0; 4],
            pal1: [0; 4],
            tilebase: 0x8000,
        }
    }

    pub fn read_byte(&self, loc: u16) -> u8 {
        match loc {
            0x8000..=0x97ff => {self.vram[(loc - 0x8000) as usize]},
            0xfe00 ..= 0xfe9f => self.voam[loc as usize - 0xfe00],
            0xff40 => {let r = self.lcdc.raw(); r}
            0xff41 => {self.lcds.stat.raw()}
            0xff42 => {self.scy}
            0xff43 => {self.scx}
            0xff44 => {self.lcds.ly}
            0xff45 => {self.lcds.lyc}
            0xff47 => {self.bgp}
            0xff4a => {self.wy}
            0xff4b => {self.wx}
            // _ => {unimplemented!("Read location not implemented! {:#04x}", loc)}
            _ => 0xFF
        }
    }
    pub fn write_byte(&mut self, loc: u16, data: u8) {
        match loc {
            0x8000..=0x97ff => {self.vram[(loc - 0x8000) as usize] = data},
            0xfe00 ..= 0xfe9f => {self.voam[loc as usize - 0xfe00] = data},
            // 0xff40 => {self.lcdc = LCDC::new(data)}
            0xff40 => {
                let orig = self.lcdc.lcd_enable;
                self.lcdc = LCDC::new(data);
                if orig && !self.lcdc.lcd_enable {
                    self.modeclock = 0;
                    self.lcds.ly = 0; 
                    self.mode = Mode::HBlank;
                    self.clear_screen();
                }
            }
            0xff41 => {if (self.lcds.stat.raw() & 0b111) == (data & 0b111) {self.lcds.stat = STAT::new(data)} else {panic!("Read only")}} // Check bits 0, 1, and 2 haven't been written to because they are read only.
            0xff42 => {self.scy = data}
            0xff43 => {self.scx = data}
            0xff44 => {panic!("Read only")}
            0xff45 => {self.lcds.lyc = data}
            0xff47 => {self.bgp = data}
            0xff4a => {self.wy = data}
            0xff4b => {self.wx = data}
            // _ => {unimplemented!("Write location not implemented! {}", loc)}
            _ => {}
        }
    }

    // pub fn cycle(&mut self, ticks: u32) {
    //     self.lcds.stat.lcy_eq_ly_flag = self.lcds.ly == self.lcds.lyc;


    // }
    fn check_interrupt_lyc(&mut self) {
        if self.lcds.stat.lcy_eq_ly_interrupt && self.lcds.ly == self.lcds.lyc {
            self.interrupt |= 0x02;
        }
    }

    pub fn do_cycle(&mut self, ticks: u32) {
        if !self.lcdc.lcd_enable { return }
        self.hblank = false;

        let mut ticksleft = ticks;

        while ticksleft > 0 {
            let curticks = if ticksleft >= 80 { 80 } else { ticksleft };
            self.modeclock += curticks;
            ticksleft -= curticks;

            // Full line takes 114 ticks
            if self.modeclock >= 456 {
                self.modeclock -= 456;
                self.lcds.ly = (self.lcds.ly + 1) % 154;
                self.check_interrupt_lyc();

                // This is a VBlank line
                if self.lcds.ly >= 144 && self.mode != Mode::VBlank {
                    self.change_mode(Mode::VBlank);
                }
            }

            // This is a normal line
            if self.lcds.ly < 144 {
                if self.modeclock <= 80 {
                    if self.mode != Mode::OAMSearch { self.change_mode(Mode::OAMSearch); }
                } else if self.modeclock <= (80 + 172) { // 252 cycles
                    if self.mode != Mode::PixelTransfer { self.change_mode(Mode::PixelTransfer); }
                } else { // the remaining 204
                    if self.mode != Mode::HBlank { self.change_mode(Mode::HBlank); }
                }
            }
        }
    }

    fn clear_screen(&mut self) {
        for v in self.data.iter_mut() {
            *v = 255;
        }
        self.updated = true;
    }
 
    // pub fn do_cycle(&mut self, ticks: u32) {
    //     if self.lcdc.lcd_enable == false {
    //         return
    //     }
    //     self.lcds.stat.lcy_eq_ly_flag = self.lcds.ly == self.lcds.lyc;

    //     if ticks == 40 {
    //         self.change_mode(Mode::OAMSearch);
            
    //     }

    //     self.x += 1;
    //     if self.x == 160 {
    //         self.change_mode(Mode::HBlank);
    //     }

    //     if ticks >= 456 {
    //         self.x = 0;
    //         if self.lcds.ly == 144 {
    //             self.change_mode(Mode::VBlank);
    //         } else {
    //             self.change_mode(Mode::OAMSearch);
    //         }
    //     }
        


    //     // match self.mode {
    //     //     Mode::HBlank => {
    //     //         if ticks == SCREEN_WIDTH {
    //     //             self.change_mode(Mode::HBlank);
    //     //         }
    //     //     },
    //     //     Mode::VBlank => {
    //     //         if ticks == 456
    //     //     }
    //     //     _ => {unimplemented!("Unimplemented mode")}
    //     // }
    // }
    
    fn change_mode(&mut self, mode: Mode) {
        self.mode = mode;

        if match self.mode {
            Mode::HBlank => {
                self.renderscan();
                self.hblank = true;
                self.lcds.stat.mode0
            },
            Mode::VBlank => { // Vertical blank
                // self.wy_trigger = false;
                self.interrupt |= 0x01;
                self.updated = true;
                self.lcds.stat.mode1
            },
            Mode::OAMSearch => self.lcds.stat.mode2,
            Mode::PixelTransfer => {
                // if self.win_on && self.wy_trigger == false && self.lcds.ly == self.winy {
                //     self.wy_trigger = true;
                //     self.wy = -1;
                // }
                false
            }
            _ => false,
        } {
            self.interrupt |= 0x02;
        }
    }

    fn renderscan(&mut self) {
        // self.draw_bg(); 
        let mut background_map: [u8; 32] = [0; 32];

        for x in background_map {
            // let pixel_x = (self.scx / 8 + x) & 0x1F;
            // let pixel_y = (self.scy + self.lcds.ly) & 0xFF;

            // println!("{} {}", pixel_x, pixel_y);
            // self.setcolor(x as usize, self.vram[(pixel_x as usize + ((pixel_y as usize) * 256))]);

            let winx = - ((self.wx as i32) - 7) + (x as i32);
            let tilex = (winx as u16 >> 3);
            let tilemapbase = self.lcdc.bg_and_window_display;
            

        }
    }





    pub fn in_hblank(&self) -> bool {
        return self.hblank;
    }


}

#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn set_bit(){
        assert_eq!(wbit(0b01010101, 3, true), 0b01011101);
        assert_eq!(wbit(0b01000101, 2, false), 0b01000001);
        assert_eq!(wbit(0b01000101, 7, false), 0b01000101);
    }
}