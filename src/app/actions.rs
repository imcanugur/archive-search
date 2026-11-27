use egui::{Key, Modifiers};

use crate::i18n::{strings, Strings};
use crate::messages::WorkerCmd;
use crate::ui::command_palette::PaletteCmd;
use crate::ui::grid_panel::ViewMode;

use super::ArchiveApp;

impl ArchiveApp {
    pub(super) fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| {
            i.modifiers.matches_logically(Modifiers::CTRL)
                && (i.key_pressed(Key::K) || i.key_pressed(Key::P))
        }) {
            self.palette.toggle();
            return;
        }
        if ctx.wants_keyboard_input() {
            return;
        }
        if ctx.input(|i| i.key_pressed(Key::F2)) {
            self.dual_mode = !self.dual_mode;
        }
        if ctx.input(|i| i.key_pressed(Key::Space))
            && (self.selected_file.is_some() || self.preview_modal.is_open())
        {
            self.preview_modal.toggle();
        }
        if self.dual_mode && ctx.input(|i| i.key_pressed(Key::M)) {
            self.merge_modal
                .open_picker(self.dual_pane.dir_a, self.dual_pane.dir_b);
        }
        if ctx
            .input(|i| i.modifiers.matches_logically(Modifiers::CTRL) && i.key_pressed(Key::Z))
        {
            self.trigger_undo();
        }
    }

    pub(super) fn apply_palette_cmd(&mut self, cmd: PaletteCmd, tr: &Strings) {
        match cmd {
            PaletteCmd::OpenFolder => self.folder_modal.open_at(None, tr),
            PaletteCmd::RescanFolder => self.rescan_current(),
            PaletteCmd::ToggleDualPane => self.dual_mode = !self.dual_mode,
            PaletteCmd::OpenMerge => self.open_merge_modal(),
            PaletteCmd::UndoMerge => self.trigger_undo(),
            PaletteCmd::IndexVisual => self.start_visual_index(),
            PaletteCmd::FindSimilar => {
                if let Some(fid) = self.selected_file {
                    self.run_find_similar(fid);
                }
            }
            PaletteCmd::FindDuplicates => self.run_find_duplicates(),
            PaletteCmd::ViewList => self.view_mode = ViewMode::List,
            PaletteCmd::ViewGrid => self.view_mode = ViewMode::Grid,
            PaletteCmd::SmartImages => self.set_smart_query("type:photo"),
            PaletteCmd::SmartVideos => self.set_smart_query("type:video"),
            PaletteCmd::SmartLarge => self.set_smart_query("size:>50mb"),
            PaletteCmd::ClearFilters => {
                self.query.clear();
                self.selected_dir = None;
                self.dispatch_query();
            }
        }
    }

    fn set_smart_query(&mut self, q: &str) {
        self.query = q.to_owned();
        self.dispatch_query();
    }

    pub(super) fn open_merge_modal(&mut self) {
        if self.dual_mode {
            self.merge_modal
                .open_picker(self.dual_pane.dir_a, self.dual_pane.dir_b);
        } else {
            self.merge_modal.open_picker(self.selected_dir, None);
        }
    }

    pub(super) fn navigate_parent_dir(&mut self) {
        if let (Some(st), Some(d)) = (self.store.as_deref(), self.selected_dir) {
            let p = st.dir_parent.get(d as usize).copied().unwrap_or(0);
            self.selected_dir = (p != 0).then_some(p);
            self.dispatch_query();
        }
    }

    pub(super) fn render_modals(&mut self, ctx: &egui::Context, active: &[u32]) {
        let tr = strings(self.locale);
        if let Some(cmd) = self.palette.show(ctx, tr) {
            self.apply_palette_cmd(cmd, tr);
        }
        if let Some(dir) = self.folder_modal.show(ctx, tr) {
            let _ = self.cmd_tx.send(WorkerCmd::ScanDir(dir));
        }
        if let Some(ref st) = self.store {
            self.preview_modal
                .show(ctx, tr, st, active, &mut self.selected_file);
            if let Some(entry) =
                self.merge_modal
                    .show(ctx, tr, st, &self.snapshot.visible_dirs)
            {
                self.journal.push(entry);
                self.rescan_current();
            }
        }
    }
}
