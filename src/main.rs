use crate::cartridge::Cartridge;
use crate::cpu::CPU;
use crate::ui::{Pane, TreeManager};
use egui::{Button, CentralPanel, Spacing, Stroke, TopBottomPanel, Widget};
use std::fs::File;
use std::io::Read;
use std::ops::BitAndAssign;
use eframe::epaint::Color32;
use egui::debug_text::print;
use egui_tiles::{Container, Linear, LinearDir, Tile, Tiles};
use crate::cartridge::licensee::Licensee;
use crate::ui::windows::{Breakpoints, Disassembly, GameWindow, MemoryDump, Registers, TileMapViewer};

mod cpu;
mod mmu;
mod io;
mod ppu;
mod gbmode;
mod mbc;
mod cartridge;
mod disassembler;
mod ui;
mod assembler;

#[inline(always)]
pub fn bit(condition: bool) -> u8 {
    if condition { 1 } else { 0 }
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1600.0, 900.0]),
        ..Default::default()
    };
    eframe::run_native(
        "GameBoy Debugger",
        options,
        Box::new(|cc| {
            Ok(Box::new(Application::new(cc, None)))
        }),
    ).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
    });
}

struct Application {
    tree: egui_tiles::Tree<Pane>,
    tree_manager: TreeManager,
}

impl Application {
    pub fn new(cc: &eframe::CreationContext<'_>, cpu: Option<Box<CPU>>) -> Self {
        setup_fonts(&cc.egui_ctx);
        set_theme(&cc.egui_ctx);
        // catppuccin_egui::set_theme(&cc.egui_ctx, catppuccin_egui::MOCHA);
        let manager = TreeManager::new(cc, cpu);
        // let mut tree = DockState::new(vec!["Memory Dump".to_owned(), "Tile Map Viewer".to_owned()]);
        // let [_, _] = tree.main_surface_mut().split_left(NodeIndex::root(), 0.585, vec!["Disassembly".to_owned()]);
        // let [_, b] = tree.main_surface_mut().split_left(NodeIndex::root(), 0.208, vec!["Game Window".to_owned()]);
        // let [_, c] = tree.main_surface_mut().split_below(b, 0.37, vec!["Breakpoints".to_owned()]);
        // let [_, _] = tree.main_surface_mut().split_below(c, 0.39, vec!["Registers".to_owned()]);
        let mut tiles = Tiles::default();

        let game_window = tiles.insert_pane(Pane::GameWindow(GameWindow::new()));
        let breakpoints = tiles.insert_pane(Pane::Breakpoints(Breakpoints::new()));
        let registers = tiles.insert_pane(Pane::Registers(Registers::new()));
        let disassembly = tiles.insert_pane(Pane::Disassembly(Disassembly::new(&manager.state)));
        let memory_dump = tiles.insert_pane(Pane::MemoryDump(MemoryDump::new()));
        let tile_map_viewer = tiles.insert_pane(Pane::TileMapViewer(TileMapViewer::new(&cc.egui_ctx)));

        let mut left_inner = Linear {
            children: vec![game_window, breakpoints, registers],
            dir: LinearDir::Vertical,
            ..Default::default()
        };
        left_inner.shares.set_share(game_window, 0.395);
        left_inner.shares.set_share(breakpoints, 0.305);
        left_inner.shares.set_share(registers, 0.3);
        let left = tiles.insert_new(Tile::Container(Container::Linear(left_inner)));

        let right_tabs = tiles.insert_tab_tile(vec![memory_dump, tile_map_viewer]);
        let mut inner_right = Linear {
            children: vec![disassembly, right_tabs],
            dir: LinearDir::Horizontal,
            ..Default::default()
        };
        inner_right.shares.set_share(disassembly, 0.58);
        inner_right.shares.set_share(right_tabs, 0.42);
        let right = tiles.insert_new(Tile::Container(Container::Linear(inner_right)));

        let mut root_inner = Linear {
            children: vec![left, right],
            dir: LinearDir::Horizontal,
            ..Default::default()
        };
        root_inner.shares.set_share(left, 0.205);
        root_inner.shares.set_share(right, 0.795);
        let root = tiles.insert_new(Tile::Container(Container::Linear(root_inner)));

        Self {
            tree: egui_tiles::Tree::new("tree", root, tiles),
            tree_manager: manager,
        }
    }

