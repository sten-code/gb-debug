#[derive(Copy, Clone, PartialEq)]
pub enum GbMode {
    Classic,
    Color,
    ColorAsClassic,
}

#[derive(Copy, Clone, PartialEq)]
pub enum GbSpeed {
    Single = 1,
    Double = 2,
}
