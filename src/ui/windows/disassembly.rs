﻿use crate::disassembler::{disassemble, DisassembledLine, LineType};
use crate::ui::windows::Window;
use crate::ui::State;
use eframe::emath::{Align, NumExt, Pos2};
use eframe::epaint::Color32;
use egui::scroll_area::ScrollAreaOutput;
use egui::{Rect, RichText, ScrollArea, Sense, TextStyle, TextWrapMode, Ui, Vec2, WidgetInfo, WidgetText, WidgetType};

pub struct Disassembly {
    disassembly: Vec<DisassembledLine>,
    scroll_area_output: Option<ScrollAreaOutput<()>>,
}

impl Disassembly {
    pub fn new(state: &State) -> Disassembly {
        Disassembly {
            disassembly: disassemble(&state.cpu),
            scroll_area_output: None,
        }
    }

    pub fn disassemble(&mut self, state: &State) {
        self.disassembly = disassemble(&state.cpu);
    }
}

impl Window for Disassembly {
    fn show(&mut self, state: &mut State, ui: &mut Ui) {
        const LABEL_HEIGHT: f32 = 19.5;
        let height = ui.available_height();
        let output = ScrollArea::vertical()
            .id_salt("disassembly")
            .auto_shrink(false)
            .animated(false)
            .drag_to_scroll(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.allocate_space(Vec2::new(0.0, self.disassembly.len() as f32 * LABEL_HEIGHT + 52.0));
                    ui.vertical(|ui| {
                        if let Some(output) = &mut self.scroll_area_output {
                            ui.add_space(output.state.offset.y);

                            let index = self.disassembly.iter().position(|(line)| line.address == state.cpu.registers.pc).unwrap_or(0);
                            let y = index as f32 * LABEL_HEIGHT + 52.0;
                            let rel_y = y - output.state.offset.y;
                            let rect = Rect::from_min_max(Pos2::new(0.0, rel_y), Pos2::new(0.0, rel_y));
                            let is_visible = y > output.state.offset.y && y < output.state.offset.y + height;
                            // println!("Is Visible: {} Y: {}, Scroll Y: {}", is_visible, y, output.state.offset.y);
                            if state.should_scroll_disasm && !is_visible {
                                ui.scroll_to_rect(rect, Some(Align::TOP));
                            }
                            // println!("count: {}", height / LABEL_HEIGHT);
                            for line in self.disassembly
                                .iter()
                                .skip((output.state.offset.y / LABEL_HEIGHT) as usize)
                                .take((height / LABEL_HEIGHT) as usize) {
                                let text = if line.address == state.cpu.registers.pc {
                                    format!("> {:04X} {}", line.address, line.text)
                                } else {
                                    format!("  {:04X} {}", line.address, line.text)
                                };

                                let widget_text: WidgetText = (if let LineType::Label(_) = line.line_type {
                                    line.text.clone().into()
                                } else if state.breakpoints.contains(&line.address) {
                                    RichText::new(text).color(Color32::LIGHT_RED).into()
                                } else if line.address == state.cpu.registers.pc {
                                    RichText::new(text).color(Color32::LIGHT_GREEN).into()
                                } else {
                                    text.into()
                                });

                                let galley = widget_text.into_galley(ui, Some(TextWrapMode::Extend), ui.available_width(), TextStyle::Button);
                                let (rect, response) = ui.allocate_at_least(galley.size(), Sense::click());
                                response.widget_info(|| {
                                    WidgetInfo::selected(
                                        WidgetType::SelectableLabel,
                                        ui.is_enabled(),
                                        false,
                                        galley.text(),
                                    )
                                });

                                let text_pos = ui.layout().align_size_within_rect(galley.size(), rect.shrink2(Vec2::ZERO)).min;
                                let visuals = ui.style().interact_selectable(&response, false);
                                ui.painter().galley(text_pos, galley, visuals.text_color());
                                response.context_menu(|ui| {
                                    ui.set_width(200.0);
                                    let has_breakpoint = state.breakpoints.contains(&line.address);
                                    if has_breakpoint {
                                        if ui.button("Remove Breakpoint").clicked() {
                                            state.breakpoints.retain(|x| *x != line.address);
                                            ui.close_menu();
                                        }
                                    } else {
                                        if ui.button("Add Breakpoint").clicked() {
                                            state.breakpoints.push(line.address);
                                            ui.close_menu();
                                        }
                                    }
                                    if ui.button("Copy").clicked() {
                                        ui.output_mut(|writer| {
                                            writer.copied_text = line.text.to_string();
                                        });
                                        ui.close_menu();
                                    }
                                    if ui.button("Copy Address").clicked() {
                                        ui.output_mut(|writer| {
                                            writer.copied_text = format!("{:04X}", line.address);
                                        });
                                        ui.close_menu();
                                    }
                                });
                            }
                        }
                    });
                });
            });
        self.scroll_area_output = Some(output);

        state.should_scroll_disasm = false;
    }
}