// pub fn new_mbc(rom: &[u8]) -> Mbc {
//     // Mbc::new(rom, rom[0x147])
// }

const RAM_SIZE: usize = 8191;

pub struct Mbc<'a> {
    rom: &'a [u8],
    ram: [u8; RAM_SIZE],
    mbc_type: u8 
}

// impl Mbc<'_> {
//     fn new(rom: &[u8], mbc_type: u8) -> Self {
//         let ram = [0; RAM_SIZE];
//         Mbc {rom, ram, mbc_type}
//     }
// }