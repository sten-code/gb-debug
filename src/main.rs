use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::time::Instant;
use eframe::emath::Align;
use eframe::epaint::{Color32, Rounding, Stroke, TextureHandle};
use eframe::epaint::textures::TextureOptions;
use egui::{Button, CentralPanel, Frame, Layout, RichText, ScrollArea, Sense, TextStyle, TopBottomPanel, Ui, Widget, WidgetInfo, WidgetText, WidgetType};
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer};
use crate::cartridge::Cartridge;
use crate::cpu::CPU;
use crate::cpu::instruction::Instruction;
use crate::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};

mod cpu;
mod mmu;
mod io;
mod ppu;
mod gbmode;
mod mbc;
mod cartridge;
mod disassembler;

const ONE_SECOND_IN_MICROS: usize = 1000000000;
const ONE_SECOND_IN_CYCLES: usize = 4190000;
const ONE_FRAME_IN_CYCLES: usize = 70224;

#[inline(always)]
pub fn bit(condition: bool) -> u8 {
    if condition { 1 } else { 0 }
}

fn main() {
    let mut file = File::open("roms/games/PokemonRed.gb").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let cartridge = Cartridge::new(buffer);
    let mut cpu = CPU::new(cartridge);
    while cpu.registers.pc != 0x0100 {
        cpu.step();
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Disassembler",
        options,
        Box::new(|cc| {
            Ok(Box::new(Application::new(cc, cpu)))
        }),
    ).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
    });
}

struct Context {
    disassembly: Vec<(u16, Option<Instruction>, String)>,
    cpu: CPU,
    texture: TextureHandle,
    cycles_elapsed_in_frame: usize,
    now: Instant,
    breakpoints: Vec<u16>,
    running: bool,
    should_scroll_disasm: bool,
    should_scroll_dump: bool,
    focussed_address: u16,
    show_message_box: bool,
    breakpoint_address_input: String,
}

impl TabViewer for Context {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.as_str().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab.as_str() {
            "Game Window" => self.game_window(ui),
            "Disassembly" => self.disassembly(ui),
            "Breakpoints" => self.breakpoints(ui),
            "Registers" => self.registers(ui),
            "Memory Dump" => self.memory_dump(ui),
            _ => {}
        }
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }
}

impl Context {
    fn step(&mut self) -> u8 {
        let cycles_elapsed = self.cpu.step();
        self.should_scroll_disasm = true;
        self.should_scroll_dump = true;
        self.focussed_address = self.cpu.registers.pc;

        // let state = self.cpu.export_state();
        // const PATH: &str = "C:\\Users\\stent\\Desktop\\Programming\\Rust\\gameboy-rboy\\state.txt";
        // let mut file = File::open(PATH).unwrap();
        // let mut buffer = String::new();
        // file.read_to_string(&mut buffer).unwrap();
        // println!("{}", buffer);
        // println!("{}", state);
        // println!("{}", buffer == state);
        //
        // if buffer == state {
        //     fs::remove_file(PATH).unwrap();
        // } else {
        //     self.running = false;
        // }

        cycles_elapsed
    }

    fn game_window(&mut self, ui: &mut Ui) {
        let input = ui.ctx().input(|i| i.clone());
        self.cpu.mmu.joypad.up = input.key_down(egui::Key::ArrowUp);
        self.cpu.mmu.joypad.down = input.key_down(egui::Key::ArrowDown);
        self.cpu.mmu.joypad.left = input.key_down(egui::Key::ArrowLeft);
        self.cpu.mmu.joypad.right = input.key_down(egui::Key::ArrowRight);
        self.cpu.mmu.joypad.a = input.key_down(egui::Key::Z);
        self.cpu.mmu.joypad.b = input.key_down(egui::Key::X);
        self.cpu.mmu.joypad.start = input.key_down(egui::Key::Enter);
        self.cpu.mmu.joypad.select = input.key_down(egui::Key::Space);

        if self.running {
            let time_delta = self.now.elapsed().subsec_nanos();
            self.now = Instant::now();
            let delta = time_delta as f64 / ONE_SECOND_IN_MICROS as f64;
            let cycles_to_run = delta * ONE_SECOND_IN_CYCLES as f64;

            let mut cycles_elapsed = 0;
            while cycles_elapsed <= cycles_to_run as usize {
                if self.breakpoints.contains(&self.cpu.registers.pc) || !self.running {
                    self.running = false;
                    self.cycles_elapsed_in_frame += cycles_elapsed;
                    break;
                }
                cycles_elapsed += self.step() as usize;
            }
            self.cycles_elapsed_in_frame += cycles_elapsed;
        }

        // Render the frame to a texture
        if self.cycles_elapsed_in_frame >= ONE_FRAME_IN_CYCLES {
            let color_image = egui::ColorImage::from_rgb([SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize], &self.cpu.mmu.ppu.screen_buffer);
            self.texture.set(color_image, TextureOptions::default());
            self.cycles_elapsed_in_frame = 0;
        }

        ui.image(&self.texture);
    }

