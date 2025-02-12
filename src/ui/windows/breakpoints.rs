use eframe::emath::Align;
use eframe::egui;
use eframe::egui::{Button, CentralPanel, Layout, Widget};
use crate::ui::State;
use crate::ui::windows::Window;

pub struct Breakpoints {
    pub show_message_box: bool,
    pub breakpoint_address_input: String,
}

impl Breakpoints {
    pub fn new() -> Self {
        Self {
            show_message_box: false,
            breakpoint_address_input: String::new(),
        }
    }
}

impl Window for Breakpoints {
    fn show(&mut self, state: &mut State, ui: &mut egui::Ui) {
        let mut deletion = Vec::new();
        for bp in state.breakpoints.iter() {
            ui.horizontal(|ui| {
                if ui.button("Remove").clicked() {
                    deletion.push(*bp);
                }

                ui.label(format!("{:04X}", bp));
            });
        }

        state.breakpoints.retain(|x| !deletion.contains(x));
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.add_space(5.0);
            if ui.button("Add Breakpoint").clicked() {
                self.show_message_box = true;
                if let Some(cpu) = &state.cpu {
                    self.breakpoint_address_input = format!("{:04x}", cpu.registers.pc);
                }
            }
        });

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
                                    state.breakpoints.push(addr);
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
}
