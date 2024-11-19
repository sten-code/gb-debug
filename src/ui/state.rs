use std::time::Instant;
use eframe::epaint::TextureHandle;
use eframe::epaint::textures::TextureOptions;
use crate::cpu::CPU;
use crate::cpu::instruction::Instruction;
use crate::disassembler;
use crate::disassembler::DisassembledLine;
use crate::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub struct State {
    pub cpu: Option<Box<CPU>>,
    pub texture: TextureHandle,
    pub cycles_elapsed_in_frame: usize,
    pub breakpoints: Vec<u16>,
    pub jp_hl_targets: Vec<u16>,
    pub disassembly: Vec<DisassembledLine>,
    pub running: bool,
    pub should_scroll_disasm: bool,
    pub should_scroll_dump: bool,
    pub focussed_address: u16,
}

impl State {
    pub fn new(cc: &eframe::CreationContext<'_>, cpu: Option<Box<CPU>>) -> Self {
        let buffer = [0u8, 0u8, 0u8, 255u8].iter().cloned().cycle().take(SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize * 4).collect::<Vec<u8>>();
        let color_image = egui::ColorImage::from_rgba_unmultiplied([SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize], &buffer);
        let texture = cc.egui_ctx.load_texture("color_buffer", color_image, TextureOptions::NEAREST);
        let (pc, disassembly) = if let Some(cpu) = &cpu {
            (cpu.registers.pc, disassembler::disassemble(cpu))
        } else {
            (0, Vec::new())
        };
        Self {
            cpu,
            texture,
            cycles_elapsed_in_frame: 0,
            breakpoints: Vec::new(),
            disassembly,
            jp_hl_targets: Vec::new(),
            running: false,
            should_scroll_disasm: true,
            should_scroll_dump: true,
            focussed_address: pc,
        }
    }

    pub fn step(&mut self) -> u8 {
        if let Some(cpu) = &mut self.cpu {
            let instruction_byte = cpu.mmu.read_byte(cpu.registers.pc);
            if let Some(instruction) = Instruction::from_byte(instruction_byte, false) {
                if matches!(instruction, Instruction::JPHL) {
                    println!("Disassembling JP HL branch");
                    let hl = cpu.registers.get_hl();
                    self.jp_hl_targets.push(hl);
                    self.disassembly = disassembler::disassemble_extra(cpu, vec![hl]);
                }
            }
            let cycles_elapsed = cpu.step();
            self.should_scroll_disasm = true;
            self.should_scroll_dump = true;
            self.focussed_address = cpu.registers.pc;
            cycles_elapsed
        } else {
            0
        }
    }
}