use std::time;
use crate::mbc;
use crate::mbc::MBC;

#[inline(always)]
fn is_set(byte: u8, position: u8) -> bool {
    (byte >> position) & 1 == 1
}

pub struct MBC3 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enabled: bool,
    selected_rom_bank: u8,
    selected_ram_bank: u8,
    ram_bank_count: u8,
    rtc_selected: bool,
    rtc_ram: [u8; 5],
    rtc_ram_latch: [u8; 5],
    rtc_zero: Option<u64>,
}

impl MBC3 {
    pub fn new(data: Vec<u8>) -> Self {
        let ram_bank_count = mbc::ram_bank_count(data[0x149]);
        MBC3 {
            rom: data,
            ram: vec![0; ram_bank_count as usize * 0x2000],
            ram_enabled: true,
            selected_rom_bank: 1,
            selected_ram_bank: 0,
            ram_bank_count,
            rtc_selected: false,
            rtc_ram: [0; 5],
            rtc_ram_latch: [0; 5],
            rtc_zero: None,
        }
    }

    fn latch_rtc_reg(&mut self) {
        self.calc_rtc_reg();
        self.rtc_ram_latch.clone_from_slice(&self.rtc_ram);
    }

    fn calc_rtc_reg(&mut self) {
        // Do not modify regs when halted
        if is_set(self.rtc_ram[4], 6) {
            return;
        }

        let tzero = match self.rtc_zero {
            Some(t) => time::UNIX_EPOCH + time::Duration::from_secs(t),
            None => return,
        };

        if self.compute_time_diff() == self.rtc_zero {
            // No time has passed. Do not alter registers
            return;
        }

        let time_diff = match time::SystemTime::now().duration_since(tzero) {
            Ok(n) => { n.as_secs() }
            _ => { 0 }
        };
        self.rtc_ram[0] = (time_diff % 60) as u8;
        self.rtc_ram[1] = ((time_diff / 60) % 60) as u8;
        self.rtc_ram[2] = ((time_diff / 3600) % 24) as u8;
        let days = time_diff / (3600 * 24);
        self.rtc_ram[3] = days as u8;
        self.rtc_ram[4] = (self.rtc_ram[4] & 0xFE) | (((days >> 8) & 0x01) as u8);
        if days >= 512 {
            self.rtc_ram[4] |= 0x80;
            self.calc_rtc_zero();
        }
    }

    fn compute_time_diff(&self) -> Option<u64> {
        if self.rtc_zero.is_none() { return None; }
        let mut time_diff = match time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
            Ok(t) => t.as_secs(),
            Err(_) => panic!("System clock is set to a time before the unix epoch (1970-01-01)"),
        };
        time_diff -= self.rtc_ram[0] as u64;
        time_diff -= (self.rtc_ram[1] as u64) * 60;
        time_diff -= (self.rtc_ram[2] as u64) * 3600;
        let days = ((self.rtc_ram[4] as u64 & 0x1) << 8) | (self.rtc_ram[3] as u64);
        time_diff -= days * 3600 * 24;
        Some(time_diff)
    }

    fn calc_rtc_zero(&mut self) {
        self.rtc_zero = self.compute_time_diff();
    }
}

impl MBC for MBC3 {
    fn force_write_rom(&mut self, address: u16, value: u8) {
        self.rom[address as usize] = value;
    }

    fn read_rom(&self, address: u16) -> u8 {
        let index = if address < 0x4000 {
            address as usize
        } else {
            ((self.selected_rom_bank as usize) * 0x4000) | ((address as usize) & 0x3FFF)
            // (self.selected_rom_bank as usize) * 0x4000 + (address as usize - 0x4000)
        };
        self.rom.get(index).copied().unwrap_or(0xFF)
    }

    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled {
            return 0xFF;
        }
        if !self.rtc_selected && self.selected_ram_bank < self.ram_bank_count {
            self.ram[((self.selected_ram_bank as usize) * 0x2000) | ((address as usize) & 0x1FFF)]
            // self.ram[(self.selected_ram_bank as usize) * 0x2000 + (address as usize)]
        } else if self.rtc_selected && self.selected_ram_bank < 5 {
            self.rtc_ram_latch[self.selected_ram_bank as usize]
        } else {
            0xFF
        }
    }

    fn write_rom(&mut self, address: u16, value: u8) {
        match address {
            // https://gbdev.io/pandocs/MBC3.html#0000-1fff---ram-and-timer-enable-write-only
            // Writing any value with the lower 4 bits being 0xA enables the RAM and RTC registers.
            0x0000..=0x1FFF => self.ram_enabled = value & 0xF == 0xA,

            0x2000..=0x3FFF => self.selected_rom_bank = match value & 0x7F {
                0 => 1,
                n => n
            },
            0x4000..=0x5FFF => {
                self.rtc_selected = is_set(value, 3);
                self.selected_ram_bank = value & 0x7;
            }
            0x6000..=0x7FFF => self.latch_rtc_reg(),
            _ => panic!("Invalid address: {:04X} (MBC3)", address),
        }
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        if !self.ram_enabled {
            return;
        }
        if !self.rtc_selected && self.selected_ram_bank < self.ram_bank_count {
            self.ram[(self.selected_ram_bank as usize * 0x2000) | ((address as usize) & 0x1FFF)] = value;
            // self.ram[(self.selected_ram_bank as usize) * 0x2000 + (address as usize)] = value;
        } else if self.rtc_selected && self.selected_ram_bank < 5 {
            self.calc_rtc_zero();
            let mask = match self.selected_ram_bank {
                0 | 1 => 0x3F,
                2 => 0x1F,
                4 => 0xC1,
                _ => 0xFF,
            };
            self.rtc_ram[self.selected_ram_bank as usize] = value & mask;
            self.calc_rtc_zero();
        }
    }

    fn get_selected_rom_bank(&self) -> u8 { self.selected_rom_bank }
    fn get_selected_ram_bank(&self) -> u8 { self.selected_ram_bank }
}
