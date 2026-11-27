use egui::{ComboBox, RichText, ScrollArea, Ui};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::index::VisibleDir;
use crate::ui::os_open::handle_file_interaction;
use crate::ui::theme::TEXT_MUTED;

const ROW_H: f32 = 24.0;

#[derive(Default)]
pub struct DualPaneState {
    pub dir_a: Option<u32>,
    pub dir_b: Option<u32>,
}

#[derive(Default)]
pub struct DualPaneAction {
    pub open_merge: bool,
    pub find_similar: Option<u32>,
}

pub fn show_dual_pane(
    ui: &mut Ui,
    tr: &Strings,
    store: Option<&ArchiveStore>,
    visible_dirs: &[VisibleDir],
    state: &mut DualPaneState,
    selected_file: &mut Option<u32>,
) -> DualPaneAction {
    let mut act = DualPaneAction::default();
    let Some(st) = store else {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new(tr.empty_subtitle).color(TEXT_MUTED));
        });
        return act;
    };

    ui.horizontal(|ui| {
        let can_merge = matches!((state.dir_a, state.dir_b), (Some(a), Some(b)) if a != b);
        if ui
            .add_enabled(can_merge, egui::Button::new(tr.dual_merge_btn))
            .clicked()
        {
            act.open_merge = true;
        }
    });
    ui.separator();

    let half_w = ((ui.available_width() - 12.0) * 0.5).max(180.0);
    ui.columns(2, |cols| {
        let a = render_pane(
            &mut cols[0],
            ("pane_a", tr.dual_pane_a, half_w),
            tr,
            st,
            visible_dirs,
            &mut state.dir_a,
            selected_file,
        );
        let b = render_pane(
            &mut cols[1],
            ("pane_b", tr.dual_pane_b, half_w),
            tr,
            st,
            visible_dirs,
            &mut state.dir_b,
            selected_file,
        );
        act.find_similar = a.or(b);
    });
    act
}

fn render_pane(
    ui: &mut Ui,
    cfg: (&str, &str, f32),
    tr: &Strings,
    store: &ArchiveStore,
    dirs: &[VisibleDir],
    selected_dir: &mut Option<u32>,
    selected_file: &mut Option<u32>,
) -> Option<u32> {
    let (id, title, width) = cfg;
    ui.set_min_width(width);
    let label = selected_dir.map_or(title, |d| store.dir_name(d));
    ui.horizontal(|ui| {
        ui.label(RichText::new(title).strong());
        ComboBox::from_id_salt(id)
            .selected_text(label)
            .width((width - 90.0).max(120.0))
            .show_ui(ui, |ui| {
                for row in dirs.iter().take(500) {
                    ui.selectable_value(
                        selected_dir,
                        Some(row.dir_id),
                        store.dir_name(row.dir_id),
                    );
                }
            });
    });
    ui.separator();

    let dir_id = (*selected_dir)?;
    let files: Vec<u32> = store
        .file_dir
        .iter()
        .enumerate()
        .filter_map(|(i, &d)| (d == dir_id).then_some(i as u32))
        .collect();

    let mut find_sim = None;
    ScrollArea::vertical()
        .id_salt(format!("{id}_scroll"))
        .auto_shrink([false, false])
        .show_rows(ui, ROW_H, files.len(), |ui, range| {
            for &fid in files.get(range).unwrap_or(&[]) {
                let resp =
                    ui.selectable_label(*selected_file == Some(fid), store.file_name(fid));
                if let Some(sim) =
                    handle_file_interaction(&resp, tr, store, fid, selected_file)
                {
                    find_sim = Some(sim);
                }
            }
        });
    find_sim
}
