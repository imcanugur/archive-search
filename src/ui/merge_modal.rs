use egui::{Button, Color32, Context, RichText, Ui, Window};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::index::VisibleDir;
use crate::ops::merger::{execute_merge, plan_merge};
use crate::ops::{JournalEntry, MergePlan, MergeTemplate};
use crate::ui::merge_view::{
    compute_merge_target, dir_combo, render_plan_summary, render_template_row,
};
use crate::ui::theme::{ACCENT, TEXT_MUTED, TEXT_PRIMARY};

pub struct MergeModal {
    open: bool,
    dir_a: Option<u32>,
    dir_b: Option<u32>,
    template: MergeTemplate,
    custom_fmt: String,
    plan: Option<MergePlan>,
    error: Option<String>,
}

impl Default for MergeModal {
    fn default() -> Self {
        Self {
            open: false,
            dir_a: None,
            dir_b: None,
            template: MergeTemplate::DashJoin,
            custom_fmt: "{A}-{B}".to_owned(),
            plan: None,
            error: None,
        }
    }
}

impl MergeModal {
    pub fn open_picker(&mut self, initial_a: Option<u32>, initial_b: Option<u32>) {
        self.dir_a = initial_a;
        self.dir_b = initial_b;
        self.plan = None;
        self.error = None;
        self.open = true;
    }

    pub fn show(
        &mut self,
        ctx: &Context,
        tr: &Strings,
        store: &ArchiveStore,
        dirs: &[VisibleDir],
    ) -> Option<JournalEntry> {
        if !self.open {
            return None;
        }
        let mut committed = None;
        let mut keep = self.open;
        Window::new(tr.merge_title)
            .open(&mut keep)
            .collapsible(false)
            .resizable(true)
            .default_size([560.0, 440.0])
            .show(ctx, |ui| committed = self.render_body(ui, tr, store, dirs));
        self.open = keep && committed.is_none();
        committed
    }

    fn render_body(
        &mut self,
        ui: &mut Ui,
        tr: &Strings,
        store: &ArchiveStore,
        dirs: &[VisibleDir],
    ) -> Option<JournalEntry> {
        let mut changed = false;
        ui.horizontal(|ui| {
            changed |= dir_combo(ui, "merge_a", tr.dual_pane_a, store, dirs, &mut self.dir_a);
            changed |= dir_combo(ui, "merge_b", tr.dual_pane_b, store, dirs, &mut self.dir_b);
        });
        changed |= render_template_row(ui, tr, &mut self.template, &mut self.custom_fmt);
        if changed {
            self.plan = None;
        }
        self.render_controls(ui, tr, store);
        self.render_plan_actions(ui, tr)
    }

    fn render_controls(&mut self, ui: &mut Ui, tr: &Strings, store: &ArchiveStore) {
        let ready = matches!((self.dir_a, self.dir_b), (Some(a), Some(b)) if a != b);
        let target = compute_merge_target(
            store,
            self.dir_a,
            self.dir_b,
            self.template,
            &self.custom_fmt,
        );
        if let Some(ref t) = target {
            let msg = format!("{} {}", tr.merge_target, t.display());
            ui.label(RichText::new(msg).small().color(TEXT_MUTED));
        }
        if ui
            .add_enabled(ready, Button::new(tr.merge_dry_run_btn))
            .clicked()
        {
            self.run_dry_run(store, target);
        }
        if let Some(ref err) = self.error {
            ui.label(RichText::new(err).color(Color32::LIGHT_RED).small());
        }
        ui.separator();
    }

    fn run_dry_run(&mut self, store: &ArchiveStore, target: Option<std::path::PathBuf>) {
        let (Some(a), Some(b), Some(t)) = (self.dir_a, self.dir_b, target) else {
            return;
        };
        match plan_merge(&store.dir_full_path(a), &store.dir_full_path(b), t) {
            Ok(p) => {
                self.plan = Some(p);
                self.error = None;
            }
            Err(err) => self.error = Some(err.to_string()),
        }
    }

    fn render_plan_actions(&mut self, ui: &mut Ui, tr: &Strings) -> Option<JournalEntry> {
        let plan = self.plan.as_ref()?;
        let mut committed = None;
        render_plan_summary(ui, tr, plan);
        ui.separator();
        ui.horizontal(|ui| {
            let lbl = RichText::new(tr.merge_execute_btn)
                .strong()
                .color(TEXT_PRIMARY);
            if ui.add(Button::new(lbl).fill(ACCENT)).clicked() {
                match execute_merge(plan) {
                    Ok(entry) => committed = Some(entry),
                    Err(err) => self.error = Some(err.to_string()),
                }
            }
            if ui.button(tr.modal_cancel_btn).clicked() {
                self.open = false;
            }
        });
        committed
    }
}
