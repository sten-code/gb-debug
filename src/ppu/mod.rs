use std::cmp::Ordering;
use crate::gbmode::GbMode;

#[inline(always)]
fn bit(value: bool, position: u8) -> u8 {
    if value { 1 << position } else { 0 }
}

#[inline(always)]
fn is_set(byte: u8, position: u8) -> bool {
    (byte >> position) & 1 == 1
}

pub const SCREEN_WIDTH: u8 = 160;
pub const SCREEN_HEIGHT: u8 = 144;

#[derive(Copy, Clone, PartialEq)]
enum PriorityType {
    Color0,
    PriorityFlag,
    Normal,
}

pub struct PPU {
    pub vram: [[u8; 0x2000]; 2], // Video RAM, 2 banks of 0x2000 bytes
    oam: [u8; 0xA0], // Object Attribute Memory
    pub selected_vram_bank: bool, // 0 or 1
    lcd_on: bool,
    win_tilemap: u16,
    win_enabled: bool,
    pub tile_data_addr: u16,
    pub bg_tilemap_addr: u16,
    sprite_size: u8,
    sprite_enabled: bool,
    bg_enabled: bool,

    lyc_interrupt: bool,
    mode_2_interrupt: bool,
    mode_1_interrupt: bool,
    mode_0_interrupt: bool,

    ly: u8, // LCD Y-Coordinate, the current horizontal line being drawn, range 0-153, 144-153 are the VBlank period: https://gbdev.io/pandocs/STAT.html#ff44--ly-lcd-y-coordinate-read-only
    lyc: u8, // LY Compare, when LY == LYC, the STAT interrupt is triggered if enabled: https://gbdev.io/pandocs/STAT.html#ff45--lyc-ly-compare
    mode: u8, // 0: HBlank, 1: VBlank, 2: OAM Search, 3: Pixel Transfer: https://gbdev.io/pandocs/Rendering.html#ppu-modes
    pub scy: u8, // aka Viewport Y or Scroll Y
    pub scx: u8, // aka Viewport X or Scroll X

    // https://gbdev.io/pandocs/Palettes.html#lcd-monochrome-palettes
    pub bg_palette: u8, // (BGP) Background Palette Data, DMG only
    pub obj_palette0: u8, // (OBP0) Object Palette 0 Data, DMG only
    pub obj_palette1: u8, // (OBP1) Object Palette 1 Data, DMG only

    winy: u8, // Window Y Position: https://gbdev.io/pandocs/Scrolling.html#ff4aff4b--wy-wx-window-y-position-x-position-plus-7
    winx: u8, // Window X Position + 7

    // https://gbdev.io/pandocs/Palettes.html#lcd-color-palettes-cgb-only
    cbg_palette_auto_increment: bool,
    cbg_palette_index: u8, // (BGPI) Background palette index
    pub cbg_palette: [[[u8; 3]; 4]; 8], // (BGPD) Background palette data
    cobj_palette_auto_increment: bool,
    cobj_palette_index: u8, // (OBPI) Object palette index
    pub cobj_palette: [[[u8; 3]; 4]; 8], // (OBPD) Object palette Data

    wy_trigger: bool,
    pub wy_pos: i16,
    pub interrupt: u8,
    pub hblank: bool, // True if the PPU is in HBlank mode
    dots: u16, // Number of cycles since the last mode change

    pub screen_buffer: [u8; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize * 3],
    pub screen_buffer_updated: bool,
    bg_priority: [PriorityType; SCREEN_WIDTH as usize],
    gb_mode: GbMode,
}

