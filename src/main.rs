mod app;
mod error;
mod i18n;
mod index;
mod messages;
mod ui;

use std::path::PathBuf;

use anyhow::Result;

use crate::app::ArchiveApp;
use crate::i18n::{strings, Locale};

fn main() -> Result<()> {
    let initial_root = std::env::args_os().nth(1).map(PathBuf::from);
    let tr = strings(Locale::default());
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1360.0, 860.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title(tr.app_title),
        ..Default::default()
    };

    eframe::run_native(
        tr.app_title,
        options,
        Box::new(move |cc| Ok(Box::new(ArchiveApp::new(cc, initial_root)))),
    )
    .map_err(|err| anyhow::anyhow!("GUI startup error: {err}"))
}
