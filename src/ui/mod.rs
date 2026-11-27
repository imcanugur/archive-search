pub mod folder_modal;
pub mod grid_panel;
pub mod os_open;
pub mod status_bar;
pub mod theme;
pub mod top_bar;
pub mod tree_panel;

use egui::Color32;

use crate::index::MediaKind;

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
        MediaKind::Image => Color32::from_rgb(76, 166, 255),
        MediaKind::Video => Color32::from_rgb(245, 158, 66),
        MediaKind::Other => Color32::from_rgb(142, 152, 174),
    }
}
