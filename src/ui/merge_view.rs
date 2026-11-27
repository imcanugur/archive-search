use std::path::PathBuf;

use egui::{ComboBox, RichText, ScrollArea, TextEdit, Ui};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::index::VisibleDir;
use crate::ops::{ConflictAction, MergePlan, MergeTemplate};
use crate::ui::theme::TEXT_MUTED;

pub fn compute_merge_target(
    store: &ArchiveStore,
    dir_a: Option<u32>,
    dir_b: Option<u32>,
    template: MergeTemplate,
    custom_fmt: &str,
) -> Option<PathBuf> {
    let (Some(a), Some(b)) = (dir_a, dir_b) else {
        return None;
    };
    let path_a = store.dir_full_path(a);
    let merged = template.format_name(store.dir_name(a), store.dir_name(b), custom_fmt);
    Some(
        path_a
            .parent()
            .map_or_else(|| PathBuf::from(&merged), |p| p.join(&merged)),
    )
}

pub fn render_template_row(
    ui: &mut Ui,
    tr: &Strings,
    template: &mut MergeTemplate,
    custom_fmt: &mut String,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(tr.merge_template);
        changed |= ui
            .selectable_value(template, MergeTemplate::DashJoin, "{A}-{B}")
            .changed();
        changed |= ui
            .selectable_value(template, MergeTemplate::UnderscoreJoin, "{A}_{B}")
            .changed();
        changed |= ui
            .selectable_value(template, MergeTemplate::Custom, "Custom")
            .changed();
        if *template == MergeTemplate::Custom {
            changed |= ui
                .add(TextEdit::singleline(custom_fmt).desired_width(130.0))
                .changed();
        }
    });
    changed
}

pub fn dir_combo(
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

pub fn render_plan_summary(ui: &mut Ui, tr: &Strings, plan: &MergePlan) {
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
