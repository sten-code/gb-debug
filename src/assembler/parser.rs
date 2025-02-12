use crate::assembler::lexer::{Register, Token};
use crate::cpu::instruction::{
    IncDecTarget, Instruction, JumpTest, LoadType, Source8Bit, Target8Bit,
};

macro_rules! expect {
    ($self:ident, $pattern:pat) => {{
        let token = $self.at();
        match token {
            $pattern => {
                $self.next();
                token
            }
            _ => panic!("Expected {:?}, got: {:?}", stringify!($pattern), $self.at()),
        }
    }};
}

macro_rules! parse_arithmetic {
    ($self:ident, $instruction:ident) => {{
        let (source, imm8) = $self.parse_arithmetic();
        if source == Source8Bit::N8 {
            $self.add(Instruction::$instruction(source), vec![imm8]);
        } else {
            $self.add_instruction(Instruction::$instruction(source));
        }
    }};
}

macro_rules! parse_bitwise {
    ($self:ident, $instruction:ident) => {{
        let token = $self.at();
        let bit = match token {
            Token::Imm8(bit) => {
                $self.next();
                if bit > 7 {
                    panic!("Invalid bit number: {}", bit);
                }
                bit
            }
            _ => panic!("Expected imm8, got: {:?}", token),
        };
        expect!($self, Token::Comma);
        let target = $self.parse_target();
        $self.add_instruction(Instruction::$instruction(bit, target));
    }};
}

#[derive(Debug)]
pub struct FullInstruction {
    pub instruction: Instruction,
    pub operands: Vec<u8>,
}

impl FullInstruction {
    pub fn new(instruction: Instruction, operands: Vec<u8>) -> Self {
        Self {
            instruction,
            operands,
        }
    }

