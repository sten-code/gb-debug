use crate::disassembler::LineType;
use crate::ui::windows::Window;
use crate::ui::State;
use eframe::egui::scroll_area::ScrollAreaOutput;
use eframe::egui::{
    Rect, RichText, ScrollArea, Sense, TextStyle, TextWrapMode, Ui, Vec2, WidgetInfo, WidgetText,
    WidgetType,
};
use eframe::emath::{Align, Pos2};
use eframe::epaint::Color32;

pub struct Disassembly {
    scroll_area_output: Option<ScrollAreaOutput<()>>,
}

impl Disassembly {
    pub fn new() -> Self {
        Self {
            scroll_area_output: None,
        }
    }

    pub fn convert_to_nop(state: &mut State, address: u16) {
        // state.disassembly.retain(|x| x.address != address);
        // if let Some(cpu) = &mut state.cpu {
        //     let mut instruction_byte = cpu.mmu.read_byte(address);
        //     let is_prefixed = if instruction_byte == 0xCB {
        //         instruction_byte = cpu.mmu.read_byte(address + 1);
        //         true
        //     } else {
        //         false
        //     };
        //
        //     if let Some(instruction) = Instruction::from_byte(instruction_byte, is_prefixed) {
        //         let size = instruction.size();
        //         for i in 0..size {
        //             let sub_address = address + i as u16;
        //             cpu.mmu.cartridge.data[sub_address as usize] = 0x00;
        //             cpu.mmu.cartridge.mbc.force_write_rom(sub_address, 0x00);
        //             state.disassembly.push(DisassembledLine {
        //                 address: sub_address,
        //                 text: format!("{:<7} NOP", "00"),
        //                 line_type: LineType::Instruction(instruction),
        //                 bytes: vec![0x00],
        //             });
        //         }
        //     }
        // }
        //
        // state.disassembly.sort_by(|a, b| {
        //     if matches!(a.line_type, LineType::Label(_)) && a.address == b.address {
        //         Ordering::Less
        //     } else if matches!(b.line_type, LineType::Label(_)) && a.address == b.address {
        //         Ordering::Greater
        //     } else {
        //         a.address.cmp(&b.address)
        //     }
        // });
    }

    pub fn assemble_at(state: &mut State, mut address: u16) {
        println!("Assembling at ${:04X}", address);
        // let instructions = assembler::assemble("LD A, $00");
        // let begin_address = address;
        // let mut end_address = address;
        // for full_instruction in &instructions {
        //     end_address += full_instruction.to_bytes().len() as u16;
        // }
        // state.disassembly.retain(|instr| (instr.address < begin_address || instr.address >= end_address) && matches!(instr.line_type, LineType::Instruction(_)));
        //
        // if let Some(cpu) = &mut state.cpu {
        //     for full_instruction in instructions {
        //         let bytes = full_instruction.to_bytes();
        //         let byte_count = bytes.len() as u16;
        //
        //         let mut bytes_str = String::new();
        //         for (i, byte) in bytes.iter().enumerate() {
        //             let address = address + i as u16;
        //             cpu.mmu.cartridge.data[address as usize] = *byte;
        //             cpu.mmu.cartridge.mbc.force_write_rom(address, *byte);
        //             bytes_str.push_str(&format!("{:02X}", byte));
        //         }
        //
        //         state.disassembly.push(DisassembledLine {
        //             address,
        //             text: format!(
        //                 "{:<7} {}",
        //                 bytes_str,
        //                 full_instruction.instruction.to_string(
        //                     *full_instruction.operands.first().unwrap_or(&0u8),
        //                     *full_instruction.operands.get(1).unwrap_or(&0u8),
        //                     address
        //                 )
        //             ),
        //             line_type: LineType::Instruction(full_instruction.instruction),
        //             bytes,
        //         });
        //
        //         address += byte_count;
        //     }
        // }
        //
        // state.disassembly.sort_by(|a, b| {
        //     if matches!(a.line_type, LineType::Label(_)) && a.address == b.address {
        //         Ordering::Less
        //     } else if matches!(b.line_type, LineType::Label(_)) && a.address == b.address {
        //         Ordering::Greater
        //     } else {
        //         a.address.cmp(&b.address)
        //     }
        // });
    }
}

