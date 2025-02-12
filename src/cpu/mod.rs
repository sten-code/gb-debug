use crate::cartridge::Cartridge;
use crate::cpu::instruction::{Instruction, Source8Bit, Reg16Bit, IncDecTarget, Target8Bit, LoadType, DerefTarget, JumpTest, StackTarget};
use crate::cpu::register::Registers;
use crate::gbmode::GbMode;
use crate::io::sound::AudioPlayer;
use crate::mbc::MBC;
use crate::mmu::MMU;

mod register;
pub mod instruction;

macro_rules! apply_work_8bit_register {
    ($self:ident : $source:ident => $work:ident) => {
        {
            let value = $self.registers.$source;
            $self.$work(value)
        }
    };

    ($self:ident : $source:ident => $work:ident => $destination:ident) => {
        {
            let result = apply_work_8bit_register!($self: $source => $work);
            $self.registers.$destination = result;
        }
    };
}

macro_rules! apply_work_16bit_register {
    ($self:ident : $getter:ident => $work:ident) => {
        {
            let value = $self.registers.$getter();
            $self.$work(value)
        }
    };

    ($self:ident : $getter:ident => $work:ident => $setter:ident) => {
        {
            let result = apply_work_16bit_register!($self: $getter => $work);
            $self.registers.$setter(result);
        }
    };
}

macro_rules! arithmetic_instruction {
    // arithmetic_instruction!(source, self.alu_add)

    ($source:ident, $self:ident.$work:ident) => {
        {
            match $source {
                Source8Bit::A => apply_work_8bit_register!($self: a => $work),
                Source8Bit::B => apply_work_8bit_register!($self: b => $work),
                Source8Bit::C => apply_work_8bit_register!($self: c => $work),
                Source8Bit::D => apply_work_8bit_register!($self: d => $work),
                Source8Bit::E => apply_work_8bit_register!($self: e => $work),
                Source8Bit::H => apply_work_8bit_register!($self: h => $work),
                Source8Bit::L => apply_work_8bit_register!($self: l => $work),
                Source8Bit::HLP => {
                    let value = $self.mmu.read_byte($self.registers.get_hl());
                    $self.$work(value);
                }
                Source8Bit::N8 => {
                    let value = $self.read_next_byte();
                    $self.$work(value);
                }
            };
            match $source {
                Source8Bit::N8 => ($self.registers.pc.wrapping_add(2), 8),
                Source8Bit::HLP => ($self.registers.pc.wrapping_add(1), 8),
                _ => ($self.registers.pc.wrapping_add(1), 4),
            }
        }
    };

}

