use egui::{Context, Key, RichText, ScrollArea, TextEdit, Window};

use crate::i18n::Strings;
use crate::ui::theme::TEXT_MUTED;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteCmd {
    OpenFolder,
    RescanFolder,
    OpenMerge,
    UndoMerge,
    IndexVisual,
    FindSimilar,
    FindDuplicates,
    ViewList,
    ViewGrid,
    SmartImages,
    SmartVideos,
    SmartLarge,
    ClearFilters,
}

struct PaletteEntry {
    label: &'static str,
    hint: &'static str,
    cmd: PaletteCmd,
}

const ENTRIES: &[PaletteEntry] = &[
    PaletteEntry {
        label: "Open Folder...",
        hint: "Browse",
        cmd: PaletteCmd::OpenFolder,
    },
    PaletteEntry {
        label: "Rescan Folder (Refresh Cache)",
        hint: "Disk",
        cmd: PaletteCmd::RescanFolder,
    },
    PaletteEntry {
        label: "Merge Folders (Dry-Run)...",
        hint: "Ops",
        cmd: PaletteCmd::OpenMerge,
    },
    PaletteEntry {
        label: "Undo Last Merge",
        hint: "Ctrl+Z",
        cmd: PaletteCmd::UndoMerge,
    },
    PaletteEntry {
        label: "Build Visual Vector Index (FP16)",
        hint: "Visual",
        cmd: PaletteCmd::IndexVisual,
    },
    PaletteEntry {
        label: "Find Similar to Selected Image",
        hint: "Visual",
        cmd: PaletteCmd::FindSimilar,
    },
    PaletteEntry {
        label: "Find Visual Duplicates (Dedup)",
        hint: "Visual",
        cmd: PaletteCmd::FindDuplicates,
    },
    PaletteEntry {
        label: "Switch to Details List View",
        hint: "View",
        cmd: PaletteCmd::ViewList,
    },
    PaletteEntry {
        label: "Switch to Thumbnail Grid View",
        hint: "View",
        cmd: PaletteCmd::ViewGrid,
    },
    PaletteEntry {
        label: "Smart View: Images (ext:jpg)",
        hint: "Filter",
        cmd: PaletteCmd::SmartImages,
    },
    PaletteEntry {
        label: "Smart View: Videos (ext:mp4)",
        hint: "Filter",
        cmd: PaletteCmd::SmartVideos,
    },
    PaletteEntry {
        label: "Smart View: Large Files (>50MB)",
        hint: "Filter",
        cmd: PaletteCmd::SmartLarge,
    },
    PaletteEntry {
        label: "Clear Search & Folder Filter",
        hint: "Reset",
        cmd: PaletteCmd::ClearFilters,
    },
];

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

        let mut clicked = None;
        ScrollArea::vertical().show(ui, |ui| {
            for (idx, entry) in filtered.iter().enumerate() {
                let is_sel = idx == self.selected;
                ui.horizontal(|ui| {
                    if ui.selectable_label(is_sel, entry.label).clicked() {
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
}
