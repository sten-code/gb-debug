use crate::gbmode::GbMode;

#[inline(always)]
fn bit(condition: bool) -> u8 {
    if condition { 1 } else { 0 }
}

#[inline(always)]
fn is_set(byte: u8, position: u8) -> bool {
    (byte >> position) & 1 == 1
}

#[derive(Copy, Clone)]
pub struct FlagsRegister {
    pub zero: bool,
    pub subtract: bool,
    pub half_carry: bool,
    pub carry: bool,
}

impl FlagsRegister {
    pub fn new() -> FlagsRegister {
        FlagsRegister {
            zero: false,
            subtract: false,
            half_carry: false,
            carry: false,
        }
    }
}

impl From<FlagsRegister> for u8 {
    fn from(flag: FlagsRegister) -> u8 {
        bit(flag.zero) << 7
            | bit(flag.subtract) << 6
            | bit(flag.half_carry) << 5
            | bit(flag.carry) << 4
    }
}

impl From<u8> for FlagsRegister {
    fn from(byte: u8) -> Self {
        FlagsRegister {
            zero: is_set(byte, 7),
            subtract: is_set(byte, 6),
            half_carry: is_set(byte, 5),
            carry: is_set(byte, 4),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: FlagsRegister,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

impl Registers {
    pub fn new(gb_mode: GbMode) -> Registers {
        match gb_mode {
            GbMode::Classic => {
                Registers {
                    a: 0x01,
                    f: FlagsRegister {
                        zero: true,
                        subtract: false,
                        half_carry: true,
                        carry: true,
                    },
                    b: 0x00,
                    c: 0x13,
                    d: 0x00,
                    e: 0xD8,
                    h: 0x01,
                    l: 0x4D,
                    pc: 0x0100,
                    sp: 0xFFFE,
                }
            },
            GbMode::Color => {
                Registers {
                    a: 0x11,
                    f: FlagsRegister {
                        zero: true,
                        subtract: false,
                        half_carry: false,
                        carry: false,
                    },
                    b: 0x00,
                    c: 0x00,
                    d: 0xFF,
                    e: 0x56,
                    h: 0x00,
                    l: 0x0D,
                    pc: 0x0100,
                    sp: 0xFFFE,
                }
            },
        }
    }

    pub fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | u8::from(self.f) as u16
    }
    pub fn set_af(&mut self, value: u16) {
        self.a = ((value & 0xFF00) >> 8) as u8;
        self.f = FlagsRegister::from((value & 0xFF) as u8);
    }

    pub fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }

    pub fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }
    pub fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xFF00) >> 8) as u8;
        self.e = (value & 0xFF) as u8;
    }

    pub fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }
    pub fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xFF00) >> 8) as u8;
        self.l = (value & 0xFF) as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_u8() {
        let mut flags = FlagsRegister::new();
        flags.zero = true;
        flags.carry = true;
        let result: u8 = flags.into();
        assert_eq!(result, 0b1001_0000u8);
    }

    #[test]
    fn from_u8() {
        let result: FlagsRegister = 0b1001_0000.into();
        assert_eq!(result.zero, true, "zero");
        assert_eq!(result.subtract, false, "subtract");
        assert_eq!(result.half_carry, false, "half_carry");
        assert_eq!(result.carry, true, "carry");
    }

    #[test]
    fn set_bc() {
        let mut registers = Registers::new();
        registers.set_bc(0b1010_1111_1100_1100);
        assert_eq!(registers.b, 0b1010_1111u8, "b");
        assert_eq!(registers.c, 0b1100_1100u8, "c");
    }

    #[test]
    fn set_f_from_u8() {
        let mut registers = Registers::new();
        let value = 0b1100_0000;
        registers.f = value.into();
        let result: u8 = registers.f.into();
        assert_eq!(result, value);
        assert_eq!(registers.f.zero, true, "zero");
        assert_eq!(registers.f.subtract, true, "subtract");
        assert_eq!(registers.f.half_carry, false, "half_carry");
        assert_eq!(registers.f.carry, false, "carry");
    }

    #[test]
    fn set_f() {
        let mut registers = Registers::new();
        let value: FlagsRegister = 0b0011_0000u8.into();
        registers.f = value;
        assert_eq!(registers.f.zero, false, "zero");
        assert_eq!(registers.f.subtract, false, "subtract");
        assert_eq!(registers.f.half_carry, true, "half_carry");
        assert_eq!(registers.f.carry, true, "carry");
    }
}