impl PPU {
    pub fn new(gb_mode: GbMode) -> PPU {
        PPU {
            vram: [[0; 0x2000]; 2],
            oam: [0; 0xA0],
            selected_vram_bank: false,
            lcd_on: false,
            win_tilemap: 0x9C00,
            win_enabled: false,
            tile_data_addr: 0x8000,
            bg_tilemap_addr: 0x9C00,
            sprite_size: 8,
            sprite_enabled: false,
            bg_enabled: false,

            lyc_interrupt: false,
            mode_2_interrupt: false,
            mode_1_interrupt: false,
            mode_0_interrupt: false,

            ly: 0,
            lyc: 0,
            mode: 0,
            scy: 0,
            scx: 0,
            bg_palette: 0,
            obj_palette0: 0,
            obj_palette1: 0,
            winy: 0,
            winx: 0,

            cbg_palette_auto_increment: false,
            cbg_palette_index: 0,
            cbg_palette: [[[0; 3]; 4]; 8],

            cobj_palette_auto_increment: false,
            cobj_palette_index: 0,
            cobj_palette: [[[0; 3]; 4]; 8],

            wy_trigger: false,
            wy_pos: 0,
            interrupt: 0,
            hblank: false,
            dots: 0,

            screen_buffer: [0; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize * 3],
            screen_buffer_updated: false,
            bg_priority: [PriorityType::Normal; SCREEN_WIDTH as usize],
            gb_mode,
        }
    }

    pub fn step(&mut self, cycles: u32) {
        if !self.lcd_on {
            return;
        }
        self.hblank = false;

        // https://gbdev.io/pandocs/Rendering.html#ppu-modes
        let mut cycles_left = cycles;
        while cycles_left > 0 {
            // Taking more than 80 dots in 1 step, risks skipping mode 2
            let current_cycles = cycles_left.min(80);
            self.dots += current_cycles as u16;
            cycles_left -= current_cycles;

            // If 1 full horizontal line is done, go to the next line
            if self.dots >= 456 {
                self.dots -= 456;
                self.ly = (self.ly + 1) % 154;
                self.check_lyc_interrupt();

                // If this is a VBlank line, go into VBlank mode
                if self.ly >= 144 && self.mode != 1 {
                    self.change_mode(1);
                }
            }

            // Update the PPU mode based on the current line and dots
            if self.ly < 144 {
                if self.dots <= 80 {
                    if self.mode != 2 {
                        self.change_mode(2);
                    }
                } else if self.dots <= 252 { // 80 + 172
                    if self.mode != 3 {
                        self.change_mode(3);
                    }
                } else {
                    if self.mode != 0 {
                        self.change_mode(0);
                    }
                }
            }
        }
    }

    fn check_lyc_interrupt(&mut self) {
        if self.lyc_interrupt && self.ly == self.lyc {
            self.interrupt |= bit(true, 1); // Cause the LCD interrupt handler to be called
        }
    }

    pub fn get_monochrome_palette_color(value: u8, index: u8) -> u8 {
        match (value >> 2 * index) & 0x03 {
            0 => 255,
            1 => 192,
            2 => 96,
            _ => 0
        }
    }

    fn change_mode(&mut self, mode: u8) {
        assert!(mode <= 3, "Mode must be 0-3");
        self.mode = mode;

        match self.mode {
            0 => {
                self.render_scanline();
                self.hblank = true;
                if self.mode_0_interrupt {
                    self.interrupt |= bit(true, 1);
                }
            }
            1 => {
                self.wy_trigger = false;
                self.interrupt |= bit(true, 0);
                if self.mode_1_interrupt {
                    self.interrupt |= bit(true, 1);
                }
                self.screen_buffer_updated = true;
            }
            2 => if self.mode_2_interrupt {
                self.interrupt |= bit(true, 1);
            }
            3 => {
                if self.win_enabled && !self.wy_trigger && self.ly == self.winy {
                    self.wy_trigger = true;
                    self.wy_pos = -1;
                }
            }
            _ => unreachable!()
        }
    }

    fn render_scanline(&mut self) {
        for x in 0..SCREEN_WIDTH {
            self.set_color(x, 255);
        }
        self.draw_bg();
        self.draw_sprites();
    }

    fn clear_screen(&mut self) {
        for v in self.screen_buffer.iter_mut() {
            *v = 255;
        }
    }

    fn set_color(&mut self, x: u8, color: u8) {
        self.screen_buffer[self.ly as usize * SCREEN_WIDTH as usize * 3 + x as usize * 3 + 0] = color;
        self.screen_buffer[self.ly as usize * SCREEN_WIDTH as usize * 3 + x as usize * 3 + 1] = color;
        self.screen_buffer[self.ly as usize * SCREEN_WIDTH as usize * 3 + x as usize * 3 + 2] = color;
    }

