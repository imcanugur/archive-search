use std::path::Path;
use std::process::Command;
use std::thread;

use egui::Response;

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;

pub fn open_path(path: &Path) {
    let owned = path.to_path_buf();
    let _ = thread::Builder::new()
        .name("os-open".to_owned())
        .spawn(move || {
            if open_wsl_windows(&owned) {
                return;
            }
            let _ = Command::new("xdg-open").arg(&owned).spawn();
        });
}

pub fn handle_file_interaction(
    resp: &Response,
    tr: &Strings,
    store: &ArchiveStore,
    fid: u32,
    selected_file: &mut Option<u32>,
) {
    if resp.clicked() {
        *selected_file = Some(fid);
    }
    if resp.double_clicked() {
        open_path(&store.file_full_path(fid));
    }
    resp.context_menu(|ui| {
        if ui.button(tr.ctx_open).clicked() {
            open_path(&store.file_full_path(fid));
            ui.close_menu();
        }
        if ui.button(tr.ctx_open_folder).clicked() {
            let path = store.file_full_path(fid);
            if let Some(parent) = path.parent() {
                open_path(parent);
            }
            ui.close_menu();
        }
        if ui.button(tr.ctx_copy_path).clicked() {
            ui.ctx()
                .copy_text(store.file_full_path(fid).to_string_lossy().into_owned());
            ui.close_menu();
        }
    });
}

fn open_wsl_windows(path: &Path) -> bool {
    let Ok(out) = Command::new("wslpath").arg("-w").arg(path).output() else {
        return false;
    };
    if !out.status.success() {
        return false;
    }
    let win_path = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    if win_path.is_empty() {
        return false;
    }
    Command::new("explorer.exe").arg(win_path).spawn().is_ok()
}
