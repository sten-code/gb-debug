use std::fmt::Display;
use eframe::egui;
use eframe::emath::Pos2;
use eframe::epaint::{Color32, Stroke};
use eframe::epaint::textures::TextureOptions;
use eframe::egui::{ComboBox, Frame, Image, Rect, TextureHandle, Ui, Widget};
use crate::cpu::CPU;
use crate::gbmode::GbMode;
use crate::ppu::PPU;
use crate::ui::State;
use crate::ui::windows::Window;

#[inline(always)]
fn bit(value: bool, position: u8) -> u8 {
    if value { 1 << position } else { 0 }
}

#[inline(always)]
fn is_set(byte: u8, position: u8) -> bool {
    (byte >> position) & 1 == 1
}

pub struct Tile {
    buffer: Vec<u8>,
    raw_buffer: Vec<u8>,
    texture: TextureHandle,
}

#[derive(PartialEq, Debug)]
enum ClassicPalette {
    BGP,
    OBP0,
    OBP1,
}

#[derive(PartialEq, Debug)]
enum ColorPalette {
    BCP0,
    BCP1,
    BCP2,
    BCP3,
    BCP4,
    BCP5,
    BCP6,
    BCP7,
    OCP0,
    OCP1,
    OCP2,
    OCP3,
    OCP4,
    OCP5,
    OCP6,
    OCP7,
}

#[derive(PartialEq, Debug)]
enum SelectedTab {
    Tiles,
    Background,
}

#[derive(PartialEq, Debug)]
enum TileDataAddress {
    Auto,
    X8000,
    X8800,
}

impl Display for TileDataAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TileDataAddress::Auto => write!(f, "Auto"),
            TileDataAddress::X8000 => write!(f, "$8000"),
            TileDataAddress::X8800 => write!(f, "$8800"),
        }
    }
}


#[derive(PartialEq, Debug)]
enum TileMapAddress {
    Auto,
    X9800,
    X9C00,
}

impl Display for TileMapAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TileMapAddress::Auto => write!(f, "Auto"),
            TileMapAddress::X9800 => write!(f, "$9800"),
            TileMapAddress::X9C00 => write!(f, "$9C00"),
        }
    }
}

pub struct TileMapViewer {
    tiles: Vec<Tile>,
    selected_classic_palette: ClassicPalette,
    selected_color_palette: ColorPalette,
    show_grid: bool,
    selected_tab: SelectedTab,
    show_screen_grid: bool,
    tile_data_address: TileDataAddress,
    tile_map_address: TileMapAddress,
}

const TILE_IMAGE_SIZE: f32 = 8.0;
impl TileMapViewer {
    pub fn new(ctx: &egui::Context) -> Self {
        let mut tiles = Vec::new();
        for i in 0..128 * 3 {
            let buffer = [0u8, 0u8, 0u8].iter().cloned().cycle().take(64 * 3).collect::<Vec<u8>>();
            let color_image = egui::ColorImage::from_rgb([8, 8], &buffer);
            let texture = ctx.load_texture(format!("tile_{}", i), color_image, TextureOptions::default());
            tiles.push(Tile {
                buffer,
                raw_buffer: vec![0u8; 64],
                texture,
            });
        }

        Self {
            tiles,
            selected_classic_palette: ClassicPalette::BGP,
            selected_color_palette: ColorPalette::BCP0,
            show_grid: true,
            selected_tab: SelectedTab::Background,
            show_screen_grid: true,
            tile_data_address: TileDataAddress::Auto,
            tile_map_address: TileMapAddress::Auto,
        }
    }

    fn set_pixel(buffer: &mut [u8], index: usize, r: u8, g: u8, b: u8) {
        // RGB555 to RGB888
        buffer[index + 0] = ((r as u32 * 13 + g as u32 * 2 + b as u32) >> 1) as u8;
        buffer[index + 1] = ((g as u32 * 3 + b as u32) << 1) as u8;
        buffer[index + 2] = ((r as u32 * 3 + g as u32 * 2 + b as u32 * 11) >> 1) as u8;
    }

