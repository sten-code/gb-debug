use crate::cpu::CPU;
use crate::cpu::instruction::{Instruction, JumpTest};

pub fn disassemble(cpu: &CPU) -> Vec<(u16, Option<Instruction>, String)> {
    disassemble_from_address(0x0100, cpu)
}

pub fn disassemble_from_address(instruction_addr: u16, cpu: &CPU) -> Vec<(u16, Option<Instruction>, String)> {
    let mut instructions: Vec<(u16, Option<Instruction>, String)> = Vec::new();
    disassemble_branch(&mut instructions, instruction_addr, cpu);
    instructions.sort_by(|a, b| a.0.cmp(&b.0));
    let mut skip = 0;
    for i in 0..instructions.len() {
        if skip > 0 {
            skip -= 1;
            continue;
        }
        let size = instructions[i + 0].1.unwrap().size() as u16;
        let address1 = instructions[i + 0].0;
        let address2 = instructions[i + 1].0;

        if address1 + size < address2 {
            let block_size = address2 - (address1 + size);
            let temp = (block_size / 16).max(1);
            for line_index in 0..temp {
                let line_offset = line_index * 16;
                let mut chunk = ".DB".to_owned();
                for offset in 0..(block_size - line_offset).min(16) {
                    chunk = format!("{chunk} ${:02X},", cpu.mmu.read_byte(address1 + size + offset + line_offset));
                }
                let _ = chunk.split_off(chunk.len() - 1);
                instructions.insert(i + line_index as usize + 1, (address1 + size + line_offset, None, chunk));
                skip += 1;
            }
        }
    }
    instructions
}

fn explored_address(instructions: &Vec<(u16, Option<Instruction>, String)>, address: u16) -> bool {
    instructions.iter().any(|(addr, _, _)| *addr == address)
}

fn disassemble_branch(instructions: &mut Vec<(u16, Option<Instruction>, String)>, mut instruction_addr: u16, cpu: &CPU) {
    if explored_address(&instructions, instruction_addr) {
        return;
    }

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
                if !explored_address(&instructions, jump_address) && instruction_addr != jump_address {
                    disassemble_branch(instructions, jump_address, cpu);
                }
            }
            Instruction::CALL(_) => {
                let jump_address = cpu.mmu.read_word(operand_addr);
                if !explored_address(&instructions, jump_address) && instruction_addr != jump_address {
                    disassemble_branch(instructions, jump_address, cpu);
                }
            }
            Instruction::JR(_) => {
                let byte = cpu.mmu.read_byte(operand_addr);
                let jump_address = if byte as i8 >= 0 {
                    instruction_addr.wrapping_add(2).wrapping_add(byte as i8 as u16)
                } else {
                    instruction_addr.wrapping_add(2).wrapping_sub((byte as i8 as i16).abs() as u16)
                };
                if !explored_address(&instructions, jump_address) && instruction_addr != jump_address {
                    disassemble_branch(instructions, jump_address, cpu);
                }
            }
            _ => {}
        }

        if explored_address(&instructions, instruction_addr) {
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
        let line = format!("{:<7} {}", instruction_bytes, instruction.to_string(instruction_arr[0], instruction_arr[1], instruction_addr));
        instructions.push((instruction_addr, Some(instruction), line));

        // If it always jumps when it reaches this instruction, it means the branch has ended
        match instruction {
            Instruction::JP(JumpTest::Always) => break,
            Instruction::RET(JumpTest::Always) => break,
            _ => {}
        }

        instruction_addr = instruction_addr.wrapping_add(size as u16);
    }
}