use egui::{
    pos2, vec2, Align2, Color32, FontId, Painter, Rect, Rounding, Sense, Stroke, Ui, Vec2,
};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::index::MediaKind;
use crate::ui::card_paint::{paint_badges, paint_text_lines};
use crate::ui::os_open::handle_file_interaction;
use crate::ui::theme::{
    BG_CARD_HOVER, BORDER_STRONG, INVERSE_SURFACE, INVERSE_TEXT, OUTLINE_VARIANT, PRIMARY,
    SURFACE_HIGH, SURFACE_LOWEST,
};
use crate::ui::thumbs::ThumbCache;

pub const MIN_CARD_WIDTH: f32 = 216.0;

pub struct CardSpec {
    pub fid: u32,
    pub width: f32,
    pub is_visual_match: bool,
}

pub fn card_height(card_w: f32) -> f32 {
    ((card_w - 12.0) * 9.0 / 16.0).round() + 52.0
}

pub fn render_card(
    ui: &mut Ui,
    tr: &Strings,
    store: &ArchiveStore,
    spec: CardSpec,
    selected_file: &mut Option<u32>,
    thumbs: &mut ThumbCache,
) -> Option<u32> {
    let size_vec = Vec2::new(spec.width, card_height(spec.width));
    let (rect, resp) = ui.allocate_exact_size(size_vec, Sense::click());
    let is_sel = *selected_file == Some(spec.fid);
    if ui.is_rect_visible(rect) {
        paint_card(ui, store, &spec, rect, is_sel, resp.hovered(), thumbs);
    }
    handle_file_interaction(&resp, tr, store, spec.fid, selected_file)
}

fn paint_card(
    ui: &mut Ui,
    store: &ArchiveStore,
    spec: &CardSpec,
    rect: Rect,
    is_sel: bool,
    hovered: bool,
    thumbs: &mut ThumbCache,
) {
    let p = ui.painter();
    let (fill, stroke) = card_style(is_sel, hovered);
    p.rect(rect, Rounding::same(8.0), fill, stroke);

    let thumb_w = (rect.width() - 12.0).max(40.0);
    let thumb_h = (thumb_w * 9.0 / 16.0).round();
    let thumb_rect = Rect::from_min_size(rect.min + vec2(6.0, 6.0), vec2(thumb_w, thumb_h));
    let kind = store
        .file_kind
        .get(spec.fid as usize)
        .copied()
        .unwrap_or(MediaKind::Other);
    let ext_id = store.file_ext.get(spec.fid as usize).copied().unwrap_or(0);
    let ext = store.ext_name(ext_id).to_ascii_uppercase();

    paint_thumb_area(p, store, spec.fid, kind, &ext, thumb_rect, thumbs);
    paint_badges(p, store, spec.fid, kind, spec.is_visual_match, thumb_rect);
    let tr_min = pos2(rect.min.x + 8.0, thumb_rect.max.y + 5.0);
    let text_rect = Rect::from_min_max(tr_min, rect.max - vec2(8.0, 4.0));
    paint_text_lines(p, store, spec.fid, text_rect);
}

fn card_style(is_sel: bool, hovered: bool) -> (Color32, Stroke) {
    if is_sel {
        (SURFACE_HIGH, Stroke::new(2.0_f32, PRIMARY))
    } else if hovered {
        (BG_CARD_HOVER, Stroke::new(1.0_f32, BORDER_STRONG))
    } else {
        (SURFACE_LOWEST, Stroke::new(1.0_f32, OUTLINE_VARIANT))
    }
}

fn paint_thumb_area(
    p: &Painter,
    store: &ArchiveStore,
    fid: u32,
    kind: MediaKind,
    ext: &str,
    rect: Rect,
    thumbs: &mut ThumbCache,
) {
    p.rect_filled(rect, Rounding::same(6.0), INVERSE_SURFACE);
    let tex = matches!(kind, MediaKind::Image | MediaKind::Video)
        .then(|| thumbs.get_or_request(fid, store.file_full_path(fid), kind))
        .flatten();
    if let Some(t) = tex {
        let sz = t.size_vec2();
        let s = (rect.width() / sz.x.max(1.0)).min(rect.height() / sz.y.max(1.0));
        let img_rect = Rect::from_center_size(rect.center(), sz * s);
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        p.image(t.id(), img_rect, uv, Color32::WHITE);
    } else {
        let lbl = if ext.is_empty() { kind.badge() } else { ext };
        p.text(
            rect.center(),
            Align2::CENTER_CENTER,
            lbl,
            FontId::monospace(12.5),
            INVERSE_TEXT,
        );
    }
}
