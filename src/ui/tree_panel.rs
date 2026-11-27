use egui::{RichText, ScrollArea, Ui};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::index::VisibleDir;
use crate::ui::theme::TEXT_MUTED;

const ROW_HEIGHT: f32 = 24.0;

pub fn show_tree_panel(
    ui: &mut Ui,
    tr: &Strings,
    store: Option<&ArchiveStore>,
    visible_dirs: &[VisibleDir],
    selected_dir: Option<u32>,
) -> Option<Option<u32>> {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new(tr.folders_heading).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{}", visible_dirs.len()))
                    .small()
                    .color(TEXT_MUTED),
            );
        });
    });
    ui.separator();

    let Some(st) = store else {
        ui.add_space(8.0);
        ui.label(RichText::new(tr.pick_folder_prompt).color(TEXT_MUTED));
        return None;
    };

    let mut next_selection: Option<Option<u32>> = None;
    ScrollArea::vertical()
        .id_salt("dir_tree_scroll")
        .auto_shrink([false, false])
        .show_rows(ui, ROW_HEIGHT, visible_dirs.len(), |ui, row_range| {
            for row in visible_dirs.get(row_range).unwrap_or(&[]) {
                if render_dir_row(ui, st, *row, selected_dir) {
                    let toggled = if selected_dir == Some(row.dir_id) {
                        None
                    } else {
                        Some(row.dir_id)
                    };
                    next_selection = Some(toggled);
                }
            }
        });
    next_selection
}

fn render_dir_row(
    ui: &mut Ui,
    store: &ArchiveStore,
    row: VisibleDir,
    selected_dir: Option<u32>,
) -> bool {
    let is_selected = selected_dir == Some(row.dir_id);
    let indent = f32::from(row.depth.min(12)) * 12.0;
    let mut clicked = false;

    ui.horizontal(|ui| {
        ui.add_space(indent);
        if ui
            .selectable_label(is_selected, store.dir_name(row.dir_id))
            .clicked()
        {
            clicked = true;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{}", row.match_count))
                    .small()
                    .color(TEXT_MUTED),
            );
        });
    });
    clicked
}
