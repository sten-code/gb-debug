mod disassembly;
pub use disassembly::*;
mod game_window;
pub use game_window::*;
mod breakpoints;
pub use breakpoints::*;
mod registers;
pub use registers::*;
mod memory_view;
pub use memory_view::*;
mod tile_map_viewer;
pub use tile_map_viewer::*;

use crate::ui::State;
use eframe::egui;

pub trait Window {
    fn show(&mut self, state: &mut State, ui: &mut egui::Ui);
}
