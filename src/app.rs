mod actions;
mod events;
mod panels;
mod visual;
mod workspace;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crossbeam_channel::{unbounded, Receiver, Sender};

use crate::gpu::vram::VectorIndex;
use crate::gpu::{spawn_gpu_worker, GpuCmd, GpuEvent};
use crate::i18n::Locale;
use crate::index::store::ArchiveStore;
use crate::index::worker::spawn_worker;
use crate::messages::{SearchSnapshot, UiEvent, WorkerCmd};
use crate::ops::journal::UndoJournal;
use crate::telemetry::{spawn_telemetry, ResourceStats};
use crate::ui::command_palette::CommandPalette;
use crate::ui::dual_pane::DualPaneState;
use crate::ui::folder_modal::FolderModal;
use crate::ui::grid_panel::ViewMode;
use crate::ui::inspector_panel::InspectorPanel;
use crate::ui::merge_modal::MergeModal;
use crate::ui::preview_modal::PreviewModal;
use crate::ui::theme::apply_theme;
use crate::ui::thumbs::ThumbCache;

pub struct ArchiveApp {
    pub(super) locale: Locale,
    pub(super) cmd_tx: Sender<WorkerCmd>,
    pub(super) ev_rx: Receiver<UiEvent>,
    pub(super) gpu_tx: Sender<GpuCmd>,
    pub(super) gpu_rx: Receiver<GpuEvent>,
    pub(super) gpu_cancel: Arc<AtomicBool>,
    pub(super) telemetry_rx: Receiver<ResourceStats>,
    pub(super) resources: ResourceStats,
    pub(super) scan_cancel: Arc<AtomicBool>,
    pub(super) store: Option<Arc<ArchiveStore>>,
    pub(super) vector_index: Option<Arc<VectorIndex>>,
    pub(super) visual_progress: Option<(u32, u32)>,
    pub(super) visual_override: Option<Arc<[u32]>>,
    pub(super) pending_similar: Option<u32>,
    pub(super) snapshot: SearchSnapshot,
    pub(super) folder_modal: FolderModal,
    pub(super) preview_modal: PreviewModal,
    pub(super) merge_modal: MergeModal,
    pub(super) inspector: InspectorPanel,
    pub(super) show_inspector: bool,
    pub(super) palette: CommandPalette,
    pub(super) dual_pane: DualPaneState,
    pub(super) dual_mode: bool,
    pub(super) merge_queue: Vec<u32>,
    pub(super) journal: UndoJournal,
    pub(super) thumbs: ThumbCache,
    pub(super) view_mode: ViewMode,
    pub(super) query: String,
    pub(super) gen_id: u64,
    pub(super) selected_dir: Option<u32>,
    pub(super) selected_file: Option<u32>,
    pub(super) scanning: bool,
    pub(super) scan_dirs: u32,
    pub(super) scan_files: u32,
    pub(super) last_scan_ms: Option<u64>,
    pub(super) error_msg: Option<String>,
}

impl ArchiveApp {
    pub fn new(cc: &eframe::CreationContext<'_>, initial_root: Option<PathBuf>) -> Self {
        apply_theme(&cc.egui_ctx);
        let (cmd_tx, cmd_rx) = unbounded();
        let (ev_tx, ev_rx) = unbounded();
        let gpu_cancel = Arc::new(AtomicBool::new(false));
        let (gpu_tx, gpu_rx) = spawn_gpu_worker(cc.egui_ctx.clone(), Arc::clone(&gpu_cancel));
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
            gpu_tx,
            gpu_rx,
            gpu_cancel,
            telemetry_rx,
            resources: ResourceStats::default(),
            scan_cancel,
            store: None,
            vector_index: None,
            visual_progress: None,
            visual_override: None,
            pending_similar: None,
            snapshot: SearchSnapshot::default(),
            folder_modal: FolderModal::default(),
            preview_modal: PreviewModal::new(cc.egui_ctx.clone()),
            merge_modal: MergeModal::default(),
            inspector: InspectorPanel::new(cc.egui_ctx.clone()),
            show_inspector: true,
            palette: CommandPalette::default(),
            dual_pane: DualPaneState::default(),
            dual_mode: false,
            merge_queue: Vec::new(),
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
}

impl eframe::App for ArchiveApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_events();
        self.thumbs.poll_uploads(ctx);
        self.inspector.poll();
        self.handle_shortcuts(ctx);
        self.render_bars(ctx);

        let active_files: Arc<[u32]> = self
            .visual_override
            .as_ref()
            .map_or_else(|| Arc::clone(&self.snapshot.matched_files), Arc::clone);
        self.render_workspace(ctx, &active_files);
        self.render_modals(ctx, &active_files);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.gpu_cancel.store(true, Ordering::Relaxed);
        let _ = self.gpu_tx.send(GpuCmd::Shutdown);
        let _ = self.cmd_tx.send(WorkerCmd::Shutdown);
    }
}