impl Window for Disassembly {
    fn show(&mut self, state: &mut State, ui: &mut Ui) {
        let cpu = if let Some(cpu) = &mut state.cpu {
            cpu
        } else {
            return;
        };

        let bank = cpu.get_current_bank();
        let disassembly =
            if let Some(disassembly) = state.disassembler.disassembly.get(bank as usize) {
                disassembly
            } else {
                return;
            };

        const LABEL_HEIGHT: f32 = 19.5;
        let height = ui.available_height();
        let output = ScrollArea::vertical()
            .id_salt("disassembly")
            .auto_shrink(false)
            .animated(false)
            .drag_to_scroll(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.allocate_space(Vec2::new(
                        0.0,
                        disassembly.len() as f32 * LABEL_HEIGHT + 52.0,
                    ));
                    ui.vertical(|ui| {
                        if let Some(output) = &mut self.scroll_area_output {
                            ui.add_space(output.state.offset.y);

                            let pc_index = disassembly
                                .iter()
                                .position(|(line)| line.address == cpu.registers.pc)
                                .unwrap_or(0);
                            let y = pc_index as f32 * LABEL_HEIGHT + 52.0;
                            let rel_y = y - output.state.offset.y;
                            let rect =
                                Rect::from_min_max(Pos2::new(0.0, rel_y), Pos2::new(0.0, rel_y));
                            let is_visible =
                                y > output.state.offset.y && y < output.state.offset.y + height;
                            if state.should_scroll_disasm && !is_visible {
                                ui.scroll_to_rect(rect, Some(Align::TOP));
                                state.should_scroll_disasm = false;
                            }

                            // let mut convert_nop_addr: Option<u16> = None;
                            // let mut assemble_addr: Option<u16> = None;
                            let offset = (output.state.offset.y / LABEL_HEIGHT) as usize;
                            for (i, line) in disassembly
                                .iter()
                                .skip(offset)
                                .take((height / LABEL_HEIGHT) as usize)
                                .enumerate()
                            {
                                let text = if line.address == cpu.registers.pc {
                                    format!("> {:04X} {}", line.address, line.text)
                                } else {
                                    format!("  {:04X} {}", line.address, line.text)
                                };

                                let widget_text: WidgetText =
                                    (if let LineType::Label(_) = line.line_type {
                                        line.text.clone().into()
                                    } else if state.breakpoints.contains(&line.address) {
                                        RichText::new(text).color(Color32::LIGHT_RED).into()
                                    } else if line.address == cpu.registers.pc {
                                        RichText::new(text).color(Color32::LIGHT_GREEN).into()
                                    } else {
                                        text.into()
                                    });

                                let galley = widget_text.into_galley(
                                    ui,
                                    Some(TextWrapMode::Extend),
                                    ui.available_width(),
                                    TextStyle::Button,
                                );
                                let (rect, response) =
                                    ui.allocate_at_least(galley.size(), Sense::click());
                                response.widget_info(|| {
                                    WidgetInfo::selected(
                                        WidgetType::SelectableLabel,
                                        ui.is_enabled(),
                                        false,
                                        galley.text(),
                                    )
                                });

                                let text_pos = ui
                                    .layout()
                                    .align_size_within_rect(galley.size(), rect.shrink2(Vec2::ZERO))
                                    .min;
                                let visuals = ui.style().interact_selectable(&response, false);
                                ui.painter().galley(text_pos, galley, visuals.text_color());

                                let next = disassembly.iter().skip(offset + i + 1).find(|line| {
                                    matches!(line.line_type, LineType::Instruction(_))
                                });

                                if let Some(next) = next {
                                    if let LineType::Instruction(instruction) = line.line_type {
                                        if next.address > line.address + instruction.size() as u16 {
                                            let cursor = ui.cursor();
                                            ui.painter().line_segment(
                                                [cursor.min, Pos2::new(cursor.max.x, cursor.min.y)],
                                                (1.0, Color32::DARK_GRAY),
                                            );
                                        }
                                    }
                                }

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
                                    ui.menu_button("Patch", |ui| {
                                        ui.set_width(200.0);
                                        if ui.button("Convert to NOP").clicked() {
                                            // convert_nop_addr = Some(line.address);
                                            ui.close_menu();
                                        }
                                        if ui.button("Assemble").clicked() {
                                            // assemble_addr = Some(line.address);
                                            ui.close_menu();
                                        }
                                    });
                                });
                            }
                            // if let Some(address) = convert_nop_addr {
                            //     Disassembly::convert_to_nop(state, address)
                            // }
                            // if let Some(address) = assemble_addr {
                            //     Disassembly::assemble_at(state, address);
                            // }
                        }
                    });
                });
            });
        self.scroll_area_output = Some(output);
    }
}
