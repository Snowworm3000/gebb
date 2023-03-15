const VRAM_SIZE: usize = 0x4000;
const VOAM_SIZE: usize = 0xA0;

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

struct LCDC { //LCD Control registers
    original: u8,
    lcd_enable: bool,
    w_tile_map_area: usize,
    window_enable: bool,
    bg_and_win_tile_data_area: usize,
    bg_tile_map_area: usize,
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

impl LCDC { //LCD Control registers
    
    fn new(lcdc: u8) -> Self {
        Self {
            original: lcdc,
            lcd_enable: rbit(lcdc, 7),
            w_tile_map_area: if rbit(lcdc, 6) {0x9800} else {0x9c00},
            window_enable: rbit(lcdc, 5),
            bg_and_win_tile_data_area: if rbit(lcdc, 4) {0x8800} else {0x8000},
            bg_tile_map_area: if rbit(lcdc, 3) {0x9800} else {0x9c00},
            obj_size: rbit(lcdc, 2),
            obj_enable: rbit(lcdc, 1),
            bg_and_window_display: rbit(lcdc, 0),
        }
    }
}

struct LCDS { //LCD Status registers
    ly: u8,
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
}

pub struct PPU {
    vram: [u8; VRAM_SIZE],
    voam: [u8; VOAM_SIZE],
    lcdc: LCDC,

    pub updated: bool,
    pub interrupt: u8,
    pub data: Vec<u8>,
    pub modeclock: u32,
    pub line: u8,
    hblank: bool,
    mode: u8,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            vram: [0; VRAM_SIZE],
            voam: [0; VOAM_SIZE],
            lcdc: LCDC::new(0),

            updated: false,
            interrupt: 0,
            data: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT * 3],
            modeclock: 0,
            line: 0,
            hblank: false,
            mode: 0,
        }
    }

    pub fn read_byte(&self, loc: u16) -> u8 {
        match loc {
            0x8000..=0x97ff => {self.vram[(loc - 0x8000) as usize]},

            0xff40 => {self.lcdc.original}
            _ => {unimplemented!("Read location not implemented! {}", loc)}
        }
    }
    pub fn write_byte(&mut self, loc: u16, data: u8) {
        match loc {
            0x8000..=0x97ff => {self.vram[(loc - 0x8000) as usize] = data},

            0xff40 => {self.lcdc = LCDC::new(data)}
            _ => {unimplemented!("Write location not implemented! {}", loc)}
        }
    }

    pub fn cycle(&mut self, ticks: u32) {

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
        assert_eq!(wbit(0b01011101, 3, true), 0b01010101);
        assert_eq!(wbit(0b01000001, 2, false), 0b01000101);
        assert_eq!(wbit(0b01000101, 7, false), 0b01000101);
    }
}