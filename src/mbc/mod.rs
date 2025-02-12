pub mod mbc3;
pub mod mbc0;
mod mbc1;

use std::{fs, io::{Read, Write}, path};

use anyhow::{Result, anyhow};

// https://gbdev.io/pandocs/MBCs.html
pub trait MBC: Send {
    fn force_write_rom(&mut self, address: u16, value: u8);
    fn has_battery(&self) -> bool;
    fn load_ram(&mut self, data: &[u8]) -> Result<()>;
    fn dump_ram(&self) -> Vec<u8>;
    fn get_rom(&self) -> &Vec<u8>;

    fn read_rom(&self, address: u16) -> u8;
    fn read_ram(&self, address: u16) -> u8;
    fn write_rom(&mut self, address: u16, value: u8);
    fn write_ram(&mut self, address: u16, value: u8);

    fn get_selected_rom_bank(&self) -> u8;
    fn get_selected_ram_bank(&self) -> u8;
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

pub struct FileBackedMBC {
    ram_path: std::path::PathBuf,
    mbc: Box<dyn MBC>,
}

impl FileBackedMBC {
    pub fn new(rom_path: path::PathBuf) -> Result<FileBackedMBC> {
        let mut data = vec![];
        fs::File::open(&rom_path).and_then(|mut f| f.read_to_end(&mut data))?;
        let mut mbc = new_mbc(data);

        let ram_path = rom_path.with_extension("gbsave");

        if mbc.has_battery() {
            match fs::File::open(&ram_path) {
                Ok(mut file) => {
                    let mut ram_data: Vec<u8> = vec![];
                    match file.read_to_end(&mut ram_data) {
                        Err(..) => return Err(anyhow!("Error while reading existing save file")),
                        Ok(..) => { mbc.load_ram(&ram_data)?; },
                    }
                },
                Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {},
                Err(_) => return Err(anyhow!("Error loading existing save file")),
            }
        }

        Ok(FileBackedMBC { ram_path, mbc })
    }
}

impl MBC for FileBackedMBC {
    fn force_write_rom(&mut self, address: u16, value: u8) {
        self.mbc.force_write_rom(address, value);
    }

    fn has_battery(&self) -> bool {
        self.mbc.has_battery()
    }

    fn load_ram(&mut self, data: &[u8]) -> Result<()> {
        self.mbc.load_ram(data)
    }

    fn dump_ram(&self) -> Vec<u8> {
        self.mbc.dump_ram()
    }

    fn get_rom(&self) -> &Vec<u8> {
        self.mbc.get_rom()
    }

    fn read_rom(&self, address: u16) -> u8 {
        self.mbc.read_rom(address)
    }

    fn read_ram(&self, address: u16) -> u8 {
        self.mbc.read_ram(address)
    }

    fn write_rom(&mut self, address: u16, value: u8) {
        self.mbc.write_rom(address, value);
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        self.mbc.write_ram(address, value);
    }

    fn get_selected_rom_bank(&self) -> u8 {
        self.mbc.get_selected_rom_bank()
    }

    fn get_selected_ram_bank(&self) -> u8 {
        self.mbc.get_selected_ram_bank()
    }
}

impl Drop for FileBackedMBC {
    fn drop(&mut self) {
        if self.mbc.has_battery() {
            // TODO: error handling
            let mut file = match fs::File::create(&self.ram_path) {
                Ok(f) => f,
                Err(..) => return,
            };
            let _ = file.write_all(&self.mbc.dump_ram());
        }
    }
}
