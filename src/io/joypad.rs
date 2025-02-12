#[inline(always)]
fn bit(condition: bool) -> u8 {
    if condition {
        1
    } else {
        0
    }
}

#[inline(always)]
fn is_set(byte: u8, position: u8) -> bool {
    (byte >> position) & 1 == 1
}

#[derive(Debug)]
pub enum Selected {
    Buttons,
    DPad,
    Both,
    None,
}

pub struct Joypad {
    selected: Selected,
    pub start: bool,
    pub select: bool,
    pub b: bool,
    pub a: bool,
    pub down: bool,
    pub up: bool,
    pub left: bool,
    pub right: bool,
}

impl Joypad {
    pub fn new() -> Joypad {
        Joypad {
            selected: Selected::Buttons,
            start: false,
            select: false,
            b: false,
            a: false,
            down: false,
            up: false,
            left: false,
            right: false,
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        // If the 5th bit is set, the a, b, select and start buttons will be put into the hardware register
        // If the 4th bit is set, the up, down, left and right buttons will be put into the hardware register
        // If neither are set, the hardware register will be set to 0xF
        // The lower 4 bits are read-only
        let button_selected = !is_set(byte, 5);
        let dpad_selected = !is_set(byte, 4);

        if button_selected && dpad_selected {
            self.selected = Selected::Both;
        } else if button_selected {
            self.selected = Selected::Buttons;
        } else if dpad_selected {
            self.selected = Selected::DPad;
        } else {
            self.selected = Selected::None;
        }
    }

    pub fn read_byte(&self) -> u8 {
        // https://gbdev.io/pandocs/Joypad_Input.html
        match self.selected {
            Selected::Buttons => {
                // 1 means the button is released, 0 means it's pressed
                let start_bit = bit(!self.start) << 3;
                let select_bit = bit(!self.select) << 2;
                let b_bit = bit(!self.b) << 1;
                let a_bit = bit(!self.a);

                0b1101_0000 | start_bit | select_bit | b_bit | a_bit
            }
            Selected::DPad => {
                // 1 means the button is released, 0 means it's pressed
                let down_bit = bit(!self.down) << 3;
                let up_bit = bit(!self.up) << 2;
                let left_bit = bit(!self.left) << 1;
                let right_bit = bit(!self.right);

                0b1110_0000 | down_bit | up_bit | left_bit | right_bit
            }
            Selected::Both => {
                // 1 means the button is released, 0 means it's pressed
                let start_bit = bit(!self.start) << 3;
                let select_bit = bit(!self.select) << 2;
                let b_bit = bit(!self.b) << 1;
                let a_bit = bit(!self.a);

                let down_bit = bit(!self.down) << 3;
                let up_bit = bit(!self.up) << 2;
                let left_bit = bit(!self.left) << 1;
                let right_bit = bit(!self.right);

                0b1100_0000
                    | start_bit
                    | select_bit
                    | b_bit
                    | a_bit
                    | down_bit
                    | up_bit
                    | left_bit
                    | right_bit
            }
            Selected::None => 0xFF, // All bits set, meaning no buttons are pressed
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn keys_buttons() {
        let mut joypad = Joypad::new();
        for i in 0..4 {
            match i {
                0 => joypad.a = true,
                1 => joypad.b = true,
                2 => joypad.select = true,
                3 => joypad.start = true,
                _ => unreachable!(),
            };

            joypad.write_byte(0x00);
            assert_eq!(
                joypad.read_byte(),
                0xCF,
                "i: {}, selected: {:?}",
                i,
                joypad.selected
            );

            joypad.write_byte(0x10);
            assert_eq!(
                joypad.read_byte(),
                0xDF & !(1 << i),
                "i: {}, selected: {:?}",
                i,
                joypad.selected
            );

            joypad.write_byte(0x20);
            assert_eq!(
                joypad.read_byte(),
                0xEF,
                "i: {}, selected: {:?}",
                i,
                joypad.selected
            );

            joypad.write_byte(0x30);
            assert_eq!(
                joypad.read_byte(),
                0xFF,
                "i: {}, selected: {:?}",
                i,
                joypad.selected
            );

            match i {
                0 => joypad.a = false,
                1 => joypad.b = false,
                2 => joypad.select = false,
                3 => joypad.start = false,
                _ => unreachable!(),
            };
        }
    }

    #[test]
    fn keys_direction() {
        let mut joypad = Joypad::new();
        for i in 0..4 {
            match i {
                0 => joypad.right = true,
                1 => joypad.left = true,
                2 => joypad.up = true,
                3 => joypad.down = true,
                _ => unreachable!(),
            };

            joypad.write_byte(0x00);
            assert_eq!(
                joypad.read_byte(),
                0xCF,
                "i: {}, selected: {:?}",
                i,
                joypad.selected
            );

            joypad.write_byte(0x10);
            assert_eq!(
                joypad.read_byte(),
                0xDF,
                "i: {}, selected: {:?}",
                i,
                joypad.selected
            );

            joypad.write_byte(0x20);
            assert_eq!(
                joypad.read_byte(),
                0xEF & !(1 << i),
                "i: {}, selected: {:?}",
                i,
                joypad.selected
            );

            joypad.write_byte(0x30);
            assert_eq!(
                joypad.read_byte(),
                0xFF,
                "i: {}, selected: {:?}",
                i,
                joypad.selected
            );

            match i {
                0 => joypad.right = false,
                1 => joypad.left = false,
                2 => joypad.up = false,
                3 => joypad.down = false,
                _ => unreachable!(),
            };
        }
    }
}
