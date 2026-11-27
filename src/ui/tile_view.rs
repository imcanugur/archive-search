use egui::{Frame, RichText, Rounding, Sense, Stroke, Ui, Vec2};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::index::MediaKind;
use crate::ui::badge_color;
use crate::ui::os_open::handle_file_interaction;
use crate::ui::theme::{ACCENT, BG_SELECTED, BG_SURFACE, BORDER_SUBTLE, TEXT_MUTED};
use crate::ui::thumbs::ThumbCache;

pub const CARD_WIDTH: f32 = 176.0;
pub const CARD_HEIGHT: f32 = 148.0;

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
    let fill = if is_sel { BG_SELECTED } else { BG_SURFACE };
    let stroke = Stroke::new(1.0_f32, if is_sel { ACCENT } else { BORDER_SUBTLE });
    let resp = Frame::none()
        .fill(fill)
        .stroke(stroke)
        .rounding(Rounding::same(4.0))
        .inner_margin(6.0)
        .show(ui, |ui| {
            ui.set_min_size(Vec2::new(CARD_WIDTH - 12.0, CARD_HEIGHT - 12.0));
            ui.set_max_size(Vec2::new(CARD_WIDTH - 12.0, CARD_HEIGHT - 12.0));
            render_tile_body(ui, store, fid, dir_id, kind, thumbs);
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
    thumbs: &mut ThumbCache,
) {
    ui.vertical(|ui| {
        let tex = matches!(kind, MediaKind::Image | MediaKind::Video)
            .then(|| thumbs.get_or_request(fid, store.file_full_path(fid), kind))
            .flatten();
        let box_sz = Vec2::new(CARD_WIDTH - 14.0, 92.0);
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
                            .strong()
                            .color(badge_color(kind)),
                    );
                }
            });
        });
        ui.add(egui::Label::new(store.file_name(fid)).truncate());
        ui.add(
            egui::Label::new(
                RichText::new(store.dir_name(dir_id))
                    .small()
                    .color(TEXT_MUTED),
            )
            .truncate(),
        );
    });
}
