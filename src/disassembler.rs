use std::cmp::Ordering;
use std::collections::HashMap;

use crate::cpu::instruction::{Instruction, JumpTest};
use crate::cpu::CPU;

pub enum LineType {
    Label(String),
    Instruction(Instruction),
    DataBlock,
}

pub struct DisassembledLine {
    pub address: u16,
    pub line_type: LineType,
    pub bytes: Vec<u8>,
    pub text: String,
}

fn add_label(instructions: &mut Vec<DisassembledLine>, name: &str, address: u16) {
    let label = format!("{}_{:04X}", name, address);
    instructions.push(DisassembledLine {
        address,
        line_type: LineType::Label(label.clone()),
        bytes: Vec::new(),
        text: format!("{}:", label),
    });
}

fn disassemble_function(
    instructions: &mut Vec<DisassembledLine>,
    address: u16,
    name: &str,
    cpu: &CPU,
) {
    println!("Disassembling {}_{:04X}...", name, address);
    add_label(instructions, name, address);
    disassemble_branch(instructions, address, cpu);
}

pub fn disassemble(cpu: &CPU) -> Vec<DisassembledLine> {
    disassemble_extra(cpu, Vec::new())
}

pub fn disassemble_extra(cpu: &CPU, extra_addresses: Vec<u16>) -> Vec<DisassembledLine> {
    println!("Disassembling Instruction Tree...");
    let mut instructions = Vec::new();

    // explore rst vectors
    disassemble_function(&mut instructions, 0x00, "rst", cpu);
    disassemble_function(&mut instructions, 0x08, "rst", cpu);
    disassemble_function(&mut instructions, 0x10, "rst", cpu);
    disassemble_function(&mut instructions, 0x18, "rst", cpu);
    disassemble_function(&mut instructions, 0x20, "rst", cpu);
    disassemble_function(&mut instructions, 0x28, "rst", cpu);
    disassemble_function(&mut instructions, 0x30, "rst", cpu);
    disassemble_function(&mut instructions, 0x38, "rst", cpu);

    // explorer interrupt vectors
    disassemble_function(&mut instructions, 0x40, "interrupt", cpu);
    disassemble_function(&mut instructions, 0x48, "interrupt", cpu);
    disassemble_function(&mut instructions, 0x50, "interrupt", cpu);
    disassemble_function(&mut instructions, 0x58, "interrupt", cpu);
    disassemble_function(&mut instructions, 0x60, "interrupt", cpu);

    // explore main function
    disassemble_function(&mut instructions, 0x0100, "main", cpu);

    for address in extra_addresses {
        disassemble_function(&mut instructions, address, "indirect", cpu);
    }

    println!("Cleaning up labels...");
    let mut seen: HashMap<u16, bool> = HashMap::new();
    instructions.retain(|instruction| {
        !matches!(instruction.line_type, LineType::Label(_)) || seen.insert(instruction.address, true).is_none()
    });
    println!("Sorting...");
    instructions.sort_by(|a, b| {
        if matches!(a.line_type, LineType::Label(_)) && a.address == b.address {
            Ordering::Less
        } else if matches!(b.line_type, LineType::Label(_)) && a.address == b.address {
            Ordering::Greater
        } else {
            a.address.cmp(&b.address)
        }
    });
    println!("Generating data blocks...");
    let mut skip = 0;
    for i in 0..instructions.len() {
        if skip > 0 {
            skip -= 1;
            continue;
        }
        if let LineType::Instruction(instruction) = instructions[i].line_type {
            let size = instruction.size() as u16;
            let address1 = instructions[i].address;
            let address2 = instructions[i + 1].address;

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
                    instructions.insert(
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
    println!("Finished");
    instructions
}

fn explored_address(instructions: &[DisassembledLine], address: u16) -> bool {
    instructions.iter().any(|(line)| {
        line.address == address && matches!(line.line_type, LineType::Instruction(_))
    })
}

pub fn disassemble_branch(
    instructions: &mut Vec<DisassembledLine>,
    start_addr: u16,
    cpu: &CPU,
) {
    // Stack of addresses to visit
    let mut stack = vec![start_addr];

    while let Some(mut instruction_addr) = stack.pop() {
        while instruction_addr < 0xFFFF {
            // Check if we've already explored this address to prevent reprocessing
            if explored_address(instructions, instruction_addr) {
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
                    add_label(instructions, "addr", jump_address);
                    if !explored_address(instructions, jump_address) && instruction_addr != jump_address
                    {
                        stack.push(jump_address);
                    }
                }
                Instruction::CALL(_) => {
                    let jump_address = cpu.mmu.read_word(operand_addr);
                    add_label(instructions, "addr", jump_address);
                    if !explored_address(instructions, jump_address) && instruction_addr != jump_address
                    {
                        stack.push(jump_address);
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
                    add_label(instructions, "addr", jump_address);
                    if !explored_address(instructions, jump_address) && instruction_addr != jump_address
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
            instructions.push(DisassembledLine {
                address: instruction_addr,
                line_type: LineType::Instruction(instruction),
                bytes: instruction_bytes,
                text: format!("{:<7} {}", instruction_bytes_str, instruction.to_string(instruction_arr[0], instruction_arr[1], instruction_addr)),
            });

            // If it always jumps when it reaches this instruction, it means the branch has ended
            // Call is expected to return when it finishes, so don't end it.
            match instruction {
                Instruction::JP(JumpTest::Always) => break,
                Instruction::RET(JumpTest::Always) => break,
                Instruction::RST(_) => break,
                _ => {}
            }

            // Move to the next instruction address
            instruction_addr = instruction_addr.wrapping_add(size as u16);
        }
    }
}

