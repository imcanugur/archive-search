use egui::{Frame, RichText, Rounding, ScrollArea, Sense, Ui};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::index::MediaKind;
use crate::ui::os_open::handle_file_interaction;
use crate::ui::theme::{BG_SELECTED, TEXT_MUTED};
use crate::ui::{badge_color, format_bytes};

const ROW_HEIGHT: f32 = 25.0;

pub fn render_list_view(
    ui: &mut Ui,
    tr: &Strings,
    store: &ArchiveStore,
    matched_files: &[u32],
    selected_file: &mut Option<u32>,
) -> Option<u32> {
    let w = ui.available_width().max(400.0);
    let cols = [(w * 0.48).max(180.0), (w * 0.28).max(110.0), 64.0_f32];
    render_list_header(ui, tr, cols);

    let mut find_sim = None;
    ScrollArea::vertical()
        .id_salt("file_list_scroll")
        .auto_shrink([false, false])
        .show_rows(ui, ROW_HEIGHT, matched_files.len(), |ui, range| {
            for &fid in matched_files.get(range).unwrap_or(&[]) {
                if let Some(id) = render_list_row(ui, tr, store, fid, cols, selected_file) {
                    find_sim = Some(id);
                }
            }
        });
    find_sim
}

fn render_list_header(ui: &mut Ui, tr: &Strings, cols: [f32; 3]) {
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
}

fn render_list_row(
    ui: &mut Ui,
    tr: &Strings,
    store: &ArchiveStore,
    fid: u32,
    cols: [f32; 3],
    selected_file: &mut Option<u32>,
) -> Option<u32> {
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
    handle_file_interaction(&resp, tr, store, fid, selected_file)
}
