use egui::{Button, Color32, Frame, RichText, Rounding, Stroke, Ui};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::ui::theme::{
    ON_SURFACE, ON_SURFACE_VARIANT, OUTLINE_VARIANT, PRIMARY, PRIMARY_DARK, PRIMARY_FIXED,
    SECONDARY, SECONDARY_BG, SURFACE_HIGH, SURFACE_LOWEST,
};

#[derive(Default)]
pub struct DrawerAction {
    pub open_dry_run: bool,
    pub merge_now: bool,
    pub trigger_undo: bool,
    pub clear_queue: bool,
}

pub fn show_merge_drawer(
    ui: &mut Ui,
    tr: &Strings,
    store: &ArchiveStore,
    dir_a: u32,
    dir_b: u32,
    can_undo: bool,
) -> DrawerAction {
    let mut act = DrawerAction::default();
    let name_a = store.dir_name(dir_a);
    let name_b = store.dir_name(dir_b);
    let merged = format!("{name_a}-{name_b}");

    Frame::none()
        .fill(SURFACE_LOWEST)
        .stroke(Stroke::new(1.0_f32, OUTLINE_VARIANT))
        .rounding(Rounding::same(10.0))
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                Frame::none()
                    .fill(SECONDARY_BG)
                    .rounding(Rounding::same(5.0))
                    .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("Folder Merge Preview")
                                .small()
                                .strong()
                                .color(SECONDARY),
                        );
                    });
                ui.label(
                    RichText::new("Pattern: {A}-{B}")
                        .monospace()
                        .color(ON_SURFACE_VARIANT),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("x").clicked() {
                        act.clear_queue = true;
                    }
                    let merge_btn = Button::new(
                        RichText::new(tr.merge_execute_btn)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(PRIMARY_DARK);
                    if ui.add(merge_btn).clicked() {
                        act.merge_now = true;
                    }
                    if ui.button(tr.merge_dry_run_btn).clicked() {
                        act.open_dry_run = true;
                    }
                    if can_undo && ui.button(tr.merge_undo_btn).clicked() {
                        act.trigger_undo = true;
                    }
                });
            });
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                pill(ui, name_a, SURFACE_HIGH, ON_SURFACE);
                ui.label(RichText::new("+").strong().color(PRIMARY));
                pill(ui, name_b, SURFACE_HIGH, ON_SURFACE);
                ui.label(RichText::new("->").strong().color(ON_SURFACE_VARIANT));
                pill(ui, &merged, PRIMARY_FIXED, PRIMARY);
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Safe BLAKE3 dedup & conflict suffixing enabled")
                        .small()
                        .color(SECONDARY),
                );
            });
        });
    act
}

fn pill(ui: &mut Ui, text: &str, fill: Color32, col: Color32) {
    Frame::none()
        .fill(fill)
        .rounding(Rounding::same(5.0))
        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
        .show(ui, |ui| {
            ui.label(RichText::new(text).small().strong().color(col));
        });
}
