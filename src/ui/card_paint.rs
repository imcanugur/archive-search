use egui::{pos2, vec2, Align2, Color32, FontId, Painter, Pos2, Rect, Rounding};

use crate::index::store::ArchiveStore;
use crate::index::MediaKind;
use crate::ui::format_bytes;
use crate::ui::theme::{
    ON_SURFACE, ON_SURFACE_VARIANT, PRIMARY, SECONDARY, TERTIARY, TERTIARY_BG,
};

pub fn paint_badges(
    p: &Painter,
    store: &ArchiveStore,
    fid: u32,
    kind: MediaKind,
    is_visual_match: bool,
    rect: Rect,
) {
    let ext_id = store.file_ext.get(fid as usize).copied().unwrap_or(0);
    if is_visual_match {
        pill(
            p,
            rect.min + vec2(6.0, 6.0),
            false,
            "MATCH",
            PRIMARY,
            Color32::WHITE,
        );
    } else if store.is_raw_ext_id(ext_id) {
        pill(
            p,
            rect.min + vec2(6.0, 6.0),
            false,
            "RAW",
            TERTIARY_BG,
            TERTIARY,
        );
    } else if kind == MediaKind::Video {
        pill(
            p,
            rect.min + vec2(6.0, 6.0),
            false,
            "VID",
            SECONDARY,
            Color32::WHITE,
        );
    }
    let ext = store.ext_name(ext_id).to_ascii_uppercase();
    let size = store.file_size.get(fid as usize).copied().unwrap_or(0);
    let tag = if !ext.is_empty() {
        ext
    } else if size > 0 {
        format_bytes(size)
    } else {
        String::new()
    };
    if !tag.is_empty() {
        let bg = Color32::from_rgba_unmultiplied(15, 23, 42, 205);
        pill(
            p,
            pos2(rect.max.x - 6.0, rect.max.y - 6.0),
            true,
            &tag,
            bg,
            Color32::WHITE,
        );
    }
}

fn pill(p: &Painter, anchor: Pos2, right_bot: bool, text: &str, bg: Color32, fg: Color32) {
    let galley = p.layout_no_wrap(text.to_owned(), FontId::monospace(10.5), fg);
    let sz = galley.size() + vec2(10.0, 4.0);
    let min = if right_bot { anchor - sz } else { anchor };
    let r = Rect::from_min_size(min, sz);
    p.rect_filled(r, Rounding::same(4.0), bg);
    p.galley(r.min + vec2(5.0, 2.0), galley, fg);
}

pub fn paint_text_lines(p: &Painter, store: &ArchiveStore, fid: u32, rect: Rect) {
    let cp = p.with_clip_rect(rect);
    let max_chars = ((rect.width() / 7.0) as usize).max(12);
    let name = truncate_chars(store.file_name(fid), max_chars);
    cp.text(
        rect.min,
        Align2::LEFT_TOP,
        name,
        FontId::proportional(13.0),
        ON_SURFACE,
    );

    let dir_id = store.file_dir.get(fid as usize).copied().unwrap_or(0);
    let size = store.file_size.get(fid as usize).copied().unwrap_or(0);
    let sub_raw = if size > 0 {
        format!("{} • {}", format_bytes(size), store.dir_name(dir_id))
    } else {
        store.dir_name(dir_id).to_owned()
    };
    let sub = truncate_chars(&sub_raw, max_chars + 4);
    cp.text(
        rect.min + vec2(0.0, 18.0),
        Align2::LEFT_TOP,
        sub,
        FontId::proportional(11.5),
        ON_SURFACE_VARIANT,
    );
}

fn truncate_chars(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_owned();
    }
    let mut out: String = s.chars().take(max_chars.saturating_sub(1)).collect();
    out.push('…');
    out
}
