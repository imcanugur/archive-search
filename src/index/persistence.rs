use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use rkyv::{Archive, Deserialize, Serialize};

use crate::error::AppError;
use crate::index::arena::ByteArena;
use crate::index::store::ArchiveStore;
use crate::index::{MediaKind, Span32};

const CACHE_VERSION: u32 = 1;

#[derive(Archive, Serialize, Deserialize)]
struct CachedStore {
    version: u32,
    root: String,
    raw_bytes: Vec<u8>,
    fold_bytes: Vec<u8>,
    ext_table: Vec<String>,
    cam_table: Vec<String>,
    dir_parent: Vec<u32>,
    dir_raw_name: Vec<Span32>,
    dir_fold_name: Vec<Span32>,
    dir_depth: Vec<u16>,
    file_dir: Vec<u32>,
    file_raw_name: Vec<Span32>,
    file_fold_name: Vec<Span32>,
    file_ext: Vec<u16>,
    file_size: Vec<u64>,
    file_mtime: Vec<i64>,
    file_year: Vec<u16>,
    file_cam: Vec<u16>,
    file_kind: Vec<MediaKind>,
}

pub fn save_store(store: &ArchiveStore) -> Result<(), AppError> {
    let path = cache_file_path(store.root())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let cached = CachedStore {
        version: CACHE_VERSION,
        root: store.root.to_string_lossy().into_owned(),
        raw_bytes: store.raw_arena.as_bytes().to_vec(),
        fold_bytes: store.fold_arena.as_bytes().to_vec(),
        ext_table: store.ext_table.clone(),
        cam_table: store.cam_table.clone(),
        dir_parent: store.dir_parent.clone(),
        dir_raw_name: store.dir_raw_name.clone(),
        dir_fold_name: store.dir_fold_name.clone(),
        dir_depth: store.dir_depth.clone(),
        file_dir: store.file_dir.clone(),
        file_raw_name: store.file_raw_name.clone(),
        file_fold_name: store.file_fold_name.clone(),
        file_ext: store.file_ext.clone(),
        file_size: store.file_size.clone(),
        file_mtime: store.file_mtime.clone(),
        file_year: store.file_year.clone(),
        file_cam: store.file_cam.clone(),
        file_kind: store.file_kind.clone(),
    };
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&cached)
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, &bytes)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

pub fn load_store(root: &Path) -> Option<ArchiveStore> {
    let path = cache_file_path(root).ok()?;
    let bytes = fs::read(&path).ok()?;
    let cached = rkyv::from_bytes::<CachedStore, rkyv::rancor::Error>(&bytes).ok()?;
    if cached.version != CACHE_VERSION || Path::new(&cached.root) != root {
        return None;
    }
    let mut ext_map = HashMap::with_capacity(cached.ext_table.len());
    for (idx, ext) in cached.ext_table.iter().enumerate() {
        if let Ok(id) = u16::try_from(idx) {
            ext_map.insert(ext.clone(), id);
        }
    }
    Some(ArchiveStore {
        root: PathBuf::from(cached.root),
        raw_arena: ByteArena::from_bytes(cached.raw_bytes),
        fold_arena: ByteArena::from_bytes(cached.fold_bytes),
        ext_table: cached.ext_table,
        ext_map,
        cam_table: cached.cam_table,
        dir_parent: cached.dir_parent,
        dir_raw_name: cached.dir_raw_name,
        dir_fold_name: cached.dir_fold_name,
        dir_depth: cached.dir_depth,
        file_dir: cached.file_dir,
        file_raw_name: cached.file_raw_name,
        file_fold_name: cached.file_fold_name,
        file_ext: cached.file_ext,
        file_size: cached.file_size,
        file_mtime: cached.file_mtime,
        file_year: cached.file_year,
        file_cam: cached.file_cam,
        file_kind: cached.file_kind,
    })
}

fn cache_file_path(root: &Path) -> Result<PathBuf, AppError> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
        .unwrap_or_else(std::env::temp_dir);
    let hash = blake3::hash(root.to_string_lossy().as_bytes()).to_hex();
    Ok(base.join("archive-search").join(format!("{hash}.rkyv")))
}

#[cfg(test)]
mod tests {
    use super::{load_store, save_store};
    use crate::index::store::{ArchiveStore, FileRecord};

    #[test]
    fn rkyv_cache_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let root = std::env::temp_dir().join("archive_search_cache_unit_test");
        let mut store = ArchiveStore::new(root.clone());
        let mut buf = String::new();
        store.add_dir(0, "root", 0, &mut buf)?;
        let rec = FileRecord {
            dir_id: 0,
            name: "IMG_20240501.jpg",
            ext: "jpg",
            size: 42,
            mtime: 1,
        };
        store.add_file(rec, &mut buf)?;
        save_store(&store)?;
        let loaded = load_store(&root);
        assert!(loaded.is_some());
        if let Some(st) = loaded {
            assert_eq!(st.file_count(), 1);
            assert_eq!(st.file_name(0), "IMG_20240501.jpg");
            assert_eq!(st.file_year.first().copied(), Some(2024));
        }
        Ok(())
    }
}
