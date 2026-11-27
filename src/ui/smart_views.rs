use egui::{pos2, vec2, Color32, Frame, Rect, RichText, Rounding, Stroke, Ui};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::ui::theme::{
    ON_SURFACE, ON_SURFACE_VARIANT, OUTLINE_VARIANT, PRIMARY, PRIMARY_FIXED, SECONDARY,
    SURFACE_CONTAINER, SURFACE_HIGH, SURFACE_LOWEST, TERTIARY, TERTIARY_BG,
};
use crate::ui::tree_panel::TreeAction;

pub fn render_smart_views_card(
    ui: &mut Ui,
    tr: &Strings,
    store: Option<&ArchiveStore>,
    active_query: &str,
    visual_active: bool,
    act: &mut TreeAction,
) {
    let q = active_query.trim();
    Frame::none()
        .fill(SURFACE_LOWEST)
        .stroke(Stroke::new(1.0_f32, OUTLINE_VARIANT))
        .rounding(Rounding::same(8.0))
        .inner_margin(10.0)
        .show(ui, |ui| {
            smart_header(ui, tr);
            ui.add_space(4.0);
            let total = store.map_or(0, ArchiveStore::file_count);
            if smart_row(
                ui,
                "All Media",
                &format!("{total}"),
                q.is_empty() && !visual_active,
                false,
            ) {
                act.smart_query = Some("");
            }
            if smart_row(
                ui,
                "All Photos",
                "type:photo",
                q.contains("type:photo"),
                false,
            ) {
                act.smart_query = Some("type:photo");
            }
            if smart_row(ui, "RAW Camera", "type:raw", q.contains("type:raw"), false) {
                act.smart_query = Some("type:raw");
            }
            if smart_row(
                ui,
                "All Videos",
                "type:video",
                q.contains("type:video"),
                false,
            ) {
                act.smart_query = Some("type:video");
            }
            if smart_row(
                ui,
                "Large (>50MB)",
                "size:>50mb",
                q.contains("size:>50mb"),
                false,
            ) {
                act.smart_query = Some("size:>50mb");
            }
            if smart_row(ui, "Visual Duplicates", "Dedup", visual_active, true) {
                act.find_duplicates = true;
            }
        });
}

fn smart_header(ui: &mut Ui, tr: &Strings) {
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
}

fn smart_row(ui: &mut Ui, label: &str, badge: &str, active: bool, warn: bool) -> bool {
    let fill = if active {
        SURFACE_HIGH
    } else if warn {
        TERTIARY_BG
    } else {
        Color32::TRANSPARENT
    };
    let col = if active {
        PRIMARY
    } else if warn {
        TERTIARY
    } else {
        ON_SURFACE
    };
    let mut clicked = false;
    let resp = Frame::none()
        .fill(fill)
        .rounding(Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(8.0, 3.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                clicked = ui
                    .selectable_label(false, RichText::new(label).color(col))
                    .clicked();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let bg = if active {
                        PRIMARY_FIXED
                    } else {
                        SURFACE_CONTAINER
                    };
                    Frame::none()
                        .fill(bg)
                        .rounding(Rounding::same(8.0))
                        .inner_margin(egui::Margin::symmetric(6.0, 1.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new(badge).monospace().color(col));
                        });
                });
            });
        })
        .response;
    if active {
        let bar = Rect::from_min_max(
            resp.rect.min + vec2(0.0, 3.0),
            pos2(resp.rect.min.x + 3.0, resp.rect.max.y - 3.0),
        );
        ui.painter().rect_filled(bar, Rounding::same(1.5), PRIMARY);
    }
    clicked
}
