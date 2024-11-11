use crate::cartridge::Cartridge;
use crate::io::joypad::Joypad;
use crate::mmu::timer::Timer;
use crate::ppu::PPU;

mod timer;

#[inline(always)]
fn bit(value: bool, position: u8) -> u8 {
    if value { 1 << position } else { 0 }
}

#[inline(always)]
fn is_set(byte: u8, position: u8) -> bool {
    (byte >> position) & 1 == 1
}

#[derive(PartialEq)]
enum DMAType {
    NoDMA,
    GDMA,
    HDMA,
}

pub const BOOT_ROM: [u8; 256] = [0x31, 0xfe, 0xff, 0xaf, 0x21, 0xff, 0x9f, 0x32, 0xcb, 0x7c, 0x20, 0xfb, 0x21, 0x26, 0xff, 0xe, 0x11, 0x3e, 0x80, 0x32, 0xe2, 0xc, 0x3e, 0xf3, 0xe2, 0x32, 0x3e, 0x77, 0x77, 0x3e, 0xfc, 0xe0, 0x47, 0x11, 0x4, 0x1, 0x21, 0x10, 0x80, 0x1a, 0xcd, 0x95, 0x0, 0xcd, 0x96, 0x0, 0x13, 0x7b, 0xfe, 0x34, 0x20, 0xf3, 0x11, 0xd8, 0x0, 0x6, 0x8, 0x1a, 0x13, 0x22, 0x23, 0x5, 0x20, 0xf9, 0x3e, 0x19, 0xea, 0x10, 0x99, 0x21, 0x2f, 0x99, 0xe, 0xc, 0x3d, 0x28, 0x8, 0x32, 0xd, 0x20, 0xf9, 0x2e, 0xf, 0x18, 0xf3, 0x67, 0x3e, 0x64, 0x57, 0xe0, 0x42, 0x3e, 0x91, 0xe0, 0x40, 0x4, 0x1e, 0x2, 0xe, 0xc, 0xf0, 0x44, 0xfe, 0x90, 0x20, 0xfa, 0xd, 0x20, 0xf7, 0x1d, 0x20, 0xf2, 0xe, 0x13, 0x24, 0x7c, 0x1e, 0x83, 0xfe, 0x62, 0x28, 0x6, 0x1e, 0xc1, 0xfe, 0x64, 0x20, 0x6, 0x7b, 0xe2, 0xc, 0x3e, 0x87, 0xe2, 0xf0, 0x42, 0x90, 0xe0, 0x42, 0x15, 0x20, 0xd2, 0x5, 0x20, 0x4f, 0x16, 0x20, 0x18, 0xcb, 0x4f, 0x6, 0x4, 0xc5, 0xcb, 0x11, 0x17, 0xc1, 0xcb, 0x11, 0x17, 0x5, 0x20, 0xf5, 0x22, 0x23, 0x22, 0x23, 0xc9, 0xce, 0xed, 0x66, 0x66, 0xcc, 0xd, 0x0, 0xb, 0x3, 0x73, 0x0, 0x83, 0x0, 0xc, 0x0, 0xd, 0x0, 0x8, 0x11, 0x1f, 0x88, 0x89, 0x0, 0xe, 0xdc, 0xcc, 0x6e, 0xe6, 0xdd, 0xdd, 0xd9, 0x99, 0xbb, 0xbb, 0x67, 0x63, 0x6e, 0xe, 0xec, 0xcc, 0xdd, 0xdc, 0x99, 0x9f, 0xbb, 0xb9, 0x33, 0x3e, 0x3c, 0x42, 0xb9, 0xa5, 0xb9, 0xa5, 0x42, 0x3c, 0x21, 0x4, 0x1, 0x11, 0xa8, 0x0, 0x1a, 0x13, 0xbe, 0x20, 0xfe, 0x23, 0x7d, 0xfe, 0x34, 0x20, 0xf5, 0x6, 0x19, 0x78, 0x86, 0x23, 0x5, 0x20, 0xfb, 0x86, 0x20, 0xfe, 0x3e, 0x1, 0xe0, 0x50];
pub struct MMU {
    cartridge: Cartridge,
    boot_rom: Option<[u8; 256]>,
    wram: [[u8; 0x1000]; 8], // Working RAM, 8 banks total
    hram: [u8; 0x7F], // aka High Ram or Zero Page

    hdma: [u8; 0x10], // HDMA registers
    hdma_src: u16, // HDMA source address
    hdma_dst: u16, // HDMA destination address
    hdma_len: u8, // HDMA length
    hdma_status: DMAType, // HDMA status

    selected_wram_bank: u8, // 1-7 banks, bank 0 is always available
    pub interrupt_flags: u8, // 7-5: Unused, 4: Joypad, 3: Serial, 2: Timer, 1: LCD, 0: VBlank
    pub interrupt_enable: u8, // Controls whether the interrupt handler should be called, same layout as interrupt flags
    pub joypad: Joypad,
    pub ppu: PPU,
    pub timer: Timer,
}

