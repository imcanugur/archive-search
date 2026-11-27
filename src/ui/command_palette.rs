use egui::{Context, Key, RichText, ScrollArea, TextEdit, Window};

use crate::i18n::Strings;
pub use crate::ui::palette_entries::PaletteCmd;
use crate::ui::palette_entries::{PaletteEntry, ENTRIES};
use crate::ui::theme::TEXT_MUTED;

#[derive(Default)]
pub struct CommandPalette {
    open: bool,
    filter: String,
    selected: usize,
    just_opened: bool,
}

impl CommandPalette {
    pub fn toggle(&mut self) {
        self.open = !self.open;
        if self.open {
            self.filter.clear();
            self.selected = 0;
            self.just_opened = true;
        }
    }

    pub fn show(&mut self, ctx: &Context, tr: &Strings) -> Option<PaletteCmd> {
        if !self.open {
            return None;
        }
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.open = false;
            return None;
        }
        let mut chosen = None;
        let mut keep = self.open;
        Window::new(tr.palette_title)
            .open(&mut keep)
            .collapsible(false)
            .resizable(false)
            .fixed_size([460.0, 320.0])
            .show(ctx, |ui| chosen = self.render_body(ui, ctx, tr));
        self.open = keep && chosen.is_none();
        chosen
    }

    fn render_body(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &Context,
        tr: &Strings,
    ) -> Option<PaletteCmd> {
        let edit = ui.add_sized(
            [ui.available_width(), 28.0],
            TextEdit::singleline(&mut self.filter).hint_text(tr.palette_hint),
        );
        if self.just_opened {
            edit.request_focus();
            self.just_opened = false;
        }
        ui.separator();

        let needle = self.filter.to_ascii_lowercase();
        let filtered: Vec<&PaletteEntry> = ENTRIES
            .iter()
            .filter(|e| needle.is_empty() || e.label.to_ascii_lowercase().contains(&needle))
            .collect();

        if let Some(cmd) = self.handle_list_keys(ctx, &filtered) {
            return Some(cmd);
        }
        render_filtered_list(ui, &filtered, self.selected)
    }

    fn handle_list_keys(
        &mut self,
        ctx: &Context,
        filtered: &[&PaletteEntry],
    ) -> Option<PaletteCmd> {
        if self.selected >= filtered.len() {
            self.selected = 0;
        }
        if ctx.input(|i| i.key_pressed(Key::ArrowDown)) && !filtered.is_empty() {
            self.selected = (self.selected + 1) % filtered.len();
        }
        if ctx.input(|i| i.key_pressed(Key::ArrowUp)) && !filtered.is_empty() {
            self.selected = (self.selected + filtered.len() - 1) % filtered.len();
        }
        if ctx.input(|i| i.key_pressed(Key::Enter)) {
            return filtered.get(self.selected).map(|e| e.cmd);
        }
        None
    }
}

fn render_filtered_list(
    ui: &mut egui::Ui,
    filtered: &[&PaletteEntry],
    selected: usize,
) -> Option<PaletteCmd> {
    let mut clicked = None;
    ScrollArea::vertical().show(ui, |ui| {
        for (idx, entry) in filtered.iter().enumerate() {
            ui.horizontal(|ui| {
                if ui.selectable_label(idx == selected, entry.label).clicked() {
                    clicked = Some(entry.cmd);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(entry.hint).small().color(TEXT_MUTED));
                });
            });
        }
    });
    clicked
}