    pub fn open_dialog(&mut self, ctx: &egui::Context) {
        if let Ok(Some(path)) = native_dialog::FileDialog::new()
            .set_title("Open ROM")
            .add_filter("GameBoy ROM", &["gb", "gbc"])
            .show_open_single_file() {
            let mut file = File::open(path).unwrap();
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).unwrap();

            let cartridge = Cartridge::new(buffer);
            let mut title = format!("GameBoy Debugger | {}", cartridge.get_title());
            if let Some(licensee) = cartridge.get_licensee() {
                title += &format!(" | {}", licensee);
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));

            self.tree_manager.state.cpu = Some(Box::new(CPU::new(cartridge, false)));
            self.tree_manager.state.disassembly = disassembler::disassemble(&self.tree_manager.state.cpu.as_ref().unwrap());
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::O) && i.modifiers.ctrl) {
            self.open_dialog(ctx);
        }

        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            let mut style = ui.style_mut();
            style.visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
            style.visuals.widgets.hovered.bg_stroke = Stroke::new(0.0, Color32::TRANSPARENT);
            style.visuals.widgets.hovered.weak_bg_fill = Color32::TRANSPARENT;

            ui.horizontal(|ui| {
                ui.add_space(5.0);
                ui.menu_button("File", |ui| {
                    ui.set_width(200.0);
                    if ui.button("Open ROM        (Ctrl+O)").clicked() {
                        ui.close_menu();
                        self.open_dialog(ctx);
                    }
                });
                ui.menu_button("Debug", |ui| {
                    ui.set_width(200.0);
                    if ui.button("Disassemble").clicked() {
                        ui.close_menu();
                        if let Some(cpu) = &self.tree_manager.state.cpu {
                            self.tree_manager.state.disassembly = disassembler::disassemble_extra(cpu, self.tree_manager.state.jp_hl_targets.clone());
                        }
                    }
                });
            });
        });
        CentralPanel::default().show(ctx, |ui| {
            self.tree.ui(&mut self.tree_manager, ui);
        });

        ctx.request_repaint();
    }
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "JetBrainsMono".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../assets/JetBrainsMono.ttf"
        )),
    );

    fonts.families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "JetBrainsMono".to_owned());

    fonts.families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("JetBrainsMono".to_owned());

    ctx.set_fonts(fonts);
}

fn set_theme(ctx: &egui::Context) {
    ctx.style_mut(|style| {
        // style.visuals.override_text_color = Some(egui::Color32::from_rgb(0, 0, 0));
        // style.visuals.hyperlink_color = egui::Color32::from_rgb(0, 0, 255);
        // style.visuals.faint_bg_color = egui::Color32::from_rgb(0, 0, 0);
        // style.visuals.extreme_bg_color = egui::Color32::from_rgb(28, 33, 35);
        // style.visuals.code_bg_color = egui::Color32::from_rgb(0, 0, 0);
        // style.visuals.warn_fg_color = egui::Color32::from_rgb(255, 0, 0);
        // style.visuals.error_fg_color = egui::Color32::from_rgb(255, 0, 0);
        // style.visuals.window_fill = egui::Color32::from_rgb(28, 33, 35);
        // style.visuals.panel_fill = egui::Color32::from_rgb(28, 33, 35);
        // style.visuals.window_stroke = egui::Stroke {
        //     width: 1.0,
        //     color: egui::Color32::from_rgb(0, 0, 0),
        // };
        // style.visuals.selection.bg_fill = egui::Color32::from_rgba_premultiplied(0, 0, 255, 128);
        // style.visuals.selection.stroke = egui::Stroke {
        //     width: 1.0,
        //     color: egui::Color32::from_rgb(0, 0, 0),
        // };
        //
        // style.visuals.window_shadow.color = egui::Color32::from_rgb(0, 0, 0);
        // style.visuals.popup_shadow.color = egui::Color32::from_rgb(0, 0, 0);
        // style.visuals.dark_mode = true;
    });
}
