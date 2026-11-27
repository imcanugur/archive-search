use egui::{pos2, vec2, Color32, Frame, Rect, RichText, Rounding, Stroke, Ui};

use crate::index::store::ArchiveStore;
use crate::index::VisibleDir;
use crate::ui::theme::{
    ON_SURFACE_VARIANT, OUTLINE_VARIANT, PRIMARY, PRIMARY_FIXED, SURFACE_HIGH, SURFACE_LOW,
};

pub fn render_dir_row(
    ui: &mut Ui,
    store: &ArchiveStore,
    row: VisibleDir,
    selected_dir: Option<u32>,
    merge_queue: &mut Vec<u32>,
) -> bool {
    let is_sel = selected_dir == Some(row.dir_id);
    let mut queued = merge_queue.contains(&row.dir_id);
    let indent = f32::from(row.depth.min(8)) * 12.0;
    let fill = if queued || is_sel {
        SURFACE_HIGH
    } else {
        Color32::TRANSPARENT
    };
    let mut clicked = false;
    let resp = Frame::none()
        .fill(fill)
        .rounding(Rounding::same(5.0))
        .inner_margin(egui::Margin::symmetric(4.0, 2.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(indent);
                if row.dir_id > 0 && ui.checkbox(&mut queued, "").changed() {
                    update_merge_queue(merge_queue, row.dir_id, queued);
                }
                paint_folder_icon(ui, is_sel || queued);
                clicked = ui
                    .selectable_label(is_sel, store.dir_name(row.dir_id))
                    .clicked();
                dir_count_pill(ui, row.match_count, queued || is_sel);
            });
        })
        .response;
    paint_tree_guides(ui, resp.rect, row.depth);
    clicked
}

fn update_merge_queue(merge_queue: &mut Vec<u32>, dir_id: u32, queued: bool) {
    if queued {
        if merge_queue.len() >= 2 {
            merge_queue.remove(0);
        }
        merge_queue.push(dir_id);
    } else {
        merge_queue.retain(|&id| id != dir_id);
    }
}

fn paint_folder_icon(ui: &mut Ui, accent: bool) {
    let (r, _) = ui.allocate_exact_size(vec2(14.0, 12.0), egui::Sense::hover());
    let col = if accent { PRIMARY } else { ON_SURFACE_VARIANT };
    let tab = Rect::from_min_size(r.min, vec2(6.0, 3.0));
    let body = Rect::from_min_size(r.min + vec2(0.0, 2.5), vec2(13.0, 8.5));
    ui.painter().rect_filled(tab, Rounding::same(1.5), col);
    ui.painter().rect_filled(body, Rounding::same(2.0), col);
}

fn paint_tree_guides(ui: &Ui, rect: Rect, depth: u16) {
    let stroke = Stroke::new(1.0_f32, OUTLINE_VARIANT);
    for lvl in 1..=depth.min(8) {
        let x = rect.min.x + f32::from(lvl) * 12.0 - 4.0;
        ui.painter()
            .line_segment([pos2(x, rect.min.y), pos2(x, rect.max.y)], stroke);
    }
}

fn dir_count_pill(ui: &mut Ui, match_count: u32, highlight: bool) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        let col = if highlight {
            PRIMARY
        } else {
            ON_SURFACE_VARIANT
        };
        let bg = if highlight {
            PRIMARY_FIXED
        } else {
            SURFACE_LOW
        };
        Frame::none()
            .fill(bg)
            .rounding(Rounding::same(8.0))
            .inner_margin(egui::Margin::symmetric(5.0, 0.0))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(format!("{match_count}"))
                        .monospace()
                        .color(col),
                );
            });
    });
}
