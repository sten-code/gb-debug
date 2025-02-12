use eframe::egui::Ui;
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

            if let Some(cpu) = &state.cpu {
                ui.vertical(|ui| {
                    ui.label(format!("{:02X}", cpu.registers.a));
                    ui.label(format!("{:02X}", cpu.registers.b));
                    ui.label(format!("{:02X}", cpu.registers.c));
                    ui.label(format!("{:02X}", cpu.registers.d));
                    ui.label(format!("{:02X}", cpu.registers.e));
                    ui.label(format!("{:02X}", u8::from(cpu.registers.f)));
                    ui.label(format!("{:02X}", cpu.registers.h));
                    ui.label(format!("{:02X}", cpu.registers.l));
                    ui.label(format!("{:04X}", cpu.registers.sp));
                    ui.label(format!("{:04X}", cpu.registers.pc));
                    ui.label(format!("{}", bit(cpu.registers.f.zero)));
                    ui.label(format!("{}", bit(cpu.registers.f.subtract)));
                    ui.label(format!("{}", bit(cpu.registers.f.half_carry)));
                    ui.label(format!("{}", bit(cpu.registers.f.carry)));
                });

                ui.vertical(|ui| {
                    ui.label(format!("{}", cpu.registers.a));
                    ui.label(format!("{}", cpu.registers.b));
                    ui.label(format!("{}", cpu.registers.c));
                    ui.label(format!("{}", cpu.registers.d));
                    ui.label(format!("{}", cpu.registers.e));
                    ui.label(format!("{}", u8::from(cpu.registers.f)));
                    ui.label(format!("{}", cpu.registers.h));
                    ui.label(format!("{}", cpu.registers.l));
                    ui.label(format!("{}", cpu.registers.sp));
                    ui.label(format!("{}", cpu.registers.pc));
                    ui.label(format!("{}", bit(cpu.registers.f.zero)));
                    ui.label(format!("{}", bit(cpu.registers.f.subtract)));
                    ui.label(format!("{}", bit(cpu.registers.f.half_carry)));
                    ui.label(format!("{}", bit(cpu.registers.f.carry)));
                });

                ui.vertical(|ui| {
                    ui.label(format!("{:0>8b}", cpu.registers.a));
                    ui.label(format!("{:0>8b}", cpu.registers.b));
                    ui.label(format!("{:0>8b}", cpu.registers.c));
                    ui.label(format!("{:0>8b}", cpu.registers.d));
                    ui.label(format!("{:0>8b}", cpu.registers.e));
                    ui.label(format!("{:0>8b}", u8::from(cpu.registers.f)));
                    ui.label(format!("{:0>8b}", cpu.registers.h));
                    ui.label(format!("{:0>8b}", cpu.registers.l));
                    ui.label(format!("{:0>16b}", cpu.registers.sp));
                    ui.label(format!("{:0>16b}", cpu.registers.pc));
                    ui.label(format!("{}", bit(cpu.registers.f.zero)));
                    ui.label(format!("{}", bit(cpu.registers.f.subtract)));
                    ui.label(format!("{}", bit(cpu.registers.f.half_carry)));
                    ui.label(format!("{}", bit(cpu.registers.f.carry)));
                });
            }
        });
    }
}
