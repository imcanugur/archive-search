use std::fs;
use std::path::{Path, PathBuf};

use egui::{RichText, ScrollArea, Ui};

use crate::i18n::Strings;
use crate::ui::theme::TEXT_MUTED;

pub fn default_start_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from("/"))
}

pub fn read_subfolders(path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut dirs: Vec<PathBuf> = fs::read_dir(path)?
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .map(|e| e.path())
        .collect();
    dirs.sort();
    Ok(dirs)
}

pub fn detect_quick_places() -> Vec<(String, PathBuf)> {
    let mut places = Vec::new();
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        places.push(("Home (~)".to_owned(), home));
    }
    places.push(("/".to_owned(), PathBuf::from("/")));
    if let Ok(mounts) = read_subfolders(Path::new("/mnt")) {
        for m in mounts {
            if let Some(name) = m
                .file_name()
                .and_then(|n| n.to_str())
                .filter(|n| n.len() == 1)
            {
                places.push((format!("{}: (/mnt/{name})", name.to_uppercase()), m));
            }
        }
    }
    places
}

pub fn render_quick_places(
    ui: &mut Ui,
    tr: &Strings,
    places: &[(String, PathBuf)],
) -> Option<PathBuf> {
    let mut jump = None;
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new(tr.modal_quick_access)
                .small()
                .color(TEXT_MUTED),
        );
        for (label, path) in places {
            if ui.small_button(label).clicked() {
                jump = Some(path.clone());
            }
        }
    });
    jump
}

pub fn render_dir_list(ui: &mut Ui, tr: &Strings, entries: &[PathBuf]) -> Option<PathBuf> {
    let mut next = None;
    ScrollArea::vertical().max_height(270.0).show(ui, |ui| {
        if entries.is_empty() {
            ui.label(RichText::new(tr.modal_no_subfolders).color(TEXT_MUTED));
        }
        for dir in entries {
            let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("...");
            if ui.selectable_label(false, name).clicked() {
                next = Some(dir.clone());
            }
        }
    });
    next
}