    pub fn from_instr(instruction: Instruction) -> Self {
        Self {
            instruction,
            operands: Vec::new(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        if self.instruction.is_prefixed() {
            bytes.push(0xCB);
        }
        bytes.push(self.instruction.to_byte());
        for operand in &self.operands {
            bytes.push(operand.clone());
        }
        bytes
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    pub instructions: Vec<FullInstruction>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
            instructions: Vec::new(),
        }
    }

    fn at(&self) -> Token {
        self.tokens[self.position].clone()
    }

    fn next(&mut self) {
        self.position += 1;
    }

    fn add_instruction(&mut self, instruction: Instruction) {
        self.instructions
            .push(FullInstruction::from_instr(instruction));
    }

    fn add(&mut self, instruction: Instruction, operands: Vec<u8>) {
        self.instructions
            .push(FullInstruction::new(instruction, operands));
    }

    pub fn parse(&mut self) {
        while self.position < self.tokens.len() {
            match self.at() {
                Token::Mnemonic(mnemonic) => {
                    self.next();
                    match mnemonic.as_str() {
                        "LD" => self.parse_ld(),
                        "INC" => {
                            let target = self.parse_inc_dec();
                            self.add_instruction(Instruction::INC(target))
                        }
                        "DEC" => {
                            let target = self.parse_inc_dec();
                            self.add_instruction(Instruction::DEC(target))
                        }
                        "ADD" => self.parse_add(),
                        "ADC" => parse_arithmetic!(self, ADC),
                        "SUB" => parse_arithmetic!(self, SUB),
                        "SBC" => parse_arithmetic!(self, SBC),
                        "AND" => parse_arithmetic!(self, AND),
                        "XOR" => parse_arithmetic!(self, XOR),
                        "OR" => parse_arithmetic!(self, OR),
                        "CP" => parse_arithmetic!(self, CP),
                        "RET" => match self.at() {
                            Token::JPCondition(condition) => {
                                self.next();
                                self.add_instruction(Instruction::RET(condition.into()));
                            }
                            _ => self.add_instruction(Instruction::RET(JumpTest::Always)),
                        },
                        "PUSH" => {
                            let token = self.at();
                            match token {
                                Token::Register(reg) => {
                                    self.next();
                                    self.add_instruction(Instruction::PUSH(reg.into()));
                                }
                                _ => panic!("Expected BC, DE, HL or AF register, got: {:?}", token),
                            }
                        }
                        "POP" => {
                            let token = self.at();
                            match token {
                                Token::Register(reg) => {
                                    self.next();
                                    self.add_instruction(Instruction::POP(reg.into()));
                                }
                                _ => panic!("Expected BC, DE, HL or AF register, got: {:?}", token),
                            }
                        }
                        "JR" => self.parse_jr(),
                        "JP" => self.parse_jp(),
                        "CALL" => self.parse_call(),
                        "RST" => {
                            let token = self.at();
                            match token {
                                Token::Imm8(imm8) => {
                                    if imm8 % 8 != 0 && imm8 <= 0x38 {
                                        panic!("Invalid RST vector address, got: ${:02X}", imm8);
                                    }
                                    self.next();
                                    self.add_instruction(Instruction::RST(imm8));
                                }
                                _ => panic!("Expected imm8, got: {:?}", token),
                            }
                        }
                        "RETI" => self.add_instruction(Instruction::RETI),
                        "DI" => self.add_instruction(Instruction::DI),
                        "EI" => self.add_instruction(Instruction::EI),
                        "NOP" => self.add_instruction(Instruction::NOP),
                        "STOP" => self.add_instruction(Instruction::STOP),
                        "HALT" => self.add_instruction(Instruction::HALT),
                        "RLCA" => self.add_instruction(Instruction::RLCA),
                        "RLA" => self.add_instruction(Instruction::RLA),
                        "DAA" => self.add_instruction(Instruction::DAA),
                        "SCF" => self.add_instruction(Instruction::SCF),
                        "RRCA" => self.add_instruction(Instruction::RRCA),
                        "RRA" => self.add_instruction(Instruction::RRA),
                        "CPL" => self.add_instruction(Instruction::CPL),
                        "CCF" => self.add_instruction(Instruction::CCF),

                        "RLC" => {
                            let target = self.parse_target();
                            self.add_instruction(Instruction::RLC(target));
                        }
                        "RRC" => {
                            let target = self.parse_target();
                            self.add_instruction(Instruction::RRC(target));
                        }
                        "RL" => {
                            let target = self.parse_target();
                            self.add_instruction(Instruction::RL(target));
                        }
                        "RR" => {
                            let target = self.parse_target();
                            self.add_instruction(Instruction::RR(target));
                        }
                        "SLA" => {
                            let target = self.parse_target();
                            self.add_instruction(Instruction::SLA(target));
                        }
                        "SRA" => {
                            let target = self.parse_target();
                            self.add_instruction(Instruction::SRA(target));
                        }
                        "SWAP" => {
                            let target = self.parse_target();
                            self.add_instruction(Instruction::SWAP(target));
                        }
                        "SRL" => {
                            let target = self.parse_target();
                            self.add_instruction(Instruction::SRL(target));
                        }
                        "BIT" => parse_bitwise!(self, BIT),
                        "RES" => parse_bitwise!(self, RES),
                        "SET" => parse_bitwise!(self, SET),

                        _ => panic!("Invalid mnemonic: {}", mnemonic),
                    }
                }
                Token::NewLine => self.next(),
                _ => panic!("Unexpected token: {:?}", self.at()),
            }
        }
    }

    fn parse_ld(&mut self) {
        let token = self.at();
        match token {
            Token::Register(destination_reg) => {
                self.next();
                expect!(self, Token::Comma);
                match self.at() {
                    Token::Imm8(imm8) => {
                        self.next();
                        self.add(
                            Instruction::LD(LoadType::ByteFromImm(destination_reg.into())),
                            vec![imm8],
                        );
                    }
                    Token::Imm16(imm16) => {
                        self.next();
                        self.add(
                            Instruction::LD(LoadType::WordFromImm(destination_reg.into())),
                            vec![(imm16 & 0xFF) as u8, (imm16 >> 8) as u8],
                        );
                    }
                    Token::Register(source_reg) => {
                        self.next();
                        if destination_reg == Register::SP && source_reg == Register::HL {
                            self.add_instruction(Instruction::LD(LoadType::SPFromHL));
                        } else if destination_reg == Register::HL && source_reg == Register::SP {
                            expect!(self, Token::Plus);
                            match self.at() {
                                Token::Imm8(imm8) => {
                                    self.next();
                                    self.add(Instruction::LD(LoadType::HLFromSPE8), vec![imm8]);
                                }
                                _ => panic!("Expected imm8, got: {:?}", self.at()),
                            }
                        } else {
                            self.add_instruction(Instruction::LD(LoadType::Byte(
                                destination_reg.into(),
                                source_reg.into(),
                            )));
                        }
                    }
                    Token::OpenBracket => {
                        self.next();
                        let token = self.at();
                        match token {
                            Token::Register(deref_reg) => {
                                if deref_reg == Register::HL {
                                    // LD r8, [HL]
                                    self.next();
                                    expect!(self, Token::CloseBracket);
                                    self.add_instruction(Instruction::LD(LoadType::Byte(
                                        destination_reg.into(),
                                        Target8Bit::HLP,
                                    )));
                                } else if destination_reg != Register::A {
                                    panic!("You can only dereference a register into the A register, got: {:?}", destination_reg);
                                } else if deref_reg == Register::C {
                                    // LD A, [C]
                                    self.next();
                                    expect!(self, Token::CloseBracket);
                                    self.add_instruction(Instruction::LD(LoadType::AFromDerefC));
                                } else {
                                    // LD r8, [r16]
                                    self.next();
                                    expect!(self, Token::CloseBracket);
                                    self.add_instruction(Instruction::LD(LoadType::AFromDeref(
                                        deref_reg.into(),
                                    )));
                                }
                            }
                            Token::Imm16(imm16) => {
                                // LD A, [n16]
                                self.next();
                                expect!(self, Token::CloseBracket);
                                self.instructions.push(FullInstruction {
                                    instruction: Instruction::LD(LoadType::AFromA16),
                                    // Split 16-bit immediate into two 8-bit immediate values
                                    operands: vec![(imm16 & 0xFF) as u8, (imm16 >> 8) as u8],
                                });
                            }
                            Token::Imm8(imm8) => {
                                // LD A, [n8]
                                if destination_reg != Register::A {
                                    panic!("You can only dereference an a8 into the A register, got: {:?}", destination_reg);
                                }
                                self.next();
                                expect!(self, Token::CloseBracket);
                                self.instructions.push(FullInstruction {
                                    instruction: Instruction::LD(LoadType::AFromA8),
                                    operands: vec![imm8],
                                });
                            }
                            _ => panic!("Expected register or imm16, found {:?}", self.at()),
                        }
                    }
                    _ => panic!("Expected immediate or register, found {:?}", self.at()),
                }
            }
            Token::OpenBracket => {
                self.next();
                let token = self.at();
                match token {
                    Token::Register(destination_reg) => {
                        if destination_reg == Register::HL {
                            self.next();
                            expect!(self, Token::CloseBracket);
                            expect!(self, Token::Comma);
                            match self.at() {
                                Token::Register(source_reg) => {
                                    // LD [HL], r8
                                    self.next();
                                    self.add_instruction(Instruction::LD(LoadType::Byte(
                                        Target8Bit::HLP,
                                        source_reg.into(),
                                    )));
                                }
                                Token::Imm8(imm8) => {
                                    // LD [HL], n8
                                    self.next();
                                    self.add(
                                        Instruction::LD(LoadType::ByteFromImm(Target8Bit::HLP)),
                                        vec![imm8],
                                    );
                                }
                                _ => panic!("Expected register, found {:?}", self.at()),
                            }
                        } else if destination_reg == Register::C {
                            // LD [C], A
                            self.next();
                            expect!(self, Token::CloseBracket);
                            expect!(self, Token::Comma);
                            expect!(self, Token::Register(Register::A));
                            self.add_instruction(Instruction::LD(LoadType::DerefCFromA));
                        } else {
                            // LD [r16], A
                            self.next();
                            expect!(self, Token::CloseBracket);
                            expect!(self, Token::Comma);
                            expect!(self, Token::Register(Register::A));
                            self.add_instruction(Instruction::LD(LoadType::DerefFromA(
                                destination_reg.into(),
                            )));
                        }
                    }
                    Token::Imm16(imm16) => {
                        self.next();
                        expect!(self, Token::CloseBracket);
                        expect!(self, Token::Comma);

                        let token = self.at();
                        match token {
                            Token::Register(Register::A) => {
                                self.next();
                                self.add(
                                    Instruction::LD(LoadType::A16FromA),
                                    vec![(imm16 & 0xFF) as u8, (imm16 >> 8) as u8],
                                );
                            }
                            Token::Register(Register::SP) => {
                                self.next();
                                self.add(
                                    Instruction::LD(LoadType::A16FromSP),
                                    vec![(imm16 & 0xFF) as u8, (imm16 >> 8) as u8],
                                );
                            }
                            _ => panic!("Expected register A or SP, found {:?}", token),
                        }
                    }
                    Token::Imm8(imm8) => {
                        self.next();
                        expect!(self, Token::CloseBracket);
                        expect!(self, Token::Comma);
                        expect!(self, Token::Register(Register::A));
                        self.add(Instruction::LD(LoadType::A8FromA), vec![imm8]);
                    }
                    _ => panic!("Expected register or imm16, found {:?}", token),
                }
            }
            _ => panic!("Expected register or open bracket, found {:?}", token),
        }
    }

    fn parse_inc_dec(&mut self) -> IncDecTarget {
        let token = self.at();
        match token {
            Token::Register(reg) => {
                self.next();
                reg.into()
            }
            Token::OpenBracket => {
                self.next();
                let token = self.at();
                match token {
                    Token::Register(reg) => {
                        if reg != Register::HL {
                            panic!("You can only dereference HL with an INC or DEC instruction, got: {:?}", reg);
                        }
                        self.next();
                        expect!(self, Token::CloseBracket);
                        IncDecTarget::HLP
                    }
                    _ => panic!("Expected register, found {:?}", token),
                }
            }
            _ => panic!("Expected register or open bracket, found {:?}", token),
        }
    }

    fn parse_jr(&mut self) {
        let token = self.at();
        match token {
            Token::JPCondition(condition) => {
                self.next();
                expect!(self, Token::Comma);
                match self.at() {
                    Token::Imm8(imm8) => {
                        self.next();
                        self.add(Instruction::JR(condition.into()), vec![imm8]);
                    }
                    _ => panic!("Expected imm8, got: {:?}", self.at()),
                }
            }
            Token::Imm8(imm8) => {
                self.next();
                self.add(Instruction::JR(JumpTest::Always), vec![imm8]);
            }
            _ => panic!("Expected condition or imm8, got: {:?}", token),
        }
    }

    fn parse_arithmetic(&mut self) -> (Source8Bit, u8) {
        expect!(self, Token::Register(Register::A));
        expect!(self, Token::Comma);
        let token = self.at();
        match token {
            Token::Register(source_reg) => {
                self.next();
                (source_reg.into(), 0)
            }
            Token::Imm8(imm8) => {
                self.next();
                (Source8Bit::N8, imm8)
            }
            Token::OpenBracket => {
                self.next();
                expect!(self, Token::Register(Register::HL));
                expect!(self, Token::CloseBracket);
                (Source8Bit::HLP, 0)
            }
            _ => panic!("Expected register or imm8, found {:?}", token),
        }
    }

    fn parse_add(&mut self) {
        let token = self.at();
        if token == Token::Register(Register::SP) {
            self.next();
            expect!(self, Token::Comma);
            let imm16 = match self.at() {
                Token::Imm16(imm16) => imm16,
                _ => panic!("Expected imm16, got: {:?}", self.at()),
            };
            self.next();
            self.add(
                Instruction::ADDSP,
                vec![(imm16 & 0xFF) as u8, (imm16 >> 8) as u8],
            );
        } else {
            let (source, imm8) = self.parse_arithmetic();
            if source == Source8Bit::N8 {
                self.add(Instruction::ADD(source), vec![imm8]);
            } else {
                self.add_instruction(Instruction::ADD(source));
            }
        }
    }

    fn parse_jp(&mut self) {
        let token = self.at();
        match token {
            Token::JPCondition(condition) => {
                self.next();
                expect!(self, Token::Comma);
                match self.at() {
                    Token::Imm16(imm16) => {
                        self.next();
                        self.add(
                            Instruction::JP(condition.into()),
                            vec![(imm16 & 0xFF) as u8, (imm16 >> 8) as u8],
                        );
                    }
                    _ => panic!("Expected imm16, got: {:?}", self.at()),
                }
            }
            Token::Imm16(imm16) => {
                self.next();
                self.add(
                    Instruction::JP(JumpTest::Always),
                    vec![(imm16 & 0xFF) as u8, (imm16 >> 8) as u8],
                );
            }
            Token::Register(Register::HL) => {
                self.next();
                self.add_instruction(Instruction::JPHL);
            }
            _ => panic!("Expected condition or imm16, got: {:?}", token),
        }
    }

    fn parse_call(&mut self) {
        let token = self.at();
        match token {
            Token::JPCondition(condition) => {
                self.next();
                expect!(self, Token::Comma);
                match self.at() {
                    Token::Imm16(imm16) => {
                        self.next();
                        self.add(
                            Instruction::CALL(condition.into()),
                            vec![(imm16 & 0xFF) as u8, (imm16 >> 8) as u8],
                        );
                    }
                    _ => panic!("Expected imm16, got: {:?}", self.at()),
                }
            }
            Token::Imm16(imm16) => {
                self.next();
                self.add(
                    Instruction::CALL(JumpTest::Always),
                    vec![(imm16 & 0xFF) as u8, (imm16 >> 8) as u8],
                );
            }
            _ => panic!("Expected condition or imm16, got: {:?}", token),
        }
    }

    fn parse_target(&mut self) -> Target8Bit {
        let token = self.at();
        match token {
            Token::Register(reg) => {
                self.next();
                reg.into()
            }
            Token::OpenBracket => {
                self.next();
                expect!(self, Token::Register(Register::HL));
                expect!(self, Token::CloseBracket);
                Target8Bit::HLP
            }
            _ => panic!("Expected register or open bracket, found {:?}", token),
        }
    }
}
