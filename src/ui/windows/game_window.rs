use crate::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::ui::windows::Window;
use crate::ui::State;
use eframe::egui::widgets::Image;
use eframe::egui::{self, Id, Modal};
use eframe::egui::{Button, DragValue, Ui, Widget};
use eframe::epaint::textures::TextureOptions;
use std::time::Instant;

pub struct GameWindow {
    now: Instant,
    pub emulation_speed: f32,
    pub fullscreen: bool,
    pub fullscreen_scale: f32,
    pub normal_scale: f32,
}

impl GameWindow {
    pub fn new() -> Self {
        Self {
            now: Instant::now(),
            emulation_speed: 1.0,
            fullscreen: false,
            fullscreen_scale: 7.0,
            normal_scale: 2.0,
        }
    }
}

const ONE_SECOND_IN_MICROS: usize = 1000000000;
const ONE_SECOND_IN_CYCLES: usize = 4190000;
const ONE_FRAME_IN_CYCLES: usize = 70224;

impl GameWindow {
    fn show_control_buttons(&mut self, state: &mut State, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add_space(5.0);
            let run_btn = Button::new(if state.running { "Stop" } else { "Run" })
                .min_size([50.0, 0.0].into())
                .ui(ui);
            if run_btn.clicked() {
                state.running = !state.running;
                state.cycles_elapsed_in_frame += state.step() as usize;
            }

            let step_btn = Button::new("Step").min_size([50.0, 0.0].into()).ui(ui);
            if step_btn.clicked() {
                state.cycles_elapsed_in_frame += state.step() as usize;
            }

            let reset_btn = Button::new("Reset").min_size([50.0, 0.0].into()).ui(ui);
            if reset_btn.clicked() {
                if let Some(cpu) = &mut state.cpu {
                    cpu.reset();
                    state.extra_targets.clear();
                    state.disassembler.disassembly.clear();
                    state.disassembler.disassemble(cpu);
                    state.should_scroll_disasm = true;
                }
            }

            ui.add(
                DragValue::new(&mut self.emulation_speed)
                    .speed(0.01)
                    .range(0.0..=30.0),
            );

            let fullscreen_btn = Button::new("Fullscreen")
                .min_size([50.0, 0.0].into())
                .ui(ui);
            if fullscreen_btn.clicked() {
                self.fullscreen = !self.fullscreen;
            }

            ui.add(
                DragValue::new(if self.fullscreen {
                    &mut self.fullscreen_scale
                } else {
                    &mut self.normal_scale
                })
                .speed(0.01)
                .range(1.0..=10.0),
            );
        });
    }
}

impl Window for GameWindow {
    fn show(&mut self, state: &mut State, ui: &mut Ui) {
        let input = ui.ctx().input(|i| i.clone());
        if let Some(cpu) = &mut state.cpu {
            cpu.mmu.joypad.up = input.key_down(egui::Key::ArrowUp);
            cpu.mmu.joypad.down = input.key_down(egui::Key::ArrowDown);
            cpu.mmu.joypad.left = input.key_down(egui::Key::ArrowLeft);
            cpu.mmu.joypad.right = input.key_down(egui::Key::ArrowRight);
            cpu.mmu.joypad.a = input.key_down(egui::Key::X);
            cpu.mmu.joypad.b = input.key_down(egui::Key::Z);
            cpu.mmu.joypad.start = input.key_down(egui::Key::Enter);
            cpu.mmu.joypad.select = input.key_down(egui::Key::Space);
        }

        if state.running {
            let time_delta = self.now.elapsed().subsec_nanos() as f32 * self.emulation_speed;
            self.now = Instant::now();
            let delta = time_delta as f64 / ONE_SECOND_IN_MICROS as f64;
            let cycles_to_run = delta * ONE_SECOND_IN_CYCLES as f64;

            let mut cycles_elapsed = 0;
            while cycles_elapsed <= cycles_to_run as usize {
                if let Some(cpu) = &mut state.cpu {
                    if state.breakpoints.contains(&cpu.registers.pc) || !state.running {
                        state.running = false;
                        state.cycles_elapsed_in_frame += cycles_elapsed;
                        break;
                    }
                }
                cycles_elapsed += state.step() as usize;
            }
            state.cycles_elapsed_in_frame += cycles_elapsed;
        }

        // Render the frame to a texture
        if state.cycles_elapsed_in_frame >= ONE_FRAME_IN_CYCLES {
            if let Some(cpu) = &mut state.cpu {
                if cpu.mmu.ppu.screen_buffer_updated {
                    let color_image = egui::ColorImage::from_rgb(
                        [SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize],
                        &cpu.mmu.ppu.screen_buffer,
                    );
                    state.texture.set(color_image, TextureOptions::NEAREST);
                    state.cycles_elapsed_in_frame = 0;
                    cpu.mmu.ppu.screen_buffer_updated = false;
                }
            }
        }

        if !self.fullscreen {
            Image::new(&state.texture)
                .fit_to_exact_size([SCREEN_WIDTH as f32 * 2.0, SCREEN_HEIGHT as f32 * 2.0].into())
                .ui(ui);
            self.show_control_buttons(state, ui);
        } else {
            let modal = Modal::new(Id::new("Game")).show(ui.ctx(), |ui| {
                ui.set_width(SCREEN_WIDTH as f32 * self.fullscreen_scale);
                Image::new(&state.texture)
                    .fit_to_exact_size(
                        [
                            SCREEN_WIDTH as f32 * self.fullscreen_scale,
                            SCREEN_HEIGHT as f32 * self.fullscreen_scale,
                        ]
                        .into(),
                    )
                    .ui(ui);
                self.show_control_buttons(state, ui);
            });
            if modal.should_close() {
                self.fullscreen = false;
            }
        }
    }
}
