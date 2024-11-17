use crate::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::ui::State;
use crate::ui::windows::Window;
use eframe::epaint::textures::TextureOptions;
use egui::widgets::Image;
use egui::{Ui, Widget};
use std::time::Instant;

pub struct GameWindow {
    now: Instant,
}

impl GameWindow {
    pub fn new() -> Self {
        Self {
            now: Instant::now(),
        }
    }
}

const ONE_SECOND_IN_MICROS: usize = 1000000000;
const ONE_SECOND_IN_CYCLES: usize = 4190000;
const ONE_FRAME_IN_CYCLES: usize = 70224;

impl Window for GameWindow {
    fn show(&mut self, state: &mut State, ui: &mut Ui) {
        let input = ui.ctx().input(|i| i.clone());
        state.cpu.mmu.joypad.up = input.key_down(egui::Key::ArrowUp);
        state.cpu.mmu.joypad.down = input.key_down(egui::Key::ArrowDown);
        state.cpu.mmu.joypad.left = input.key_down(egui::Key::ArrowLeft);
        state.cpu.mmu.joypad.right = input.key_down(egui::Key::ArrowRight);
        state.cpu.mmu.joypad.a = input.key_down(egui::Key::X);
        state.cpu.mmu.joypad.b = input.key_down(egui::Key::Z);
        state.cpu.mmu.joypad.start = input.key_down(egui::Key::Enter);
        state.cpu.mmu.joypad.select = input.key_down(egui::Key::Space);

        if state.running {
            let time_delta = self.now.elapsed().subsec_nanos();
            self.now = Instant::now();
            let delta = time_delta as f64 / ONE_SECOND_IN_MICROS as f64;
            let cycles_to_run = delta * ONE_SECOND_IN_CYCLES as f64;

            let mut cycles_elapsed = 0;
            while cycles_elapsed <= cycles_to_run as usize {
                if state.breakpoints.contains(&state.cpu.registers.pc) || !state.running {
                    state.running = false;
                    state.cycles_elapsed_in_frame += cycles_elapsed;
                    break;
                }
                cycles_elapsed += state.cpu.step() as usize;
            }
            state.cycles_elapsed_in_frame += cycles_elapsed;
        }

        // Render the frame to a texture
        if state.cycles_elapsed_in_frame >= ONE_FRAME_IN_CYCLES {
            let color_image = egui::ColorImage::from_rgb([SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize], &state.cpu.mmu.ppu.screen_buffer);
            state.texture.set(color_image, TextureOptions::NEAREST);
            state.cycles_elapsed_in_frame = 0;
        }

        Image::new(&state.texture)
            .fit_to_exact_size([SCREEN_WIDTH as f32 * 2.0, SCREEN_HEIGHT as f32 * 2.0].into())
            .ui(ui);
    }
}