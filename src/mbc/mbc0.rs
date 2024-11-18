use crate::mbc::MBC;

pub struct MBC0 {
    rom: Vec<u8>,
}

impl MBC0 {
    pub fn new(data: Vec<u8>) -> Self {
        MBC0 {
            rom: data
        }
    }
}

impl MBC for MBC0 {
    fn force_write_rom(&mut self, address: u16, value: u8) {
        self.rom[address as usize] = value;
    }

    fn read_rom(&self, address: u16) -> u8 { self.rom[address as usize] }
    fn read_ram(&self, _: u16) -> u8 { 0 }
    fn write_rom(&mut self, _: u16, _: u8) {}
    fn write_ram(&mut self, _: u16, _: u8) {}
}
