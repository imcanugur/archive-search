use std::fs;
use std::path::{Path, PathBuf};

use egui::{Button, Context, RichText, ScrollArea, TextEdit, Ui, Window};

use crate::i18n::Strings;
use crate::ui::theme::{ACCENT, TEXT_MUTED, TEXT_PRIMARY};

pub struct FolderModal {
    open: bool,
    current: PathBuf,
    path_input: String,
    entries: Vec<PathBuf>,
    quick_places: Vec<(String, PathBuf)>,
    error: Option<String>,
}

impl Default for FolderModal {
    fn default() -> Self {
        let start = std::env::var_os("HOME")
            .map(PathBuf::from)
            .filter(|p| p.exists())
            .unwrap_or_else(|| PathBuf::from("/"));
        Self {
            open: false,
            path_input: start.to_string_lossy().into_owned(),
            current: start,
            entries: Vec::new(),
            quick_places: detect_quick_places(),
            error: None,
        }
    }
}

impl FolderModal {
    pub fn open_at(&mut self, initial: Option<&Path>, tr: &Strings) {
        self.open = true;
        let target = initial
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.current.clone());
        self.navigate(target, tr);
    }

    pub fn show(&mut self, ctx: &Context, tr: &Strings) -> Option<PathBuf> {
        if !self.open {
            return None;
        }
        let mut chosen = None;
        let mut keep_open = self.open;
        Window::new(tr.modal_title)
            .open(&mut keep_open)
            .collapsible(false)
            .resizable(true)
            .default_size([640.0, 440.0])
            .show(ctx, |ui| chosen = self.render_body(ui, tr));
        self.open = keep_open && chosen.is_none();
        chosen
    }

    fn render_body(&mut self, ui: &mut Ui, tr: &Strings) -> Option<PathBuf> {
        let mut chosen = None;
        ui.horizontal(|ui| {
            if ui.button(tr.modal_up_btn).clicked() {
                if let Some(parent) = self.current.parent() {
                    self.navigate(parent.to_path_buf(), tr);
                }
            }
            let edit = ui.add_sized(
                [ui.available_width() - 56.0, 26.0],
                TextEdit::singleline(&mut self.path_input),
            );
            if ui.button(tr.modal_go_btn).clicked()
                || (edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            {
                self.navigate(PathBuf::from(self.path_input.trim()), tr);
            }
        });
        self.render_quick_places(ui, tr);
        if let Some(ref err) = self.error {
            ui.label(RichText::new(err).color(egui::Color32::LIGHT_RED).small());
        }
        ui.separator();
        self.render_dir_list(ui, tr);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(self.current.to_string_lossy())
                    .small()
                    .color(TEXT_MUTED),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let ok = Button::new(
                    RichText::new(tr.modal_scan_this_btn)
                        .strong()
                        .color(TEXT_PRIMARY),
                )
                .fill(ACCENT);
                if ui.add(ok).clicked() {
                    chosen = Some(self.current.clone());
                }
                if ui.button(tr.modal_cancel_btn).clicked() {
                    self.open = false;
                }
            });
        });
        chosen
    }

    fn render_quick_places(&mut self, ui: &mut Ui, tr: &Strings) {
        let mut jump = None;
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(tr.modal_quick_access)
                    .small()
                    .color(TEXT_MUTED),
            );
            for (label, path) in &self.quick_places {
                if ui.small_button(label).clicked() {
                    jump = Some(path.clone());
                }
            }
        });
        if let Some(target) = jump {
            self.navigate(target, tr);
        }
    }

    fn render_dir_list(&mut self, ui: &mut Ui, tr: &Strings) {
        let mut next = None;
        ScrollArea::vertical().max_height(270.0).show(ui, |ui| {
            if self.entries.is_empty() {
                ui.label(RichText::new(tr.modal_no_subfolders).color(TEXT_MUTED));
            }
            for dir in &self.entries {
                let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("...");
                if ui.selectable_label(false, name).clicked() {
                    next = Some(dir.clone());
                }
            }
        });
        if let Some(dir) = next {
            self.navigate(dir, tr);
        }
    }

    fn navigate(&mut self, path: PathBuf, tr: &Strings) {
        match fs::read_dir(&path) {
            Ok(rd) => {
                let mut dirs: Vec<PathBuf> = rd
                    .flatten()
                    .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
                    .map(|e| e.path())
                    .collect();
                dirs.sort();
                self.current = path.clone();
                self.path_input = path.to_string_lossy().into_owned();
                self.entries = dirs;
                self.error = None;
            }
            Err(err) => self.error = Some(format!("{}: {err}", tr.modal_open_err)),
        }
    }
}

fn detect_quick_places() -> Vec<(String, PathBuf)> {
    let mut places = Vec::new();
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        places.push(("Home (~)".to_owned(), home));
    }
    places.push(("/".to_owned(), PathBuf::from("/")));
    if let Ok(rd) = fs::read_dir("/mnt") {
        let mut mounts: Vec<PathBuf> = rd
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
            .map(|e| e.path())
            .collect();
        mounts.sort();
        for m in mounts {
            if let Some(name) = m.file_name().and_then(|n| n.to_str()) {
                if name.len() == 1 {
                    places.push((format!("{}: (/mnt/{name})", name.to_uppercase()), m));
                }
            }
        }
    }
    places
}
