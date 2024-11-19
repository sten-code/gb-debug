use eframe::emath::Align;
use eframe::epaint::{Color32, Rounding, Stroke};
use egui::{ScrollArea, Sense, TextStyle, WidgetInfo, WidgetText, WidgetType};
use crate::ui::State;
use crate::ui::windows::Window;

pub struct MemoryDump {}

impl MemoryDump {
    pub fn new() -> Self {
        Self {}
    }
}

impl Window for MemoryDump {
    fn show(&mut self, state: &mut State, ui: &mut egui::Ui) {
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
                if let Some(cpu) = &state.cpu {
                    for row_addr in (start..=end).step_by(BYTES_PER_LINE) {
                        let distance = ((row_addr as i64 - focussed_row_addr as i64).abs() / 16) as usize;
                        if distance > 50 {
                            continue;
                        }

                        let bytes = (row_addr..=row_addr + BYTES_PER_LINE - 1).map(|addr| cpu.mmu.read_byte(addr as u16)).collect::<Vec<u8>>();

                        ui.horizontal(|ui| {
                            ui.add_space(5.0);
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

                                if state.should_scroll_dump && row_addr + i == state.focussed_address as usize && !ui.is_rect_visible(response.rect) {
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
                                    } else if row_addr + i == state.focussed_address as usize {
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
                }
            });
        state.should_scroll_dump = false;
    }
}