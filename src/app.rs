use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crossbeam_channel::{unbounded, Receiver, Sender};

use crate::i18n::{strings, Locale};
use crate::index::store::ArchiveStore;
use crate::index::worker::spawn_worker;
use crate::messages::{SearchSnapshot, UiEvent, WorkerCmd};
use crate::ui::folder_modal::FolderModal;
use crate::ui::grid_panel::{show_grid_panel, ViewMode};
use crate::ui::status_bar::{show_status_bar, StatusInfo};
use crate::ui::theme::{apply_theme, BG_CANVAS};
use crate::ui::top_bar::show_top_bar;
use crate::ui::tree_panel::show_tree_panel;

pub struct ArchiveApp {
    locale: Locale,
    cmd_tx: Sender<WorkerCmd>,
    ev_rx: Receiver<UiEvent>,
    scan_cancel: Arc<AtomicBool>,
    store: Option<Arc<ArchiveStore>>,
    snapshot: SearchSnapshot,
    folder_modal: FolderModal,
    view_mode: ViewMode,
    query: String,
    gen_id: u64,
    selected_dir: Option<u32>,
    selected_file: Option<u32>,
    scanning: bool,
    scan_dirs: u32,
    scan_files: u32,
    last_scan_ms: Option<u64>,
    error_msg: Option<String>,
}

impl ArchiveApp {
    pub fn new(cc: &eframe::CreationContext<'_>, initial_root: Option<PathBuf>) -> Self {
        apply_theme(&cc.egui_ctx);
        let (cmd_tx, cmd_rx) = unbounded();
        let (ev_tx, ev_rx) = unbounded();
        let scan_cancel = Arc::new(AtomicBool::new(false));
        spawn_worker(cmd_rx, ev_tx, cc.egui_ctx.clone(), Arc::clone(&scan_cancel));
        if let Some(root) = initial_root {
            let _ = cmd_tx.send(WorkerCmd::ScanDir(root));
        }
        Self {
            locale: Locale::default(),
            cmd_tx,
            ev_rx,
            scan_cancel,
            store: None,
            snapshot: SearchSnapshot::default(),
            folder_modal: FolderModal::default(),
            view_mode: ViewMode::default(),
            query: String::new(),
            gen_id: 0,
            selected_dir: None,
            selected_file: None,
            scanning: false,
            scan_dirs: 0,
            scan_files: 0,
            last_scan_ms: None,
            error_msg: None,
        }
    }

    fn drain_events(&mut self) {
        while let Ok(ev) = self.ev_rx.try_recv() {
            match ev {
                UiEvent::ScanStarted => {
                    self.scanning = true;
                    self.scan_dirs = 0;
                    self.scan_files = 0;
                    self.error_msg = None;
                    self.selected_dir = None;
                    self.selected_file = None;
                }
                UiEvent::ScanProgress {
                    dirs_found,
                    files_found,
                } => {
                    self.scan_dirs = dirs_found;
                    self.scan_files = files_found;
                }
                UiEvent::ScanComplete { store, elapsed_ms } => {
                    self.store = Some(store);
                    self.scanning = false;
                    self.last_scan_ms = Some(elapsed_ms);
                }
                UiEvent::ScanError(msg) => {
                    self.scanning = false;
                    self.error_msg = Some(msg);
                }
                UiEvent::SearchResult(snap) => {
                    if snap.gen_id >= self.snapshot.gen_id || snap.gen_id == 0 {
                        self.snapshot = snap;
                    }
                }
            }
        }
    }

    fn dispatch_query(&mut self) {
        self.gen_id = self.gen_id.saturating_add(1);
        let _ = self.cmd_tx.send(WorkerCmd::Query {
            gen_id: self.gen_id,
            raw: self.query.clone(),
            selected_dir: self.selected_dir,
        });
    }

    fn render_top_and_bottom(&mut self, ctx: &egui::Context) {
        let tr = strings(self.locale);
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            let act = show_top_bar(
                ui,
                tr,
                &mut self.query,
                self.store.as_deref(),
                self.selected_dir,
                &mut self.view_mode,
                self.scanning,
            );
            if act.pick_folder {
                self.folder_modal.open_at(None, tr);
            }
            if act.go_up {
                if let (Some(st), Some(dir_id)) = (self.store.as_deref(), self.selected_dir) {
                    let parent = st.dir_parent.get(dir_id as usize).copied().unwrap_or(0);
                    self.selected_dir = if parent == 0 { None } else { Some(parent) };
                    self.dispatch_query();
                }
            } else if act.clear_selected_dir {
                self.selected_dir = None;
                self.dispatch_query();
            } else if act.query_changed {
                self.dispatch_query();
            }
        });
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            let info = StatusInfo {
                store: self.store.as_deref(),
                snapshot: &self.snapshot,
                selected_file: self.selected_file,
                scanning: self.scanning,
                scan_dirs: self.scan_dirs,
                scan_files: self.scan_files,
                last_scan_ms: self.last_scan_ms,
                error_msg: self.error_msg.as_deref(),
            };
            if show_status_bar(ui, tr, &info) {
                self.scan_cancel.store(true, Ordering::Relaxed);
            }
        });
    }
}

impl eframe::App for ArchiveApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let tr = strings(self.locale);
        self.drain_events();
        self.render_top_and_bottom(ctx);
        egui::SidePanel::left("tree_panel")
            .default_width(270.0)
            .resizable(true)
            .show(ctx, |ui| {
                if let Some(next_dir) = show_tree_panel(
                    ui,
                    tr,
                    self.store.as_deref(),
                    &self.snapshot.visible_dirs,
                    self.selected_dir,
                ) {
                    self.selected_dir = next_dir;
                    self.dispatch_query();
                }
            });
        egui::CentralPanel::default()
            .frame(
                egui::Frame::central_panel(&ctx.style())
                    .fill(BG_CANVAS)
                    .inner_margin(8.0),
            )
            .show(ctx, |ui| {
                let pick = show_grid_panel(
                    ui,
                    tr,
                    self.store.as_deref(),
                    &self.snapshot.matched_files,
                    &mut self.selected_file,
                    self.view_mode,
                );
                if pick {
                    self.folder_modal.open_at(None, tr);
                }
            });
        if let Some(dir) = self.folder_modal.show(ctx, tr) {
            let _ = self.cmd_tx.send(WorkerCmd::ScanDir(dir));
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = self.cmd_tx.send(WorkerCmd::Shutdown);
    }
}
