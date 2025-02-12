use crate::mbc;
use crate::mbc::MBC;

use anyhow::{Result, anyhow};

pub struct MBC1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enabled: bool,
    selected_rom_bank: u8,
    selected_ram_bank: u8,
    ram_bank_count: u8,
    rom_bank_count: u8,
    banking_mode: u8,
    has_battery: bool,
}

impl MBC1 {
    pub fn new(data: Vec<u8>) -> Self {
        let ram_bank_count = mbc::ram_bank_count(data[0x149]);
        let rom_bank_count = mbc::rom_bank_count(data[0x148]);
        let has_battery = match data[0x147] {
            0x03 => true,
            _ => false,
        };
        MBC1 {
            rom: data,
            ram: vec![0; ram_bank_count as usize * 0x2000],
            ram_enabled: false,
            selected_rom_bank: 1,
            selected_ram_bank: 0,
            ram_bank_count,
            rom_bank_count,
            banking_mode: 0,
            has_battery,
        }
    }
}

impl MBC for MBC1 {
    fn force_write_rom(&mut self, address: u16, value: u8) {
        self.rom[address as usize] = value;
    }
    fn has_battery(&self) -> bool {
        self.has_battery
    }
    fn load_ram(&mut self, data: &[u8]) -> Result<()> {
        if data.len() != self.ram.len() {
            return Err(anyhow!("Loaded RAM has incorrect length"));
        }

        self.ram = data.to_vec();

        Ok(())
    }
    fn dump_ram(&self) -> Vec<u8> {
        self.ram.to_vec()
    }
    fn get_rom(&self) -> &Vec<u8> {
        &self.rom
    }

    fn read_rom(&self, address: u16) -> u8 {
        let bank = if address < 0x4000 {
            if self.banking_mode == 0 {
                0
            } else {
                self.selected_rom_bank & 0xE0
            }
        } else {
            self.selected_rom_bank
        };
        let idx = bank as usize * 0x4000 | ((address as usize) & 0x3FFF);
        *self.rom.get(idx).unwrap_or(&0xFF)
    }

    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled {
            return 0xFF;
        }
        let ram_bank = if self.banking_mode == 1 {
            self.selected_ram_bank
        } else {
            0
        };
        self.ram[(ram_bank as usize * 0x2000) | ((address & 0x1FFF) as usize)]
    }

    fn write_rom(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.ram_enabled = value & 0xF == 0xA;
            }
            0x2000..=0x3FFF => {
                let lower_bits = match value & 0x1F {
                    0 => 1,
                    n => n,
                };
                self.selected_rom_bank =
                    ((self.selected_rom_bank & 0x60) | lower_bits) % self.rom_bank_count;
            }
            0x4000..=0x5FFF => {
                if self.rom_bank_count > 0x20 {
                    let upper_bits = value & 0x03 % (self.rom_bank_count >> 5);
                    self.selected_rom_bank = self.selected_rom_bank & 0x1F | (upper_bits << 5)
                }
                if self.rom_bank_count > 1 {
                    self.selected_rom_bank = value & 0x03;
                }
            }
            0x6000..=0x7FFF => {
                self.banking_mode = value & 0x01;
            }
            _ => panic!("Could not write to {:04X} (MBC1)", address),
        }
    }
    fn write_ram(&mut self, address: u16, value: u8) {
        if !self.ram_enabled {
            return;
        }
        let ram_bank = if self.banking_mode == 1 {
            self.selected_ram_bank
        } else {
            0
        };
        let address = (ram_bank as u16 * 0x2000) | (address & 0x1FFF);
        if address < self.ram.len() as u16 {
            self.ram[address as usize] = value;
        }
    }

    fn get_selected_rom_bank(&self) -> u8 {
        self.selected_rom_bank
    }
    fn get_selected_ram_bank(&self) -> u8 {
        self.selected_ram_bank
    }
}