macro_rules! bitwise_instruction {

    ($self:ident : $target:ident => $work:ident) => {
        {
            let value = match $target {
                Target8Bit::A => $self.registers.a,
                Target8Bit::B => $self.registers.b,
                Target8Bit::C => $self.registers.c,
                Target8Bit::D => $self.registers.d,
                Target8Bit::E => $self.registers.e,
                Target8Bit::H => $self.registers.h,
                Target8Bit::L => $self.registers.l,
                Target8Bit::HLP => $self.mmu.read_byte($self.registers.get_hl()),
            };

            let result = $self.$work(value);
            match $target {
                Target8Bit::A => $self.registers.a = result,
                Target8Bit::B => $self.registers.b = result,
                Target8Bit::C => $self.registers.c = result,
                Target8Bit::D => $self.registers.d = result,
                Target8Bit::E => $self.registers.e = result,
                Target8Bit::H => $self.registers.h = result,
                Target8Bit::L => $self.registers.l = result,
                Target8Bit::HLP => $self.mmu.write_byte($self.registers.get_hl(), result),
            };

            ($self.registers.pc.wrapping_add(2), match $target {
                Target8Bit::HLP => 16,
                _ => 8,
            })
        }
    };

    ($self:ident : $source:ident => ($work:ident @ $position:ident)) => {
        {
            let value = match $source {
                Target8Bit::A => $self.registers.a,
                Target8Bit::B => $self.registers.b,
                Target8Bit::C => $self.registers.c,
                Target8Bit::D => $self.registers.d,
                Target8Bit::E => $self.registers.e,
                Target8Bit::H => $self.registers.h,
                Target8Bit::L => $self.registers.l,
                Target8Bit::HLP => $self.mmu.read_byte($self.registers.get_hl()),
            };
            $self.$work($position, value);
            ($self.registers.pc.wrapping_add(2), match $source {
                Target8Bit::HLP => 12,
                _ => 8,
            })
        }
    };

        ($self:ident : $source:ident => ($work:ident @ $position:ident) => $destination:ident) => {
        {
            let value = match $source {
                Target8Bit::A => $self.registers.a,
                Target8Bit::B => $self.registers.b,
                Target8Bit::C => $self.registers.c,
                Target8Bit::D => $self.registers.d,
                Target8Bit::E => $self.registers.e,
                Target8Bit::H => $self.registers.h,
                Target8Bit::L => $self.registers.l,
                Target8Bit::HLP => $self.mmu.read_byte($self.registers.get_hl()),
            };
            let result = $self.$work($position, value);
            match $destination {
                Target8Bit::A => $self.registers.a = result,
                Target8Bit::B => $self.registers.b = result,
                Target8Bit::C => $self.registers.c = result,
                Target8Bit::D => $self.registers.d = result,
                Target8Bit::E => $self.registers.e = result,
                Target8Bit::H => $self.registers.h = result,
                Target8Bit::L => $self.registers.l = result,
                Target8Bit::HLP => $self.mmu.write_byte($self.registers.get_hl(), result),
            };
            ($self.registers.pc.wrapping_add(2), match $source {
                Target8Bit::HLP => 12,
                _ => 8,
            })
        }
    };
}

#[inline(always)]
fn is_set(byte: u8, position: u8) -> bool {
    (byte >> position) & 1 == 1
}

pub struct CPU {
    pub registers: Registers,
    pub mmu: MMU,
    pub call_stack: Vec<(u16, u16, u16)>,
    ime: bool,
    is_halted: bool,
    gb_mode: GbMode,
}

impl CPU {
    pub fn new(cartridge: Cartridge, using_boot_rom: bool, audio_player: Box<dyn AudioPlayer>) -> CPU {
        let gb_mode = match cartridge.read_rom(0x143) & 0x80 {
            0x80 => GbMode::Color,
            _ => GbMode::Classic,
        };
        println!("Gb Mode: {:?}", gb_mode);
        CPU {
            registers: Registers::new(gb_mode, using_boot_rom),
            mmu: MMU::new(cartridge, gb_mode, using_boot_rom, audio_player),
            call_stack: Vec::new(),
            ime: false,
            is_halted: false,
            gb_mode,
        }
    }

    pub fn reset(&mut self) {
        self.mmu.reset();
        self.registers.reset();
        self.call_stack.clear();
        self.ime = false;
        self.is_halted = false;
    }

    pub fn get_current_bank(&self) -> u8 {
        if self.registers.pc < 0x4000 {
            0
        } else if self.registers.pc < 0x8000 {
            self.mmu.cartridge.mbc.get_selected_rom_bank()
        } else {
            self.mmu.cartridge.mbc.get_selected_ram_bank()
        }
    }

    pub fn get_gb_mode(&self) -> GbMode {
        self.gb_mode
    }

    pub fn export_state(&self) -> String {
        format!("A: {} B: {} C: {} D: {} E: {} H: {} L: {} Z: {} N: {} H: {} C: {} SP: {} PC: {}",
                self.registers.a,
                self.registers.b,
                self.registers.c,
                self.registers.d,
                self.registers.e,
                self.registers.h,
                self.registers.l,
                self.registers.f.zero as u8,
                self.registers.f.subtract as u8,
                self.registers.f.half_carry as u8,
                self.registers.f.carry as u8,
                self.registers.sp,
                self.registers.pc)
    }