    fn disassembly(&mut self, ui: &mut egui::Ui) {
        let index = self.disassembly.iter().position(|(addr, _, _)| *addr == self.cpu.registers.pc).unwrap_or(0);
        // if let Some(index) = self.disassembly.iter().position(|(addr, _)| *addr == self.cpu.registers.pc) {
        let visible_height = ui.available_height(); // Visible height of the ScrollArea
        ScrollArea::vertical()
            .auto_shrink(false)
            .animated(false)
            .drag_to_scroll(false)
            .show(ui, |ui| {
                // println!("Count: {}", self.disassembly.len());
                for (addr, instruction, line) in self.disassembly.iter().skip(index.saturating_sub(100)).take(200) {
                    let text = if *addr == self.cpu.registers.pc {
                        format!("> {:04X} {}", *addr, line)
                    } else {
                        format!("  {:04X} {}", *addr, line)
                    };


                    // ui.label(text);
                    // continue;

                    if *addr == self.cpu.registers.pc {
                        let galley = WidgetText::from(RichText::new(text).color(
                            if self.breakpoints.contains(addr) {
                                Color32::LIGHT_RED
                            } else {
                                Color32::LIGHT_GREEN
                            }
                        )).into_galley(ui, None, ui.available_width(), TextStyle::Button);
                        let (rect, response) = ui.allocate_at_least(galley.size(), Sense::click());

                        if self.should_scroll_disasm && (!ui.is_rect_visible(response.rect) || response.rect.top() < 52.0) {
                            ui.scroll_to_rect(response.rect, Some(Align::TOP));
                        }

                        response.widget_info(|| {
                            WidgetInfo::selected(
                                WidgetType::Label,
                                ui.is_enabled(),
                                false,
                                galley.text(),
                            )
                        });

                        let text_pos = ui.layout().align_size_within_rect(galley.size(), rect.shrink2([0.0, 0.0].into())).min;
                        let visuals = ui.style().interact_selectable(&response, false);
                        ui.painter().galley(text_pos, galley, visuals.text_color());
                    } else if self.breakpoints.contains(addr) {
                        ui.label(RichText::new(text).color(Color32::LIGHT_RED));
                    } else {
                        ui.label(text);
                    }
                }
            });
        // }

        self.should_scroll_disasm = false;
    }

