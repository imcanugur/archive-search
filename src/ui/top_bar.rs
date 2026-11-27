use egui::{Frame, RichText, Rounding, Stroke, TextEdit, Ui};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::ui::grid_panel::ViewMode;
use crate::ui::theme::{BG_CANVAS, BORDER_SUBTLE, TEXT_MUTED};

#[derive(Default)]
pub struct TopBarAction {
    pub query_changed: bool,
    pub pick_folder: bool,
    pub go_up: bool,
    pub clear_selected_dir: bool,
}

pub fn show_top_bar(
    ui: &mut Ui,
    tr: &Strings,
    query: &mut String,
    store: Option<&ArchiveStore>,
    selected_dir: Option<u32>,
    view_mode: &mut ViewMode,
    scanning: bool,
) -> TopBarAction {
    let mut action = TopBarAction::default();
    ui.add_space(3.0);
    ui.horizontal(|ui| {
        if ui
            .add_enabled(!scanning, egui::Button::new(tr.open_folder_btn))
            .clicked()
        {
            action.pick_folder = true;
        }
        let can_up = selected_dir.is_some_and(|d| d > 0);
        if ui
            .add_enabled(can_up, egui::Button::new(tr.up_btn))
            .clicked()
        {
            action.go_up = true;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.selectable_value(view_mode, ViewMode::Grid, tr.view_grid);
            ui.selectable_value(view_mode, ViewMode::List, tr.view_list);
            ui.separator();

            if !query.is_empty() && ui.small_button("x").clicked() {
                query.clear();
                action.query_changed = true;
            }
            let search_resp = ui.add_sized(
                [240.0, 26.0],
                TextEdit::singleline(query)
                    .hint_text(tr.search_hint)
                    .margin(egui::vec2(8.0, 4.0)),
            );
            if search_resp.changed() {
                action.query_changed = true;
            }

            let addr_w = ui.available_width().max(120.0);
            render_address_bar(ui, tr, store, selected_dir, addr_w, &mut action);
        });
    });
    ui.add_space(3.0);
    action
}

fn render_address_bar(
    ui: &mut Ui,
    tr: &Strings,
    store: Option<&ArchiveStore>,
    selected_dir: Option<u32>,
    width: f32,
    action: &mut TopBarAction,
) {
    Frame::none()
        .fill(BG_CANVAS)
        .stroke(Stroke::new(1.0_f32, BORDER_SUBTLE))
        .rounding(Rounding::same(4.0))
        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
        .show(ui, |ui| {
            ui.set_width((width - 18.0).max(80.0));
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let text = match (store, selected_dir) {
                    (Some(st), Some(dir_id)) => st.dir_name(dir_id).to_owned(),
                    (Some(st), None) => st.dir_name(0).to_owned(),
                    (None, _) => tr.address_placeholder.to_owned(),
                };
                let color = if store.is_some() {
                    ui.visuals().text_color()
                } else {
                    TEXT_MUTED
                };
                ui.add(egui::Label::new(RichText::new(text).color(color)).truncate());
                if selected_dir.is_some() {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("x").clicked() {
                            action.clear_selected_dir = true;
                        }
                    });
                }
            });
        });
}