    pub fn step(&mut self) -> u8 {
        // println!("Executing instruction at ${:04X}", self.registers.pc);
        let mut opcode = self.mmu.read_byte(self.registers.pc);
        let prefixed = opcode == 0xCB;
        if prefixed {
            opcode = self.mmu.read_byte(self.registers.pc.wrapping_add(1));
        }

        let (next_pc, mut cycles) = if let Some(instruction) = Instruction::from_byte(opcode, prefixed) {
            self.execute(instruction)
        } else {
            panic!("Invalid opcode: ${:02X}, PC: ${:04X}", opcode, self.registers.pc);
        };

        self.mmu.step(cycles as u32);
        if self.mmu.has_interrupt() {
            self.is_halted = false;
        }
        if !self.is_halted {
            self.registers.pc = next_pc;
        }

        let mut interrupted = false;
        if self.ime {
            // VBlank
            if is_set(self.mmu.interrupt_enable, 0) && is_set(self.mmu.interrupt_flags, 0) {
                interrupted = true;
                // Turn off the bit at position 0
                self.mmu.interrupt_flags &= !1;
                self.interrupt(0x40);
            }

            // LCD STAT
            else if is_set(self.mmu.interrupt_enable, 1) && is_set(self.mmu.interrupt_flags, 1) {
                interrupted = true;
                // Turn off the bit at position 1
                self.mmu.interrupt_flags &= !2;
                self.interrupt(0x48);
            }

            // Timer
            else if is_set(self.mmu.interrupt_enable, 2) && is_set(self.mmu.interrupt_flags, 2) {
                interrupted = true;
                // Turn off the bit at position 2
                self.mmu.interrupt_flags &= !4;
                self.interrupt(0x50);
            }
        }
        if interrupted {
            cycles += 12;
        }

        cycles
    }

    fn interrupt(&mut self, address: u16) {
        self.ime = false;
        self.push(self.registers.pc);
        self.call_stack.push((self.registers.pc, address, self.registers.pc));
        self.registers.pc = address;
        self.mmu.step(12);
    }

