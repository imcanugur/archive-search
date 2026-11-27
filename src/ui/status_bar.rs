use egui::{Color32, RichText, Ui};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::messages::SearchSnapshot;
use crate::ui::format_bytes;
use crate::ui::theme::TEXT_MUTED;

pub struct StatusInfo<'a> {
    pub store: Option<&'a ArchiveStore>,
    pub snapshot: &'a SearchSnapshot,
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
            return;
        }
        let Some(st) = info.store else {
            ui.label(RichText::new(tr.status_ready).small().color(TEXT_MUTED));
            return;
        };

        let search_ms = info.snapshot.elapsed_us as f64 / 1000.0;
        let scan_label = info.last_scan_ms.map_or(String::new(), |ms| {
            format!("  |  {}: {ms} ms", tr.status_scan)
        });
        ui.label(
            RichText::new(format!(
                "{} / {} {}  |  {} / {} {}  |  {}: {:.2} ms  |  {}: {}{}",
                info.snapshot.matched_files.len(),
                st.file_count(),
                tr.status_items,
                info.snapshot.visible_dirs.len(),
                st.dir_count(),
                tr.status_folders,
                tr.status_search,
                search_ms,
                tr.status_index_ram,
                format_bytes(st.memory_bytes() as u64),
                scan_label
            ))
            .small()
            .color(TEXT_MUTED),
        );

        if let Some(fid) = info.selected_file {
            ui.separator();
            let path_text = st.file_full_path(fid).to_string_lossy().into_owned();
            ui.add(
                egui::Label::new(RichText::new(path_text).small().color(TEXT_MUTED))
                    .truncate(),
            );
        }
    });
    cancel_clicked
}