    fn breakpoints(&mut self, ui: &mut Ui) {
        let mut deletion = Vec::new();
        for bp in self.breakpoints.iter() {
            ui.horizontal(|ui| {
                if ui.button("Remove").clicked() {
                    deletion.push(*bp);
                }

                ui.label(format!("{:04X}", bp));
            });
        }
        self.breakpoints.retain(|x| !deletion.contains(x));
        if ui.button("Add Breakpoint").clicked() {
            self.show_message_box = true;
            self.breakpoint_address_input = format!("{:04x}", self.cpu.registers.pc);
        }

        if self.show_message_box {
            ui.ctx().show_viewport_immediate(
                egui::ViewportId::from_hash_of("breakpoint_message_box"),
                egui::ViewportBuilder::default()
                    .with_title("Breakpoint")

                    .with_inner_size([300.0, 100.0]),
                |ctx, class| {
                    CentralPanel::default().show(ctx, |ui| {
                        ui.label("The address of the breakpoint:");
                        ui.text_edit_singleline(&mut self.breakpoint_address_input);

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if Button::new("Add").min_size([50.0, 0.0].into()).ui(ui).clicked() {
                                if let Ok(addr) = u16::from_str_radix(&self.breakpoint_address_input, 16) {
                                    self.breakpoints.push(addr);
                                    self.breakpoint_address_input.clear();
                                    self.show_message_box = false;
                                }
                            }
                            if Button::new("Close").min_size([50.0, 0.0].into()).ui(ui).clicked() {
                                self.breakpoint_address_input.clear();
                                self.show_message_box = false;
                            }
                        });
                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_message_box = false;
                    }
                });
        }
    }

    fn registers(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("A:");
                ui.label("B:");
                ui.label("C:");
                ui.label("D:");
                ui.label("E:");
                ui.label("F:");
                ui.label("H:");
                ui.label("L:");
                ui.label("SP:");
                ui.label("PC:");
                ui.label("Z:");
                ui.label("N:");
                ui.label("H:");
                ui.label("C:");
            });

            ui.vertical(|ui| {
                ui.label(format!("{:02X}", self.cpu.registers.a));
                ui.label(format!("{:02X}", self.cpu.registers.b));
                ui.label(format!("{:02X}", self.cpu.registers.c));
                ui.label(format!("{:02X}", self.cpu.registers.d));
                ui.label(format!("{:02X}", self.cpu.registers.e));
                ui.label(format!("{:02X}", u8::from(self.cpu.registers.f)));
                ui.label(format!("{:02X}", self.cpu.registers.h));
                ui.label(format!("{:02X}", self.cpu.registers.l));
                ui.label(format!("{:04X}", self.cpu.registers.sp));
                ui.label(format!("{:04X}", self.cpu.registers.pc));
                ui.label(format!("{}", bit(self.cpu.registers.f.zero)));
                ui.label(format!("{}", bit(self.cpu.registers.f.subtract)));
                ui.label(format!("{}", bit(self.cpu.registers.f.half_carry)));
                ui.label(format!("{}", bit(self.cpu.registers.f.carry)));
            });

            ui.vertical(|ui| {
                ui.label(format!("{}", self.cpu.registers.a));
                ui.label(format!("{}", self.cpu.registers.b));
                ui.label(format!("{}", self.cpu.registers.c));
                ui.label(format!("{}", self.cpu.registers.d));
                ui.label(format!("{}", self.cpu.registers.e));
                ui.label(format!("{}", u8::from(self.cpu.registers.f)));
                ui.label(format!("{}", self.cpu.registers.h));
                ui.label(format!("{}", self.cpu.registers.l));
                ui.label(format!("{}", self.cpu.registers.sp));
                ui.label(format!("{}", self.cpu.registers.pc));
                ui.label(format!("{}", bit(self.cpu.registers.f.zero)));
                ui.label(format!("{}", bit(self.cpu.registers.f.subtract)));
                ui.label(format!("{}", bit(self.cpu.registers.f.half_carry)));
                ui.label(format!("{}", bit(self.cpu.registers.f.carry)));
            });

            ui.vertical(|ui| {
                ui.label(format!("{:0>8b}", self.cpu.registers.a));
                ui.label(format!("{:0>8b}", self.cpu.registers.b));
                ui.label(format!("{:0>8b}", self.cpu.registers.c));
                ui.label(format!("{:0>8b}", self.cpu.registers.d));
                ui.label(format!("{:0>8b}", self.cpu.registers.e));
                ui.label(format!("{:0>8b}", u8::from(self.cpu.registers.f)));
                ui.label(format!("{:0>8b}", self.cpu.registers.h));
                ui.label(format!("{:0>8b}", self.cpu.registers.l));
                ui.label(format!("{:0>8b}", self.cpu.registers.sp));
                ui.label(format!("{:0>8b}", self.cpu.registers.pc));
                ui.label(format!("{}", bit(self.cpu.registers.f.zero)));
                ui.label(format!("{}", bit(self.cpu.registers.f.subtract)));
                ui.label(format!("{}", bit(self.cpu.registers.f.half_carry)));
                ui.label(format!("{}", bit(self.cpu.registers.f.carry)));
            });
        });
    }

    fn memory_dump(&mut self, ui: &mut Ui) {
        const BYTES_PER_LINE: usize = 0x10;
        let start: usize = 0x0000;
        let end: usize = 0xFFFF;
        let focussed_row_addr = self.focussed_address & 0xFFF0;
        ui.horizontal(|ui| {
            ui.label("Addr");
            ui.add_space(5.0);
            for i in 0..BYTES_PER_LINE {
                ui.label(format!("{:02X}", i));
            }
        });
        ui.add_space(5.0);
        ScrollArea::vertical()
            .auto_shrink(false)
            .drag_to_scroll(false)
            .show(ui, |ui| {
                for row_addr in (start..=end).step_by(BYTES_PER_LINE) {
                    let distance = ((row_addr as i64 - focussed_row_addr as i64).abs() / 16) as usize;
                    if distance > 50 {
                        continue;
                    }

                    let bytes = (row_addr..=row_addr + BYTES_PER_LINE - 1).map(|addr| self.cpu.mmu.read_byte(addr as u16)).collect::<Vec<u8>>();

                    ui.horizontal(|ui| {
                        ui.label(format!("{:04X}", row_addr));
                        ui.add_space(5.0);

                        for (i, byte) in bytes.iter().enumerate() {
                            let text = WidgetText::from(format!("{:02X}", byte));
                            let galley = text.into_galley(ui, None, ui.available_width(), TextStyle::Button);

                            let desired_size = galley.size();
                            let (rect, response) = ui.allocate_at_least(desired_size, Sense::click());
                            response.widget_info(|| {
                                WidgetInfo::selected(
                                    WidgetType::SelectableLabel,
                                    ui.is_enabled(),
                                    false,
                                    galley.text(),
                                )
                            });

                            if self.should_scroll_dump && row_addr + i == self.focussed_address as usize && !ui.is_rect_visible(response.rect) {
                                ui.scroll_to_rect(response.rect, Some(Align::Center));
                            }

                            if ui.is_rect_visible(response.rect) {
                                let text_pos = ui
                                    .layout()
                                    .align_size_within_rect(galley.size(), rect.shrink2([0.0, 0.0].into()))
                                    .min;

                                let visuals = ui.style().interact_selectable(&response, false);

                                if response.hovered() || response.highlighted() || response.has_focus() {
                                    let rect = rect.expand(visuals.expansion);

                                    ui.painter().rect(
                                        rect,
                                        Rounding::default(),
                                        visuals.weak_bg_fill,
                                        Stroke::default(),
                                    );

                                    ui.painter().galley(text_pos, galley, visuals.text_color());
                                } else if row_addr + i == self.focussed_address as usize {
                                    let rect = rect.expand(visuals.expansion);
                                    ui.painter().rect(
                                        rect,
                                        Rounding::default(),
                                        Color32::LIGHT_GREEN,
                                        Stroke::default(),
                                    );

                                    ui.painter().galley(text_pos, galley, Color32::DARK_GRAY);
                                } else {
                                    ui.painter().galley(text_pos, galley, visuals.text_color());
                                }
                            }
                        }
                    });
                    // for _ in 0..(BYTES_PER_LINE - chunk.len()) {
                    //     print!("   ");
                    // }
                    // print!(" |");
                    // for byte in &bytes {
                    //     if byte.is_ascii_graphic() || byte.is_ascii_whitespace() {
                    //         print!("{}", *byte as char);
                    //     } else {
                    //         print!(".");
                    //     }
                    // }
                    // println!("|");
                }
            });
        self.should_scroll_dump = false;
    }
}

