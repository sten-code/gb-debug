use egui::{Ui, WidgetText};
use crate::cpu::CPU;
use crate::ui::state::State;
use crate::ui::windows::*;
use crate::ui::windows::GameWindow;
use crate::ui::windows::Window;

pub struct Context {
    pub state: State,
    pub game_window: GameWindow,
    pub disassembly_window: DisassemblyWindow,
    pub breakpoints_window: BreakpointsWindow,
    pub registers_window: RegistersWindow,
    pub memory_dump_window: MemoryDumpWindow,
    pub tile_map_viewer: TileMapViewer,
}

impl Context {
    pub fn new(cc: &eframe::CreationContext<'_>, cpu: Box<CPU>) -> Self {
        let state = State::new(cc, cpu);
        let disassembly_window = DisassemblyWindow::new(&state);
        Self {
            state,
            game_window: GameWindow::new(),
            disassembly_window,
            breakpoints_window: BreakpointsWindow::new(),
            registers_window: RegistersWindow {},
            memory_dump_window: MemoryDumpWindow {},
            tile_map_viewer: TileMapViewer::new(&cc.egui_ctx),
        }
    }
}

impl TabViewer for Context {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.as_str().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab.as_str() {
            "Game Window" => self.game_window.show(&mut self.state, ui),
            "Disassembly" => self.disassembly_window.show(&mut self.state, ui),
            "Breakpoints" => self.breakpoints_window.show(&mut self.state, ui),
            "Registers" => self.registers_window.show(&mut self.state, ui),
            "Memory Dump" => self.memory_dump_window.show(&mut self.state, ui),
            "Tile Map Viewer" => self.tile_map_viewer.show(&mut self.state, ui),
            _ => {}
        }
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }
}
