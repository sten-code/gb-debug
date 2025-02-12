use crate::ui::windows::Window;
use crate::ui::State;
use eframe::egui;
use eframe::egui::{CornerRadius, Frame, ScrollArea, Sense, StrokeKind, TextStyle, WidgetInfo, WidgetText, WidgetType};
use eframe::emath::Align;
use eframe::epaint::{Color32, Margin, Stroke};

#[derive(PartialEq, Debug)]
enum SelectedTab {
    MemoryDump,
    Stack,
    VRAM,
    ExternalRAM,
    WorkRAM,
    OAM,
    IORegisters,
    HighRam,
}

pub struct MemoryView {
    selected_tab: SelectedTab,
}

impl MemoryView {
    pub fn new() -> Self {
        Self {
            selected_tab: SelectedTab::MemoryDump,
        }
    }

    fn show_memory_dump(&mut self, state: &mut State, ui: &mut egui::Ui) {
        const BYTES_PER_LINE: usize = 0x10;
        let start: usize = 0x0000;
        let end: usize = 0xFFFF;
        let focussed_row_addr = state.focussed_address & 0xFFF0;
        ui.horizontal(|ui| {
            ui.add_space(5.0);
            ui.label("addr");
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
                if let Some(cpu) = &mut state.cpu {
                    for row_addr in (start..=end).step_by(BYTES_PER_LINE) {
                        let distance =
                            ((row_addr as i64 - focussed_row_addr as i64).abs() / 16) as usize;
                        if distance > 50 {
                            continue;
                        }

                        let bytes = (row_addr..=row_addr + BYTES_PER_LINE - 1)
                            .map(|addr| cpu.mmu.read_byte(addr as u16))
                            .collect::<Vec<u8>>();

                        ui.horizontal(|ui| {
                            ui.add_space(5.0);
                            ui.label(format!("{:04X}", row_addr));
                            ui.add_space(5.0);

                            for (i, byte) in bytes.iter().enumerate() {
                                let text = WidgetText::from(format!("{:02X}", byte));
                                let galley = text.into_galley(
                                    ui,
                                    None,
                                    ui.available_width(),
                                    TextStyle::Button,
                                );

                                let desired_size = galley.size();
                                let (rect, response) =
                                    ui.allocate_at_least(desired_size, Sense::click());
                                response.widget_info(|| {
                                    WidgetInfo::selected(
                                        WidgetType::SelectableLabel,
                                        ui.is_enabled(),
                                        false,
                                        galley.text(),
                                    )
                                });

                                if state.should_scroll_dump
                                    && row_addr + i == state.focussed_address as usize
                                    && !ui.is_rect_visible(response.rect)
                                {
                                    ui.scroll_to_rect(response.rect, Some(Align::Center));
                                    state.should_scroll_dump = false;
                                }

                                if ui.is_rect_visible(response.rect) {
                                    let text_pos = ui
                                        .layout()
                                        .align_size_within_rect(
                                            galley.size(),
                                            rect.shrink2([0.0, 0.0].into()),
                                        )
                                        .min;

                                    let visuals = ui.style().interact_selectable(&response, false);

                                    if response.hovered()
                                        || response.highlighted()
                                        || response.has_focus()
                                    {
                                        let rect = rect.expand(visuals.expansion);

                                        ui.painter().rect(
                                            rect,
                                            CornerRadius::default(),
                                            visuals.weak_bg_fill,
                                            Stroke::default(),
                                            egui::StrokeKind::Middle
                                        );

                                        ui.painter().galley(text_pos, galley, visuals.text_color());
                                    } else if row_addr + i == state.focussed_address as usize {
                                        let rect = rect.expand(visuals.expansion);
                                        ui.painter().rect(
                                            rect,
                                            CornerRadius::default(),
                                            Color32::LIGHT_GREEN,
                                            Stroke::default(),
                                            egui::StrokeKind::Middle
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
                }
            });
    }

    fn show_stack(&mut self, state: &mut State, ui: &mut egui::Ui) {
        Frame::default()
            .inner_margin(Margin {
                left: 5,
                right: 5,
                top: 5,
                bottom: 5,
            })
            .show(ui, |ui| {
                let Some(cpu) = &mut state.cpu else {
                    return;
                };

                ScrollArea::vertical()
                    .auto_shrink(false)
                    .drag_to_scroll(false)
                    .show(ui, |ui| {
                        for addr in (cpu.registers.sp..=0xFFFE).rev().step_by(2) {
                            let word = cpu.mmu.read_word(addr - 1);
                            let is_call = cpu.call_stack.iter().any(|item| item.stack_address == addr - 1);
                            ui.horizontal(|ui| {
                                ui.label(format!("{:04X}", addr));
                                ui.label(format!("{:04X}", word));
                                if is_call {
                                    ui.label("call");
                                }
                            });
                        }
                    });
            });
    }

    fn show_memory_range(&mut self, state: &mut State, ui: &mut egui::Ui, start: u16, end: u16, shrink: bool) {
        const BYTES_PER_LINE: usize = 0x10;

        if let Some(cpu) = &mut state.cpu {
            ui.horizontal(|ui| {
                ui.add_space(5.0);
                ui.label("addr");
                ui.add_space(5.0);
                for i in 0..BYTES_PER_LINE {
                    ui.label(format!("{:02X}", i));
                }
            });

            ScrollArea::vertical()
                .auto_shrink(shrink)
                .drag_to_scroll(false)
                .show(ui, |ui| {
                    for row_addr in (start as usize..=end as usize).step_by(BYTES_PER_LINE) {
                        let bytes = (row_addr..=row_addr + BYTES_PER_LINE - 1)
                            .map(|addr| cpu.mmu.read_byte(addr as u16))
                            .collect::<Vec<u8>>();

                        ui.horizontal(|ui| {
                            ui.add_space(5.0);
                            ui.label(format!("{:04X}", row_addr));
                            ui.add_space(5.0);

                            for byte in bytes.iter() {
                                let text = WidgetText::from(format!("{:02X}", byte));
                                let galley = text.into_galley(ui, None, ui.available_width(), TextStyle::Button);

                                let (rect, response) = ui.allocate_at_least(galley.size(), Sense::click());
                                if ui.is_rect_visible(response.rect) {
                                    let text_pos = ui.layout().align_size_within_rect(galley.size(), rect).min;

                                    let visuals = ui.style().interact_selectable(&response, false);

                                    if response.hovered() || response.highlighted() || response.has_focus() {
                                        ui.painter()
                                            .rect(rect, CornerRadius::ZERO, visuals.weak_bg_fill, Stroke::NONE, StrokeKind::Middle);

                                        ui.painter().galley(text_pos, galley, visuals.text_color());
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
        }
    }
}

impl Window for MemoryView {
    fn show(&mut self, state: &mut State, ui: &mut egui::Ui) {
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.add_space(5.0);
            ui.selectable_value(&mut self.selected_tab, SelectedTab::MemoryDump, "Memory Dump");
            ui.selectable_value(&mut self.selected_tab, SelectedTab::Stack, "Stack");
            ui.selectable_value(&mut self.selected_tab, SelectedTab::IORegisters, "I/O Registers");
            ui.selectable_value(&mut self.selected_tab, SelectedTab::VRAM, "VRAM");
        });
        ui.horizontal(|ui| {
            ui.add_space(5.0);
            ui.selectable_value(&mut self.selected_tab, SelectedTab::ExternalRAM, "External RAM");
            ui.selectable_value(&mut self.selected_tab, SelectedTab::WorkRAM, "Work RAM");
            ui.selectable_value(&mut self.selected_tab, SelectedTab::OAM, "OAM");
            ui.selectable_value(&mut self.selected_tab, SelectedTab::HighRam, "High RAM");
        });

        match self.selected_tab {
            SelectedTab::MemoryDump => self.show_memory_dump(state, ui),
            SelectedTab::Stack => self.show_stack(state, ui),
            SelectedTab::VRAM => self.show_memory_range(state, ui, 0x8000, 0x9FFF, false),
            SelectedTab::ExternalRAM => self.show_memory_range(state, ui, 0xA000, 0xBFFF, false),
            SelectedTab::WorkRAM => self.show_memory_range(state, ui, 0xC000, 0xCFFF, false),
            SelectedTab::OAM => self.show_memory_range(state, ui, 0xFE00, 0xFE9F, false),
            SelectedTab::IORegisters => self.show_memory_range(state, ui, 0xFF00, 0xFF7F, false),
            SelectedTab::HighRam => self.show_memory_range(state, ui, 0xFF80, 0xFFFE, false),
        }
    }
}