#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteCmd {
    OpenFolder,
    RescanFolder,
    ToggleDualPane,
    OpenMerge,
    UndoMerge,
    IndexVisual,
    FindSimilar,
    FindDuplicates,
    ViewList,
    ViewGrid,
    SmartImages,
    SmartVideos,
    SmartLarge,
    ClearFilters,
}

pub struct PaletteEntry {
    pub label: &'static str,
    pub hint: &'static str,
    pub cmd: PaletteCmd,
}

pub const ENTRIES: &[PaletteEntry] = &[
    PaletteEntry {
        label: "Open Folder...",
        hint: "Browse",
        cmd: PaletteCmd::OpenFolder,
    },
    PaletteEntry {
        label: "Rescan Folder (Refresh Cache)",
        hint: "Disk",
        cmd: PaletteCmd::RescanFolder,
    },
    PaletteEntry {
        label: "Toggle Dual Pane Mode",
        hint: "F2",
        cmd: PaletteCmd::ToggleDualPane,
    },
    PaletteEntry {
        label: "Merge Folders (Dry-Run)...",
        hint: "M",
        cmd: PaletteCmd::OpenMerge,
    },
    PaletteEntry {
        label: "Undo Last Merge",
        hint: "Ctrl+Z",
        cmd: PaletteCmd::UndoMerge,
    },
    PaletteEntry {
        label: "Build Visual Vector Index (FP16)",
        hint: "Visual",
        cmd: PaletteCmd::IndexVisual,
    },
    PaletteEntry {
        label: "Find Similar to Selected Image",
        hint: "Visual",
        cmd: PaletteCmd::FindSimilar,
    },
    PaletteEntry {
        label: "Find Visual Duplicates (Dedup)",
        hint: "Visual",
        cmd: PaletteCmd::FindDuplicates,
    },
    PaletteEntry {
        label: "Switch to Details List View",
        hint: "View",
        cmd: PaletteCmd::ViewList,
    },
    PaletteEntry {
        label: "Switch to Thumbnail Grid View",
        hint: "View",
        cmd: PaletteCmd::ViewGrid,
    },
    PaletteEntry {
        label: "Smart View: Images (ext:jpg)",
        hint: "Filter",
        cmd: PaletteCmd::SmartImages,
    },
    PaletteEntry {
        label: "Smart View: Videos (ext:mp4)",
        hint: "Filter",
        cmd: PaletteCmd::SmartVideos,
    },
    PaletteEntry {
        label: "Smart View: Large Files (>50MB)",
        hint: "Filter",
        cmd: PaletteCmd::SmartLarge,
    },
    PaletteEntry {
        label: "Clear Search & Folder Filter",
        hint: "Reset",
        cmd: PaletteCmd::ClearFilters,
    },
];
