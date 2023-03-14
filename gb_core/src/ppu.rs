
pub struct PPU {
    vram: [u8; 0x4000],
    voam: [u8; 0xA0],

}

impl PPU {
    pub fn read_byte(&self, loc: u16) {
        
    }
}