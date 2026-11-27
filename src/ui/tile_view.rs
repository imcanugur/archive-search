use egui::{Frame, RichText, Rounding, Sense, Stroke, Ui, Vec2};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::index::MediaKind;
use crate::ui::os_open::handle_file_interaction;
use crate::ui::theme::{
    INVERSE_SURFACE, INVERSE_TEXT, ON_SURFACE, ON_SURFACE_VARIANT, OUTLINE_VARIANT, PRIMARY,
    SURFACE_LOWEST,
};
use crate::ui::thumbs::ThumbCache;
use crate::ui::{badge_color, format_bytes};

pub const CARD_WIDTH: f32 = 192.0;
pub const CARD_HEIGHT: f32 = 176.0;

pub fn render_tile(
    ui: &mut Ui,
    tr: &Strings,
    store: &ArchiveStore,
    fid: u32,
    selected_file: &mut Option<u32>,
    thumbs: &mut ThumbCache,
) -> Option<u32> {
    let is_sel = *selected_file == Some(fid);
    let kind = store
        .file_kind
        .get(fid as usize)
        .copied()
        .unwrap_or(MediaKind::Other);
    let dir_id = store.file_dir.get(fid as usize).copied().unwrap_or(0);
    let size = store.file_size.get(fid as usize).copied().unwrap_or(0);
    let stroke = if is_sel {
        Stroke::new(2.0_f32, PRIMARY)
    } else {
        Stroke::new(1.0_f32, OUTLINE_VARIANT)
    };
    let resp = Frame::none()
        .fill(SURFACE_LOWEST)
        .stroke(stroke)
        .rounding(Rounding::same(8.0))
        .inner_margin(6.0)
        .show(ui, |ui| {
            ui.set_min_size(Vec2::new(CARD_WIDTH - 12.0, CARD_HEIGHT - 12.0));
            ui.set_max_size(Vec2::new(CARD_WIDTH - 12.0, CARD_HEIGHT - 12.0));
            render_tile_body(ui, store, fid, dir_id, kind, size, thumbs);
        })
        .response
        .interact(Sense::click());
    handle_file_interaction(&resp, tr, store, fid, selected_file)
}

fn render_tile_body(
    ui: &mut Ui,
    store: &ArchiveStore,
    fid: u32,
    dir_id: u32,
    kind: MediaKind,
    size: u64,
    thumbs: &mut ThumbCache,
) {
    ui.vertical(|ui| {
        let tex = matches!(kind, MediaKind::Image | MediaKind::Video)
            .then(|| thumbs.get_or_request(fid, store.file_full_path(fid), kind))
            .flatten();
        let box_sz = Vec2::new(CARD_WIDTH - 14.0, 112.0);
        Frame::none()
            .fill(INVERSE_SURFACE)
            .rounding(Rounding::same(6.0))
            .show(ui, |ui| {
                ui.allocate_ui(box_sz, |ui| {
                    ui.centered_and_justified(|ui| match tex {
                        Some(t) => {
                            let sz = t.size_vec2();
                            let s = (box_sz.x / sz.x.max(1.0)).min(box_sz.y / sz.y.max(1.0));
                            ui.image((t.id(), sz * s));
                        }
                        None => {
                            ui.label(
                                RichText::new(kind.badge())
                                    .monospace()
                                    .strong()
                                    .color(INVERSE_TEXT),
                            );
                        }
                    });
                });
            });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add(
                egui::Label::new(
                    RichText::new(store.file_name(fid))
                        .strong()
                        .color(ON_SURFACE),
                )
                .truncate(),
            );
        });
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(kind.badge())
                    .monospace()
                    .color(badge_color(kind)),
            );
            ui.add(
                egui::Label::new(
                    RichText::new(store.dir_name(dir_id))
                        .small()
                        .color(ON_SURFACE_VARIANT),
                )
                .truncate(),
            );
            if size > 0 {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format_bytes(size)).monospace().color(PRIMARY));
                });
            }
        });
    });
}
