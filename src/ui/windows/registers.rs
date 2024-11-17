use egui::Ui;
use crate::ui::State;
use crate::ui::windows::Window;

#[inline(always)]
pub fn bit(condition: bool) -> u8 {
    if condition { 1 } else { 0 }
}

pub struct Registers {}

impl Registers {
    pub fn new() -> Self {
        Self {}
    }
}

impl Window for Registers {
    fn show(&mut self, state: &mut State, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("A:");
                ui.label("B:");
                ui.label("C:");
                ui.label("D:");
                ui.label("E:");
                ui.label("F:");
                ui.label("H:");
                ui.label("L:");
                ui.label("SP:");
                ui.label("PC:");
                ui.label("Z:");
                ui.label("N:");
                ui.label("H:");
                ui.label("C:");
            });

            ui.vertical(|ui| {
                ui.label(format!("{:02X}", state.cpu.registers.a));
                ui.label(format!("{:02X}", state.cpu.registers.b));
                ui.label(format!("{:02X}", state.cpu.registers.c));
                ui.label(format!("{:02X}", state.cpu.registers.d));
                ui.label(format!("{:02X}", state.cpu.registers.e));
                ui.label(format!("{:02X}", u8::from(state.cpu.registers.f)));
                ui.label(format!("{:02X}", state.cpu.registers.h));
                ui.label(format!("{:02X}", state.cpu.registers.l));
                ui.label(format!("{:04X}", state.cpu.registers.sp));
                ui.label(format!("{:04X}", state.cpu.registers.pc));
                ui.label(format!("{}", bit(state.cpu.registers.f.zero)));
                ui.label(format!("{}", bit(state.cpu.registers.f.subtract)));
                ui.label(format!("{}", bit(state.cpu.registers.f.half_carry)));
                ui.label(format!("{}", bit(state.cpu.registers.f.carry)));
            });

            ui.vertical(|ui| {
                ui.label(format!("{}", state.cpu.registers.a));
                ui.label(format!("{}", state.cpu.registers.b));
                ui.label(format!("{}", state.cpu.registers.c));
                ui.label(format!("{}", state.cpu.registers.d));
                ui.label(format!("{}", state.cpu.registers.e));
                ui.label(format!("{}", u8::from(state.cpu.registers.f)));
                ui.label(format!("{}", state.cpu.registers.h));
                ui.label(format!("{}", state.cpu.registers.l));
                ui.label(format!("{}", state.cpu.registers.sp));
                ui.label(format!("{}", state.cpu.registers.pc));
                ui.label(format!("{}", bit(state.cpu.registers.f.zero)));
                ui.label(format!("{}", bit(state.cpu.registers.f.subtract)));
                ui.label(format!("{}", bit(state.cpu.registers.f.half_carry)));
                ui.label(format!("{}", bit(state.cpu.registers.f.carry)));
            });

            ui.vertical(|ui| {
                ui.label(format!("{:0>8b}", state.cpu.registers.a));
                ui.label(format!("{:0>8b}", state.cpu.registers.b));
                ui.label(format!("{:0>8b}", state.cpu.registers.c));
                ui.label(format!("{:0>8b}", state.cpu.registers.d));
                ui.label(format!("{:0>8b}", state.cpu.registers.e));
                ui.label(format!("{:0>8b}", u8::from(state.cpu.registers.f)));
                ui.label(format!("{:0>8b}", state.cpu.registers.h));
                ui.label(format!("{:0>8b}", state.cpu.registers.l));
                ui.label(format!("{:0>16b}", state.cpu.registers.sp));
                ui.label(format!("{:0>16b}", state.cpu.registers.pc));
                ui.label(format!("{}", bit(state.cpu.registers.f.zero)));
                ui.label(format!("{}", bit(state.cpu.registers.f.subtract)));
                ui.label(format!("{}", bit(state.cpu.registers.f.half_carry)));
                ui.label(format!("{}", bit(state.cpu.registers.f.carry)));
            });
        });
    }
}