    fn set_rgb(&mut self, x: u8, r: u8, g: u8, b: u8) {
        let index = self.ly as usize * SCREEN_WIDTH as usize * 3 + x as usize * 3;

        // RGB555 to RGB888
        self.screen_buffer[index + 0] = ((r as u32 * 13 + g as u32 * 2 + b as u32) >> 1) as u8;
        self.screen_buffer[index + 1] = ((g as u32 * 3 + b as u32) << 1) as u8;
        self.screen_buffer[index + 2] = ((r as u32 * 3 + g as u32 * 2 + b as u32 * 11) >> 1) as u8;
    }

    fn draw_bg(&mut self) {
        let draw_bg = self.gb_mode == GbMode::Color || self.bg_enabled;

        let win_y = if self.win_enabled && self.wy_trigger && self.winx <= 166 {
            self.wy_pos += 1;
            self.wy_pos
        } else {
            -1
        };

        if win_y < 0 && draw_bg == false {
            return;
        }

        let win_tile_y = (win_y as u16 >> 3) & 31;
        let bg_y = self.scy.wrapping_add(self.ly);
        let bg_tile_y = (bg_y as u16 >> 3) & 31;

        for x in 0..SCREEN_WIDTH {
            let win_x = -((self.winx as i16) - 7) + (x as i16);
            let bg_x = self.scx as u16 + x as u16;

            let (tile_map_base_addr, tile_y, tile_x, pixel_y, pixel_x) = if win_y >= 0 && win_x >= 0 {
                (self.win_tilemap,
                 win_tile_y,
                 win_x as u16 / 8,
                 win_y as u16 % 8,
                 win_x as u8 % 8)
            } else if draw_bg {
                (self.bg_tilemap_addr,
                 bg_tile_y,
                 (bg_x / 8) & 31,
                 bg_y as u16 % 8,
                 bg_x as u8 % 8)
            } else {
                continue;
            };

            let tile_num = self.vram[0][tile_map_base_addr as usize - 0x8000 + tile_y as usize * 32 + tile_x as usize];
            let (palette_num, vram1, x_flip, y_flip, priority) = if self.gb_mode == GbMode::Color {
                let flags = self.vram[1][tile_map_base_addr as usize - 0x8000 + tile_y as usize * 32 + tile_x as usize];
                (
                    flags & 0b111,
                    is_set(flags, 3),
                    is_set(flags, 5),
                    is_set(flags, 6),
                    is_set(flags, 7),
                )
            } else {
                (0, false, false, false, false)
            };

            let tileaddress = self.tile_data_addr + (
                if self.tile_data_addr == 0x8000 {
                    tile_num as u16
                } else {
                    (tile_num as i8 as i16 + 128) as u16
                }
            ) * 16;

            let a0 = if y_flip {
                tileaddress + 14 - pixel_y * 2
            } else {
                tileaddress + pixel_y * 2
            };

            let (b1, b2) = if vram1 {
                (self.vram[1][a0 as usize - 0x8000], self.vram[1][a0 as usize - 0x8000])
            } else {
                (self.vram[0][a0 as usize - 0x8000], self.vram[0][a0 as usize - 0x8000 + 1])
            };

            let x_bit = if x_flip {
                pixel_x
            } else {
                7 - pixel_x
            };
            let color_num = bit(is_set(b2, x_bit), 1) | bit(is_set(b1, x_bit), 0);

            self.bg_priority[x as usize] = if color_num == 0 { PriorityType::Color0 } else if priority { PriorityType::PriorityFlag } else { PriorityType::Normal };
            if self.gb_mode == GbMode::Color {
                let r = self.cbg_palette[palette_num as usize][color_num as usize][0];
                let g = self.cbg_palette[palette_num as usize][color_num as usize][1];
                let b = self.cbg_palette[palette_num as usize][color_num as usize][2];
                self.set_rgb(x, r, g, b);
            } else {
                let color = PPU::get_monochrome_palette_color(self.bg_palette, color_num);
                self.set_color(x, color);
            }
        }
    }


