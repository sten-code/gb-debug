#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GbMode {
    Classic,
    Color,
}

#[derive(Copy, Clone, PartialEq)]
pub enum GbSpeed {
    Single = 1,
    Double = 2,
}
