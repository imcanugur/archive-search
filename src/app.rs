use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crossbeam_channel::{unbounded, Receiver, Sender};
use egui::{Key, Modifiers};

use crate::i18n::{strings, Locale};
use crate::index::store::ArchiveStore;
use crate::index::worker::spawn_worker;
use crate::messages::{SearchSnapshot, UiEvent, WorkerCmd};
use crate::ops::journal::UndoJournal;
use crate::telemetry::{spawn_telemetry, ResourceStats};
use crate::ui::command_palette::{CommandPalette, PaletteCmd};
use crate::ui::folder_modal::FolderModal;
use crate::ui::grid_panel::{show_grid_panel, ViewMode};
use crate::ui::merge_modal::MergeModal;
use crate::ui::preview_modal::PreviewModal;
use crate::ui::status_bar::{show_status_bar, StatusInfo};
use crate::ui::theme::{apply_theme, BG_CANVAS};
use crate::ui::thumbs::ThumbCache;
use crate::ui::top_bar::show_top_bar;
use crate::ui::tree_panel::show_tree_panel;

pub struct ArchiveApp {
    locale: Locale,
    cmd_tx: Sender<WorkerCmd>,
    ev_rx: Receiver<UiEvent>,
    telemetry_rx: Receiver<ResourceStats>,
    resources: ResourceStats,
    scan_cancel: Arc<AtomicBool>,
    store: Option<Arc<ArchiveStore>>,
    snapshot: SearchSnapshot,
    folder_modal: FolderModal,
    preview_modal: PreviewModal,
    merge_modal: MergeModal,
    palette: CommandPalette,
    journal: UndoJournal,
    thumbs: ThumbCache,
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
        let telemetry_rx = spawn_telemetry(cc.egui_ctx.clone());
        let scan_cancel = Arc::new(AtomicBool::new(false));
        spawn_worker(cmd_rx, ev_tx, cc.egui_ctx.clone(), Arc::clone(&scan_cancel));
        if let Some(root) = initial_root {
            let _ = cmd_tx.send(WorkerCmd::ScanDir(root));
        }
        Self {
            locale: Locale::default(),
            cmd_tx,
            ev_rx,
            telemetry_rx,
            resources: ResourceStats::default(),
            scan_cancel,
            store: None,
            snapshot: SearchSnapshot::default(),
            folder_modal: FolderModal::default(),
            preview_modal: PreviewModal::new(cc.egui_ctx.clone()),
            merge_modal: MergeModal::default(),
            palette: CommandPalette::default(),
            journal: UndoJournal::default(),
            thumbs: ThumbCache::new(cc.egui_ctx.clone()),
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
        while let Ok(res) = self.telemetry_rx.try_recv() {
            self.resources = res;
        }
        while let Ok(ev) = self.ev_rx.try_recv() {
            match ev {
                UiEvent::ScanStarted => {
                    self.scanning = true;
                    self.scan_dirs = 0;
                    self.scan_files = 0;
                    self.error_msg = None;
                    self.selected_dir = None;
                    self.selected_file = None;
                    self.thumbs.clear();
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

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| {
            i.modifiers.matches_logically(Modifiers::CTRL)
                && (i.key_pressed(Key::K) || i.key_pressed(Key::P))
        }) {
            self.palette.toggle();
            return;
        }
        if ctx.wants_keyboard_input() {
            return;
        }
        if ctx.input(|i| i.key_pressed(Key::Space))
            && (self.selected_file.is_some() || self.preview_modal.is_open())
        {
            self.preview_modal.toggle();
        }
        if ctx
            .input(|i| i.modifiers.matches_logically(Modifiers::CTRL) && i.key_pressed(Key::Z))
        {
            self.trigger_undo();
        }
    }

    fn apply_palette_cmd(&mut self, cmd: PaletteCmd, tr: &crate::i18n::Strings) {
        match cmd {
            PaletteCmd::OpenFolder => self.folder_modal.open_at(None, tr),
            PaletteCmd::RescanFolder => self.rescan_current(),
            PaletteCmd::OpenMerge => self.merge_modal.open_picker(self.selected_dir),
            PaletteCmd::UndoMerge => self.trigger_undo(),
            PaletteCmd::ViewList => self.view_mode = ViewMode::List,
            PaletteCmd::ViewGrid => self.view_mode = ViewMode::Grid,
            PaletteCmd::SmartImages => {
                self.query = "ext:jpg".to_owned();
                self.dispatch_query();
            }
            PaletteCmd::SmartVideos => {
                self.query = "ext:mp4".to_owned();
                self.dispatch_query();
            }
            PaletteCmd::SmartLarge => {
                self.query = "size:>50mb".to_owned();
                self.dispatch_query();
            }
            PaletteCmd::ClearFilters => {
                self.query.clear();
                self.selected_dir = None;
                self.dispatch_query();
            }
        }
    }

    fn trigger_undo(&mut self) {
        match self.journal.undo_last() {
            Ok(Some(_)) => self.rescan_current(),
            Ok(None) => {}
            Err(err) => self.error_msg = Some(err.to_string()),
        }
    }

    fn rescan_current(&self) {
        if let Some(ref st) = self.store {
            let _ = self
                .cmd_tx
                .send(WorkerCmd::RescanDir(st.root().to_path_buf()));
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

    fn render_bars(&mut self, ctx: &egui::Context) {
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
                self.journal.can_undo(),
            );
            if act.pick_folder {
                self.folder_modal.open_at(None, tr);
            }
            if act.open_merge {
                self.merge_modal.open_picker(self.selected_dir);
            }
            if act.trigger_undo {
                self.trigger_undo();
            }
            if act.go_up {
                if let (Some(st), Some(d)) = (self.store.as_deref(), self.selected_dir) {
                    let p = st.dir_parent.get(d as usize).copied().unwrap_or(0);
                    self.selected_dir = (p != 0).then_some(p);
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
                resources: self.resources,
                thumb_vram_bytes: self.thumbs.vram_bytes(),
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
        self.thumbs.poll_uploads(ctx);
        self.handle_shortcuts(ctx);
        self.render_bars(ctx);

        egui::SidePanel::left("tree_panel")
            .default_width(260.0)
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
                if show_grid_panel(
                    ui,
                    tr,
                    self.store.as_deref(),
                    &self.snapshot.matched_files,
                    &mut self.selected_file,
                    self.view_mode,
                    &mut self.thumbs,
                ) {
                    self.folder_modal.open_at(None, tr);
                }
            });
        if let Some(cmd) = self.palette.show(ctx, tr) {
            self.apply_palette_cmd(cmd, tr);
        }
        if let Some(dir) = self.folder_modal.show(ctx, tr) {
            let _ = self.cmd_tx.send(WorkerCmd::ScanDir(dir));
        }
        if let Some(ref st) = self.store {
            self.preview_modal.show(
                ctx,
                tr,
                st,
                &self.snapshot.matched_files,
                &mut self.selected_file,
            );
            if let Some(entry) =
                self.merge_modal
                    .show(ctx, tr, st, &self.snapshot.visible_dirs)
            {
                self.journal.push(entry);
                self.rescan_current();
            }
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = self.cmd_tx.send(WorkerCmd::Shutdown);
    }
}
