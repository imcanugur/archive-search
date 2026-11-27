use egui::{Frame, RichText, Rounding, Stroke, Ui};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::index::VisibleDir;
use crate::ui::theme::{
    ON_SURFACE, ON_SURFACE_VARIANT, OUTLINE_VARIANT, PRIMARY, PRIMARY_FIXED, SECONDARY,
    SURFACE_HIGH, SURFACE_LOW, SURFACE_LOWEST, TERTIARY, TERTIARY_BG,
};
use crate::ui::tree_panel::TreeAction;

pub fn render_smart_views_card(
    ui: &mut Ui,
    tr: &Strings,
    store: Option<&ArchiveStore>,
    act: &mut TreeAction,
) {
    Frame::none()
        .fill(SURFACE_LOWEST)
        .stroke(Stroke::new(1.0_f32, OUTLINE_VARIANT))
        .rounding(Rounding::same(8.0))
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(tr.smart_views_heading)
                        .small()
                        .strong()
                        .color(ON_SURFACE_VARIANT),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new("Fast Index").monospace().color(SECONDARY));
                });
            });
            ui.add_space(4.0);
            let total = store.map_or(0, ArchiveStore::file_count);
            if smart_row(ui, "All Media", &format!("{total}"), false) {
                act.smart_query = Some("");
            }
            if smart_row(ui, "All Photos", "type:photo", false) {
                act.smart_query = Some("type:photo");
            }
            if smart_row(ui, "RAW Camera", "type:raw", false) {
                act.smart_query = Some("type:raw");
            }
            if smart_row(ui, "All Videos", "type:video", false) {
                act.smart_query = Some("type:video");
            }
            if smart_row(ui, "Large (>50MB)", "size:>50mb", false) {
                act.smart_query = Some("size:>50mb");
            }
            if smart_row(ui, "Visual Duplicates", "Dedup", true) {
                act.find_duplicates = true;
            }
        });
}

fn smart_row(ui: &mut Ui, label: &str, badge: &str, warn: bool) -> bool {
    let fill = if warn {
        TERTIARY_BG
    } else {
        egui::Color32::TRANSPARENT
    };
    let col = if warn { TERTIARY } else { ON_SURFACE };
    let mut clicked = false;
    Frame::none()
        .fill(fill)
        .rounding(Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(6.0, 3.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(false, RichText::new(label).color(col))
                    .clicked()
                {
                    clicked = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(badge).monospace().color(ON_SURFACE_VARIANT));
                });
            });
        });
    clicked
}

pub fn render_dir_row(
    ui: &mut Ui,
    store: &ArchiveStore,
    row: VisibleDir,
    selected_dir: Option<u32>,
    merge_queue: &mut Vec<u32>,
) -> bool {
    let is_sel = selected_dir == Some(row.dir_id);
    let mut queued = merge_queue.contains(&row.dir_id);
    let indent = f32::from(row.depth.min(10)) * 10.0;
    let fill = if queued {
        SURFACE_HIGH
    } else if is_sel {
        SURFACE_LOW
    } else {
        egui::Color32::TRANSPARENT
    };
    let mut clicked = false;
    Frame::none()
        .fill(fill)
        .rounding(Rounding::same(5.0))
        .inner_margin(egui::Margin::symmetric(4.0, 2.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(indent);
                if row.dir_id > 0 && ui.checkbox(&mut queued, "").changed() {
                    if queued {
                        if merge_queue.len() >= 2 {
                            merge_queue.remove(0);
                        }
                        merge_queue.push(row.dir_id);
                    } else {
                        merge_queue.retain(|&id| id != row.dir_id);
                    }
                }
                if ui
                    .selectable_label(is_sel, store.dir_name(row.dir_id))
                    .clicked()
                {
                    clicked = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let col = if queued || is_sel {
                        PRIMARY
                    } else {
                        ON_SURFACE_VARIANT
                    };
                    let bg = if queued {
                        PRIMARY_FIXED
                    } else {
                        egui::Color32::TRANSPARENT
                    };
                    Frame::none()
                        .fill(bg)
                        .rounding(Rounding::same(8.0))
                        .show(ui, |ui| {
                            let cnt = format!("{}", row.match_count);
                            ui.label(RichText::new(cnt).monospace().color(col));
                        });
                });
            });
        });
    clicked
}
