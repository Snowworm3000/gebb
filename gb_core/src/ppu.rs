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
            _ => panic!("Invalid PPU write at address {:04x}", address),
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
                    self.stat |= (1 << 6); // Set LYC=LY flag
                    if self.stat & (1 << 6) != 0 && self.stat & (1 << 5) != 0 {
                        // If LYC=LY interrupt enabled
                        // and mode 2 (OAM search) interrupt enabled
                        self.interrupt |= 0x02; // Request interrupt
                    }
                } else {
                    self.stat &= !(1 << 6); // Clear LYC=LY coincidence flag
                }
    
                if self.ly == 144 {
                    // Vblank
                    self.stat = (self.stat & !0b11) | 0b01; // Set mode to 1 (Vblank)
                    self.render_scanline();
                    self.interrupt |= 0x01;
                    self.updated = true;
                } else {
                    // Switch to OAM search
                    self.stat = (self.stat & !0b11) | 0b10; // Set mode to 2 (OAM search)
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
                self.stat = (self.stat & !0b11) | 0b11; // Set mode to 3 (LCD transfer)
            }
            // LCD transfer
            3 => {
                self.stat = (self.stat & !0b11) | 0b00; // Set mode to 0 (Hblank)
    
                if self.stat & (1 << 3) != 0 {
                    // If mode 0 (Hblank) interrupt enabled
                    self.interrupt |= 0x02; // Request interrupt
                }
            }
            _ => unreachable!(),
        }
    }
    
    fn render_scanline(&mut self) {
        let background_enabled = self.lcdc & 0b1 != 0;
        let sprites_enabled = self.lcdc & 0b10 != 0;
    
        let output: [u8; 144 * 160];
        let background_buffer;
        let sprite_buffer;
        if background_enabled {
            background_buffer = self.render_background();

            if sprites_enabled {
                sprite_buffer = self.render_sprites();
                output = self.merge(background_buffer, sprite_buffer);
            } else {
                output = background_buffer
            }
            self.render(output);
        }
    }   

    fn merge(&self, bg: [u8; 160 * 144], spr: [u8; 160 * 144]) -> [u8; 160 * 144] {
        let mut merged = [0; 160 * 144];
        for (index, bg_pixel) in bg.into_iter().enumerate() {
            let spr_pixel = spr[index];

            // Sprite is visible
            if (spr_pixel != 0) & (((self.lcdc >> 1) & 0b1) == 1) { 
                merged[index] = spr_pixel;
            } else {
                merged[index] = bg_pixel;
            }
        }
        merged
    }
    fn render(&mut self, buffer: [u8; 160 * 144]) {
        for row in 0..144 {
            for col in 0..160 {
                let pixel_offset = (row * 160 + col) * 3;
                let pixel = buffer[row * 160 + col];
                let colours = self.to_rgb(pixel);
                self.screen_buffer[pixel_offset] = colours.0;
                self.screen_buffer[pixel_offset + 1] = colours.1;
                self.screen_buffer[pixel_offset + 2] = colours.2;
            }
        }
    }

    fn render_background(&mut self) -> [u8; 160 * 144] {
        let mut background_buffer = [0; 160 * 144];
        let bg_tile_map_select = (self.lcdc >> 3) & 0b1;
        let bg_tile_set_select = (self.lcdc >> 4) & 0b1;
        let scroll_y = self.scy;
        let scroll_x = self.scx;
    
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

                let pixel_offset = (row * 160 + col);
                background_buffer[pixel_offset] = colour;
            }
        }
    
        self.updated = true;
        background_buffer
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

    fn render_sprites(&mut self) -> [u8; 144 * 160] {
        let mut sprite_buffer = [0; 144 * 160];
        let sprite_height = if self.lcdc & (1 << 2) != 0 { 16 } else { 8 };
    
        for row in 0..144 {
            for col in 0..40 {
                let sprite_addr = 0xFE00 + col * 4;
                let y_pos = self.read_byte(sprite_addr) as i16 - 16;
                let x_pos = self.read_byte(sprite_addr + 1) as i16 - 8;
                let tile_num = self.read_byte(sprite_addr + 2);
                let attributes = self.read_byte(sprite_addr + 3);
        
                // Check if sprite intersects with scanline
                if y_pos <= row as i16 && y_pos + sprite_height > row as i16 {
        
                    let tile_row = if attributes & (1 << 6) != 0 {
                        (sprite_height - 1) - (row as i16 - y_pos)
                    } else {
                        row as i16 - y_pos
                    }; 
        
                    // Read sprite tile data from VRAM
                    let tile_addr = if sprite_height == 16 {
                        0x8000 + (tile_num & 0xFE) as u16 * 16
                    } else {
                        0x8000 + tile_num as u16 * 16
                    };
                    let tile_lo = self.read_byte(tile_addr + tile_row as u16 * 2);
                    let tile_hi = self.read_byte(tile_addr + tile_row as u16 * 2 + 1);
        
                    for i in 0..8 {
                        let color_bit = 7 - i;
                        let color_num = ((tile_hi >> color_bit) & 1) << 1 | ((tile_lo >> color_bit) & 1);
                        let color_addr = if attributes & (1 << 4) != 0 { 0xFF49 } else { 0xFF48 };
                        let color = self.read_byte(color_addr) >> (color_num * 2) & 0b11;
        
                        let pixel_x = (x_pos + i) as usize;
                        let pixel_y = row as usize;
        
                        // Draw pixel if it's not transparent
                        if color_num != 0 && pixel_x < 160 && pixel_y < 144 {
                            if attributes & (1 << 5) != 0 {
                                // Flip horizontally
                                sprite_buffer[(pixel_y * 160) + (159 - pixel_x)] = color;
                            } else {
                                sprite_buffer[(pixel_y * 160) + pixel_x] = color;
                            }
                        }
                    }
                }
    
            }
        }
        sprite_buffer
    }
    
    
}