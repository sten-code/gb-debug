use std::cmp::Ordering;
use std::collections::HashMap;

use crate::cpu::instruction::{Instruction, JumpTest};
use crate::cpu::CPU;

#[derive(Debug, Clone)]
pub enum LineType {
    Label(String),
    Instruction(Instruction),
    DataBlock,
}

#[derive(Debug, Clone)]
pub struct DisassembledLine {
    pub address: u16,
    pub line_type: LineType,
    pub bytes: Vec<u8>,
    pub text: String,
}


pub struct Disassembler {
    pub disassembly: Vec<Vec<DisassembledLine>>
}

impl Disassembler {
    pub fn new() -> Self {
        let mut disassembly = Vec::new();

        // Initialize disassembly with 128 banks
        for _ in 0..0x80 {
            disassembly.push(Vec::new());
        }

        // High RAM
        disassembly.push(Vec::new());

        Self {
            disassembly,
        }
    }

    pub fn reset(&mut self, cpu: &CPU) {
        for disassembly in &mut self.disassembly {
            disassembly.clear();
        }
    }

    fn add_label(&mut self, name: &str, bank: u8, address: u16) {
        let label = format!("{}_{:04X}", name, address);
        if let Some(disassembly) = self.disassembly.get_mut(bank as usize) {
            disassembly.push(DisassembledLine {
                address,
                line_type: LineType::Label(label.clone()),
                bytes: Vec::new(),
                text: format!("{}:", label),
            });
        }
    }

    pub fn disassemble_function(&mut self, bank: u8, address: u16, name: &str, cpu: &mut CPU) {
        // println!("Disassembling {}_{:04X} in bank: {}...", name, address, bank);
        self.add_label(name, bank, address);
        self.disassemble_branch(bank, address, cpu);
    }

    pub fn explored_address(&self, bank: u8, address: u16) -> bool {
        if let Some(disassembly) = self.disassembly.get(bank as usize) {
            disassembly.iter().any(|line| {
                line.address == address && matches!(line.line_type, LineType::Instruction(_))
            })
        } else {
            false
        }
    }

    fn is_in_bank(&self, bank: u8, address: u16) -> bool {
        // bank 0 takes 0-0x3FFF, anything beyond that isn't in bank 0
        if bank == 0 && address > 0x3FFF {
            return false;
        }

        // bank 1-7F takes 0x4000-0x7FFF, anything before that is bank 0
        if bank > 0 && (address < 0x4000 || address > 0x7FFF) {
            return false;
        }

        true
    }

    fn disassemble_branch(&mut self, bank: u8, start_addr: u16, cpu: &mut CPU) {
        // Stack of addresses to visit
        let mut stack = vec![start_addr];

        while let Some(mut instruction_addr) = stack.pop() {
            while instruction_addr < 0xFFFF {
                if !self.is_in_bank(bank, instruction_addr) {
                    break;
                }

                // Check if we've already explored this address to prevent reprocessing
                if self.explored_address(bank, instruction_addr) {
                    break;
                }

                // Get the instruction at the current address
                let mut operand_addr = instruction_addr.wrapping_add(1);
                let mut byte = cpu.mmu.read_byte(instruction_addr);
                let is_prefixed = byte == 0xCB;
                if is_prefixed {
                    byte = cpu.mmu.read_byte(instruction_addr.wrapping_add(1));
                    operand_addr = operand_addr.wrapping_add(1);
                }
                let instruction = Instruction::from_byte(byte, is_prefixed).unwrap_or(Instruction::NOP);
                let size = instruction.size();

                // Explore the branch that the jump/call instruction points to
                match instruction {
                    Instruction::JP(_) => {
                        let jump_address = cpu.mmu.read_word(operand_addr);
                        if self.is_in_bank(bank, jump_address) {
                            self.add_label("addr", bank, jump_address);
                            if !self.explored_address(bank, jump_address) && instruction_addr != jump_address
                            {
                                stack.push(jump_address);
                            }
                        }
                    }
                    Instruction::CALL(_) => {
                        let jump_address = cpu.mmu.read_word(operand_addr);
                        if self.is_in_bank(bank, jump_address) {
                            self.add_label("func", bank, jump_address);
                            if !self.explored_address(bank, jump_address) && instruction_addr != jump_address
                            {
                                stack.push(jump_address);
                            }
                        }
                    }
                    Instruction::JR(_) => {
                        let byte = cpu.mmu.read_byte(operand_addr);
                        let jump_address = if byte as i8 >= 0 {
                            instruction_addr
                                .wrapping_add(2)
                                .wrapping_add(byte as i8 as u16)
                        } else {
                            instruction_addr
                                .wrapping_add(2)
                                .wrapping_sub((byte as i8 as i16).unsigned_abs())
                        };
                        if self.is_in_bank(bank, jump_address) {
                            self.add_label("addr", bank, jump_address);
                            if !self.explored_address(bank, jump_address) && instruction_addr != jump_address
                            {
                                stack.push(jump_address);
                            }
                        }
                    }
                    Instruction::RST(vector) => {
                        let jump_address = vector as u16;
                        self.add_label("rst", bank, jump_address);
                        if !self.explored_address(bank, jump_address) && instruction_addr != jump_address
                        {
                            stack.push(jump_address);
                        }
                    }
                    _ => {}
                }

                // Record the current instruction
                let mut instruction_bytes_str = format!("{:02X}", byte);
                let mut instruction_bytes = Vec::new();
                let mut instruction_arr = [0u8; 2];
                if size > 1 {
                    for i in 1..size {
                        let extra_byte = cpu.mmu.read_byte(instruction_addr.wrapping_add(i as u16));
                        instruction_bytes_str.push_str(&format!("{:02X}", extra_byte));
                        instruction_bytes.push(extra_byte);
                        instruction_arr[i as usize - 1] = extra_byte;
                    }
                }
                if let Some(disassembly) = self.disassembly.get_mut(bank as usize) {
                    disassembly.push(DisassembledLine {
                        address: instruction_addr,
                        line_type: LineType::Instruction(instruction),
                        bytes: instruction_bytes,
                        text: format!("{:<7} {}", instruction_bytes_str, instruction.to_string(instruction_arr[0], instruction_arr[1], instruction_addr)),
                    });
                }

                // If it always jumps when it reaches this instruction, it means the branch has ended.
                // call and rst are expected to return when they finish, so don't end it.
                match instruction {
                    Instruction::JP(JumpTest::Always) => break,
                    Instruction::JPHL => break,
                    Instruction::RET(JumpTest::Always) => break,
                    Instruction::RETI => break,
                    _ => {}
                }

                // Move to the next instruction address
                instruction_addr = instruction_addr.wrapping_add(size as u16);
            }
        }
    }

