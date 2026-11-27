use egui::{Frame, RichText, Rounding, ScrollArea, Sense, Stroke, Ui, Vec2};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::index::MediaKind;
use crate::ui::os_open::handle_file_interaction;
use crate::ui::theme::{ACCENT, BG_SELECTED, BG_SURFACE, BORDER_SUBTLE, TEXT_MUTED};
use crate::ui::{badge_color, format_bytes};

const ROW_HEIGHT: f32 = 25.0;
const CARD_WIDTH: f32 = 176.0;
const CARD_HEIGHT: f32 = 76.0;
const CARD_GAP: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    List,
    Grid,
}

pub fn show_grid_panel(
    ui: &mut Ui,
    tr: &Strings,
    store: Option<&ArchiveStore>,
    matched_files: &[u32],
    selected_file: &mut Option<u32>,
    view_mode: ViewMode,
) -> bool {
    let Some(st) = store else {
        return render_empty_state(ui, tr);
    };
    if matched_files.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new(tr.no_matches).color(TEXT_MUTED));
        });
        return false;
    }
    match view_mode {
        ViewMode::List => render_list_view(ui, tr, st, matched_files, selected_file),
        ViewMode::Grid => render_grid_view(ui, tr, st, matched_files, selected_file),
    }
    false
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

fn render_list_view(
    ui: &mut Ui,
    tr: &Strings,
    store: &ArchiveStore,
    matched_files: &[u32],
    selected_file: &mut Option<u32>,
) {
    let w = ui.available_width().max(400.0);
    let cols = [(w * 0.48).max(180.0), (w * 0.28).max(110.0), 64.0_f32];
    ui.horizontal(|ui| {
        ui.add_sized(
            [cols[0], 20.0],
            egui::Label::new(RichText::new(tr.col_name).small().color(TEXT_MUTED)),
        );
        ui.add_sized(
            [cols[1], 20.0],
            egui::Label::new(RichText::new(tr.col_folder).small().color(TEXT_MUTED)),
        );
        ui.add_sized(
            [cols[2], 20.0],
            egui::Label::new(RichText::new(tr.col_type).small().color(TEXT_MUTED)),
        );
        ui.label(RichText::new(tr.col_size).small().color(TEXT_MUTED));
    });
    ui.separator();

    ScrollArea::vertical()
        .id_salt("file_list_scroll")
        .auto_shrink([false, false])
        .show_rows(ui, ROW_HEIGHT, matched_files.len(), |ui, range| {
            for &fid in matched_files.get(range).unwrap_or(&[]) {
                render_list_row(ui, tr, store, fid, cols, selected_file);
            }
        });
}

fn render_list_row(
    ui: &mut Ui,
    tr: &Strings,
    store: &ArchiveStore,
    fid: u32,
    cols: [f32; 3],
    selected_file: &mut Option<u32>,
) {
    let is_sel = *selected_file == Some(fid);
    let kind = store
        .file_kind
        .get(fid as usize)
        .copied()
        .unwrap_or(MediaKind::Other);
    let size = store.file_size.get(fid as usize).copied().unwrap_or(0);
    let dir_id = store.file_dir.get(fid as usize).copied().unwrap_or(0);
    let fill = if is_sel {
        BG_SELECTED
    } else {
        egui::Color32::TRANSPARENT
    };

    let resp = Frame::none()
        .fill(fill)
        .rounding(Rounding::same(3.0))
        .inner_margin(egui::Margin::symmetric(4.0, 2.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_sized(
                    [cols[0], 20.0],
                    egui::Label::new(store.file_name(fid)).truncate(),
                );
                ui.add_sized(
                    [cols[1], 20.0],
                    egui::Label::new(RichText::new(store.dir_name(dir_id)).color(TEXT_MUTED))
                        .truncate(),
                );
                ui.add_sized(
                    [cols[2], 20.0],
                    egui::Label::new(
                        RichText::new(kind.badge()).small().color(badge_color(kind)),
                    ),
                );
                let sz = if size == 0 {
                    "-".to_owned()
                } else {
                    format_bytes(size)
                };
                ui.label(RichText::new(sz).small().color(TEXT_MUTED));
            });
        })
        .response
        .interact(Sense::click());
    handle_file_interaction(&resp, tr, store, fid, selected_file);
}

fn render_grid_view(
    ui: &mut Ui,
    tr: &Strings,
    store: &ArchiveStore,
    matched_files: &[u32],
    selected_file: &mut Option<u32>,
) {
    let cols = ((ui.available_width().max(CARD_WIDTH) / (CARD_WIDTH + CARD_GAP)).floor()
        as usize)
        .max(1);
    let rows = matched_files.len().div_ceil(cols);
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
                        render_tile(ui, tr, store, fid, selected_file);
                    }
                });
            }
        });
}

fn render_tile(
    ui: &mut Ui,
    tr: &Strings,
    store: &ArchiveStore,
    fid: u32,
    selected_file: &mut Option<u32>,
) {
    let is_sel = *selected_file == Some(fid);
    let kind = store
        .file_kind
        .get(fid as usize)
        .copied()
        .unwrap_or(MediaKind::Other);
    let dir_id = store.file_dir.get(fid as usize).copied().unwrap_or(0);
    let fill = if is_sel { BG_SELECTED } else { BG_SURFACE };
    let stroke = Stroke::new(1.0_f32, if is_sel { ACCENT } else { BORDER_SUBTLE });
    let resp = Frame::none()
        .fill(fill)
        .stroke(stroke)
        .rounding(Rounding::same(4.0))
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.set_min_size(Vec2::new(CARD_WIDTH - 16.0, CARD_HEIGHT - 16.0));
            ui.set_max_size(Vec2::new(CARD_WIDTH - 16.0, CARD_HEIGHT - 16.0));
            ui.vertical(|ui| {
                ui.label(RichText::new(kind.badge()).small().color(badge_color(kind)));
                ui.add(egui::Label::new(store.file_name(fid)).truncate());
                ui.add(
                    egui::Label::new(
                        RichText::new(store.dir_name(dir_id))
                            .small()
                            .color(TEXT_MUTED),
                    )
                    .truncate(),
                );
            });
        })
        .response
        .interact(Sense::click());
    handle_file_interaction(&resp, tr, store, fid, selected_file);
}
