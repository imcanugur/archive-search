use egui::{RichText, ScrollArea, Ui};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::ui::list_view::render_list_view;
use crate::ui::theme::TEXT_MUTED;
use crate::ui::thumbs::ThumbCache;
use crate::ui::tile_view::{render_tile, CARD_HEIGHT, CARD_WIDTH};

const CARD_GAP: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    List,
    #[default]
    Grid,
}

#[derive(Default)]
pub struct GridAction {
    pub pick_folder: bool,
    pub find_similar: Option<u32>,
}

pub fn show_grid_panel(
    ui: &mut Ui,
    tr: &Strings,
    store: Option<&ArchiveStore>,
    matched_files: &[u32],
    selected_file: &mut Option<u32>,
    view_mode: ViewMode,
    thumbs: &mut ThumbCache,
) -> GridAction {
    let mut act = GridAction::default();
    let Some(st) = store else {
        act.pick_folder = render_empty_state(ui, tr);
        return act;
    };
    if matched_files.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new(tr.no_matches).color(TEXT_MUTED));
        });
        return act;
    }
    act.find_similar = match view_mode {
        ViewMode::List => render_list_view(ui, tr, st, matched_files, selected_file),
        ViewMode::Grid => render_grid_view(ui, tr, st, matched_files, selected_file, thumbs),
    };
    act
}

fn render_empty_state(ui: &mut Ui, tr: &Strings) -> bool {
    let mut clicked = false;
    ui.vertical_centered(|ui| {
        ui.add_space((ui.available_height() * 0.35).max(40.0));
        ui.label(RichText::new(tr.empty_subtitle).color(TEXT_MUTED));
        ui.add_space(10.0);
        if ui.button(tr.select_archive_btn).clicked() {
            clicked = true;
        }
    });
    clicked
}

fn render_grid_view(
    ui: &mut Ui,
    tr: &Strings,
    store: &ArchiveStore,
    matched_files: &[u32],
    selected_file: &mut Option<u32>,
    thumbs: &mut ThumbCache,
) -> Option<u32> {
    let cols = ((ui.available_width().max(CARD_WIDTH) / (CARD_WIDTH + CARD_GAP)).floor()
        as usize)
        .max(1);
    let rows = matched_files.len().div_ceil(cols);
    let mut find_sim = None;
    ScrollArea::vertical()
        .id_salt("file_grid_scroll")
        .auto_shrink([false, false])
        .show_rows(ui, CARD_HEIGHT + CARD_GAP, rows, |ui, range| {
            for r in range {
                let start = r * cols;
                let end = (start + cols).min(matched_files.len());
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = CARD_GAP;
                    for &fid in matched_files.get(start..end).unwrap_or(&[]) {
                        if let Some(id) =
                            render_tile(ui, tr, store, fid, selected_file, thumbs)
                        {
                            find_sim = Some(id);
                        }
                    }
                });
            }
        });
    find_sim
}
