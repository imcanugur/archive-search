use crate::i18n::Strings;
use crate::ops::merger::{execute_merge, plan_merge};
use crate::ui::dual_pane::show_dual_pane;
use crate::ui::grid_panel::{show_grid_panel, GridParams};
use crate::ui::inspector::InspectorAction;
use crate::ui::merge_drawer::show_merge_drawer;
use crate::ui::theme::SURFACE;

use super::ArchiveApp;

impl ArchiveApp {
    pub(super) fn render_right_inspector(&mut self, ctx: &egui::Context, tr: &Strings) {
        let Some(st) = self.show_inspector.then(|| self.store.clone()).flatten() else {
            return;
        };
        let Some(fid) = self.selected_file else {
            return;
        };
        let frame = egui::Frame::side_top_panel(&ctx.style())
            .fill(SURFACE)
            .inner_margin(8.0);
        let mut act = InspectorAction::default();
        egui::SidePanel::right("inspector_panel")
            .frame(frame)
            .default_width(310.0)
            .min_width(260.0)
            .resizable(true)
            .show(ctx, |ui| {
                let vec_idx = self.vector_index.as_deref();
                act = self
                    .inspector
                    .show(ui, tr, &st, fid, vec_idx, &mut self.thumbs);
            });
        if act.close_panel {
            self.show_inspector = false;
        }
        if act.open_quick_look {
            self.preview_modal.toggle();
        }
        if let Some(sim_id) = act.find_similar {
            self.run_find_similar(sim_id);
        }
        if let Some(tag) = act.filter_tag {
            self.query = tag;
            self.dispatch_query();
        }
    }

    pub(super) fn render_center_pane(
        &mut self,
        ctx: &egui::Context,
        tr: &Strings,
        active: &[u32],
    ) {
        let (mut req_sim, mut req_merge) = (None, false);
        let frame = egui::Frame::central_panel(&ctx.style())
            .fill(SURFACE)
            .inner_margin(8.0);
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            if self.dual_mode {
                let d_act = show_dual_pane(
                    ui,
                    tr,
                    self.store.as_deref(),
                    &self.snapshot.visible_dirs,
                    &mut self.dual_pane,
                    &mut self.selected_file,
                );
                req_merge = d_act.open_merge;
                req_sim = d_act.find_similar;
            } else {
                self.dock_merge_drawer(ui, tr);
                let gp = GridParams {
                    store: self.store.as_deref(),
                    matched_files: active,
                    is_visual_match: self.visual_override.is_some(),
                    view_mode: self.view_mode,
                };
                let gact =
                    show_grid_panel(ui, tr, gp, &mut self.selected_file, &mut self.thumbs);
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

    fn dock_merge_drawer(&mut self, ui: &mut egui::Ui, tr: &Strings) {
        let (Some(&a), Some(&b)) = (self.merge_queue.first(), self.merge_queue.get(1)) else {
            return;
        };
        let Some(st) = self.store.clone() else {
            return;
        };
        let dock = egui::Frame::none().inner_margin(egui::Margin {
            top: 8.0,
            ..Default::default()
        });
        egui::TopBottomPanel::bottom("merge_drawer_dock")
            .frame(dock)
            .show_inside(ui, |ui| {
                let dact = show_merge_drawer(ui, tr, &st, a, b, self.journal.can_undo());
                if dact.clear_queue {
                    self.merge_queue.clear();
                } else if dact.open_dry_run {
                    self.merge_modal.open_picker(Some(a), Some(b));
                } else if dact.trigger_undo {
                    self.trigger_undo();
                } else if dact.merge_now {
                    self.execute_queued_merge(&st, a, b);
                }
            });
    }

    fn execute_queued_merge(
        &mut self,
        st: &crate::index::store::ArchiveStore,
        a: u32,
        b: u32,
    ) {
        let path_a = st.dir_full_path(a);
        let merged = format!("{}-{}", st.dir_name(a), st.dir_name(b));
        let target = path_a
            .parent()
            .map_or_else(|| std::path::PathBuf::from(&merged), |p| p.join(&merged));
        match plan_merge(&path_a, &st.dir_full_path(b), target).and_then(|p| execute_merge(&p))
        {
            Ok(entry) => {
                self.journal.push(entry);
                self.merge_queue.clear();
                self.rescan_current();
            }
            Err(err) => self.error_msg = Some(err.to_string()),
        }
    }
}
