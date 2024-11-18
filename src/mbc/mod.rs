pub mod mbc3;
pub mod mbc0;
mod mbc1;

// https://gbdev.io/pandocs/MBCs.html
pub trait MBC: Send {
    fn force_write_rom(&mut self, address: u16, value: u8);

    fn read_rom(&self, address: u16) -> u8;
    fn read_ram(&self, address: u16) -> u8;
    fn write_rom(&mut self, address: u16, value: u8);
    fn write_ram(&mut self, address: u16, value: u8);
}

pub fn new_mbc(data: Vec<u8>) -> Box<dyn MBC> {
    // https://gbdev.io/pandocs/The_Cartridge_Header.html#0147--cartridge-type
    match data[0x147] {
        0x00 => Box::new(mbc0::MBC0::new(data)),
        0x01 | 0x02 | 0x03 => Box::new(mbc1::MBC1::new(data)),
        0x0F | 0x10 | 0x11 | 0x12 | 0x13 => Box::new(mbc3::MBC3::new(data)),
        _ => panic!("Unsupported MBC type: {:02X}", data[0x147]),
    }
}

pub fn ram_bank_count(code: u8) -> u8 {
    // https://gbdev.io/pandocs/The_Cartridge_Header.html#0149--ram-size
    match code {
        1 => 1,
        2 => 2,
        3 => 4,
        4 => 16,
        5 => 8,
        _ => 0,
    }
}

pub fn rom_bank_count(code: u8) -> u8 {
    // https://gbdev.io/pandocs/The_Cartridge_Header.html#0148--rom-size
    match code {
        0 => 2,
        1 => 4,
        2 => 8,
        3 => 16,
        4 => 32,
        5 => 64,
        6 => 128,
        0x52 => 72,
        0x53 => 80,
        0x54 => 96,
        _ => 0,
    }
}