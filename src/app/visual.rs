use std::sync::Arc;

use crate::gpu::similarity::{find_similar_files, find_visual_duplicates};
use crate::gpu::GpuCmd;
use crate::index::bitset::Bitset;
use crate::ui::format_bytes;

use super::ArchiveApp;

impl ArchiveApp {
    pub(super) fn start_visual_index(&mut self) {
        if let Some(ref st) = self.store {
            self.visual_progress = Some((0, 0));
            let _ = self.gpu_tx.send(GpuCmd::IndexArchive(Arc::clone(st)));
        }
    }

    pub(super) fn run_find_similar(&mut self, fid: u32) {
        let Some(ref idx) = self.vector_index else {
            self.pending_similar = Some(fid);
            self.start_visual_index();
            return;
        };
        let Some(ref st) = self.store else {
            return;
        };
        let results = if let Some(qvec) = (!self.query.is_empty())
            .then(|| idx.vector_for_file(fid))
            .flatten()
        {
            let mut gpu_bits = idx.match_bitset(qvec, 0.90, st.file_count());
            let mut cpu_bits = Bitset::new_empty(st.file_count());
            for &id in self.snapshot.matched_files.iter() {
                cpu_bits.set(id as usize);
            }
            gpu_bits.and_with(&cpu_bits);
            gpu_bits.collect_indices()
        } else {
            find_similar_files(idx, fid, 120, 0.90)
        };
        self.visual_override = Some(Arc::from(results));
    }

    pub(super) fn run_find_duplicates(&mut self) {
        if let Some(ref idx) = self.vector_index {
            let dups = find_visual_duplicates(idx, 0.992, 500);
            self.visual_override = Some(Arc::from(dups));
        } else {
            self.start_visual_index();
        }
    }

    pub(super) fn visual_status_text(&self) -> Option<String> {
        if let Some((done, total)) = self.visual_progress {
            return Some(format!("Visual: {done}/{total}"));
        }
        self.vector_index.as_ref().map(|idx| {
            format!(
                "Visual: {} vecs ({})",
                idx.len(),
                format_bytes(idx.memory_bytes() as u64)
            )
        })
    }
}
