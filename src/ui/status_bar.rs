use egui::{Color32, RichText, Ui};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::messages::SearchSnapshot;
use crate::telemetry::ResourceStats;
use crate::ui::format_bytes;
use crate::ui::theme::TEXT_MUTED;

pub struct StatusInfo<'a> {
    pub store: Option<&'a ArchiveStore>,
    pub snapshot: &'a SearchSnapshot,
    pub resources: ResourceStats,
    pub thumb_vram_bytes: u64,
    pub selected_file: Option<u32>,
    pub scanning: bool,
    pub scan_dirs: u32,
    pub scan_files: u32,
    pub last_scan_ms: Option<u64>,
    pub error_msg: Option<&'a str>,
}

pub fn show_status_bar(ui: &mut Ui, tr: &Strings, info: &StatusInfo<'_>) -> bool {
    let mut cancel_clicked = false;
    ui.horizontal(|ui| {
        if let Some(err) = info.error_msg {
            ui.label(
                RichText::new(format!("{}: {err}", tr.status_error_prefix))
                    .small()
                    .color(Color32::LIGHT_RED),
            );
            return;
        }
        if info.scanning {
            ui.spinner();
            ui.label(
                RichText::new(format!(
                    "{}: {} {}, {} {}...",
                    tr.status_scanning,
                    info.scan_dirs,
                    tr.status_folders,
                    info.scan_files,
                    tr.status_items
                ))
                .small(),
            );
            if ui.small_button(tr.status_cancel_btn).clicked() {
                cancel_clicked = true;
            }
            render_resources_right(ui, tr, info.resources, info.thumb_vram_bytes, None);
            return;
        }
        let Some(st) = info.store else {
            ui.label(RichText::new(tr.status_ready).small().color(TEXT_MUTED));
            render_resources_right(ui, tr, info.resources, info.thumb_vram_bytes, None);
            return;
        };

        let search_ms = info.snapshot.elapsed_us as f64 / 1000.0;
        let scan_label = info.last_scan_ms.map_or(String::new(), |ms| {
            format!("  |  {}: {ms} ms", tr.status_scan)
        });
        ui.label(
            RichText::new(format!(
                "{} / {} {}  |  {} / {} {}  |  {}: {:.2} ms{}",
                info.snapshot.matched_files.len(),
                st.file_count(),
                tr.status_items,
                info.snapshot.visible_dirs.len(),
                st.dir_count(),
                tr.status_folders,
                tr.status_search,
                search_ms,
                scan_label
            ))
            .small()
            .color(TEXT_MUTED),
        );

        render_resources_right(
            ui,
            tr,
            info.resources,
            info.thumb_vram_bytes,
            Some(st.memory_bytes() as u64),
        );
    });
    let _ = info.selected_file;
    cancel_clicked
}

fn render_resources_right(
    ui: &mut Ui,
    tr: &Strings,
    res: ResourceStats,
    thumb_vram_bytes: u64,
    idx_bytes: Option<u64>,
) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        let mut parts = Vec::new();
        parts.push(format!("App CPU: {:.1}%", res.cpu_percent));
        let ram = if res.rss_bytes > 0 {
            match idx_bytes {
                Some(idx) => format!(
                    "App {}: {} (Idx {})",
                    tr.status_index_ram,
                    format_bytes(res.rss_bytes),
                    format_bytes(idx)
                ),
                None => format!(
                    "App {}: {}",
                    tr.status_index_ram,
                    format_bytes(res.rss_bytes)
                ),
            }
        } else {
            idx_bytes.map_or_else(String::new, |idx| format!("Idx: {}", format_bytes(idx)))
        };
        if !ram.is_empty() {
            parts.push(ram);
        }
        let app_vram = thumb_vram_bytes.saturating_add(res.app_cuda_vram_bytes);
        parts.push(format!("App VRAM: {}", format_bytes(app_vram)));
        let color = if res.cpu_percent > 60.0 {
            Color32::from_rgb(245, 175, 75)
        } else {
            Color32::from_rgb(130, 195, 155)
        };
        ui.label(RichText::new(parts.join("  |  ")).small().color(color));
    });
}
