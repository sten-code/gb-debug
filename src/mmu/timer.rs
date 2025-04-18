pub struct Timer {
    divider: u8,
    counter: u8,
    modulo: u8,
    enabled: bool,
    step: u32,
    internal_counter: u32,
    internal_divider: u32,
    pub interrupt: u8,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            divider: 0,
            counter: 0,
            modulo: 0,
            enabled: false,
            step: 1024,
            internal_counter: 0,
            internal_divider: 0,
            interrupt: 0,
        }
    }

    pub fn read_byte(&self, a: u16) -> u8 {
        assert!(
            a >= 0xFF04 && a <= 0xFF07,
            "Timer does not handler write {:4X}",
            a
        );
        match a {
            0xFF04 => self.divider,
            0xFF05 => self.counter,
            0xFF06 => self.modulo,
            0xFF07 => {
                0xF8 | (if self.enabled { 0x4 } else { 0 })
                    | (match self.step {
                        16 => 1,
                        64 => 2,
                        256 => 3,
                        _ => 0,
                    })
            }
            _ => unreachable!(),
        }
    }

    pub fn write_byte(&mut self, a: u16, v: u8) {
        assert!(
            a >= 0xFF04 && a <= 0xFF07,
            "Timer does not handler write {:4X}",
            a
        );
        match a {
            0xFF04 => {
                self.divider = 0;
            }
            0xFF05 => {
                self.counter = v;
            }
            0xFF06 => {
                self.modulo = v;
            }
            0xFF07 => {
                self.enabled = v & 0x4 != 0;
                self.step = match v & 0x3 {
                    1 => 16,
                    2 => 64,
                    3 => 256,
                    _ => 1024,
                };
            }
            _ => unreachable!(),
        };
    }

    pub fn step(&mut self, ticks: u32) {
        self.internal_divider += ticks;
        while self.internal_divider >= 256 {
            self.divider = self.divider.wrapping_add(1);
            self.internal_divider -= 256;
        }

        if self.enabled {
            self.internal_counter += ticks;

            while self.internal_counter >= self.step {
                self.counter = self.counter.wrapping_add(1);
                if self.counter == 0 {
                    self.counter = self.modulo;
                    self.interrupt |= 0x04;
                }
                self.internal_counter -= self.step;
            }
        }
    }
}
