use std::time::Instant;
use eframe::epaint::TextureHandle;
use eframe::epaint::textures::TextureOptions;
use crate::cpu::CPU;
use crate::cpu::instruction::Instruction;
use crate::disassembler;
use crate::disassembler::{DisassembledLine, Disassembler};
use crate::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub struct State {
    pub cpu: Option<Box<CPU>>,
    pub texture: TextureHandle,
    pub cycles_elapsed_in_frame: usize,
    pub breakpoints: Vec<u16>,
    pub extra_targets: Vec<(u8, u16)>,
    pub disassembler: Disassembler,
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
        let mut disassembler = Disassembler::new();
        let pc = if let Some(cpu) = &cpu {
            disassembler.disassemble(cpu);
            cpu.registers.pc
        } else {
            0
        };
        Self {
            cpu,
            texture,
            cycles_elapsed_in_frame: 0,
            breakpoints: Vec::new(),
            disassembler,
            extra_targets: Vec::new(),
            running: false,
            should_scroll_disasm: true,
            should_scroll_dump: true,
            focussed_address: pc,
        }
    }

    pub fn step(&mut self) -> u8 {
        if let Some(cpu) = &mut self.cpu {
            // let prev = cpu.registers.pc;
            // let byte = cpu.mmu.read_byte(cpu.registers.pc);
            let cycles_elapsed = cpu.step();

            let bank = cpu.get_current_bank();
            if !self.disassembler.explored_address(bank, cpu.registers.pc) {
                println!("Disassembling from bank: {} at ${:04X}", bank, cpu.registers.pc);
                self.extra_targets.push((bank, cpu.registers.pc));
                // let label = if let Some(instruction) = Instruction::from_byte(byte, false) {
                //     match instruction {
                //         Instruction::CALL(_) => "func",
                //         _ => "addr",
                //     }
                // } else {
                //     "addr"
                // };
                self.disassembler.disassemble_function(bank, cpu.registers.pc, "addr", cpu);
                self.disassembler.remove_duplicate_labels();
                self.disassembler.sort_disassembly();
            }

            // if !self.extra_targets.iter().any(|(to, from)| *to == cpu.registers.pc)
            //     && !self.disassembler.explored_address(0, cpu.registers.pc) {
            //     print!("Found target at ${:04X}, from: ${:04X}", cpu.registers.pc, prev);
            //     let instruction_byte = cpu.mmu.read_byte(prev);
            //     if let Some(instruction) = Instruction::from_byte(instruction_byte, false) {
            //         println!(" caused by: {:?}", instruction);
            //     } else {
            //         println!();
            //     }
            //
            //     self.extra_targets.push((cpu.registers.pc, prev));
            //     self.disassembler.disassemble_function(0, cpu.registers.pc, "indirect", cpu);
            //     self.disassembler.sort_disassembly();
            //     self.disassembler.remove_duplicate_labels();
            // }

            self.should_scroll_disasm = true;
            self.should_scroll_dump = true;
            self.focussed_address = cpu.registers.pc;
            cycles_elapsed
        } else {
            0
        }
    }
}