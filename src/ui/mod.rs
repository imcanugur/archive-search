mod card_paint;
mod cards;
pub mod command_palette;
mod dir_row;
pub mod dual_pane;
pub mod filter_bar;
mod folder_fs;
pub mod folder_modal;
pub mod grid_panel;
pub mod inspector;
mod inspector_loader;
mod inspector_preview;
mod inspector_view;
mod list_view;
pub mod merge_drawer;
pub mod merge_modal;
mod merge_view;
mod os_open;
mod palette_entries;
mod preview_loader;
pub mod preview_modal;
mod smart_views;
pub mod status_bar;
pub mod theme;
mod thumb_queue;
pub mod thumbs;
pub mod top_bar;
mod top_bar_widgets;
pub mod tree_panel;
pub mod video_thumb;

use egui::Color32;

use crate::index::MediaKind;
use crate::ui::theme::{ON_SURFACE_VARIANT, PRIMARY, TERTIARY};

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

pub fn badge_color(kind: MediaKind) -> Color32 {
    match kind {
        MediaKind::Image => PRIMARY,
        MediaKind::Video => TERTIARY,
        MediaKind::Other => ON_SURFACE_VARIANT,
    }
}

pub fn format_commas(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}