    fn draw_sprites(&mut self) {
        if !self.sprite_enabled { return; }

        let line = self.ly as i32;
        let sprite_size = self.sprite_size as i32;

        let mut sprites = [(0, 0, 0); 10];
        let mut sprite_count = 0;
        for index in 0..40 {
            let sprite_addr = (index as u16) * 4;
            let sprite_y = self.read_oam(sprite_addr + 0) as u16 as i32 - 16;
            if line < sprite_y || line >= sprite_y + sprite_size { continue; }
            let sprite_x = self.read_oam(sprite_addr + 1) as u16 as i32 - 8;
            sprites[sprite_count] = (sprite_x, sprite_y, index);
            sprite_count += 1;
            if sprite_count >= 10 {
                break;
            }
        }
        if self.gb_mode == GbMode::Color {
            sprites[..sprite_count].sort_unstable_by(|a, b| b.2.cmp(&a.2));
        } else {
            sprites[..sprite_count].sort_unstable_by(|a, b| if a.0 != b.0 {
                b.0.cmp(&a.0)
            } else {
                b.2.cmp(&a.2)
            });
        }

        for &(sprite_x, sprite_y, i) in &sprites[..sprite_count] {
            if sprite_x < -7 || sprite_x >= (SCREEN_WIDTH as i32) { continue; }

            let sprite_addr = (i as u16) * 4;
            let tile_num = (self.read_oam(sprite_addr + 2) & (if self.sprite_size == 16 { 0xFE } else { 0xFF })) as u16;
            let flags = self.read_oam(sprite_addr + 3) as usize;
            let palette_num = flags & 0x07;
            let vram1: bool = flags & (1 << 3) != 0;
            let use_palette1: bool = flags & (1 << 4) != 0;
            let x_flip: bool = flags & (1 << 5) != 0;
            let y_flip: bool = flags & (1 << 6) != 0;
            let below_bg: bool = flags & (1 << 7) != 0;

            let tile_y: u16 = if y_flip {
                (sprite_size - 1 - (line - sprite_y)) as u16
            } else {
                (line - sprite_y) as u16
            };

            let tile_address = tile_num * 16 + tile_y * 2;
            let (bit1, bit2) = if vram1 && self.gb_mode == GbMode::Color {
                (self.vram[1][tile_address as usize], self.vram[1][tile_address as usize + 1])
            } else {
                (self.vram[0][tile_address as usize], self.vram[0][tile_address as usize + 1])
            };

            for x in 0..8 {
                if sprite_x + x < 0 || sprite_x + x >= (SCREEN_WIDTH as i32) { continue; }

                let x_bit = 1 << (if x_flip { x } else { 7 - x } as u32);
                let color_num = bit(bit2 & x_bit != 0, 1) | bit(bit1 & x_bit != 0, 0);
                if color_num == 0 {
                    continue;
                }

                if self.gb_mode == GbMode::Color {
                    if self.bg_enabled && (self.bg_priority[(sprite_x + x) as usize] == PriorityType::PriorityFlag || (below_bg && self.bg_priority[(sprite_x + x) as usize] != PriorityType::Color0)) {
                        continue;
                    }
                    let r = self.cobj_palette[palette_num][color_num as usize][0];
                    let g = self.cobj_palette[palette_num][color_num as usize][1];
                    let b = self.cobj_palette[palette_num][color_num as usize][2];
                    self.set_rgb((sprite_x + x) as u8, r, g, b);
                } else {
                    if below_bg && self.bg_priority[(sprite_x + x) as usize] != PriorityType::Color0 { continue; }
                    self.set_color((sprite_x + x) as u8, PPU::get_monochrome_palette_color(
                        if use_palette1 { self.obj_palette1 } else { self.obj_palette0 },
                        color_num,
                    ));
                }
            }
        }
    }

    pub fn read_vram(&self, addr: u16) -> u8 {
        assert!(addr <= 0x2000, "VRAM is only 0x2000 bytes long");
        self.vram[self.selected_vram_bank as usize][addr as usize]
    }

    pub fn write_vram(&mut self, addr: u16, value: u8) {
        assert!(addr <= 0x2000, "VRAM is only 0x2000 bytes long");
        self.vram[self.selected_vram_bank as usize][addr as usize] = value;
    }

    pub fn read_oam(&self, addr: u16) -> u8 {
        assert!(addr <= 0xA0, "OAM is only 0xA0 bytes long");
        self.oam[addr as usize]
    }

