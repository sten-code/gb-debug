use eframe::egui;
use eframe::emath::{vec2, Vec2};
use eframe::epaint::Stroke;
use egui::{Id, Response, Sense, TextStyle, Ui};
use egui_tiles::{TabState, TileId, Tiles};
use crate::cpu::CPU;
use crate::ui::State;
use crate::ui::windows::*;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Pane {
    Disassembly(Disassembly),
    GameWindow(GameWindow),
    Breakpoints(Breakpoints),
    Registers(Registers),
    MemoryView(MemoryView),
    TileMapViewer(TileMapViewer),
}

impl Pane {
    pub fn ui(&mut self, state: &mut State, ui: &mut Ui) -> egui_tiles::UiResponse {
        match self {
            Pane::Disassembly(view) => view.show(state, ui),
            Pane::GameWindow(view) => view.show(state, ui),
            Pane::Breakpoints(view) => view.show(state, ui),
            Pane::Registers(view) => view.show(state, ui),
            Pane::MemoryView(view) => view.show(state, ui),
            Pane::TileMapViewer(view) => view.show(state, ui),
        }
        egui_tiles::UiResponse::None
    }
}

pub struct TreeManager {
    simplification_options: egui_tiles::SimplificationOptions,
    pub state: State,
}

impl TreeManager {
    pub fn new(cc: &eframe::CreationContext<'_>, cpu: Option<Box<CPU>>) -> Self {
        let mut simplification_options = egui_tiles::SimplificationOptions::default();
        simplification_options.all_panes_must_have_tabs = true;

        Self {
            simplification_options,
            state: State::new(cc, cpu),
        }
    }
}

impl egui_tiles::Behavior<Pane> for TreeManager {
    fn pane_ui(
        &mut self,
        ui: &mut Ui,
        _tile_id: TileId,
        view: &mut Pane,
    ) -> egui_tiles::UiResponse {
        view.ui(&mut self.state, ui)
    }

    fn tab_title_for_pane(&mut self, view: &Pane) -> egui::WidgetText {
        match view {
            Pane::Disassembly(_) => "Disassembly".into(),
            Pane::GameWindow(_) => "Game Window".into(),
            Pane::Breakpoints(_) => "Breakpoints".into(),
            Pane::Registers(_) => "Registers".into(),
            Pane::MemoryView(_) => "Memory View".into(),
            Pane::TileMapViewer(_) => "Tile Map Viewer".into(),
        }
    }

    fn is_tab_closable(&self, _tiles: &Tiles<Pane>, _tile_id: TileId) -> bool {
        false
    }

    #[allow(clippy::fn_params_excessive_bools)]
    fn tab_ui(
        &mut self,
        tiles: &mut Tiles<Pane>,
        ui: &mut Ui,
        id: Id,
        tile_id: TileId,
        state: &TabState,
    ) -> Response {
        let text = self.tab_title_for_tile(tiles, tile_id);
        let close_btn_size = Vec2::splat(self.close_button_outer_size());
        let close_btn_left_padding = 4.0;
        let font_id = TextStyle::Button.resolve(ui.style());
        let galley = text.into_galley(ui, Some(egui::TextWrapMode::Extend), f32::INFINITY, font_id);

        let x_margin = self.tab_title_spacing(ui.visuals());

        let button_width = galley.size().x
            + 2.0 * x_margin
            + f32::from(state.closable) * (close_btn_left_padding + close_btn_size.x);
        let (_, tab_rect) = ui.allocate_space(vec2(button_width, ui.available_height()));

        let tab_response = ui
            .interact(tab_rect, id, Sense::click_and_drag())
            .on_hover_cursor(egui::CursorIcon::PointingHand);

        // Show a gap when dragged
        if ui.is_rect_visible(tab_rect) && !state.is_being_dragged {
            let bg_color = self.tab_bg_color(ui.visuals(), tiles, tile_id, state);
            let stroke = self.tab_outline_stroke(ui.visuals(), tiles, tile_id, state);
            ui.painter()
                .rect(tab_rect.shrink(0.5), 0.0, bg_color, stroke, egui::StrokeKind::Middle);

            if state.active {
                // Make the tab name area connect with the tab ui area:
                ui.painter().hline(
                    tab_rect.x_range(),
                    tab_rect.bottom(),
                    Stroke::new(stroke.width + 1.0, bg_color),
                );
            }

            // Prepare title's text for rendering
            let text_color = self.tab_text_color(ui.visuals(), tiles, tile_id, state);
            let text_position = egui::Align2::LEFT_CENTER
                .align_size_within_rect(galley.size(), tab_rect.shrink(x_margin))
                .min;

            // Render the title
            ui.painter().galley(text_position, galley, text_color);

            // Conditionally render the close button
            if state.closable {
                let close_btn_rect = egui::Align2::RIGHT_CENTER
                    .align_size_within_rect(close_btn_size, tab_rect.shrink(x_margin));

                // Allocate
                let close_btn_id = ui.auto_id_with("tab_close_btn");
                let close_btn_response = ui
                    .interact(close_btn_rect, close_btn_id, Sense::click_and_drag())
                    .on_hover_cursor(egui::CursorIcon::Default);

                let visuals = ui.style().interact(&close_btn_response);

                // Scale based on the interaction visuals
                let rect = close_btn_rect
                    .shrink(self.close_button_inner_margin())
                    .expand(visuals.expansion);
                let stroke = visuals.fg_stroke;

                // paint the crossed lines
                ui.painter() // paints \
                    .line_segment([rect.left_top(), rect.right_bottom()], stroke);
                ui.painter() // paints /
                    .line_segment([rect.right_top(), rect.left_bottom()], stroke);

                // Give the user a chance to react to the close button being clicked
                // Only close if the user returns true (handled)
                if close_btn_response.clicked() {
                    log::debug!("Tab close requested for tile: {tile_id:?}");

                    // Close the tab if the implementation wants to
                    if self.on_tab_close(tiles, tile_id) {
                        log::debug!("Implementation confirmed close request for tile: {tile_id:?}");

                        tiles.remove(tile_id);
                    } else {
                        log::debug!("Implementation denied close request for tile: {tile_id:?}");
                    }
                }
            }
        }

        self.on_tab_button(tiles, tile_id, tab_response)
    }

    fn tab_bar_height(&self, _style: &egui::Style) -> f32 {
        24.0
    }

    fn gap_width(&self, _style: &egui::Style) -> f32 {
        2.0
    }

    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        self.simplification_options
    }
}