impl MMU {
    pub fn new(cartridge: Cartridge) -> MMU {
        MMU {
            cartridge,
            boot_rom: Some(BOOT_ROM),
            wram: [[0; 0x1000]; 8],
            hram: [0; 0x7F],

            hdma: [0; 0x10],
            hdma_src: 0,
            hdma_dst: 0,
            hdma_len: 0,
            hdma_status: DMAType::NoDMA,

            selected_wram_bank: 1,
            interrupt_flags: 0b00000,
            interrupt_enable: 0b00000,
            joypad: Joypad::new(),
            ppu: PPU::new(),
            timer: Timer::new(),
        }
    }

    pub fn step(&mut self, cycles: u32) {
        self.timer.step(cycles);
        self.interrupt_flags |= self.timer.interrupt;
        self.timer.interrupt = 0;

        self.ppu.step(cycles);
        self.interrupt_flags |= self.ppu.interrupt;
        self.ppu.interrupt = 0;
    }

    pub fn has_interrupt(&self) -> bool {
        self.interrupt_flags & self.interrupt_enable != 0
    }

    pub fn read_word(&self, addr: u16) -> u16 {
        let low = self.read_byte(addr) as u16;
        let high = self.read_byte(addr.wrapping_add(1)) as u16;
        (high << 8) | low
    }

