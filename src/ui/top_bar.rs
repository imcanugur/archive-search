use egui::{Button, RichText, Rounding, Stroke, Ui};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::ui::filter_bar::render_filter_bar;
use crate::ui::grid_panel::ViewMode;
use crate::ui::theme::{
    ON_SURFACE, ON_SURFACE_VARIANT, OUTLINE_VARIANT, PRIMARY, SECONDARY, SURFACE_LOW,
    SURFACE_LOWEST, TERTIARY,
};
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
    pub size_progress: Option<(u32, u32)>,
}

#[derive(Default)]
pub struct TopBarAction {
    pub query_changed: bool,
    pub clear_all: bool,
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
    ui.separator();
    let fb = render_filter_bar(ui, st.query, st.selected_dir.is_some(), st.size_progress);
    act.query_changed |= fb.query_changed;
    act.clear_all = fb.clear_all;
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
        if tab_pill(ui, &format!("{root_lbl} — Explorer"), !*st.dual_pane) {
            *st.dual_pane = false;
        }
        if tab_pill(ui, "Dual-Pane (F2)", *st.dual_pane) {
            *st.dual_pane = true;
        }
        let open_btn =
            Button::new(RichText::new(format!("+ {}", tr.open_folder_btn)).color(PRIMARY))
                .fill(SURFACE_LOWEST);
        if ui.add_enabled(!st.scanning, open_btn).clicked() {
            act.pick_folder = true;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let pill = Button::new(
                RichText::new("Ctrl+K Command Palette")
                    .small()
                    .color(ON_SURFACE),
            )
            .fill(SURFACE_LOW)
            .rounding(Rounding::same(14.0));
            if ui.add(pill).clicked() {
                act.open_palette = true;
            }
        });
    });
}

fn tab_pill(ui: &mut Ui, label: &str, active: bool) -> bool {
    let (fill, stroke, col) = if active {
        (
            SURFACE_LOWEST,
            Stroke::new(1.0_f32, OUTLINE_VARIANT),
            ON_SURFACE,
        )
    } else {
        (egui::Color32::TRANSPARENT, Stroke::NONE, ON_SURFACE_VARIANT)
    };
    let text = if active {
        RichText::new(label).strong().color(col)
    } else {
        RichText::new(label).color(col)
    };
    ui.add(
        Button::new(text)
            .fill(fill)
            .stroke(stroke)
            .rounding(Rounding::same(6.0)),
    )
    .clicked()
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
