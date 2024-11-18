use std::iter::Peekable;
use std::str::Chars;
use env_logger::Target;
use crate::cpu::instruction::{DerefTarget, IncDecTarget, JumpTest, Reg16Bit, Source8Bit, StackTarget, Target8Bit};

#[derive(Clone, Debug, PartialEq)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    AF,
    BC,
    DE,
    HL,
    HLI, // HL+
    HLD, // HL-
    SP,
}

impl Into<Target8Bit> for Register {
    fn into(self) -> Target8Bit {
        match self {
            Register::A => Target8Bit::A,
            Register::B => Target8Bit::B,
            Register::C => Target8Bit::C,
            Register::D => Target8Bit::D,
            Register::E => Target8Bit::E,
            Register::H => Target8Bit::H,
            Register::L => Target8Bit::L,
            Register::HLI => Target8Bit::HLP,
            _ => panic!("Invalid register: {:?}", self),
        }
    }
}

impl Into<DerefTarget> for Register {
    fn into(self) -> DerefTarget {
        match self {
            Register::BC => DerefTarget::BCP,
            Register::DE => DerefTarget::DEP,
            Register::HLI => DerefTarget::HLI,
            Register::HLD => DerefTarget::HLD,
            _ => panic!("Invalid deref target: {:?}", self),
        }
    }
}

impl Into<Reg16Bit> for Register {
    fn into(self) -> Reg16Bit {
        match self {
            Register::BC => Reg16Bit::BC,
            Register::DE => Reg16Bit::DE,
            Register::HL => Reg16Bit::HL,
            Register::SP => Reg16Bit::SP,
            _ => panic!("Invalid 16-bit register: {:?}", self),
        }
    }
}

impl Into<IncDecTarget> for Register {
    fn into(self) -> IncDecTarget {
        match self {
            Register::A => IncDecTarget::A,
            Register::B => IncDecTarget::B,
            Register::C => IncDecTarget::C,
            Register::D => IncDecTarget::D,
            Register::E => IncDecTarget::E,
            Register::H => IncDecTarget::H,
            Register::L => IncDecTarget::L,
            Register::BC => IncDecTarget::BC,
            Register::DE => IncDecTarget::DE,
            Register::HL => IncDecTarget::HL,
            Register::SP => IncDecTarget::SP,
            _ => panic!("Invalid inc/dec target: {:?}", self),
        }
    }
}

impl Into<Source8Bit> for Register {
    fn into(self) -> Source8Bit {
        match self {
            Register::A => Source8Bit::A,
            Register::B => Source8Bit::B,
            Register::C => Source8Bit::C,
            Register::D => Source8Bit::D,
            Register::E => Source8Bit::E,
            Register::H => Source8Bit::H,
            Register::L => Source8Bit::L,
            Register::HLI => Source8Bit::HLP,
            _ => panic!("Invalid source register: {:?}", self),
        }
    }
}

