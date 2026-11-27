use egui::{Button, RichText, Rounding, Ui};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::ui::grid_panel::ViewMode;
use crate::ui::theme::{ON_SURFACE, PRIMARY, SECONDARY, SURFACE_LOWEST, TERTIARY};
use crate::ui::top_bar_widgets::{render_breadcrumbs, render_search_capsule};

pub struct TopBarState<'a> {
    pub query: &'a mut String,
    pub store: Option<&'a ArchiveStore>,
    pub selected_dir: Option<u32>,
    pub elapsed_us: u64,
    pub view_mode: &'a mut ViewMode,
    pub dual_pane: &'a mut bool,
    pub show_inspector: &'a mut bool,
    pub scanning: bool,
}

#[derive(Default)]
pub struct TopBarAction {
    pub query_changed: bool,
    pub pick_folder: bool,
    pub select_dir: Option<Option<u32>>,
    pub go_up: bool,
    pub open_merge: bool,
    pub find_duplicates: bool,
    pub open_palette: bool,
}

pub fn show_top_bar(ui: &mut Ui, tr: &Strings, st: &mut TopBarState<'_>) -> TopBarAction {
    let mut act = TopBarAction::default();
    render_tab_strip(ui, tr, st, &mut act);
    ui.separator();
    render_command_strip(ui, tr, st, &mut act);
    act
}

fn render_tab_strip(
    ui: &mut Ui,
    tr: &Strings,
    st: &mut TopBarState<'_>,
    act: &mut TopBarAction,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(tr.app_title).strong().color(PRIMARY));
        ui.add_space(6.0);
        let root_lbl = st.store.map_or("All Media", |s| s.dir_name(0));
        if ui
            .selectable_label(!*st.dual_pane, format!("{root_lbl} — Explorer"))
            .clicked()
        {
            *st.dual_pane = false;
        }
        if ui
            .selectable_label(*st.dual_pane, "Dual Compare (F2)")
            .clicked()
        {
            *st.dual_pane = true;
        }
        if ui
            .add_enabled(!st.scanning, Button::new(tr.open_folder_btn))
            .clicked()
        {
            act.pick_folder = true;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let pill = Button::new(RichText::new("Ctrl+K Palette").small().color(ON_SURFACE))
                .fill(SURFACE_LOWEST)
                .rounding(Rounding::same(12.0));
            if ui.add(pill).clicked() {
                act.open_palette = true;
            }
        });
    });
}

fn render_command_strip(
    ui: &mut Ui,
    tr: &Strings,
    st: &mut TopBarState<'_>,
    act: &mut TopBarAction,
) {
    ui.horizontal(|ui| {
        let can_up = st.selected_dir.is_some_and(|d| d > 0);
        if ui.add_enabled(can_up, Button::new(tr.up_btn)).clicked() {
            act.go_up = true;
        }
        render_breadcrumbs(ui, tr, st.store, st.selected_dir, act);
        render_search_capsule(ui, tr, st.query, st.elapsed_us, act);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.selectable_label(*st.show_inspector, "Details").clicked() {
                *st.show_inspector = !*st.show_inspector;
            }
            ui.selectable_value(st.view_mode, ViewMode::List, tr.view_list);
            ui.selectable_value(st.view_mode, ViewMode::Grid, tr.view_grid);
            ui.separator();
            let enabled = st.store.is_some() && !st.scanning;
            let dup = Button::new(RichText::new("Find Duplicates").color(TERTIARY));
            if ui.add_enabled(enabled, dup).clicked() {
                act.find_duplicates = true;
            }
            let merge = Button::new(RichText::new(tr.dual_merge_btn).color(SECONDARY));
            if ui.add_enabled(enabled, merge).clicked() {
                act.open_merge = true;
            }
        });
    });
}