    pub fn write_oam(&mut self, addr: u16, value: u8) {
        assert!(addr <= 0xA0, "OAM is only 0xA0 bytes long");
        self.oam[addr as usize] = value;
    }

    pub fn read_register(&self, addr: u16) -> u8 {
        assert!((addr >= 0xFF40 && addr <= 0xFF4B) || (addr >= 0xFF68 && addr <= 0xFF6B), "PPU register out of range");
        match addr {
            0xFF40 => bit(self.lcd_on, 7) // https://gbdev.io/pandocs/LCDC.html#ff40--lcdc-lcd-control
                | bit(self.win_tilemap == 0x9C00, 6)
                | bit(self.win_enabled, 5)
                | bit(self.tile_data_addr == 0x8000, 4)
                | bit(self.bg_tilemap_addr == 0x9C00, 3)
                | bit(self.sprite_size == 16, 2)
                | bit(self.sprite_enabled, 1)
                | bit(self.bg_enabled, 0),
            0xFF41 => bit(true, 7) // https://gbdev.io/pandocs/STAT.html#ff41--stat-lcd-status
                | bit(self.lyc_interrupt, 6)
                | bit(self.mode_2_interrupt, 5)
                | bit(self.mode_1_interrupt, 4)
                | bit(self.mode_0_interrupt, 3)
                | bit(self.ly == self.lyc, 2)
                | self.mode,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF46 => 0x00, // DMA, write-only
            0xFF47 => self.bg_palette, // DMG only
            0xFF48 => self.obj_palette0, // DMG only
            0xFF49 => self.obj_palette1, // DMG only
            0xFF4A => self.winy,
            0xFF4B => self.winx,

            // CGB only
            0xFF4F..=0xFF6B if self.gb_mode != GbMode::Color => { 0xFF }
            0xFF68 => bit(self.cbg_palette_auto_increment, 7)
                | bit(true, 6)
                | self.cbg_palette_index,
            0xFF69 => {
                let palette_num = self.cbg_palette_index / 8;
                // https://gbdev.io/pandocs/Palettes.html#ff69--bcpdbgpd-cgb-mode-only-background-color-palette-data--background-palette-data

                // A single rgb color fits in 16 bits, 5 bits for red, 5 bits for green, 5 bits for blue
                // You can't return this all at once, so if the index is even, return the first 8 bits, otherwise return the last 8 bits of these 16 bits.

                // r = 00011111
                // g = 00011111
                // b = 00011111

                let color_num = (self.cbg_palette_index / 2) % 4; // Because we are doing this alternating, we need to divide by 2
                if self.cbg_palette_index % 2 == 0 {
                    // Only the first 3 bits of green
                    self.cbg_palette[palette_num as usize][color_num as usize][0] | ((self.cbg_palette[palette_num as usize][color_num as usize][1] & 0x07) << 5)
                } else {
                    // The last 2 bits of green
                    ((self.cbg_palette[palette_num as usize][color_num as usize][1] & 0x18) >> 3) | (self.cbg_palette[palette_num as usize][color_num as usize][2] << 2)
                }
            }
            0xFF6A => bit(self.cobj_palette_auto_increment, 7)
                | bit(true, 6)
                | self.cobj_palette_index,
            0xFF6B => {
                // Explanation is the same as the background palette
                let palette_num = self.cobj_palette_index / 8;
                let color_num = (self.cobj_palette_index / 2) % 4;
                if self.cobj_palette_index % 2 == 0 {
                    self.cobj_palette[palette_num as usize][color_num as usize][0] | ((self.cobj_palette[palette_num as usize][color_num as usize][1] & 0x07) << 5)
                } else {
                    ((self.cobj_palette[palette_num as usize][color_num as usize][1] & 0x18) >> 3) | (self.cobj_palette[palette_num as usize][color_num as usize][2] << 2)
                }
            }
            _ => unreachable!()
        }
    }

