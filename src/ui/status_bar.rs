use egui::{Color32, RichText, Ui};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::messages::SearchSnapshot;
use crate::telemetry::ResourceStats;
use crate::ui::format_bytes;
use crate::ui::theme::{ON_SURFACE_VARIANT, PRIMARY, SECONDARY, TERTIARY};

pub struct StatusInfo<'a> {
    pub store: Option<&'a ArchiveStore>,
    pub snapshot: &'a SearchSnapshot,
    pub resources: ResourceStats,
    pub thumb_vram_bytes: u64,
    pub visual_status: Option<String>,
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
            let msg = format!("{}: {err}", tr.status_error_prefix);
            ui.label(
                RichText::new(msg)
                    .small()
                    .color(Color32::from_rgb(186, 26, 26)),
            );
            return;
        }
        if info.scanning {
            cancel_clicked = render_scanning_left(ui, tr, info);
            render_resources_right(ui, tr, info.resources, info.thumb_vram_bytes, None);
            return;
        }
        let Some(st) = info.store else {
            ui.label(
                RichText::new(tr.status_ready)
                    .small()
                    .color(ON_SURFACE_VARIANT),
            );
            render_resources_right(ui, tr, info.resources, info.thumb_vram_bytes, None);
            return;
        };
        render_ready_left(ui, tr, st, info);
        let idx = Some(st.memory_bytes() as u64);
        render_resources_right(ui, tr, info.resources, info.thumb_vram_bytes, idx);
    });
    cancel_clicked
}

fn render_scanning_left(ui: &mut Ui, tr: &Strings, info: &StatusInfo<'_>) -> bool {
    ui.spinner();
    let msg = format!(
        "{}: {} {}, {} {}...",
        tr.status_scanning,
        info.scan_dirs,
        tr.status_folders,
        info.scan_files,
        tr.status_items
    );
    ui.label(RichText::new(msg).small().color(PRIMARY));
    ui.small_button(tr.status_cancel_btn).clicked()
}

fn render_ready_left(ui: &mut Ui, tr: &Strings, st: &ArchiveStore, info: &StatusInfo<'_>) {
    let search_ms = info.snapshot.elapsed_us as f64 / 1000.0;
    let scan_lbl = info.last_scan_ms.map_or(String::new(), |ms| {
        format!("  •  {}: {ms} ms", tr.status_scan)
    });
    let vis_lbl = info
        .visual_status
        .as_deref()
        .map_or(String::new(), |v| format!("  •  {v}"));
    ui.label(RichText::new("●").small().color(SECONDARY));
    let text = format!(
        "{} / {} {}  •  {} / {} {}  •  {}: {:.2} ms{}{}",
        info.snapshot.matched_files.len(),
        st.file_count(),
        tr.status_items,
        info.snapshot.visible_dirs.len(),
        st.dir_count(),
        tr.status_folders,
        tr.status_search,
        search_ms,
        scan_lbl,
        vis_lbl
    );
    ui.label(RichText::new(text).small().color(ON_SURFACE_VARIANT));
}

fn render_resources_right(
    ui: &mut Ui,
    tr: &Strings,
    res: ResourceStats,
    thumb_vram_bytes: u64,
    idx_bytes: Option<u64>,
) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        let app_vram = thumb_vram_bytes.saturating_add(res.app_cuda_vram_bytes);
        ui.label(
            RichText::new(format!("App VRAM: {}", format_bytes(app_vram)))
                .monospace()
                .color(PRIMARY),
        );
        ui.label(RichText::new("•").small().color(ON_SURFACE_VARIANT));
        let ram = match (res.rss_bytes > 0, idx_bytes) {
            (true, Some(idx)) => format!(
                "App {}: {} (Idx {})",
                tr.status_index_ram,
                format_bytes(res.rss_bytes),
                format_bytes(idx)
            ),
            (true, None) => format!(
                "App {}: {}",
                tr.status_index_ram,
                format_bytes(res.rss_bytes)
            ),
            (false, Some(idx)) => format!("Idx: {}", format_bytes(idx)),
            (false, None) => String::new(),
        };
        if !ram.is_empty() {
            ui.label(RichText::new(ram).monospace().color(ON_SURFACE_VARIANT));
            ui.label(RichText::new("•").small().color(ON_SURFACE_VARIANT));
        }
        let cpu_col = if res.cpu_percent > 60.0 {
            TERTIARY
        } else {
            SECONDARY
        };
        ui.label(
            RichText::new(format!("App CPU: {:.1}%", res.cpu_percent))
                .monospace()
                .color(cpu_col),
        );
    });
}
