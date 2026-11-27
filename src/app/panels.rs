use std::sync::atomic::Ordering;

use crate::i18n::{strings, Strings};
use crate::ui::status_bar::{show_status_bar, StatusInfo};
use crate::ui::theme::{SURFACE, SURFACE_LOWEST};
use crate::ui::top_bar::{show_top_bar, TopBarState};
use crate::ui::tree_panel::{show_tree_panel, TreeParams};

use super::ArchiveApp;

impl ArchiveApp {
    pub(super) fn render_bars(&mut self, ctx: &egui::Context) {
        let tr = strings(self.locale);
        let top_frame = egui::Frame::side_top_panel(&ctx.style())
            .fill(SURFACE_LOWEST)
            .inner_margin(egui::Margin::symmetric(12.0, 6.0));
        egui::TopBottomPanel::top("top_bar")
            .frame(top_frame)
            .show(ctx, |ui| self.render_top_bar_inner(ui, tr));

        let bot_frame = egui::Frame::side_top_panel(&ctx.style())
            .fill(SURFACE_LOWEST)
            .inner_margin(egui::Margin::symmetric(12.0, 4.0));
        egui::TopBottomPanel::bottom("status_bar")
            .frame(bot_frame)
            .show(ctx, |ui| self.render_status_inner(ui, tr));
    }

    fn render_top_bar_inner(&mut self, ui: &mut egui::Ui, tr: &Strings) {
        let mut st = TopBarState {
            query: &mut self.query,
            store: self.store.as_deref(),
            selected_dir: self.selected_dir,
            elapsed_us: self.snapshot.elapsed_us,
            view_mode: &mut self.view_mode,
            dual_pane: &mut self.dual_mode,
            show_inspector: &mut self.show_inspector,
            scanning: self.scanning,
            size_progress: self.size_progress,
        };
        let act = show_top_bar(ui, tr, &mut st);
        if act.pick_folder {
            self.folder_modal.open_at(None, tr);
        }
        if act.open_palette {
            self.palette.toggle();
        }
        if act.open_merge {
            self.open_merge_modal();
        }
        if act.find_duplicates {
            self.run_find_duplicates();
        }
        if act.clear_all {
            self.query.clear();
            self.selected_dir = None;
            self.dispatch_query();
        } else if act.go_up {
            self.navigate_parent_dir();
        } else if let Some(dir) = act.select_dir {
            self.selected_dir = dir;
            self.dispatch_query();
        } else if act.query_changed {
            self.dispatch_query();
        }
    }

    fn render_status_inner(&mut self, ui: &mut egui::Ui, tr: &Strings) {
        let info = StatusInfo {
            store: self.store.as_deref(),
            snapshot: &self.snapshot,
            resources: self.resources,
            thumb_vram_bytes: self.thumbs.vram_bytes(),
            visual_status: self.visual_status_text(),
            scanning: self.scanning,
            scan_dirs: self.scan_dirs,
            scan_files: self.scan_files,
            last_scan_ms: self.last_scan_ms,
            error_msg: self.error_msg.as_deref(),
        };
        if show_status_bar(ui, tr, &info) {
            self.scan_cancel.store(true, Ordering::Relaxed);
            self.gpu_cancel.store(true, Ordering::Relaxed);
        }
    }

    pub(super) fn render_workspace(&mut self, ctx: &egui::Context, active: &[u32]) {
        let tr = strings(self.locale);
        self.render_left_rail(ctx, tr);
        self.render_right_inspector(ctx, tr);
        self.render_center_pane(ctx, tr, active);
    }

    fn render_left_rail(&mut self, ctx: &egui::Context, tr: &Strings) {
        if self.dual_mode {
            return;
        }
        let frame = egui::Frame::side_top_panel(&ctx.style())
            .fill(SURFACE)
            .inner_margin(8.0);
        egui::SidePanel::left("tree_panel")
            .frame(frame)
            .default_width(265.0)
            .min_width(210.0)
            .max_width(420.0)
            .resizable(true)
            .show(ctx, |ui| {
                let tp = TreeParams {
                    store: self.store.as_deref(),
                    visible_dirs: &self.snapshot.visible_dirs,
                    selected_dir: self.selected_dir,
                    active_query: &self.query,
                    visual_active: self.visual_override.is_some(),
                };
                let act = show_tree_panel(ui, tr, tp, &mut self.merge_queue);
                if let Some(next_dir) = act.select_dir {
                    self.selected_dir = next_dir;
                    self.dispatch_query();
                }
                if let Some(q) = act.smart_query {
                    self.query = q.to_owned();
                    self.dispatch_query();
                }
                if act.find_duplicates {
                    self.run_find_duplicates();
                }
            });
    }
}
