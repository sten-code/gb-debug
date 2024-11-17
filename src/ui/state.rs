use std::time::Instant;
use eframe::epaint::TextureHandle;
use eframe::epaint::textures::TextureOptions;
use crate::cpu::CPU;
use crate::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub struct State {
    pub cpu: Box<CPU>,
    pub texture: TextureHandle,
    pub cycles_elapsed_in_frame: usize,
    pub breakpoints: Vec<u16>,
    pub running: bool,
    pub should_scroll_disasm: bool,
    pub should_scroll_dump: bool,
    pub focussed_address: u16,
}

impl State {
    pub fn new(cc: &eframe::CreationContext<'_>, cpu: Box<CPU>) -> Self {
        let buffer = [0u8, 0u8, 0u8, 255u8].iter().cloned().cycle().take(SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize * 4).collect::<Vec<u8>>();
        let color_image = egui::ColorImage::from_rgba_unmultiplied([SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize], &buffer);
        let texture = cc.egui_ctx.load_texture("color_buffer", color_image, TextureOptions::NEAREST);

        Self {
            cpu,
            texture,
            cycles_elapsed_in_frame: 0,
            breakpoints: Vec::new(),
            running: false,
            should_scroll_disasm: false,
            should_scroll_dump: false,
            focussed_address: 0,
        }
    }

    pub fn step(&mut self) -> u8 {
        let cycles_elapsed = self.cpu.step();
        self.should_scroll_disasm = true;
        self.should_scroll_dump = true;
        self.focussed_address = self.cpu.registers.pc;

        cycles_elapsed
    }
}