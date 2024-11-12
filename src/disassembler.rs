use std::cmp::Ordering;
use std::collections::HashMap;

use crate::cpu::instruction::{Instruction, JumpTest};
use crate::cpu::CPU;

pub enum LineType {
    Label(String),
    Instruction(Instruction),
    DataBlock(Vec<u8>),
}

pub fn disassemble(cpu: &CPU) -> Vec<(u16, LineType, String)> {
    disassemble_from_address(0x0100, cpu)
}

pub fn disassemble_from_address(instruction_addr: u16, cpu: &CPU) -> Vec<(u16, LineType, String)> {
    println!("Disassembling Instruction Tree...");
    let mut instructions: Vec<(u16, LineType, String)> = Vec::new();
    instructions.push((
        0x0100,
        LineType::Label("main".to_owned()),
        "main:".to_owned(),
    ));
    disassemble_branch(&mut instructions, instruction_addr, cpu);
    println!("Cleaning up labels...");
    let mut seen: HashMap<u16, bool> = HashMap::new();
    instructions.retain(|instruction| !matches!(instruction.1, LineType::Label(_)) || seen.insert(instruction.0, true).is_none());
    println!("Sorting...");
    instructions.sort_by(|a, b| if matches!(a.1, LineType::Label(_)) && a.0 == b.0 { 
        Ordering::Less
    } else if matches!(b.1, LineType::Label(_)) && a.0 == b.0 {
        Ordering::Greater
    } else {
        a.0.cmp(&b.0)
    });
    println!("Generating data blocks...");
    let mut skip = 0;
    for i in 0..instructions.len() {
        if skip > 0 {
            skip -= 1;
            continue;
        }
        if let LineType::Instruction(instruction) = instructions[i].1 {
            let size = instruction.size() as u16;
            let address1 = instructions[i].0;
            let address2 = instructions[i + 1].0;

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
                        (
                            address1 + size + line_offset,
                            LineType::DataBlock(data),
                            chunk,
                        ),
                    );
                    skip += 1;
                }
            }
        }
    }
    println!("Finished");
    instructions
}

fn explored_address(instructions: &[(u16, LineType, String)], address: u16) -> bool {
    instructions.iter().any(|(addr, line_type, _)| {
        *addr == address && matches!(line_type, LineType::Instruction(_))
    })
}

fn disassemble_branch(
    instructions: &mut Vec<(u16, LineType, String)>,
    mut instruction_addr: u16,
    cpu: &CPU,
) {
    // if explored_address(instructions, instruction_addr) {
    //     return;
    // }

    while instruction_addr < 0xFFFF {
        // Get the instruction at the current address
        let mut operand_addr = instruction_addr.wrapping_add(1);
        let mut byte = cpu.mmu.read_byte(instruction_addr);
        let is_prefixed = byte == 0xCB;
        if is_prefixed {
            byte = cpu.mmu.read_byte(instruction_addr.wrapping_add(1));
            operand_addr = instruction_addr.wrapping_add(1);
        }
        let instruction = Instruction::from_byte(byte, is_prefixed).unwrap_or(Instruction::NOP);
        let size = instruction.size();

        // Explore the branch that the jump/call instruction points to
        match instruction {
            Instruction::JP(_) => {
                let jump_address = cpu.mmu.read_word(operand_addr);
                let name = format!("Addr_{:04X}", jump_address);
                instructions.push((
                    jump_address,
                    LineType::Label(name.clone()),
                    format!("{}:", name),
                ));
                if !explored_address(instructions, jump_address) && instruction_addr != jump_address
                {
                    disassemble_branch(instructions, jump_address, cpu);
                }
            }
            Instruction::CALL(_) => {
                let jump_address = cpu.mmu.read_word(operand_addr);
                let name = format!("Addr_{:04X}", jump_address);
                instructions.push((
                    jump_address,
                    LineType::Label(name.clone()),
                    format!("{}:", name),
                ));
                if !explored_address(instructions, jump_address) && instruction_addr != jump_address
                {
                    disassemble_branch(instructions, jump_address, cpu);
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
                let name = format!("Addr_{:04X}", jump_address);
                instructions.push((
                    jump_address,
                    LineType::Label(name.clone()),
                    format!("{}:", name),
                ));
                if !explored_address(instructions, jump_address) && instruction_addr != jump_address
                {
                    disassemble_branch(instructions, jump_address, cpu);
                }
            }
            _ => {}
        }

        if explored_address(instructions, instruction_addr) {
            break;
        }

        // Add the current instruction to the list
        let mut instruction_bytes = format!("{:02X}", byte);
        let mut instruction_arr = [0u8; 2];
        if size > 1 {
            for i in 1..size {
                let extra_byte = cpu.mmu.read_byte(instruction_addr.wrapping_add(i as u16));
                instruction_bytes.push_str(&format!("{:02X}", extra_byte));
                instruction_arr[i as usize - 1] = extra_byte;
            }
        }
        let line = format!(
            "{:<7} {}",
            instruction_bytes,
            instruction.to_string(instruction_arr[0], instruction_arr[1], instruction_addr)
        );
        instructions.push((instruction_addr, LineType::Instruction(instruction), line));

        // If it always jumps when it reaches this instruction, it means the branch has ended
        match instruction {
            Instruction::JP(JumpTest::Always) => break,
            Instruction::RET(JumpTest::Always) => break,
            _ => {}
        }

        instruction_addr = instruction_addr.wrapping_add(size as u16);
    }
}
