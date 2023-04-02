pub struct PPU {
    vram: [u8; 0x2000],
    oam: [u8; 0xA0],
    lcdc: u8,
    stat: u8,
    scy: u8,
    scx: u8,
    pub ly: u8,
    lyc: u8,
    wy: u8,
    wx: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    pub screen_buffer: [u8; 160 * 144 * 3],
    pub updated: bool,
    pub interrupt: u8,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            vram: [0; 0x2000],
            oam: [0; 0xA0],
            lcdc: 0,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            wy: 0,
            wx: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            screen_buffer: [0; 160 * 144 * 3],
            updated: false,
            interrupt: 0,
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize],
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],
            0xFF40 => self.lcdc,
            0xFF41 => self.stat,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => panic!("Invalid PPU read at address {:04x}", address),
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize] = value,
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize] = value,
            0xFF40 => self.lcdc = value,
            0xFF41 => self.stat = value,
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => self.ly = 0,
            0xFF45 => self.lyc = value,
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,
            // _ => panic!("Invalid PPU write at address {:04x}", address),
            _ => {},
        }
    }

    pub fn execute(&mut self, cycles: u8) {
        let current_mode = self.stat & 0b11;

        match current_mode {
            // Hblank
            0 => {
                self.ly += 1;

                // Check for LYC coincidence
                if self.ly == self.lyc {
                    self.stat |= 0b0100_0000; // Set LYC=LY coincidence flag
                    if self.stat & 0b0100_0000 != 0 && self.stat & 0b0010_0000 != 0 {
                        // If LYC=LY coincidence interrupt enabled
                        // and mode 2 (OAM search) interrupt enabled
                        // then request interrupt
                        // todo!("request interrupt")
                        self.interrupt |= 0x02;
                    }
                } else {
                    self.stat &= !0b0100_0000; // Clear LYC=LY coincidence flag
                }
    
                if self.ly == 144 {
                    // Vblank
                    self.stat = (self.stat & !0b11) | 0b01; // Set mode to 1 (Vblank)
                    // todo!("render frame")
                    self.render_scanline();
                    self.interrupt |= 0x01;
                    self.updated = true;
                } else {
                    // Switch to OAM search
                    self.stat = (self.stat & !0b11) | 0b10; // Set mode to 2 (OAM search)
                    // self.render_scanline();
                }
            }
            // Vblank
            1 => {
                self.ly += 1;
    
                if self.ly == 154 {
                    self.ly = 0;
                    // Switch to OAM search
                    self.stat = (self.stat & !0b11) | 0b10; // Set mode to 2 (OAM search)
                }
            }
            // OAM search
            2 => {
                // TODO: OAM search
                self.stat = (self.stat & !0b11) | 0b11; // Set mode to 3 (LCD transfer)
            }
            // LCD transfer
            3 => {
                // TODO: LCD transfer
                // self.render_scanline();
                self.stat = (self.stat & !0b11) | 0b00; // Set mode to 0 (Hblank)
    
                if self.stat & 0b0000_1000 != 0 {
                    // If mode 0 (Hblank) interrupt enabled
                    // then request interrupt
                    // todo!("request interrupt")
                    self.interrupt |= 0x02;
                }
            }
            _ => unreachable!(),
        }
    }
    
    fn render_scanline(&mut self) {
        let background_enabled = self.lcdc & 0b0000_0001 != 0;
        let sprites_enabled = self.lcdc & 0b0000_0010 != 0;
    
        if background_enabled {
            // TODO: Render background
            self.render_background(self.ly as u32); 
        }
    
        if sprites_enabled {
            // TODO: Render sprites
        }
    }    
    
    fn render_background(&mut self, ly: u32) {
        let bg_enabled = self.lcdc & 0b0000_0001 != 0;
        let bg_tile_map_select = (self.lcdc & 0b0000_1000) >> 3;
        let bg_tile_set_select = (self.lcdc & 0b0001_0000) >> 4;
        let scroll_y = self.scy;
        let scroll_x = self.scx;
    
        if !bg_enabled {
            return;
        }
    
        let tile_map_base = if bg_tile_map_select == 0 {
            0x9800 - 0x8000
        } else {
            0x9C00 - 0x8000
        };
    
        let tile_set_base = if bg_tile_set_select == 0 {
            0x8800 - 0x8000
        } else {
            0x8000 - 0x8000
        };
    
        let tile_size = 8;
    
        // let row = ly as usize;
        // println!("{}", row);
        // if ly >= 144 {
        //     return;
        // }
        for row in 0..144 {
            for col in 0..160 {
                let x = col as u8;
                let y = row as u8;
                let tile_y = (y.wrapping_add(scroll_y)) / tile_size;
                let tile_x = (x.wrapping_add(scroll_x)) / tile_size;
                let tile_map_offset = tile_y as u16 * 32 + tile_x as u16;
                let tile_id = self.vram[(tile_map_base + tile_map_offset) as usize];
                let tile_offset = if tile_set_base == 0x8800 {
                    ((tile_id as i8) as i16 + 128) as u16
                } else {
                    tile_id as u16
                };
                let tile_address = tile_set_base + tile_offset * 16;
                let tile_row = ((y.wrapping_add(scroll_y)) % tile_size) * 2;
                let lsb = self.vram[(tile_address + tile_row as u16) as usize];
                let msb = self.vram[(tile_address + tile_row as u16 + 1) as usize];
    
                let pixel_x = (x.wrapping_add(scroll_x)) % tile_size;
                let colour_bit = 7 - pixel_x;
                let colour_num = ((msb >> colour_bit) & 1) << 1 | ((lsb >> colour_bit) & 1);
                let colour = match colour_num {
                    0 => self.bgp & 0b0000_0011,
                    1 => (self.bgp & 0b0000_1100) >> 2,
                    2 => (self.bgp & 0b0011_0000) >> 4,
                    3 => (self.bgp & 0b1100_0000) >> 6,
                    _ => unreachable!(),
                };
    
                let pixel_offset = (row * 160 + col) * 3;
                let colours = self.to_rgb(colour);
                self.screen_buffer[pixel_offset] = colours.0;
                self.screen_buffer[pixel_offset + 1] = colours.1;
                self.screen_buffer[pixel_offset + 2] = colours.2;
            }
        }
    
        self.updated = true;
    }

    fn to_rgb(&self, colour: u8) -> (u8, u8, u8) {
        match colour {
            0 => (255, 255, 255),
            1 => (200, 200, 200),
            2 => (100, 100, 100),
            3 => (0, 0, 0),
            _ => {panic!("undefined colour {}", colour)}
        }
    }
    
}