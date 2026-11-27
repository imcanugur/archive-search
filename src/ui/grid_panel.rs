use egui::{RichText, ScrollArea, Ui};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::ui::cards::{card_height, render_card, CardSpec, MIN_CARD_WIDTH};
use crate::ui::list_view::render_list_view;
use crate::ui::theme::TEXT_MUTED;
use crate::ui::thumbs::ThumbCache;

const CARD_GAP: f32 = 10.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    List,
    #[default]
    Grid,
}

pub struct GridParams<'a> {
    pub store: Option<&'a ArchiveStore>,
    pub matched_files: &'a [u32],
    pub is_visual_match: bool,
    pub view_mode: ViewMode,
}

#[derive(Default)]
pub struct GridAction {
    pub pick_folder: bool,
    pub find_similar: Option<u32>,
}

pub fn show_grid_panel(
    ui: &mut Ui,
    tr: &Strings,
    p: GridParams<'_>,
    selected_file: &mut Option<u32>,
    thumbs: &mut ThumbCache,
) -> GridAction {
    let mut act = GridAction::default();
    let Some(st) = p.store else {
        act.pick_folder = render_empty_state(ui, tr);
        return act;
    };
    if p.matched_files.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space((ui.available_height() * 0.35).max(40.0));
            ui.label(RichText::new(tr.no_matches).color(TEXT_MUTED));
        });
        return act;
    }
    act.find_similar = match p.view_mode {
        ViewMode::List => render_list_view(ui, tr, st, p.matched_files, selected_file),
        ViewMode::Grid => render_grid_view(
            ui,
            tr,
            st,
            p.matched_files,
            p.is_visual_match,
            selected_file,
            thumbs,
        ),
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
    is_visual_match: bool,
    selected_file: &mut Option<u32>,
    thumbs: &mut ThumbCache,
) -> Option<u32> {
    let avail = (ui.available_width() - 8.0).max(MIN_CARD_WIDTH);
    let cols = ((avail / (MIN_CARD_WIDTH + CARD_GAP)).floor() as usize).max(1);
    let card_w = ((avail - CARD_GAP * (cols.saturating_sub(1) as f32)) / cols as f32)
        .floor()
        .max(MIN_CARD_WIDTH);
    let row_h = card_height(card_w) + CARD_GAP;
    let rows = matched_files.len().div_ceil(cols);
    let mut find_sim = None;

    ScrollArea::vertical()
        .id_salt("file_grid_scroll")
        .auto_shrink([false, false])
        .show_rows(ui, row_h, rows, |ui, range| {
            for r in range {
                let start = r * cols;
                let end = (start + cols).min(matched_files.len());
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = CARD_GAP;
                    for &fid in matched_files.get(start..end).unwrap_or(&[]) {
                        let spec = CardSpec {
                            fid,
                            width: card_w,
                            is_visual_match,
                        };
                        if let Some(id) =
                            render_card(ui, tr, store, spec, selected_file, thumbs)
                        {
                            find_sim = Some(id);
                        }
                    }
                });
                ui.add_space(2.0);
            }
        });
    find_sim
}