    pub fn sort_disassembly(&mut self) {
        for disassembly in &mut self.disassembly {
            disassembly.sort_by(|a, b| {
                if matches!(a.line_type, LineType::Label(_)) && a.address == b.address {
                    Ordering::Less
                } else if matches!(b.line_type, LineType::Label(_)) && a.address == b.address {
                    Ordering::Greater
                } else {
                    a.address.cmp(&b.address)
                }
            });
        }
    }

    pub fn remove_duplicate_labels(&mut self) {
        for disassembly in &mut self.disassembly {
            let mut seen: HashMap<u16, bool> = HashMap::new();
            disassembly.retain(|instruction| {
                !matches!(instruction.line_type, LineType::Label(_)) || seen.insert(instruction.address, true).is_none()
            });
        }
    }

    /*
    fn generate_data_blocks(&mut self, cpu: &CPU) {
        self.final_disassembly = self.base_disassembly.clone();

        let mut skip = 0;
        for i in 0..self.final_disassembly.len() {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            if let LineType::Instruction(instruction) = self.final_disassembly[i].line_type {
                let size = instruction.size() as u16;
                let address1 = self.final_disassembly[i].address;
                let address2 = self.final_disassembly[i + 1].address;

                if address1 + size < address2 {
                    let block_size = address2 - (address1 + size);
                    let temp = (block_size / 16) + 1;
                    for line_index in 0..temp {
                        let line_offset = line_index * 16;
                        if block_size - line_offset == 0 {
                            break;
                        }
                        let mut chunk = ".DB".to_owned();
                        let mut data = Vec::new();
                        for offset in 0..(block_size - line_offset).min(16) {
                            let byte = cpu.mmu.read_byte(address1 + size + offset + line_offset);
                            chunk = format!("{chunk} ${:02X},", byte);
                            data.push(byte);
                        }
                        let _ = chunk.split_off(chunk.len() - 1);
                        self.final_disassembly.insert(
                            i + line_index as usize + 1,
                            DisassembledLine {
                                address: address1 + size + line_offset,
                                line_type: LineType::DataBlock,
                                bytes: data,
                                text: chunk,
                            },
                        );
                        skip += 1;
                    }
                }
            }
        }
    }*/

    pub fn disassemble_extra(&mut self, cpu: &mut CPU, extra_addresses: &Vec<(u8, u16)>) {
        println!("Disassembling Instruction Tree...");
        self.reset(cpu);

        // explorer interrupt vectors
        self.disassemble_function(0, 0x40, "interrupt", cpu);
        self.disassemble_function(0, 0x48, "interrupt", cpu);
        self.disassemble_function(0, 0x50, "interrupt", cpu);
        self.disassemble_function(0, 0x58, "interrupt", cpu);
        self.disassemble_function(0, 0x60, "interrupt", cpu);

        // explore main function
        self.disassemble_function(0, 0x0100, "main", cpu);

        for (bank, address) in extra_addresses {
            self.disassemble_function(*bank, *address, "addr", cpu);
        }

        // Each time a jump occurs, it adds a new label, so there are duplicates that need to be cleaned up
        println!("Removing duplicate labels...");
        self.remove_duplicate_labels();

        println!("Sorting...");
        self.sort_disassembly();

        // println!("Generating data blocks...");
        // self.generate_data_blocks(cpu);

        println!("Finished");
    }

    pub fn disassemble(&mut self, cpu: &mut CPU) {
        self.disassemble_extra(cpu, &Vec::new())
    }
}