struct Application {
    context: Context,
    tree: DockState<String>,
}

impl Application {
    fn new(cc: &eframe::CreationContext<'_>, cpu: CPU) -> Self {
        setup_fonts(&cc.egui_ctx);
        let buffer = [0u8, 0u8, 0u8, 255u8].iter().cloned().cycle().take(SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize * 4).collect::<Vec<u8>>();
        let color_image = egui::ColorImage::from_rgba_unmultiplied([SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize], &buffer);
        let texture = cc.egui_ctx.load_texture("color_buffer", color_image, TextureOptions::default());

        let mut tree = DockState::new(vec!["Memory Dump".to_owned()]);
        let [_, _] = tree.main_surface_mut().split_left(NodeIndex::root(), 0.55, vec!["Disassembly".to_owned()]);
        let [_, b] = tree.main_surface_mut().split_left(NodeIndex::root(), 0.21, vec!["Game Window".to_owned()]);
        let [_, c] = tree.main_surface_mut().split_below(b, 0.26, vec!["Breakpoints".to_owned()]);
        let [_, _] = tree.main_surface_mut().split_below(c, 0.39, vec!["Registers".to_owned()]);

        let context = Context {
            disassembly: disassembler::disassemble(&cpu),
            cpu,
            texture,
            cycles_elapsed_in_frame: 0,
            now: Instant::now(),
            breakpoints: vec![],
            running: false,
            should_scroll_disasm: true,
            should_scroll_dump: true,
            focussed_address: 0x0000,
            show_message_box: false,
            breakpoint_address_input: String::new(),
        };

        Self {
            context,
            tree,
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("egui_dock::MenuBar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let run_btn = Button::new(if self.context.running { "Stop" } else { "Run" })
                    .frame(false)
                    .min_size([50.0, 0.0].into())
                    .ui(ui);
                if run_btn.clicked() {
                    self.context.running = !self.context.running;
                    self.context.cycles_elapsed_in_frame += self.context.step() as usize;
                }

                let step_btn = Button::new("Step")
                    .frame(false)
                    .min_size([50.0, 0.0].into())
                    .ui(ui);
                if step_btn.clicked() {
                    self.context.cycles_elapsed_in_frame += self.context.step() as usize;
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
                    self.context.disassembly = disassembler::disassemble(&self.context.cpu);
                }
            });
        });
        CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                DockArea::new(&mut self.tree)
                    .show_inside(ui, &mut self.context);
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