impl Into<StackTarget> for Register {
    fn into(self) -> StackTarget {
        match self {
            Register::AF => StackTarget::AF,
            Register::BC => StackTarget::BC,
            Register::DE => StackTarget::DE,
            Register::HL => StackTarget::HL,
            _ => panic!("Invalid stack target: {:?}", self),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum JPCondition {
    NZ,
    Z,
    NC,
    C,
}

impl Into<JumpTest> for JPCondition {
    fn into(self) -> JumpTest {
        match self {
            JPCondition::NZ => JumpTest::NotZero,
            JPCondition::Z => JumpTest::Zero,
            JPCondition::NC => JumpTest::NotCarry,
            JPCondition::C => JumpTest::Carry,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Mnemonic(String),
    Register(Register),
    JPCondition(JPCondition),
    Imm16(u16),
    Imm8(u8),
    Comma,
    OpenBracket,
    CloseBracket,
    Plus,
    NewLine,
    EOF,
}

pub struct Lexer<'a> {
    code: &'a str,
    chars: Peekable<Chars<'a>>,
    pub tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(code: &str) -> Lexer {
        Lexer {
            code,
            chars: code.chars().peekable(),
            tokens: Vec::new(),
        }
    }

    fn tokenize_hex_immediate(&mut self) {
        // Parse hex number
        let mut hex = String::new();
        loop {
            let char = self.chars.peek();
            if char.is_none() {
                break;
            }
            let char = char.unwrap();
            if char.is_digit(16) {
                hex.push(*char);
                self.chars.next();
            } else {
                break;
            }
        }
        if hex.len() == 4 {
            self.tokens.push(Token::Imm16(u16::from_str_radix(&hex, 16).unwrap()));
        } else if hex.len() == 2 {
            self.tokens.push(Token::Imm8(u8::from_str_radix(&hex, 16).unwrap()));
        } else {
            panic!("Invalid hex number: ${}, must be either 2 or 4 characters long.", hex);
        }
    }

    fn tokenize_decimal_immediate(&mut self, first_char: char) {
        let mut decimal = String::new();
        decimal.push(first_char);
        loop {
            let char = self.chars.peek();
            if char.is_none() {
                break;
            }
            let char = char.unwrap();
            if char.is_digit(10) {
                decimal.push(*char);
                self.chars.next();
            } else {
                break;
            }
        }
        if first_char == '-' {
            let decimal = i8::from_str_radix(&decimal, 10).unwrap();
            if decimal < -128 || decimal > 127 {
                panic!("Signed decimal immediate must be between -128 and 127, got: {}", decimal);
            }
            self.tokens.push(Token::Imm8(decimal as u8));
        } else {
            let decimal = u8::from_str_radix(&decimal, 10).unwrap();
            if decimal > 0xFF {
                panic!("Decimal immediate must be between 0 and 255, got: {}", decimal);
            }
            self.tokens.push(Token::Imm8(decimal));
        }
    }

    fn tokenize_identifier(&mut self, first_char: char) -> String {
        let mut identifier = String::new();
        identifier.push(first_char);
        loop {
            let char = self.chars.peek();
            if char.is_none() {
                break;
            }
            let char = char.unwrap();
            if char.is_alphabetic() {
                identifier.push(*char);
                self.chars.next();
            } else {
                break;
            }
        }
        identifier
    }

    pub fn tokenize(&mut self) {
        while let Some(char) = self.chars.next() {
            if char == '\n' {
                self.tokens.push(Token::NewLine);
            } else if char.is_whitespace() {
                continue;
            } else if char == '$' {
                self.tokenize_hex_immediate();
            } else if char.is_digit(10) || char == '-' {
                self.tokenize_decimal_immediate(char);
            } else if char == '\'' {
                let character = self.chars.next().unwrap();
                if character == '\\' {
                    let escape = self.chars.next().unwrap();
                    let character = match escape {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '\\' => '\\',
                        '\'' => '\'',
                        _ => panic!("Invalid escape character: '{}'", escape),
                    };
                    if self.chars.next() != Some('\'') {
                        panic!("Expected closing quote, got: '{}'", char);
                    }
                    self.tokens.push(Token::Imm8(character as u8));
                } else {
                    if self.chars.next() != Some('\'') {
                        panic!("Expected closing quote, got: '{}'", char);
                    }

                    self.tokens.push(Token::Imm8(character as u8));
                }
            } else if char == ',' {
                self.tokens.push(Token::Comma);
            } else if char == '[' {
                self.tokens.push(Token::OpenBracket);
            } else if char == ']' {
                self.tokens.push(Token::CloseBracket);
            } else if char == '+' {
                self.tokens.push(Token::Plus);
            } else if char.is_alphabetic() {
                let identifier = self.tokenize_identifier(char).to_uppercase();
                if self.tokens.last() == Some(&Token::Mnemonic("JP".to_owned()))
                    || self.tokens.last() == Some(&Token::Mnemonic("CALL".to_owned()))
                    || self.tokens.last() == Some(&Token::Mnemonic("JR".to_owned()))
                    || self.tokens.last() == Some(&Token::Mnemonic("RET".to_owned())) {
                    match identifier.as_str() {
                        "NZ" => {
                            self.tokens.push(Token::JPCondition(JPCondition::NZ));
                            continue;
                        }
                        "Z" => {
                            self.tokens.push(Token::JPCondition(JPCondition::Z));
                            continue;
                        }
                        "NC" => {
                            self.tokens.push(Token::JPCondition(JPCondition::NC));
                            continue;
                        }
                        "C" => {
                            self.tokens.push(Token::JPCondition(JPCondition::C));
                            continue;
                        }
                        _ => {}
                    }
                }

                match identifier.as_str() {
                    "A" => self.tokens.push(Token::Register(Register::A)),
                    "B" => self.tokens.push(Token::Register(Register::B)),
                    "C" => self.tokens.push(Token::Register(Register::C)),
                    "D" => self.tokens.push(Token::Register(Register::D)),
                    "E" => self.tokens.push(Token::Register(Register::E)),
                    "H" => self.tokens.push(Token::Register(Register::H)),
                    "L" => self.tokens.push(Token::Register(Register::L)),
                    "AF" => self.tokens.push(Token::Register(Register::AF)),
                    "BC" => self.tokens.push(Token::Register(Register::BC)),
                    "DE" => self.tokens.push(Token::Register(Register::DE)),
                    "HL" => match self.chars.peek() {
                        Some(&'+') => {
                            self.chars.next();
                            self.tokens.push(Token::Register(Register::HLI));
                        }
                        Some(&'-') => {
                            self.chars.next();
                            self.tokens.push(Token::Register(Register::HLD));
                        }
                        _ => self.tokens.push(Token::Register(Register::HL))
                    },
                    "SP" => self.tokens.push(Token::Register(Register::SP)),
                    _ => self.tokens.push(Token::Mnemonic(identifier)),
                }
            } else {
                panic!("Invalid character: {}", char);
            }
        }
    }
}