    pub fn write_word(&mut self, addr: u16, value: u16) {
        self.write_byte(addr, value as u8);
        self.write_byte(addr.wrapping_add(1), (value >> 8) as u8);
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x00FF => if let Some(boot_ram) = self.boot_rom {
                boot_ram[addr as usize]
            } else {
                self.cartridge.read_rom(addr)
            }
            0x0000..=0x7FFF => self.cartridge.read_rom(addr),
            0x8000..=0x9FFF => self.ppu.read_vram(addr - 0x8000),
            0xA000..=0xBFFF => self.cartridge.read_ram(addr - 0xA000),
            0xC000..=0xCFFF => self.wram[0][addr as usize - 0xC000],
            0xD000..=0xDFFF => self.wram[self.selected_wram_bank as usize][addr as usize - 0xD000],
            0xE000..=0xFDFF => {
                // Echo RAM echoes wram bank 0
                let bank = (addr as usize - 0xE000) / 0x1000;
                self.wram[bank][(addr as usize - 0xE000) % 0x1000]
            }
            0xFE00..=0xFE9F => self.ppu.read_oam(addr - 0xFE00),
            0xFEA0..=0xFEFF => 0x00, // Not Usable https://gbdev.io/pandocs/Memory_Map.html#fea0feff-range

            // IO Registers: https://gbdev.io/pandocs/Hardware_Reg_List.html
            0xFF00 => self.joypad.read_byte(),
            0xFF01 => 0, // TODO: Serial Data Transfer
            0xFF02 => 0, // TODO: Serial Data Control
            0xFF03 => 0xFF, // Unused
            0xFF04 ..= 0xFF07 => self.timer.read_byte(addr),
            0xFF08..=0xFF0E => 0xFF, // Unused
            0xFF0F => self.interrupt_flags,
            0xFF10..=0xFF3F => 0, // TODO: Sound Registers
            0xFF40..=0xFF4B => self.ppu.read_register(addr),
            0xFF4C => 0xFF, // Unused
            0xFF4D => 0, // TODO: Speed Switch
            0xFF4E => 0xFF, // Unused
            0xFF4F => self.ppu.selected_vram_bank as u8,
            0xFF50 => 0xFF,
            0xFF51..=0xFF54 => self.hdma[addr as usize - 0xFF51],
            0xFF55 => self.hdma_len | bit(self.hdma_status == DMAType::NoDMA, 7),
            0xFF56 => 0, // TODO: Infrared
            0xFF57..=0xFF67 => 0xFF, // Unused
            0xFF68..=0xFF6B => self.ppu.read_register(addr),
            0xFF6C => 0, // TODO: CGB Speed Switch
            0xFF6D..=0xFF6F => 0xFF, // Unused
            0xFF70 => self.selected_wram_bank,
            0xFF71..=0xFF75 => 0xFF, // Unused
            0xFF76..=0xFF77 => 0, // TODO: Audio digital output

            0xFF78..=0xFF7F => 0xFF, // Unused
            0xFF80..=0xFFFE => self.hram[addr as usize - 0xFF80],

            0xFFFF => self.interrupt_enable,
            _ => unreachable!()
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x3FFF => {} // TODO: ROM Bank 0
            0x4000..=0x7FFF => {} // TODO: Switchable ROM Bank
            0x8000..=0x9FFF => self.ppu.write_vram(addr - 0x8000, value),
            0xA000..=0xBFFF => {} // TODO: External RAM from cartridge
            0xC000..=0xCFFF => self.wram[0][addr as usize - 0xC000] = value,
            0xD000..=0xDFFF => self.wram[self.selected_wram_bank as usize][addr as usize - 0xD000] = value,
            0xE000..=0xFDFF => {
                // Echo RAM echoes wram bank 0
                let bank = (addr as usize - 0xE000) / 0x1000;
                self.wram[bank][(addr as usize - 0xE000) % 0x1000] = value;
            }
            0xFE00..=0xFE9F => self.ppu.write_oam(addr - 0xFE00, value),
            0xFEA0..=0xFEFF => {} // Not Usable https://gbdev.io/pandocs/Memory_Map.html#fea0feff-range

            // IO Registers: https://gbdev.io/pandocs/Hardware_Reg_List.html
            0xFF00 => self.joypad.write_byte(value),
            0xFF01 => {} // TODO: Serial Data Transfer
            0xFF02 => {} // TODO: Serial Data Control
            0xFF03 => {} // Unused
            0xFF04 ..= 0xFF07 => self.timer.write_byte(addr, value),
            0xFF08..=0xFF0E => {} // Unused
            0xFF0F => self.interrupt_flags = value,
            0xFF10..=0xFF3F => {} // TODO: Sound Registers
            0xFF40..=0xFF4B => self.ppu.write_register(addr, value),
            0xFF4C => {} // Unused
            0xFF4D => {} // TODO: Speed Switch
            0xFF4E => {} // Unused
            0xFF4F => self.ppu.selected_vram_bank = value > 0,
            0xFF50 => self.boot_rom = None,
            0xFF51 => self.hdma[0] = value,
            0xFF52 => self.hdma[1] = value & 0xF0,
            0xFF53 => self.hdma[2] = value & 0x1F,
            0xFF54 => self.hdma[3] = value & 0xF0,
            0xFF55 => {
                if self.hdma_status == DMAType::HDMA {
                    if is_set(value, 7) {
                        self.hdma_status = DMAType::NoDMA;
                    }
                    return;
                }
                let src = ((self.hdma[0] as u16) << 8) | (self.hdma[1] as u16);
                let dst = ((self.hdma[2] as u16) << 8) | (self.hdma[3] as u16) | 0x8000;
                if src > 0x7FF0 && (src < 0xA000 || src > 0xDFF0) {
                    panic!("HDMA transfer with illegal start address {:04X}", src);
                }

                self.hdma_src = src;
                self.hdma_dst = dst;
                self.hdma_len = value & 0x7F;
                self.hdma_status = if is_set(value, 7) { DMAType::HDMA } else { DMAType::GDMA };
            }
            0xFF56 => {} // TODO: Infrared
            0xFF57..=0xFF67 => {} // Unused
            0xFF68..=0xFF6B => self.ppu.write_register(addr, value),
            0xFF6C => {} // TODO: CGB Object Priority Mode
            0xFF6D..=0xFF6F => {} // Unused
            0xFF70 => self.selected_wram_bank = value,
            0xFF71..=0xFF75 => {} // Unused
            0xFF76..=0xFF77 => {} // Audio digital output (read-only)
            0xFF78..=0xFF7F => {} // Unused
            0xFF80..=0xFFFE => self.hram[addr as usize - 0xFF80] = value,

            0xFFFF => self.interrupt_enable = value,
            _ => unreachable!()
        };
    }

    fn perform_vram_dma(&mut self) -> u32 {
        match self.hdma_status {
            DMAType::NoDMA => 0,
            DMAType::GDMA => self.perform_gdma(),
            DMAType::HDMA => self.perform_hdma(),
        }
    }

    fn perform_gdma(&mut self) -> u32 {
        let len = self.hdma_len as u32 + 1;
        for _ in 0..len {
            self.perform_vram_dma_row();
        }

        self.hdma_status = DMAType::NoDMA;
        len * 8
    }

    fn perform_hdma(&mut self) -> u32 {
        if !self.ppu.hblank {
            return 0;
        }

        self.perform_vram_dma_row();
        if self.hdma_len == 0x7F {
            self.hdma_status = DMAType::NoDMA;
        }
        8
    }

    fn perform_vram_dma_row(&mut self) {
        let mmu_src = self.hdma_src;
        for i in 0..0x10 {
            let byte = self.read_byte(mmu_src + i);
            self.write_byte(self.hdma_dst + i, byte);
        }
        self.hdma_src += 0x10;
        self.hdma_dst += 0x10;

        if self.hdma_len == 0 {
            self.hdma_len = 0x7F;
        } else {
            self.hdma_len = self.hdma_len.wrapping_sub(1);
        }
    }
}