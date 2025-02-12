use crate::mbc::MBC;

pub struct MBC0 {
    rom: Vec<u8>,
}

impl MBC0 {
    pub fn new(data: Vec<u8>) -> Self {
        MBC0 { rom: data }
    }
}

impl MBC for MBC0 {
    fn force_write_rom(&mut self, address: u16, value: u8) {
        self.rom[address as usize] = value;
    }
    fn has_battery(&self) -> bool {
        false
    }
    fn load_ram(&mut self, _data: &[u8]) -> anyhow::Result<()> {
        Ok(())
    }
    fn dump_ram(&self) -> Vec<u8> {
        Vec::new()
    }

    fn get_rom(&self) -> &Vec<u8> {
        &self.rom
    }

    fn read_rom(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }
    fn read_ram(&self, _: u16) -> u8 {
        0
    }
    fn write_rom(&mut self, _: u16, _: u8) {}
    fn write_ram(&mut self, _: u16, _: u8) {}

    fn get_selected_rom_bank(&self) -> u8 {
        0
    }
    fn get_selected_ram_bank(&self) -> u8 {
        0
    }
}
