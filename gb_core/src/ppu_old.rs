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
        for x in 0 .. SCREEN_WIDTH {
            self.setcolor(x, 255);
        }
        // self.draw_bg();
        let mut background_map: [u8; 32 * 32] = [0; 32 * 32];

        for x in background_map {
            let pixel_x = (self.scx / 8 + x) & 0x1F;
            // self.setcolor(x as usize, self.vram[(pixel_x as usize + ((self.lcds.ly as usize) * 256))]);
        }

        // let map_offset = if self.lcdc.bg_tile_map_area {0x9800} else {0x9c00};
        // let data_offset = if self.lcdc.bg_and_win_tile_data_area {0x8800} else {0x8000};
        // for x in background_map {

        //     // let vram = 
        //     for (i, pixel) in self.vram.into_iter().enumerate(){
        //         let pix1 = (pixel >> 6) & 0b11;
        //         let pix2 = (pixel >> 4) & 0b11;
        //         let pix3 = (pixel >> 2) & 0b11;
        //         let pix4 = (pixel) & 0b11;
        //         self.setcolor(i * 3, if pix1 != 0 {255} else {0});
        //         self.setcolor((i * 3) + 1, if pix2 != 0 {255} else {0});
        //         self.setcolor((i * 3) + 2,  if pix3 != 0 {255} else {0});
        //         self.setcolor((i * 3) + 3,  if pix4 != 0 {255} else {0});
        //     }
        // }

        // for tile_x in 0 .. (SCREEN_WIDTH / 8) {
        //     // let tile_x_offset = 32 * tile_x + x;


        //     // self.setcolor(x, pixel);
        // }
    }
    fn setcolor(&mut self, x: usize, color: u8) {
        // println!("ly {} x {}", self.lcds.ly, x);
        if self.lcds.ly as usize * SCREEN_WIDTH * 3 + x * 3 + 2 >= 69120 { return }
        self.data[self.lcds.ly as usize * SCREEN_WIDTH * 3 + x * 3 + 0] = color;
        self.data[self.lcds.ly as usize * SCREEN_WIDTH * 3 + x * 3 + 1] = color;
        self.data[self.lcds.ly as usize * SCREEN_WIDTH * 3 + x * 3 + 2] = color;
    }
    fn draw_bg(&mut self) {
        let drawbg = self.lcdc.bg_and_window_display;

        // let wx_trigger = self.winx <= 166;
        // let winy = if self.wy_trigger && wx_trigger {
        //     self.wy += 1;
        //     self.wy
        // }
        // else {
        //     -1
        // };

        // if winy < 0 && drawbg == false {
        //     return;
        // }
        let winy = self.wy;

        let wintiley = (winy as u16 >> 3) & 31;

        let bgy = self.scy.wrapping_add(self.lcds.ly);
        let bgtiley = (bgy as u16 >> 3) & 31;

        for x in 0 .. SCREEN_WIDTH {
            let winx = - ((self.wx as i32) - 7) + (x as i32);
            let bgx = self.scx as u32 + x as u32;

            let (tilemapbase, tiley, tilex, pixely, pixelx) = if winy >= 0 && winx >= 0 {
                (if self.lcdc.w_tile_map_area {0x9c00} else {0x9800},
                wintiley,
                (winx as u16 >> 3),
                winy as u16 & 0x07,
                winx as u8 & 0x07)
            } else if drawbg {
                (if self.lcdc.bg_tile_map_area {0x9c00} else {0x9800},
                bgtiley,
                (bgx as u16 >> 3) & 31,
                bgy as u16 & 0x07,
                bgx as u8 & 0x07)
            } else {
                continue;
            };

            let tilenr: u8 = self.rbvram0(tilemapbase + tiley * 32 + tilex);

            let (palnr, vram1, xflip, yflip, prio) = (0, false, false, false, false);

            let tileaddress = self.tilebase
            + (if self.tilebase == 0x8000 {
                tilenr as u16
            } else {
                (tilenr as i8 as i16 + 128) as u16
            }) * 16;

            let a0 = match yflip {
                false => tileaddress + (pixely * 2),
                true => tileaddress + (14 - (pixely * 2)),
            };

            let (b1, b2) = match vram1 {
                false => (self.rbvram0(a0), self.rbvram0(a0 + 1)),
                true => (self.rbvram1(a0), self.rbvram1(a0 + 1)),
            };

            let xbit = match xflip {
                true => pixelx,
                false => 7 - pixelx,
            } as u32;
            let colnr = if b1 & (1 << xbit) != 0 { 1 } else { 0 }
                | if b2 & (1 << xbit) != 0 { 2 } else { 0 };

            // self.bgprio[x] =
            //     if colnr == 0 { PrioType::Color0 }
            //     else if prio { PrioType::PrioFlag }
            //     else { PrioType::Normal };

            let color = self.palb[colnr];
            self.setcolor(x, color);
            
        }
    }
    fn update_pal(&mut self) {
        for i in 0 .. 4 {
            self.palb[i] = PPU::get_monochrome_pal_val(self.palbr, i);
            self.pal0[i] = PPU::get_monochrome_pal_val(self.pal0r, i);
            self.pal1[i] = PPU::get_monochrome_pal_val(self.pal1r, i);
        }
    }

    fn get_monochrome_pal_val(value: u8, index: usize) -> u8 {
        match (value >> 2*index) & 0x03 {
            0 => 255,
            1 => 192,
            2 => 96,
            _ => 0
        }
    }

    fn rbvram0(&self, a: u16) -> u8 {
        if a < 0x8000 || a >= 0xA000 { panic!("Shouldn't have used rbvram0"); }
        self.vram[a as usize & 0x1FFF]
    }
    fn rbvram1(&self, a: u16) -> u8 {
        if a < 0x8000 || a >= 0xA000 { panic!("Shouldn't have used rbvram1"); }
        self.vram[0x2000 + (a as usize & 0x1FFF)]
    }

    pub fn in_hblank(&self) -> bool {
        return self.hblank;
    }

    pub fn may_hdma(&self) -> bool { 
        false
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