    fn execute(&mut self, instruction: Instruction) -> (u16, u8) {
        match instruction {
            Instruction::ADC(source) => arithmetic_instruction!(source, self.alu_adc),
            Instruction::ADD(source) => arithmetic_instruction!(source, self.alu_add),
            Instruction::ADDHL(source) => {
                let value = match source {
                    Reg16Bit::BC => self.registers.get_bc(),
                    Reg16Bit::DE => self.registers.get_de(),
                    Reg16Bit::HL => self.registers.get_hl(),
                    Reg16Bit::SP => self.registers.sp,
                };
                let hl = self.registers.get_hl();
                let result = hl.wrapping_add(value);
                self.registers.f.subtract = false;
                self.registers.f.half_carry = (hl & 0xFFF) + (value & 0xFFF) > 0xFFF;
                self.registers.f.carry = (hl as u32 + value as u32) > 0xFFFF;
                self.registers.set_hl(result);
                (self.registers.pc.wrapping_add(1), 8)
            }
            Instruction::ADDSP => {
                let value = self.read_next_byte() as i8 as i16 as u16;
                self.registers.f.zero = false;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = (self.registers.sp & 0xF) + (value & 0xF) > 0xF;
                self.registers.f.carry = (self.registers.sp & 0xFF) + (value & 0xFF) > 0xFF;
                self.registers.sp = self.registers.sp.wrapping_add(value);
                (self.registers.pc.wrapping_add(2), 16)
            }
            Instruction::AND(source) => arithmetic_instruction!(source, self.alu_and),
            Instruction::CP(source) => arithmetic_instruction!(source, self.alu_cp),
            Instruction::INC(target) => {
                match target {
                    IncDecTarget::A => apply_work_8bit_register!(self: a => alu_inc_8bit => a),
                    IncDecTarget::B => apply_work_8bit_register!(self: b => alu_inc_8bit => b),
                    IncDecTarget::C => apply_work_8bit_register!(self: c => alu_inc_8bit => c),
                    IncDecTarget::D => apply_work_8bit_register!(self: d => alu_inc_8bit => d),
                    IncDecTarget::E => apply_work_8bit_register!(self: e => alu_inc_8bit => e),
                    IncDecTarget::H => apply_work_8bit_register!(self: h => alu_inc_8bit => h),
                    IncDecTarget::L => apply_work_8bit_register!(self: l => alu_inc_8bit => l),
                    IncDecTarget::HLP => {
                        let address = self.registers.get_hl();
                        let value = self.mmu.read_byte(address);
                        let result = self.alu_inc_8bit(value);
                        self.mmu.write_byte(address, result);
                    }
                    IncDecTarget::BC => apply_work_16bit_register!(self: get_bc => alu_inc_16bit => set_bc),
                    IncDecTarget::DE => apply_work_16bit_register!(self: get_de => alu_inc_16bit => set_de),
                    IncDecTarget::HL => apply_work_16bit_register!(self: get_hl => alu_inc_16bit => set_hl),
                    IncDecTarget::SP => self.registers.sp = self.alu_inc_16bit(self.registers.sp),
                };

                (self.registers.pc.wrapping_add(1), match target {
                    IncDecTarget::BC | IncDecTarget::DE | IncDecTarget::HL | IncDecTarget::SP => 8,
                    IncDecTarget::HLP => 12,
                    _ => 4,
                })
            }
            Instruction::DEC(target) => {
                match target {
                    IncDecTarget::A => apply_work_8bit_register!(self: a => alu_dec_8bit => a),
                    IncDecTarget::B => apply_work_8bit_register!(self: b => alu_dec_8bit => b),
                    IncDecTarget::C => apply_work_8bit_register!(self: c => alu_dec_8bit => c),
                    IncDecTarget::D => apply_work_8bit_register!(self: d => alu_dec_8bit => d),
                    IncDecTarget::E => apply_work_8bit_register!(self: e => alu_dec_8bit => e),
                    IncDecTarget::H => apply_work_8bit_register!(self: h => alu_dec_8bit => h),
                    IncDecTarget::L => apply_work_8bit_register!(self: l => alu_dec_8bit => l),
                    IncDecTarget::HLP => {
                        let address = self.registers.get_hl();
                        let value = self.mmu.read_byte(address);
                        let result = self.alu_dec_8bit(value);
                        self.mmu.write_byte(address, result);
                    }
                    IncDecTarget::BC => apply_work_16bit_register!(self: get_bc => alu_dec_16bit => set_bc),
                    IncDecTarget::DE => apply_work_16bit_register!(self: get_de => alu_dec_16bit => set_de),
                    IncDecTarget::HL => apply_work_16bit_register!(self: get_hl => alu_dec_16bit => set_hl),
                    IncDecTarget::SP => self.registers.sp = self.alu_dec_16bit(self.registers.sp),
                };

                (self.registers.pc.wrapping_add(1), match target {
                    IncDecTarget::BC | IncDecTarget::DE | IncDecTarget::HL | IncDecTarget::SP => 8,
                    IncDecTarget::HLP => 12,
                    _ => 4,
                })
            }
            Instruction::OR(source) => arithmetic_instruction!(source, self.alu_or),
            Instruction::SBC(source) => arithmetic_instruction!(source, self.alu_sbc),
            Instruction::SUB(source) => arithmetic_instruction!(source, self.alu_sub),
            Instruction::XOR(source) => arithmetic_instruction!(source, self.alu_xor),

            Instruction::RL(target) => bitwise_instruction!(self: target => alu_rl),
            Instruction::RLA => {
                let value = self.registers.a;
                let result = self.alu_rl(value);
                self.registers.f.zero = false;
                self.registers.a = result;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::RLC(target) => bitwise_instruction!(self: target => alu_rlc),
            Instruction::RLCA => {
                let value = self.registers.a;
                let result = self.alu_rlc(value);
                self.registers.f.zero = false;
                self.registers.a = result;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::RR(target) => bitwise_instruction!(self: target => alu_rr),
            Instruction::RRA => {
                let value = self.registers.a;
                let result = self.alu_rr(value);
                self.registers.f.zero = false;
                self.registers.a = result;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::RRC(target) => bitwise_instruction!(self: target => alu_rrc),
            Instruction::RRCA => {
                let value = self.registers.a;
                let result = self.alu_rrc(value);
                self.registers.f.zero = false;
                self.registers.a = result;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::SLA(target) => bitwise_instruction!(self: target => alu_sla),
            Instruction::SRA(target) => bitwise_instruction!(self: target => alu_sra),
            Instruction::SWAP(target) => bitwise_instruction!(self: target => alu_swap),
            Instruction::SRL(target) => bitwise_instruction!(self: target => alu_srl),

            Instruction::BIT(position, target) => bitwise_instruction!(self: target => (alu_bit @ position)),
            Instruction::RES(position, target) => bitwise_instruction!(self: target => (alu_res @ position) => target),
            Instruction::SET(position, target) => bitwise_instruction!(self: target => (alu_set @ position) => target),

            Instruction::LD(load_type) => {
                match load_type {
                    LoadType::Byte(destination, source) => {
                        let value = match source {
                            Target8Bit::A => self.registers.a,
                            Target8Bit::B => self.registers.b,
                            Target8Bit::C => self.registers.c,
                            Target8Bit::D => self.registers.d,
                            Target8Bit::E => self.registers.e,
                            Target8Bit::H => self.registers.h,
                            Target8Bit::L => self.registers.l,
                            Target8Bit::HLP => self.mmu.read_byte(self.registers.get_hl()),
                        };

                        self.set_target_8bit(destination, value);

                        (
                            self.registers.pc.wrapping_add(1),
                            if destination == Target8Bit::HLP || source == Target8Bit::HLP { 8 } else { 4 }
                        )
                    }
                    LoadType::ByteFromImm(destination) => {
                        let value = self.read_next_byte();
                        self.set_target_8bit(destination, value);
                        (self.registers.pc.wrapping_add(2), 8)
                    }
                    LoadType::AFromDeref(source) => {
                        // LD A, [BC | DE | HL+ | HL-]
                        let address = self.get_value_deref(source);
                        let value = self.mmu.read_byte(address);
                        self.registers.a = value;
                        (self.registers.pc.wrapping_add(1), 8)
                    }
                    LoadType::DerefFromA(target) => {
                        // LD [BC | DE | HL+ | HL-], A
                        let address = self.get_value_deref(target);
                        self.mmu.write_byte(address, self.registers.a);
                        (self.registers.pc.wrapping_add(1), 8)
                    }
                    LoadType::AFromDerefC => {
                        // LD A, [0xFF00 + C]
                        let address = 0xFF00u16.wrapping_add(self.registers.c as u16);
                        let value = self.mmu.read_byte(address);
                        self.registers.a = value;
                        (self.registers.pc.wrapping_add(1), 8)
                    }
                    LoadType::DerefCFromA => {
                        // LD [0xFF00 + C], A
                        let address = 0xFF00u16.wrapping_add(self.registers.c as u16);
                        self.mmu.write_byte(address, self.registers.a);
                        (self.registers.pc.wrapping_add(1), 8)
                    }
                    LoadType::A8FromA => {
                        // LD [0xFF00 + A8], A
                        let address = 0xFF00u16.wrapping_add(self.read_next_byte() as u16);
                        self.mmu.write_byte(address, self.registers.a);
                        (self.registers.pc.wrapping_add(2), 12)
                    }
                    LoadType::AFromA8 => {
                        // LD A, [0xFF00 + A8]
                        let address = 0xFF00u16.wrapping_add(self.read_next_byte() as u16);
                        self.registers.a = self.mmu.read_byte(address);
                        (self.registers.pc.wrapping_add(2), 12)
                    }
                    LoadType::A16FromA => {
                        // LD [A16], A
                        let address = self.read_next_word();
                        self.mmu.write_byte(address, self.registers.a);
                        (self.registers.pc.wrapping_add(3), 16)
                    }
                    LoadType::AFromA16 => {
                        // LD A, [A16]
                        let address = self.read_next_word();
                        self.registers.a = self.mmu.read_byte(address);
                        (self.registers.pc.wrapping_add(3), 16)
                    }

                    LoadType::WordFromImm(destination) => {
                        // LD (BC | DE | HL | SP), N16
                        let value = self.read_next_word();
                        match destination {
                            Reg16Bit::BC => self.registers.set_bc(value),
                            Reg16Bit::DE => self.registers.set_de(value),
                            Reg16Bit::HL => self.registers.set_hl(value),
                            Reg16Bit::SP => self.registers.sp = value,
                        }
                        (self.registers.pc.wrapping_add(3), 12)
                    }
                    LoadType::SPFromHL => {
                        // LD SP, HL
                        self.registers.sp = self.registers.get_hl();
                        (self.registers.pc.wrapping_add(1), 8)
                    }
                    LoadType::HLFromSPE8 => {
                        // LD HL, SP + E8
                        let value = self.read_next_byte() as i8 as i16 as u16;
                        let hl = self.registers.sp.wrapping_add(value);
                        self.registers.f.zero = false;
                        self.registers.f.subtract = false;
                        self.registers.f.half_carry = (self.registers.sp & 0xF) + (value & 0xF) > 0xF;
                        self.registers.f.carry = (self.registers.sp & 0xFF) + (value & 0xFF) > 0xFF;
                        self.registers.set_hl(hl);
                        (self.registers.pc.wrapping_add(2), 12)
                    }
                    LoadType::A16FromSP => {
                        // LD [A16], SP
                        let address = self.read_next_word();
                        self.mmu.write_word(address, self.registers.sp);
                        (self.registers.pc.wrapping_add(3), 20)
                    }
                }
            }

            Instruction::CALL(condition) => {
                let should_jump = self.check_condition(condition);
                if should_jump {
                    let address = self.read_next_word();
                    let return_address = self.registers.pc.wrapping_add(3);
                    self.push(return_address);
                    self.call_stack.push((self.registers.pc, address, return_address));
                    (address, 24)
                } else {
                    (self.registers.pc.wrapping_add(3), 12)
                }
            }
            Instruction::JP(condition) => {
                let should_jump = self.check_condition(condition);
                if should_jump {
                    (self.read_next_word(), 16)
                } else {
                    (self.registers.pc.wrapping_add(3), 12)
                }
            }
            Instruction::JPHL => {
                (self.registers.get_hl(), 4)
            }
            Instruction::JR(condition) => {
                let should_jump = self.check_condition(condition);
                if should_jump {
                    let offset = self.read_next_byte() as i8;
                    (if offset >= 0 {
                        self.registers.pc.wrapping_add(2).wrapping_add(offset as u16)
                    } else {
                        self.registers.pc.wrapping_add(2).wrapping_sub((offset as i16).abs() as u16)
                    }, 16)
                } else {
                    (self.registers.pc.wrapping_add(2), 12)
                }
            }
            Instruction::RET(condition) => {
                let should_jump = self.check_condition(condition);
                if should_jump {
                    self.call_stack.pop();
                    (self.pop(), if condition == JumpTest::Always { 16 } else { 20 })
                } else {
                    (self.registers.pc.wrapping_add(1), 8)
                }
            }
            Instruction::RETI => {
                self.ime = true;
                self.call_stack.pop();
                (self.pop(), 16)
            }
            Instruction::RST(vec) => {
                let return_address = self.registers.pc.wrapping_add(1);
                self.push(return_address);
                self.call_stack.push((self.registers.pc, vec as u16, return_address));
                (vec as u16, 16)
            }

            Instruction::POP(source) => {
                let value = self.pop();
                match source {
                    StackTarget::AF => self.registers.set_af(value),
                    StackTarget::BC => self.registers.set_bc(value),
                    StackTarget::DE => self.registers.set_de(value),
                    StackTarget::HL => self.registers.set_hl(value),
                };
                (self.registers.pc.wrapping_add(1), 12)
            }
            Instruction::PUSH(source) => {
                let value = match source {
                    StackTarget::AF => self.registers.get_af(),
                    StackTarget::BC => self.registers.get_bc(),
                    StackTarget::DE => self.registers.get_de(),
                    StackTarget::HL => self.registers.get_hl(),
                };
                self.push(value);
                (self.registers.pc.wrapping_add(1), 16)
            }

            Instruction::CCF => {
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = !self.registers.f.carry;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::CPL => {
                self.registers.a = !self.registers.a;
                self.registers.f.subtract = true;
                self.registers.f.half_carry = true;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::DAA => {
                let mut a = self.registers.a;
                let mut adjust = 0;
                if self.registers.f.half_carry || (!self.registers.f.subtract && (a & 0xF) > 9) {
                    adjust = 0x6;
                }
                if self.registers.f.carry || (!self.registers.f.subtract && a > 0x99) {
                    adjust |= 0x60;
                    self.registers.f.carry = true;
                }
                if self.registers.f.subtract {
                    a = a.wrapping_sub(adjust);
                } else {
                    a = a.wrapping_add(adjust);
                }
                self.registers.a = a;
                self.registers.f.zero = a == 0;
                self.registers.f.half_carry = false;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::DI => {
                self.ime = false;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::EI => {
                self.ime = true;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::HALT => {
                self.is_halted = true;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::NOP => {
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::SCF => {
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = true;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::STOP => {
                (self.registers.pc.wrapping_add(2), 4)
            }
        }
    }

    fn pop(&mut self) -> u16 {
        let value = self.mmu.read_word(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(2);
        value
    }

    fn push(&mut self, value: u16) {
        self.registers.sp = self.registers.sp.wrapping_sub(2);
        self.mmu.write_word(self.registers.sp, value);
    }

    pub fn check_condition(&self, condition: JumpTest) -> bool {
        match condition {
            JumpTest::NotZero => !self.registers.f.zero,
            JumpTest::Zero => self.registers.f.zero,
            JumpTest::NotCarry => !self.registers.f.carry,
            JumpTest::Carry => self.registers.f.carry,
            JumpTest::Always => true,
        }
    }

    fn get_value_deref(&mut self, target: DerefTarget) -> u16 {
        match target {
            DerefTarget::BCP => self.registers.get_bc(),
            DerefTarget::DEP => self.registers.get_de(),
            DerefTarget::HLI => {
                let value = self.registers.get_hl();
                self.registers.set_hl(value.wrapping_add(1));
                value
            }
            DerefTarget::HLD => {
                let value = self.registers.get_hl();
                self.registers.set_hl(value.wrapping_sub(1));
                value
            }
        }
    }

    fn set_target_8bit(&mut self, target: Target8Bit, value: u8) {
        match target {
            Target8Bit::A => self.registers.a = value,
            Target8Bit::B => self.registers.b = value,
            Target8Bit::C => self.registers.c = value,
            Target8Bit::D => self.registers.d = value,
            Target8Bit::E => self.registers.e = value,
            Target8Bit::H => self.registers.h = value,
            Target8Bit::L => self.registers.l = value,
            Target8Bit::HLP => self.mmu.write_byte(self.registers.get_hl(), value),
        };
    }

    fn alu_add(&mut self, value: u8) {
        let (result, did_overflow) = self.registers.a.overflowing_add(value);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (self.registers.a & 0xF) + (value & 0xF) > 0xF;
        self.registers.f.carry = did_overflow;
        self.registers.a = result;
    }

    fn alu_adc(&mut self, value: u8) {
        let carry = if self.registers.f.carry { 1 } else { 0 };
        let result = self.registers.a.wrapping_add(value).wrapping_add(carry);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (self.registers.a & 0xF) + (value & 0xF) + carry > 0xF;
        self.registers.f.carry = self.registers.a as u16 + value as u16 + carry as u16 > 0xFF;
        self.registers.a = result;
    }

    fn alu_and(&mut self, value: u8) {
        self.registers.a &= value;
        self.registers.f.zero = self.registers.a == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = true;
        self.registers.f.carry = false;
    }

    fn alu_cp(&mut self, value: u8) {
        let result = self.registers.a.wrapping_sub(value);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (self.registers.a & 0xF) < (value & 0xF);
        self.registers.f.carry = self.registers.a < value;
    }

    fn alu_inc_8bit(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (value & 0xF) == 0xF;
        result
    }

    fn alu_inc_16bit(&mut self, value: u16) -> u16 {
        value.wrapping_add(1)
    }

    fn alu_inc(&mut self, value: u8) {
        let result = self.alu_inc_8bit(value);
        self.registers.a = result;
    }

    fn alu_dec_8bit(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (value & 0xF) == 0;
        result
    }

    fn alu_dec_16bit(&mut self, value: u16) -> u16 {
        value.wrapping_sub(1)
    }

    fn alu_dec(&mut self, value: u8) {
        let result = self.alu_dec_8bit(value);
        self.registers.a = result;
    }

    fn alu_or(&mut self, value: u8) {
        self.registers.a |= value;
        self.registers.f.zero = self.registers.a == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;
    }

    fn alu_sbc(&mut self, value: u8) {
        let carry = if self.registers.f.carry { 1 } else { 0 };
        let result = self.registers.a.wrapping_sub(value).wrapping_sub(carry);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (self.registers.a & 0xF) < (value & 0xF) + carry;
        self.registers.f.carry = (self.registers.a as u16) < (value as u16) + (carry as u16);
        self.registers.a = result;
    }

    fn alu_sub(&mut self, value: u8) {
        let (result, did_overflow) = self.registers.a.overflowing_sub(value);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (self.registers.a & 0xF) < (value & 0xF);
        self.registers.f.carry = did_overflow;
        self.registers.a = result;
    }

    fn alu_xor(&mut self, value: u8) {
        self.registers.a ^= value;
        self.registers.f.zero = self.registers.a == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;
    }

    fn alu_rl(&mut self, value: u8) -> u8 {
        let carry_bit = if self.registers.f.carry { 1 } else { 0 };
        let result = (value << 1) | carry_bit;
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = is_set(value, 7);
        result
    }

    fn alu_rlc(&mut self, value: u8) -> u8 {
        let result = value.rotate_left(1);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = is_set(value, 7);
        result
    }

    fn alu_rr(&mut self, value: u8) -> u8 {
        let carry_bit = if self.registers.f.carry { 1 } else { 0 };
        let result = (value >> 1) | (carry_bit << 7);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = is_set(value, 0);
        result
    }

    fn alu_rrc(&mut self, value: u8) -> u8 {
        let result = value.rotate_right(1);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = is_set(value, 0);
        result
    }

    fn alu_sla(&mut self, value: u8) -> u8 {
        let result = value << 1;
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = is_set(value, 7);
        result
    }

    fn alu_sra(&mut self, value: u8) -> u8 {
        let result = (value >> 1) | (value & 0x80);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = is_set(value, 0);
        result
    }

    fn alu_swap(&mut self, value: u8) -> u8 {
        let result = (value >> 4) | (value << 4);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;
        result
    }

    fn alu_srl(&mut self, value: u8) -> u8 {
        let result = value >> 1;
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = is_set(value, 0);
        result
    }

    fn alu_bit(&mut self, position: u8, value: u8) {
        let mask = 1 << position;
        self.registers.f.zero = (value & mask) == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = true;
    }

    fn alu_res(&mut self, position: u8, value: u8) -> u8 {
        let mask = !(1 << position);
        value & mask
    }

    fn alu_set(&mut self, position: u8, value: u8) -> u8 {
        let mask = 1 << position;
        value | mask
    }

    fn read_next_byte(&mut self) -> u8 {
        self.mmu.read_byte(self.registers.pc.wrapping_add(1))
    }

    fn read_next_word(&mut self) -> u16 {
        self.mmu.read_word(self.registers.pc.wrapping_add(1))
    }
}
