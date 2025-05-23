pub mod licensee;

use crate::mbc::FileBackedMBC;
use crate::mbc::MBC;
use std::path::PathBuf;
use crate::cartridge::licensee::Licensee;

pub fn has_battery(cartridge_type: u8) -> bool {
    match cartridge_type {
        0x03 | 0x06 | 0x09 | 0x0D | 0x0F | 0x10 | 0x13 | 0x1B | 0x1E | 0x22 | 0xFF => true,
        _ => false,
    }
}

pub struct Cartridge {
    pub rom_path: PathBuf,
    pub mbc: FileBackedMBC,
}

impl Cartridge {
    pub fn new(rom_path: PathBuf) -> Cartridge {
        let mbc = FileBackedMBC::new(rom_path.clone()).expect("Couldn't read rom");
        //let mbc: Box<dyn MBC> = mbc::new_mbc(data.clone());

        Cartridge {
            rom_path,
            mbc,
        }
    }

    pub fn reset(&mut self) {
        self.mbc = FileBackedMBC::new(self.rom_path.clone()).expect("Couldn't read rom to reset");
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

    pub fn get_title(&self) -> String {
        let title = &self.mbc.get_rom()[0x134..0x143];
        title.iter().take_while(|&&c| c != 0).map(|&c| c as char).collect()
    }

    pub fn get_manufacturer_code(&self) -> String {
        let code = &self.mbc.get_rom()[0x13F..0x142];
        code.iter().map(|&c| c as char).collect()
    }

    pub fn has_battery(&self) -> bool {
        self.mbc.has_battery()
    }

    pub fn get_cgb_flag(&self) -> u8 {
        self.mbc.get_rom()[0x143]
    }

    pub fn get_mbc_type(&self) -> u8 {
        self.mbc.get_rom()[0x147]
    }

    pub fn get_new_licensee_code(&self) -> String {
        let code = &self.mbc.get_rom()[0x144..0x146];
        code.iter().map(|&c| c as char).collect()
    }

    pub fn get_old_licensee_code(&self) -> u8 {
        self.mbc.get_rom()[0x14B]
    }

    pub fn get_licensee(&self) -> Option<Licensee> {
        let old_code = self.get_old_licensee_code();
        if old_code == 0x33 {
            let new_code = self.get_new_licensee_code();
            licensee::from_new_code(new_code.as_str())
        } else {
            licensee::from_old_code(old_code)
        }
    }

    pub fn get_sgb_flag(&self) -> u8 {
        self.mbc.get_rom()[0x146]
    }

    pub fn get_cartridge_type(&self) -> u8 {
        self.mbc.get_rom()[0x147]
    }

    pub fn get_rom_size_flag(&self) -> u8 {
        self.mbc.get_rom()[0x148]
    }

    pub fn get_ram_size_flag(&self) -> u8 {
        self.mbc.get_rom()[0x149]
    }

    pub fn get_destination_code(&self) -> u8 {
        self.mbc.get_rom()[0x14A]
    }

    pub fn get_rom_version_number(&self) -> u8 {
        self.mbc.get_rom()[0x14C]
    }

    pub fn compute_header_checksum(&self) -> u8 {
        let mut sum: u8 = 0;
        for i in 0x134..0x14C {
            sum = sum.wrapping_sub(self.mbc.get_rom()[i]);
        }
        sum.wrapping_sub(0x19)
    }

    pub fn get_header_checksum(&self) -> u8 {
        self.mbc.get_rom()[0x14D]
    }

    pub fn compute_global_checksum(&self) -> u16 {
        let mut sum: u16 = 0;
        for i in 0..0x14E {
            sum = sum.wrapping_add(self.mbc.get_rom()[i] as u16);
        }
        for i in 0x150..self.mbc.get_rom().len() {
            sum = sum.wrapping_add(self.mbc.get_rom()[i] as u16);
        }
        sum
    }

    pub fn get_global_checksum(&self) -> u16 {
        (self.mbc.get_rom()[0x14E] as u16) << 8 | self.mbc.get_rom()[0x14F] as u16
    }
}
