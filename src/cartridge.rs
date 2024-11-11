use crate::mbc;
use crate::mbc::MBC;
use crate::mbc::mbc0::MBC0;
use crate::mbc::mbc3::MBC3;

pub fn has_battery(cartridge_type: u8) -> bool {
    match cartridge_type {
        0x03 | 0x06 | 0x09 | 0x0D | 0x0F | 0x10 | 0x13 | 0x1B | 0x1E | 0x22 | 0xFF => true,
        _ => false,
    }
}

pub struct Cartridge {
    data: Vec<u8>,
    mbc: Box<dyn MBC>,
    has_battery: bool
}

impl Cartridge {
    pub fn new(data: Vec<u8>) -> Cartridge {
        let mbc_type = data[0x147];
        let has_battery = has_battery(mbc_type);
        let mbc: Box<dyn MBC> = mbc::new_mbc(data.clone());

        Cartridge {
            data,
            mbc,
            has_battery
        }
    }

    pub fn read_rom(&self, address: u16) -> u8 {
        self.mbc.read_rom(address)
    }

    pub fn read_ram(&self, address: u16) -> u8 {
        self.mbc.read_ram(address)
    }

    pub fn write_rom(&mut self, address: u16, value: u8) {
        self.mbc.write_rom(address, value)
    }

    pub fn write_ram(&mut self, address: u16, value: u8) {
        self.mbc.write_ram(address, value)
    }
}