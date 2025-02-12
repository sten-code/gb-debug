use crate::ui::windows::Window;
use crate::ui::State;
use eframe::egui;
use eframe::egui::{Button, ComboBox, Id, Modal, Sides, Widget};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BreakpointType {
    Address,
    Instruction,
}

pub enum Breakpoint {
    Address(u16),
    Instruction(u8),
}

impl BreakpointType {
    pub const VALUES: [BreakpointType; 2] = [BreakpointType::Address, BreakpointType::Instruction];
}

pub struct Breakpoints {
    pub show_message_box: bool,
    pub breakpoint_input_buffer: String,
    breakpoint_type: BreakpointType,
}

impl Breakpoints {
    pub fn new() -> Self {
        Self {
            show_message_box: false,
            breakpoint_input_buffer: String::new(),
            breakpoint_type: BreakpointType::Address,
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
                    self.breakpoint_input_buffer = format!("{:04x}", cpu.registers.pc);
                }
            }
        });

        if self.show_message_box {
            let modal = Modal::new(Id::new("add_breakpoint_modal")).show(ui.ctx(), |ui| {
                ui.set_width(300.0);

                ComboBox::new("breakpoint_type", "Breakpoint Type")
                    .selected_text(format!("{:?}", self.breakpoint_type))
                    .show_ui(ui, |ui| {
                        for breakpoint_type in BreakpointType::VALUES {
                            ui.selectable_value(
                                &mut self.breakpoint_type,
                                breakpoint_type,
                                format!("{:?}", breakpoint_type),
                            );
                        }
                    });
                ui.separator();

                match self.breakpoint_type {
                    BreakpointType::Address => {
                        ui.label("The address of the breakpoint:");
                        ui.text_edit_singleline(&mut self.breakpoint_input_buffer);
                    }
                    BreakpointType::Instruction => {
                        ui.label("The instruction opcode of the breakpoint:");
                        ui.text_edit_singleline(&mut self.breakpoint_input_buffer);
                    }
                }
                ui.separator();
                Sides::new().show(
                    ui,
                    |ui| {},
                    |ui| {
                        if Button::new("Add").min_size([50.0, 0.0].into()).ui(ui).clicked() {
                            let Ok(addr) = u16::from_str_radix(&self.breakpoint_input_buffer, 16) else {
                                return;
                            };

                            state.breakpoints.push(addr);
                            self.breakpoint_input_buffer.clear();
                            self.show_message_box = false;
                        }
                        if Button::new("Close").min_size([50.0, 0.0].into()).ui(ui).clicked() {
                            self.breakpoint_input_buffer.clear();
                            self.show_message_box = false;
                        }
                    },
                );
            });
            if modal.should_close() {
                self.show_message_box = false;
            }
            // ui.ctx().show_viewport_immediate(
            //     egui::ViewportId::from_hash_of("breakpoint_message_box"),
            //     egui::ViewportBuilder::default()
            //         .with_title("Breakpoint")
            //         .with_inner_size([300.0, 100.0]),
            //     |ctx, class| {
            //         CentralPanel::default().show(ctx, |ui| {
            //             ui.label("The address of the breakpoint:");
            //             ui.text_edit_singleline(&mut self.breakpoint_address_input);
            //
            //             ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            //                 if Button::new("Add").min_size([50.0, 0.0].into()).ui(ui).clicked() {
            //                     if let Ok(addr) = u16::from_str_radix(&self.breakpoint_address_input, 16) {
            //                         state.breakpoints.push(addr);
            //                         self.breakpoint_address_input.clear();
            //                         self.show_message_box = false;
            //                     }
            //                 }
            //                 if Button::new("Close").min_size([50.0, 0.0].into()).ui(ui).clicked() {
            //                     self.breakpoint_address_input.clear();
            //                     self.show_message_box = false;
            //                 }
            //             });
            //         });
            //
            //         if ctx.input(|i| i.viewport().close_requested()) {
            //             self.show_message_box = false;
            //         }
            //     },
            // );
        }
    }
}