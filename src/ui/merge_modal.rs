use std::path::PathBuf;

use egui::{Button, Color32, ComboBox, Context, RichText, ScrollArea, TextEdit, Ui, Window};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::index::VisibleDir;
use crate::ops::merger::{execute_merge, plan_merge};
use crate::ops::{ConflictAction, JournalEntry, MergePlan, MergeTemplate};
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
    pub fn open_picker(&mut self, initial_a: Option<u32>) {
        self.dir_a = initial_a;
        self.dir_b = None;
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
        let mut committed = None;
        let mut changed = false;
        ui.horizontal(|ui| {
            changed |= dir_combo(ui, "merge_a", tr.dual_pane_a, store, dirs, &mut self.dir_a);
            changed |= dir_combo(ui, "merge_b", tr.dual_pane_b, store, dirs, &mut self.dir_b);
        });
        ui.horizontal(|ui| {
            ui.label(tr.merge_template);
            changed |= ui
                .selectable_value(&mut self.template, MergeTemplate::DashJoin, "{A}-{B}")
                .changed();
            changed |= ui
                .selectable_value(&mut self.template, MergeTemplate::UnderscoreJoin, "{A}_{B}")
                .changed();
            changed |= ui
                .selectable_value(&mut self.template, MergeTemplate::Custom, "Custom")
                .changed();
            if self.template == MergeTemplate::Custom {
                changed |= ui
                    .add(TextEdit::singleline(&mut self.custom_fmt).desired_width(130.0))
                    .changed();
            }
        });
        if changed {
            self.plan = None;
        }

        let ready = matches!((self.dir_a, self.dir_b), (Some(a), Some(b)) if a != b);
        if let Some(target) = self.compute_target(store) {
            ui.label(
                RichText::new(format!("{} {}", tr.merge_target, target.display()))
                    .small()
                    .color(TEXT_MUTED),
            );
        }
        if ui
            .add_enabled(ready, Button::new(tr.merge_dry_run_btn))
            .clicked()
        {
            self.run_dry_run(store);
        }
        if let Some(ref err) = self.error {
            ui.label(RichText::new(err).color(Color32::LIGHT_RED).small());
        }
        ui.separator();

        if let Some(ref plan) = self.plan {
            render_plan_summary(ui, tr, plan);
            ui.separator();
            ui.horizontal(|ui| {
                let exec = Button::new(
                    RichText::new(tr.merge_execute_btn)
                        .strong()
                        .color(TEXT_PRIMARY),
                )
                .fill(ACCENT);
                if ui.add(exec).clicked() {
                    match execute_merge(plan) {
                        Ok(entry) => committed = Some(entry),
                        Err(err) => self.error = Some(err.to_string()),
                    }
                }
                if ui.button(tr.modal_cancel_btn).clicked() {
                    self.open = false;
                }
            });
        }
        committed
    }

    fn compute_target(&self, store: &ArchiveStore) -> Option<PathBuf> {
        let (Some(a), Some(b)) = (self.dir_a, self.dir_b) else {
            return None;
        };
        let path_a = store.dir_full_path(a);
        let merged =
            self.template
                .format_name(store.dir_name(a), store.dir_name(b), &self.custom_fmt);
        Some(
            path_a
                .parent()
                .map_or_else(|| PathBuf::from(&merged), |p| p.join(&merged)),
        )
    }

    fn run_dry_run(&mut self, store: &ArchiveStore) {
        let (Some(a), Some(b)) = (self.dir_a, self.dir_b) else {
            return;
        };
        let Some(target) = self.compute_target(store) else {
            return;
        };
        match plan_merge(&store.dir_full_path(a), &store.dir_full_path(b), target) {
            Ok(p) => {
                self.plan = Some(p);
                self.error = None;
            }
            Err(err) => self.error = Some(err.to_string()),
        }
    }
}

fn dir_combo(
    ui: &mut Ui,
    id: &str,
    label: &str,
    store: &ArchiveStore,
    dirs: &[VisibleDir],
    selected: &mut Option<u32>,
) -> bool {
    let mut changed = false;
    let cur = selected.map_or(label, |d| store.dir_name(d));
    ComboBox::from_id_salt(id)
        .selected_text(cur)
        .width(220.0)
        .show_ui(ui, |ui| {
            for row in dirs.iter().take(500) {
                if ui
                    .selectable_value(selected, Some(row.dir_id), store.dir_name(row.dir_id))
                    .changed()
                {
                    changed = true;
                }
            }
        });
    changed
}

fn render_plan_summary(ui: &mut Ui, tr: &Strings, plan: &MergePlan) {
    ui.label(format!("{}: {}", tr.merge_summary_moves, plan.move_count));
    ui.label(format!(
        "{}: {}",
        tr.merge_summary_dups, plan.duplicate_count
    ));
    ui.label(format!(
        "{}: {}",
        tr.merge_summary_renames, plan.rename_count
    ));
    ScrollArea::vertical().max_height(170.0).show(ui, |ui| {
        for op in plan.ops.iter().take(200) {
            let tag = match op.action {
                ConflictAction::MoveDirect => "[MOVE]",
                ConflictAction::SkipDuplicate => "[SKIP DUP]",
                ConflictAction::RenameSuffix => "[RENAME]",
            };
            let dst_name = op.dst.file_name().and_then(|n| n.to_str()).unwrap_or("");
            ui.label(
                RichText::new(format!("{tag}  {dst_name}"))
                    .small()
                    .color(TEXT_MUTED),
            );
        }
    });
}
