use std::collections::HashMap;
use std::mem::size_of;
use std::path::PathBuf;

use crate::error::AppError;
use crate::index::arena::{fold_into, ByteArena};
use crate::index::{MediaKind, Span32};

pub struct FileRecord<'a> {
    pub dir_id: u32,
    pub name: &'a str,
    pub ext: &'a str,
    pub size: u64,
    pub mtime: i64,
}

#[derive(Debug)]
pub struct ArchiveStore {
    root: PathBuf,
    raw_arena: ByteArena,
    fold_arena: ByteArena,
    ext_table: Vec<String>,
    ext_map: HashMap<String, u16>,
    pub(crate) dir_parent: Vec<u32>,
    dir_raw_name: Vec<Span32>,
    dir_fold_name: Vec<Span32>,
    pub(crate) dir_depth: Vec<u16>,
    pub(crate) file_dir: Vec<u32>,
    file_raw_name: Vec<Span32>,
    file_fold_name: Vec<Span32>,
    pub(crate) file_ext: Vec<u16>,
    pub(crate) file_size: Vec<u64>,
    file_mtime: Vec<i64>,
    pub(crate) file_kind: Vec<MediaKind>,
}

impl ArchiveStore {
    pub fn new(root: PathBuf) -> Self {
        let mut ext_map = HashMap::with_capacity(64);
        ext_map.insert(String::new(), 0);
        Self {
            root,
            raw_arena: ByteArena::with_capacity(1024 * 1024),
            fold_arena: ByteArena::with_capacity(1024 * 1024),
            ext_table: vec![String::new()],
            ext_map,
            dir_parent: Vec::with_capacity(4096),
            dir_raw_name: Vec::with_capacity(4096),
            dir_fold_name: Vec::with_capacity(4096),
            dir_depth: Vec::with_capacity(4096),
            file_dir: Vec::with_capacity(65_536),
            file_raw_name: Vec::with_capacity(65_536),
            file_fold_name: Vec::with_capacity(65_536),
            file_ext: Vec::with_capacity(65_536),
            file_size: Vec::with_capacity(65_536),
            file_mtime: Vec::with_capacity(65_536),
            file_kind: Vec::with_capacity(65_536),
        }
    }

    pub fn add_dir(
        &mut self,
        parent: u32,
        name: &str,
        depth: u16,
        fold_buf: &mut String,
    ) -> Result<u32, AppError> {
        let id = u32::try_from(self.dir_parent.len()).map_err(|_| AppError::ArenaOverflow)?;
        let raw_span = self.raw_arena.push_str(name)?;
        fold_into(name, fold_buf);
        let fold_span = self.fold_arena.push_str(fold_buf)?;
        self.dir_parent.push(parent);
        self.dir_raw_name.push(raw_span);
        self.dir_fold_name.push(fold_span);
        self.dir_depth.push(depth);
        Ok(id)
    }

    pub fn add_file(
        &mut self,
        rec: FileRecord<'_>,
        fold_buf: &mut String,
    ) -> Result<u32, AppError> {
        let id = u32::try_from(self.file_dir.len()).map_err(|_| AppError::ArenaOverflow)?;
        let raw_span = self.raw_arena.push_str(rec.name)?;
        fold_into(rec.name, fold_buf);
        let fold_span = self.fold_arena.push_str(fold_buf)?;
        let ext_id = self.intern_ext(rec.ext);
        self.file_dir.push(rec.dir_id);
        self.file_raw_name.push(raw_span);
        self.file_fold_name.push(fold_span);
        self.file_ext.push(ext_id);
        self.file_size.push(rec.size);
        self.file_mtime.push(rec.mtime);
        self.file_kind.push(MediaKind::from_ext(rec.ext));
        Ok(id)
    }

    fn intern_ext(&mut self, ext_lower: &str) -> u16 {
        if let Some(&id) = self.ext_map.get(ext_lower) {
            return id;
        }
        let Ok(id) = u16::try_from(self.ext_table.len()) else {
            return 0;
        };
        let owned = ext_lower.to_owned();
        self.ext_table.push(owned.clone());
        self.ext_map.insert(owned, id);
        id
    }

    pub fn root(&self) -> &std::path::Path {
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
            + size_of::<u16>()
            + size_of::<u64>()
            + size_of::<MediaKind>();
        self.raw_arena.len_bytes()
            + self.fold_arena.len_bytes()
            + self.dir_parent.len() * dir_row
            + self.file_dir.len() * file_row
            + self.file_mtime.len() * size_of::<i64>()
    }
}
