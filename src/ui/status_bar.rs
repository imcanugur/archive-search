use egui::{Color32, RichText, Ui};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::messages::SearchSnapshot;
use crate::telemetry::ResourceStats;
use crate::ui::theme::{ON_SURFACE_VARIANT, PRIMARY, SECONDARY};
use crate::ui::{format_bytes, format_commas};

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
            render_telemetry_right(ui, tr, info);
            return;
        }
        if let Some(st) = info.store {
            render_ready_left(ui, st, info);
        } else {
            ui.label(RichText::new("●").small().color(ON_SURFACE_VARIANT));
            ui.label(
                RichText::new(tr.status_ready)
                    .small()
                    .color(ON_SURFACE_VARIANT),
            );
        }
        render_telemetry_right(ui, tr, info);
    });
    cancel_clicked
}

fn render_scanning_left(ui: &mut Ui, tr: &Strings, info: &StatusInfo<'_>) -> bool {
    ui.spinner();
    let msg = format!(
        "{}: {} {}, {} {}...",
        tr.status_scanning,
        format_commas(info.scan_dirs as usize),
        tr.status_folders,
        format_commas(info.scan_files as usize),
        tr.status_items
    );
    ui.label(RichText::new(msg).small().color(PRIMARY));
    ui.small_button(tr.status_cancel_btn).clicked()
}

fn render_ready_left(ui: &mut Ui, st: &ArchiveStore, info: &StatusInfo<'_>) {
    ui.label(RichText::new("●").small().color(SECONDARY));
    let matched = info.snapshot.matched_files.len();
    let total = st.file_count();
    let vdirs = info.snapshot.visible_dirs.len();
    let tdirs = st.dir_count();
    let summary = if matched == total && vdirs == tdirs {
        format!(
            "{} files • {} dirs indexed",
            format_commas(total),
            format_commas(tdirs)
        )
    } else {
        format!(
            "{} / {} files • {} / {} dirs",
            format_commas(matched),
            format_commas(total),
            format_commas(vdirs),
            format_commas(tdirs)
        )
    };
    let vis = info
        .visual_status
        .as_deref()
        .map_or(String::new(), |v| format!(" • {v}"));
    ui.label(
        RichText::new(format!("{summary}{vis}"))
            .small()
            .color(ON_SURFACE_VARIANT),
    );
}

fn render_telemetry_right(ui: &mut Ui, tr: &Strings, info: &StatusInfo<'_>) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        let app_vram = info
            .thumb_vram_bytes
            .saturating_add(info.resources.app_cuda_vram_bytes);
        let tot = info.resources.gpu_total_vram_bytes;
        let vram_lbl = if tot > 0 {
            let gb = 1024.0 * 1024.0 * 1024.0;
            format!(
                "RTX 3070 VRAM: {:.2} / {:.1} GB",
                app_vram as f64 / gb,
                tot as f64 / gb
            )
        } else {
            format!("VRAM: {}", format_bytes(app_vram))
        };
        ui.label(RichText::new(vram_lbl).monospace().color(PRIMARY));
        sep(ui);
        let ram = if info.resources.rss_bytes > 0 {
            format!(
                "{}: {}",
                tr.status_index_ram,
                format_bytes(info.resources.rss_bytes)
            )
        } else {
            let idx = info.store.map_or(0, ArchiveStore::memory_bytes) as u64;
            format!("Idx: {}", format_bytes(idx))
        };
        ui.label(RichText::new(ram).monospace().color(ON_SURFACE_VARIANT));
        if let Some(ms) = info.last_scan_ms {
            sep(ui);
            let s = format!("{}: {ms} ms", tr.status_scan);
            ui.label(RichText::new(s).monospace().color(ON_SURFACE_VARIANT));
        }
        sep(ui);
        let ms = info.snapshot.elapsed_us as f64 / 1000.0;
        ui.label(
            RichText::new(format!("{}: {ms:.2} ms", tr.status_search))
                .monospace()
                .color(SECONDARY),
        );
    });
}

fn sep(ui: &mut Ui) {
    ui.label(RichText::new("•").small().color(ON_SURFACE_VARIANT));
}