    pub fn write_register(&mut self, addr: u16, value: u8) {
        assert!((addr >= 0xFF40 && addr <= 0xFF4B) || (addr >= 0xFF68 && addr <= 0xFF6B), "PPU register out of range: {:04X}", addr);
        match addr {
            0xFF40 => {
                let orig_lcd_on = self.lcd_on;
                self.lcd_on = is_set(value, 7);
                self.win_tilemap = if is_set(value, 6) { 0x9C00 } else { 0x9800 };
                self.win_enabled = is_set(value, 5);
                self.tile_data_addr = if is_set(value, 4) { 0x8000 } else { 0x8800 };
                self.bg_tilemap_addr = if is_set(value, 3) { 0x9C00 } else { 0x9800 };
                self.sprite_size = if is_set(value, 2) { 16 } else { 8 };
                self.sprite_enabled = is_set(value, 1);
                self.bg_enabled = is_set(value, 0);
                if orig_lcd_on && !self.lcd_on {
                    self.dots = 0;
                    self.ly = 0;
                    self.mode = 0;
                    self.wy_trigger = false;
                    self.clear_screen();
                }
                if !orig_lcd_on && self.lcd_on {
                    self.change_mode(2);
                    self.dots = 4;
                }
            }
            0xFF41 => {
                self.lyc_interrupt = is_set(value, 6);
                self.mode_2_interrupt = is_set(value, 5);
                self.mode_1_interrupt = is_set(value, 4);
                self.mode_0_interrupt = is_set(value, 3);
            }
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => {} // LY is Read-only
            0xFF45 => {
                self.lyc = value;
                self.check_lyc_interrupt();
            }
            0xFF46 => panic!("ppu.write_register(0xFF46, value): DMA should be handled by the MMU"),
            0xFF47 => self.bg_palette = value,
            0xFF48 => self.obj_palette0 = value,
            0xFF49 => self.obj_palette1 = value,
            0xFF4A => self.winy = value,
            0xFF4B => self.winx = value,

            // CGB only
            0xFF4F..=0xFF6B if self.gb_mode != GbMode::Color => {}
            0xFF68 => {
                // https://gbdev.io/pandocs/Palettes.html#ff68--bcpsbgpi-cgb-mode-only-background-color-palette-specification--background-palette-index
                self.cbg_palette_index = value & 0x3F; // The index is the first 6 bits
                self.cbg_palette_auto_increment = is_set(value, 7);
            }
            0xFF69 => {
                // Explanation is in the read_register function
                let palette_num = self.cbg_palette_index / 8;
                let color_num = (self.cbg_palette_index / 2) % 4;
                if self.cbg_palette_index % 2 == 0 {
                    self.cbg_palette[palette_num as usize][color_num as usize][0] = value & 0x1F;
                    self.cbg_palette[palette_num as usize][color_num as usize][1] = (self.cbg_palette[palette_num as usize][color_num as usize][1] & 0x18) | (value >> 5);
                } else {
                    self.cbg_palette[palette_num as usize][color_num as usize][1] = (self.cbg_palette[palette_num as usize][color_num as usize][1] & 0x07) | ((value & 0x03) << 3);
                    self.cbg_palette[palette_num as usize][color_num as usize][2] = (value >> 2) & 0x1F;
                }
                if self.cbg_palette_auto_increment {
                    self.cbg_palette_index = (self.cbg_palette_index + 1) & 0x3F;
                }
            }
            0xFF6A => {
                self.cobj_palette_index = value & 0x3F;
                self.cobj_palette_auto_increment = is_set(value, 7);
            }
            0xFF6B => {
                // Explanation is in the read_register function of the background palette, as it's the same for the objects.
                let palette_num = self.cobj_palette_index / 8;
                let color_num = (self.cobj_palette_index / 2) % 4;
                if self.cobj_palette_index % 2 == 0 {
                    self.cobj_palette[palette_num as usize][color_num as usize][0] = value & 0x1F;
                    self.cobj_palette[palette_num as usize][color_num as usize][1] = (self.cobj_palette[palette_num as usize][color_num as usize][1] & 0x18) | (value >> 5);
                } else {
                    self.cobj_palette[palette_num as usize][color_num as usize][1] = (self.cobj_palette[palette_num as usize][color_num as usize][1] & 0x07) | ((value & 0x03) << 3);
                    self.cobj_palette[palette_num as usize][color_num as usize][2] = (value >> 2) & 0x1F;
                }
                if self.cobj_palette_auto_increment {
                    self.cobj_palette_index = (self.cobj_palette_index + 1) & 0x3F;
                }
            }
            _ => unreachable!()
        }
    }
}
