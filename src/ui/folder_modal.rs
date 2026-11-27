use std::path::{Path, PathBuf};

use egui::{Button, Context, RichText, TextEdit, Ui, Window};

use crate::i18n::Strings;
use crate::ui::folder_fs::{
    default_start_dir, detect_quick_places, read_subfolders, render_dir_list,
    render_quick_places,
};
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
        let start = default_start_dir();
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
        self.render_path_bar(ui, tr);
        if let Some(target) = render_quick_places(ui, tr, &self.quick_places) {
            self.navigate(target, tr);
        }
        if let Some(ref err) = self.error {
            ui.label(RichText::new(err).color(egui::Color32::LIGHT_RED).small());
        }
        ui.separator();
        if let Some(dir) = render_dir_list(ui, tr, &self.entries) {
            self.navigate(dir, tr);
        }
        ui.separator();
        self.render_footer(ui, tr)
    }

    fn render_path_bar(&mut self, ui: &mut Ui, tr: &Strings) {
        ui.horizontal(|ui| {
            if let Some(parent) = ui
                .button(tr.modal_up_btn)
                .clicked()
                .then(|| self.current.parent().map(Path::to_path_buf))
                .flatten()
            {
                self.navigate(parent, tr);
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
    }

    fn render_footer(&mut self, ui: &mut Ui, tr: &Strings) -> Option<PathBuf> {
        let mut chosen = None;
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

    fn navigate(&mut self, path: PathBuf, tr: &Strings) {
        match read_subfolders(&path) {
            Ok(dirs) => {
                self.current = path.clone();
                self.path_input = path.to_string_lossy().into_owned();
                self.entries = dirs;
                self.error = None;
            }
            Err(err) => self.error = Some(format!("{}: {err}", tr.modal_open_err)),
        }
    }
}