    pub fn update_textures(&mut self, cpu: &mut CPU) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            let address = 0x8000 + (i as u16 * 16);
            for row in 0..8 {
                let byte1 = cpu.mmu.read_byte(address + row * 2);
                let byte2 = cpu.mmu.read_byte(address + row * 2 + 1);
                for pixel in 0..8 {
                    let color_num = ((is_set(byte2, 7 - pixel) as u8) << 1) | (is_set(byte1, 7 - pixel) as u8);
                    tile.raw_buffer[row as usize * 8 + pixel as usize] = color_num;

                    match cpu.get_gb_mode() {
                        GbMode::Color => {
                            let palette = match self.selected_color_palette {
                                ColorPalette::BCP0 => &cpu.mmu.ppu.cbg_palette[0],
                                ColorPalette::BCP1 => &cpu.mmu.ppu.cbg_palette[1],
                                ColorPalette::BCP2 => &cpu.mmu.ppu.cbg_palette[2],
                                ColorPalette::BCP3 => &cpu.mmu.ppu.cbg_palette[3],
                                ColorPalette::BCP4 => &cpu.mmu.ppu.cbg_palette[4],
                                ColorPalette::BCP5 => &cpu.mmu.ppu.cbg_palette[5],
                                ColorPalette::BCP6 => &cpu.mmu.ppu.cbg_palette[6],
                                ColorPalette::BCP7 => &cpu.mmu.ppu.cbg_palette[7],
                                ColorPalette::OCP0 => &cpu.mmu.ppu.cobj_palette[0],
                                ColorPalette::OCP1 => &cpu.mmu.ppu.cobj_palette[1],
                                ColorPalette::OCP2 => &cpu.mmu.ppu.cobj_palette[2],
                                ColorPalette::OCP3 => &cpu.mmu.ppu.cobj_palette[3],
                                ColorPalette::OCP4 => &cpu.mmu.ppu.cobj_palette[4],
                                ColorPalette::OCP5 => &cpu.mmu.ppu.cobj_palette[5],
                                ColorPalette::OCP6 => &cpu.mmu.ppu.cobj_palette[6],
                                ColorPalette::OCP7 => &cpu.mmu.ppu.cobj_palette[7],
                            };

                            TileMapViewer::set_pixel(
                                &mut tile.buffer,
                                row as usize * 8 * 3 + pixel as usize * 3,
                                palette[color_num as usize][0],
                                palette[color_num as usize][1],
                                palette[color_num as usize][2],
                            );
                        }
                        GbMode::Classic => {
                            let color = PPU::get_monochrome_palette_color(match self.selected_classic_palette {
                                ClassicPalette::BGP => cpu.mmu.ppu.bg_palette,
                                ClassicPalette::OBP0 => cpu.mmu.ppu.obj_palette0,
                                ClassicPalette::OBP1 => cpu.mmu.ppu.obj_palette1,
                            }, color_num);

                            tile.buffer[row as usize * 8 * 3 + pixel as usize * 3 + 0] = color;
                            tile.buffer[row as usize * 8 * 3 + pixel as usize * 3 + 1] = color;
                            tile.buffer[row as usize * 8 * 3 + pixel as usize * 3 + 2] = color;
                        }
                    }
                }
            }

