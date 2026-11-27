use std::mem::size_of;
use std::path::{Path, PathBuf};

use crate::index::store::ArchiveStore;
use crate::index::{MediaKind, Span32};

impl ArchiveStore {
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn dir_count(&self) -> usize {
        self.dir_parent.len()
    }

    pub fn file_count(&self) -> usize {
        self.file_dir.len()
    }

    pub fn find_ext_id(&self, ext_lower: &str) -> Option<u16> {
        self.ext_map.get(ext_lower).copied()
    }

    pub fn is_raw_ext_id(&self, id: u16) -> bool {
        let ext = self.ext_table.get(id as usize).map_or("", String::as_str);
        matches!(
            ext,
            "raw" | "cr2" | "cr3" | "nef" | "arw" | "dng" | "orf" | "rw2"
        )
    }

    pub fn sizes_populated(&self) -> bool {
        self.file_mtime.is_empty() || self.file_mtime.iter().all(|&m| m != 0)
    }

    pub fn cam_name(&self, id: u16) -> &str {
        self.cam_table.get(id as usize).map_or("", String::as_str)
    }

    pub fn dir_name(&self, id: u32) -> &str {
        self.dir_raw_name
            .get(id as usize)
            .map_or("", |&s| self.raw_arena.get_str(s))
    }

    pub fn dir_folded(&self, id: u32) -> &[u8] {
        self.dir_fold_name
            .get(id as usize)
            .map_or(&[], |&s| self.fold_arena.get_bytes(s))
    }

    pub fn file_name(&self, id: u32) -> &str {
        self.file_raw_name
            .get(id as usize)
            .map_or("", |&s| self.raw_arena.get_str(s))
    }

    pub fn file_folded(&self, id: u32) -> &[u8] {
        self.file_fold_name
            .get(id as usize)
            .map_or(&[], |&s| self.fold_arena.get_bytes(s))
    }

    pub fn dir_full_path(&self, id: u32) -> PathBuf {
        let mut parts = Vec::new();
        let mut cur = id;
        while cur != 0 {
            parts.push(self.dir_name(cur));
            cur = self.dir_parent.get(cur as usize).copied().unwrap_or(0);
        }
        let mut path = self.root.clone();
        parts.into_iter().rev().for_each(|p| path.push(p));
        path
    }

    pub fn file_full_path(&self, id: u32) -> PathBuf {
        let dir_id = self.file_dir.get(id as usize).copied().unwrap_or(0);
        let mut path = self.dir_full_path(dir_id);
        path.push(self.file_name(id));
        path
    }

    pub fn memory_bytes(&self) -> usize {
        let dir_row = size_of::<u32>() + size_of::<Span32>() * 2 + size_of::<u16>();
        let file_row = size_of::<u32>()
            + size_of::<Span32>() * 2
            + size_of::<u16>() * 3
            + size_of::<u64>()
            + size_of::<i64>()
            + size_of::<MediaKind>();
        self.raw_arena.len_bytes()
            + self.fold_arena.len_bytes()
            + self.dir_parent.len() * dir_row
            + self.file_dir.len() * file_row
    }
}
