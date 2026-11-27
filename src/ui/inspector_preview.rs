use egui::{Button, Color32, Frame, RichText, Rounding, Stroke, Ui, Vec2};

use crate::index::store::ArchiveStore;
use crate::index::MediaKind;
use crate::ui::inspector::InspectorAction;
use crate::ui::theme::{
    INVERSE_SURFACE, ON_SURFACE_VARIANT, OUTLINE_VARIANT, PRIMARY, SURFACE_LOW,
};
use crate::ui::thumbs::ThumbCache;

pub fn render_preview_box(
    ui: &mut Ui,
    store: &ArchiveStore,
    fid: u32,
    thumbs: &mut ThumbCache,
    act: &mut InspectorAction,
) {
    let kind = store
        .file_kind
        .get(fid as usize)
        .copied()
        .unwrap_or(MediaKind::Other);
    let w = ui.available_width();
    let box_sz = Vec2::new(w, (w * 9.0 / 16.0).clamp(130.0, 190.0));
    render_preview_frame(ui, store, fid, kind, box_sz, thumbs);
    ui.add_space(6.0);
    let lbl = RichText::new("Find Visually Similar (GPU)")
        .strong()
        .color(Color32::WHITE);
    if ui
        .add_sized([w, 30.0], Button::new(lbl).fill(PRIMARY))
        .clicked()
    {
        act.find_similar = Some(fid);
    }
}

fn render_preview_frame(
    ui: &mut Ui,
    store: &ArchiveStore,
    fid: u32,
    kind: MediaKind,
    box_sz: Vec2,
    thumbs: &mut ThumbCache,
) {
    let tex = matches!(kind, MediaKind::Image | MediaKind::Video)
        .then(|| thumbs.get_or_request(fid, store.file_full_path(fid), kind))
        .flatten();
    if let Some(t) = tex {
        Frame::none()
            .fill(INVERSE_SURFACE)
            .rounding(Rounding::same(8.0))
            .show(ui, |ui| {
                ui.allocate_ui(box_sz, |ui| {
                    let sz = t.size_vec2();
                    let s = (box_sz.x / sz.x.max(1.0)).min(box_sz.y / sz.y.max(1.0));
                    ui.centered_and_justified(|ui| ui.image((t.id(), sz * s)));
                });
            });
        return;
    }
    render_placeholder(ui, store, fid, kind, box_sz);
}

fn render_placeholder(
    ui: &mut Ui,
    store: &ArchiveStore,
    fid: u32,
    kind: MediaKind,
    box_sz: Vec2,
) {
    let ext_id = store.file_ext.get(fid as usize).copied().unwrap_or(0);
    let ext = store.ext_name(ext_id).to_ascii_uppercase();
    let lbl = if ext.is_empty() { kind.badge() } else { &ext };
    Frame::none()
        .fill(SURFACE_LOW)
        .stroke(Stroke::new(1.0_f32, OUTLINE_VARIANT))
        .rounding(Rounding::same(8.0))
        .show(ui, |ui| {
            ui.allocate_ui(box_sz, |ui| {
                ui.centered_and_justified(|ui| {
                    if matches!(kind, MediaKind::Image | MediaKind::Video) {
                        ui.spinner();
                    } else {
                        let text = RichText::new(lbl)
                            .heading()
                            .monospace()
                            .color(ON_SURFACE_VARIANT);
                        ui.label(text);
                    }
                });
            });
        });
}
