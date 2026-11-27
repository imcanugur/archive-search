use std::sync::atomic::Ordering;

use crate::i18n::{strings, Strings};
use crate::ui::dual_pane::show_dual_pane;
use crate::ui::grid_panel::show_grid_panel;
use crate::ui::status_bar::{show_status_bar, StatusInfo};
use crate::ui::theme::BG_CANVAS;
use crate::ui::top_bar::{show_top_bar, TopBarState};
use crate::ui::tree_panel::show_tree_panel;

use super::ArchiveApp;

impl ArchiveApp {
    pub(super) fn render_bars(&mut self, ctx: &egui::Context) {
        let tr = strings(self.locale);
        self.render_top_bar(ctx, tr);
        self.render_status_bar(ctx, tr);
    }

    fn render_top_bar(&mut self, ctx: &egui::Context, tr: &Strings) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            let mut st = TopBarState {
                query: &mut self.query,
                store: self.store.as_deref(),
                selected_dir: self.selected_dir,
                view_mode: &mut self.view_mode,
                dual_pane: &mut self.dual_mode,
                scanning: self.scanning,
                can_undo: self.journal.can_undo(),
            };
            let act = show_top_bar(ui, tr, &mut st);
            if act.pick_folder {
                self.folder_modal.open_at(None, tr);
            }
            if act.open_merge {
                self.open_merge_modal();
            }
            if act.trigger_undo {
                self.trigger_undo();
            }
            if act.go_up {
                self.navigate_parent_dir();
            } else if act.clear_selected_dir {
                self.selected_dir = None;
                self.dispatch_query();
            } else if act.query_changed {
                self.dispatch_query();
            }
        });
    }

    fn render_status_bar(&mut self, ctx: &egui::Context, tr: &Strings) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
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
        });
    }

    pub(super) fn render_workspace(&mut self, ctx: &egui::Context, active: &[u32]) {
        let tr = strings(self.locale);
        if !self.dual_mode {
            egui::SidePanel::left("tree_panel")
                .default_width(260.0)
                .resizable(true)
                .show(ctx, |ui| {
                    if let Some(next_dir) = show_tree_panel(
                        ui,
                        tr,
                        self.store.as_deref(),
                        &self.snapshot.visible_dirs,
                        self.selected_dir,
                    ) {
                        self.selected_dir = next_dir;
                        self.dispatch_query();
                    }
                });
        }
        let (mut req_sim, mut req_merge) = (None, false);
        let frame = egui::Frame::central_panel(&ctx.style())
            .fill(BG_CANVAS)
            .inner_margin(8.0);
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            if self.dual_mode {
                let act = show_dual_pane(
                    ui,
                    tr,
                    self.store.as_deref(),
                    &self.snapshot.visible_dirs,
                    &mut self.dual_pane,
                    &mut self.selected_file,
                );
                req_merge = act.open_merge;
                req_sim = act.find_similar;
            } else {
                let gact = show_grid_panel(
                    ui,
                    tr,
                    self.store.as_deref(),
                    active,
                    &mut self.selected_file,
                    self.view_mode,
                    &mut self.thumbs,
                );
                if gact.pick_folder {
                    self.folder_modal.open_at(None, tr);
                }
                req_sim = gact.find_similar;
            }
        });
        if req_merge {
            self.open_merge_modal();
        }
        if let Some(fid) = req_sim {
            self.run_find_similar(fid);
        }
    }
}