            let color_image = egui::ColorImage::from_rgb([8, 8], &tile.buffer);
            tile.texture.set(color_image, TextureOptions::NEAREST_REPEAT);
        }
    }

    pub fn show_tiles(&mut self, state: &mut State, ui: &mut Ui) {
        if let Some(cpu) = &state.cpu {
            ui.horizontal(|ui| {
                ui.add_space(5.0);
                ui.checkbox(&mut self.show_grid, "Show Grid");
                match cpu.get_gb_mode() {
                    GbMode::Classic => ComboBox::from_label("Palette")
                        .selected_text(format!("{:?}", self.selected_classic_palette))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.selected_classic_palette, ClassicPalette::BGP, format!("{:?}", ClassicPalette::BGP));
                            ui.selectable_value(&mut self.selected_classic_palette, ClassicPalette::OBP0, format!("{:?}", ClassicPalette::OBP0));
                            ui.selectable_value(&mut self.selected_classic_palette, ClassicPalette::OBP1, format!("{:?}", ClassicPalette::OBP1));
                        }),
                    GbMode::Color => ComboBox::from_label("Palette")
                        .selected_text(format!("{:?}", self.selected_color_palette))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.selected_color_palette, ColorPalette::BCP0, format!("{:?}", ColorPalette::BCP0));
                            ui.selectable_value(&mut self.selected_color_palette, ColorPalette::BCP1, format!("{:?}", ColorPalette::BCP1));
                            ui.selectable_value(&mut self.selected_color_palette, ColorPalette::BCP2, format!("{:?}", ColorPalette::BCP2));
                            ui.selectable_value(&mut self.selected_color_palette, ColorPalette::BCP3, format!("{:?}", ColorPalette::BCP3));
                            ui.selectable_value(&mut self.selected_color_palette, ColorPalette::BCP4, format!("{:?}", ColorPalette::BCP4));
                            ui.selectable_value(&mut self.selected_color_palette, ColorPalette::BCP5, format!("{:?}", ColorPalette::BCP5));
                            ui.selectable_value(&mut self.selected_color_palette, ColorPalette::BCP6, format!("{:?}", ColorPalette::BCP6));
                            ui.selectable_value(&mut self.selected_color_palette, ColorPalette::BCP7, format!("{:?}", ColorPalette::BCP7));
                            ui.selectable_value(&mut self.selected_color_palette, ColorPalette::OCP0, format!("{:?}", ColorPalette::OCP0));
                            ui.selectable_value(&mut self.selected_color_palette, ColorPalette::OCP1, format!("{:?}", ColorPalette::OCP1));
                            ui.selectable_value(&mut self.selected_color_palette, ColorPalette::OCP2, format!("{:?}", ColorPalette::OCP2));
                            ui.selectable_value(&mut self.selected_color_palette, ColorPalette::OCP3, format!("{:?}", ColorPalette::OCP3));
                            ui.selectable_value(&mut self.selected_color_palette, ColorPalette::OCP4, format!("{:?}", ColorPalette::OCP4));
                            ui.selectable_value(&mut self.selected_color_palette, ColorPalette::OCP5, format!("{:?}", ColorPalette::OCP5));
                            ui.selectable_value(&mut self.selected_color_palette, ColorPalette::OCP6, format!("{:?}", ColorPalette::OCP6));
                            ui.selectable_value(&mut self.selected_color_palette, ColorPalette::OCP7, format!("{:?}", ColorPalette::OCP7));
                        }),
                };
            });

            ui.spacing_mut().item_spacing = [0.0, 0.0].into();
            ui.spacing_mut().interact_size = [0.0, 0.0].into();
            ui.vertical(|ui| {
                for (i, row) in self.tiles.chunks(16).enumerate() {
                    ui.horizontal(|ui| {
                        ui.add_space(5.0);
                        for tile in row {
                            let response = if self.show_grid {
                                Frame::new()
                                    .stroke(Stroke::new(1.0, Color32::BLACK))
                                    .show(ui, |ui| {
                                        Image::new(&tile.texture)
                                            .fit_to_exact_size([16.0, 16.0].into())
                                            .ui(ui);
                                    }).response
                            } else {
                                Image::new(&tile.texture)
                                    .fit_to_exact_size([16.0, 16.0].into())
                                    .ui(ui)
                            };
                            if response.hovered() {}
                        }
                    });
                    if i % 8 == 7 {
                        ui.add_space(5.0);
                    }
                }
            });
        }
    }

    pub fn show_background(&mut self, state: &mut State, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add_space(5.0);
            ui.vertical(|ui| {
                ui.checkbox(&mut self.show_grid, "Show Grid");
                ui.checkbox(&mut self.show_screen_grid, "Show Screen Grid");
            });
            ui.add_space(5.0);
            ui.vertical(|ui| {
                ComboBox::from_label("Tile Data Address")
                    .selected_text(self.tile_data_address.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.tile_data_address, TileDataAddress::Auto, TileDataAddress::Auto.to_string());
                        ui.selectable_value(&mut self.tile_data_address, TileDataAddress::X8000, TileDataAddress::X8000.to_string());
                        ui.selectable_value(&mut self.tile_data_address, TileDataAddress::X8800, TileDataAddress::X8800.to_string());
                    });
                ComboBox::from_label("Tile Map Address")
                    .selected_text(self.tile_map_address.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.tile_map_address, TileMapAddress::Auto, TileMapAddress::Auto.to_string());
                        ui.selectable_value(&mut self.tile_map_address, TileMapAddress::X9800, TileMapAddress::X9800.to_string());
                        ui.selectable_value(&mut self.tile_map_address, TileMapAddress::X9C00, TileMapAddress::X9C00.to_string());
                    });
            });
        });

        if let Some(cpu) = &mut state.cpu {
            let x = ui.cursor().min.x + 5.0 + (cpu.mmu.ppu.scx as f32 * (TILE_IMAGE_SIZE / 8.0));
            let y = ui.cursor().min.y + (cpu.mmu.ppu.scy as f32 * (TILE_IMAGE_SIZE / 8.0));
            ui.spacing_mut().item_spacing = [0.0, 0.0].into();
            ui.spacing_mut().interact_size = [0.0, 0.0].into();
            ui.vertical(|ui| {
                for y in 0..32 {
                    ui.horizontal(|ui| {
                        ui.add_space(5.0);
                        for x in 0..32 {
                            let offset = y * 32 + x;
                            let address = match self.tile_map_address {
                                TileMapAddress::Auto => cpu.mmu.ppu.bg_tilemap_addr + offset,
                                TileMapAddress::X9800 => 0x9800 + offset,
                                TileMapAddress::X9C00 => 0x9C00 + offset,
                            };
                            let tile_id = cpu.mmu.read_byte(address);
                            let tile = match self.tile_data_address {
                                TileDataAddress::Auto => {
                                    if cpu.mmu.ppu.tile_data_addr == 0x8000 {
                                        &mut self.tiles[128 + tile_id as usize]
                                    } else {
                                        &mut self.tiles[(256 + tile_id as i8 as i16) as usize]
                                    }
                                }
                                TileDataAddress::X8000 => &mut self.tiles[tile_id as usize],
                                TileDataAddress::X8800 => &mut self.tiles[(128 + tile_id as i8 as i16) as usize],
                            };

                            if cpu.get_gb_mode() == GbMode::Color {
                                let attributes = cpu.mmu.ppu.vram[1][offset as usize];
                                let palette = attributes & 0b111;
                                for (i, color_num) in tile.raw_buffer.iter().enumerate() {
                                    TileMapViewer::set_pixel(
                                        &mut tile.buffer,
                                        i * 3,
                                        cpu.mmu.ppu.cbg_palette[palette as usize][*color_num as usize][0],
                                        cpu.mmu.ppu.cbg_palette[palette as usize][*color_num as usize][1],
                                        cpu.mmu.ppu.cbg_palette[palette as usize][*color_num as usize][2],
                                    );
                                }
                                tile.texture.set(egui::ColorImage::from_rgb([8, 8], &tile.buffer), TextureOptions::NEAREST);
                            }

                            if self.show_grid {
                                Frame::new()
                                    .stroke(Stroke::new(1.0, Color32::BLACK))
                                    .show(ui, |ui| {
                                        Image::new(&tile.texture)
                                            .fit_to_exact_size([TILE_IMAGE_SIZE, TILE_IMAGE_SIZE].into())
                                            .ui(ui);
                                    });
                            } else {
                                Image::new(&tile.texture)
                                    .fit_to_exact_size([TILE_IMAGE_SIZE, TILE_IMAGE_SIZE].into())
                                    .ui(ui);
                            }
                        }
                    });
                }
            });
            if self.show_screen_grid {
                ui.painter().rect_stroke(Rect::from_min_max(
                    Pos2::new(x, y),
                    Pos2::new(x + 20.0 * TILE_IMAGE_SIZE, y + 18.0 * TILE_IMAGE_SIZE),
                ), 0.0, Stroke::new(1.0, Color32::GREEN), egui::StrokeKind::Middle);
            }
        }
    }
}

impl Window for TileMapViewer {
    fn show(&mut self, state: &mut State, ui: &mut Ui) {
        if let Some(cpu) = &mut state.cpu {
            self.update_textures(cpu);
        }

        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.add_space(5.0);
            ui.selectable_value(&mut self.selected_tab, SelectedTab::Tiles, "Tiles");
            ui.selectable_value(&mut self.selected_tab, SelectedTab::Background, "Background");
        });

        match self.selected_tab {
            SelectedTab::Tiles => self.show_tiles(state, ui),
            SelectedTab::Background => self.show_background(state, ui),
        }
    }
}
