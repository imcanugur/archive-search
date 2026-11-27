use egui::{Frame, RichText, Rounding, ScrollArea, Stroke, Ui};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::index::VisibleDir;
use crate::ui::dir_row::render_dir_row;
use crate::ui::smart_views::render_smart_views_card;
use crate::ui::theme::{
    ON_SURFACE_VARIANT, OUTLINE_VARIANT, PRIMARY, SURFACE_CONTAINER, SURFACE_LOWEST,
};

const ROW_HEIGHT: f32 = 26.0;

pub struct TreeParams<'a> {
    pub store: Option<&'a ArchiveStore>,
    pub visible_dirs: &'a [VisibleDir],
    pub selected_dir: Option<u32>,
    pub active_query: &'a str,
    pub visual_active: bool,
}

#[derive(Default)]
pub struct TreeAction {
    pub select_dir: Option<Option<u32>>,
    pub smart_query: Option<&'static str>,
    pub find_duplicates: bool,
}

pub fn show_tree_panel(
    ui: &mut Ui,
    tr: &Strings,
    p: TreeParams<'_>,
    merge_queue: &mut Vec<u32>,
) -> TreeAction {
    let mut act = TreeAction::default();
    render_smart_views_card(ui, tr, p.store, p.active_query, p.visual_active, &mut act);
    ui.add_space(6.0);
    render_dir_tree_card(
        ui,
        tr,
        p.store,
        p.visible_dirs,
        p.selected_dir,
        merge_queue,
        &mut act,
    );
    act
}

fn render_dir_tree_card(
    ui: &mut Ui,
    tr: &Strings,
    store: Option<&ArchiveStore>,
    visible_dirs: &[VisibleDir],
    selected_dir: Option<u32>,
    merge_queue: &mut Vec<u32>,
    act: &mut TreeAction,
) {
    Frame::none()
        .fill(SURFACE_LOWEST)
        .stroke(Stroke::new(1.0_f32, OUTLINE_VARIANT))
        .rounding(Rounding::same(8.0))
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(tr.folders_heading)
                        .small()
                        .strong()
                        .color(ON_SURFACE_VARIANT),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let txt = format!("{} dirs", visible_dirs.len());
                    ui.label(RichText::new(txt).monospace().color(ON_SURFACE_VARIANT));
                });
            });
            ui.separator();
            let Some(st) = store else {
                ui.label(RichText::new(tr.pick_folder_prompt).color(ON_SURFACE_VARIANT));
                return;
            };
            ScrollArea::vertical()
                .id_salt("dir_tree_scroll")
                .auto_shrink([false, false])
                .show_rows(ui, ROW_HEIGHT, visible_dirs.len(), |ui, range| {
                    for row in visible_dirs.get(range).unwrap_or(&[]) {
                        if render_dir_row(ui, st, *row, selected_dir, merge_queue) {
                            let toggled =
                                (selected_dir != Some(row.dir_id)).then_some(row.dir_id);
                            act.select_dir = Some(toggled);
                        }
                    }
                });
            if merge_queue.len() == 2 {
                Frame::none()
                    .fill(SURFACE_CONTAINER)
                    .rounding(Rounding::same(6.0))
                    .inner_margin(6.0)
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("2 folders queued for batch merge")
                                .small()
                                .strong()
                                .color(PRIMARY),
                        );
                    });
            }
        });
}
