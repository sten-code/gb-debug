use crate::cartridge::Cartridge;
use crate::cpu::CPU;
use crate::ui::{Pane, TreeManager};
use egui::{Button, CentralPanel, TopBottomPanel, Widget};
use std::fs::File;
use std::io::Read;
use std::ops::BitAndAssign;
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
    let mut file = File::open("roms/games/PokemonRed.gb").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let cartridge = Cartridge::new(buffer);
    println!("Title: {}", cartridge.get_title());
    println!("Licensee: {}", cartridge.get_licensee().unwrap_or(Licensee::None));


    let mut title = format!("GameBoy Debugger | {}", cartridge.get_title());
    if let Some(licensee) = cartridge.get_licensee() {
        title += &format!(" | {}", licensee);
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1600.0, 900.0]),
        ..Default::default()
    };
    eframe::run_native(
        &title,
        options,
        Box::new(|cc| {
            let cpu = Box::new(CPU::new(cartridge));
            Ok(Box::new(Application::new(cc, cpu)))
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
    fn new(cc: &eframe::CreationContext<'_>, cpu: Box<CPU>) -> Self {
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
        left_inner.shares.set_share(game_window, 0.37);
        left_inner.shares.set_share(breakpoints, 0.33);
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
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("egui_dock::MenuBar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let run_btn = Button::new(if self.tree_manager.state.running { "Stop" } else { "Run" })
                    .frame(false)
                    .min_size([50.0, 0.0].into())
                    .ui(ui);
                if run_btn.clicked() {
                    self.tree_manager.state.running = !self.tree_manager.state.running;
                    self.tree_manager.state.cycles_elapsed_in_frame += self.tree_manager.state.step() as usize;
                }

                let step_btn = Button::new("Step")
                    .frame(false)
                    .min_size([50.0, 0.0].into())
                    .ui(ui);
                if step_btn.clicked() {
                    self.tree_manager.state.cycles_elapsed_in_frame += self.tree_manager.state.step() as usize;
                }

                // let reset_btn = Button::new("Reset")
                //     .frame(false)
                //     .min_size([50.0, 0.0].into())
                //     .ui(ui);
                // if reset_btn.clicked() {
                //     self.context.cpu.reset();
                //     self.context.disassembly = disassemble(&self.context.cpu);
                // }

                let disassemble_btn = Button::new("Disassemble")
                    .frame(false)
                    .min_size([50.0, 0.0].into())
                    .ui(ui);
                if disassemble_btn.clicked() {
                    for tile in self.tree.tiles.iter_mut() {
                        if let Tile::Pane(pane) = tile.1 {
                            if let Pane::Disassembly(disassembly) = pane {
                                disassembly.disassemble(&self.tree_manager.state);
                            }
                        }
                    }
                }
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