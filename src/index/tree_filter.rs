use memchr::memmem::Finder;

use crate::index::bitset::Bitset;
use crate::index::query::ParsedQuery;
use crate::index::store::ArchiveStore;
use crate::index::VisibleDir;

pub fn build_dir_allowed(store: &ArchiveStore, query: &ParsedQuery) -> Bitset {
    let count = store.dir_count();
    let mut allowed = Bitset::new_empty(count);
    for d in 0..count {
        let ok = match query.selected_dir {
            None => true,
            Some(sel) => {
                let parent = store.dir_parent.get(d).copied().unwrap_or(0) as usize;
                (d as u32) == sel || (d > 0 && allowed.get(parent))
            }
        };
        if ok {
            allowed.set(d);
        }
    }
    for dir_term in &query.dir_terms {
        let finder = Finder::new(dir_term.as_bytes());
        allowed.and_with(&propagate_dir_finder(store, &finder));
    }
    allowed
}

pub fn propagate_dir_finder(store: &ArchiveStore, finder: &Finder<'_>) -> Bitset {
    let count = store.dir_count();
    let mut mask = Bitset::new_empty(count);
    for d in 0..count {
        let parent = store.dir_parent.get(d).copied().unwrap_or(0) as usize;
        if (d > 0 && mask.get(parent)) || finder.find(store.dir_folded(d as u32)).is_some() {
            mask.set(d);
        }
    }
    mask
}

pub fn collapse_dir_tree(
    store: &ArchiveStore,
    matched: &[u32],
    selected_dir: Option<u32>,
) -> Vec<VisibleDir> {
    let dir_count = store.dir_count();
    let mut counts = vec![0u32; dir_count];
    for &fid in matched {
        let d = store.file_dir.get(fid as usize).copied().unwrap_or(0) as usize;
        if let Some(c) = counts.get_mut(d) {
            *c = c.saturating_add(1);
        }
    }
    for d in (1..dir_count).rev() {
        let sub = counts.get(d).copied().unwrap_or(0);
        let parent = store.dir_parent.get(d).copied().unwrap_or(0) as usize;
        if let Some(pc) = counts.get_mut(parent) {
            *pc = pc.saturating_add(sub);
        }
    }
    counts
        .into_iter()
        .enumerate()
        .filter(|&(d, cnt)| cnt > 0 || d == 0 || selected_dir == Some(d as u32))
        .map(|(d, match_count)| VisibleDir {
            dir_id: d as u32,
            depth: store.dir_depth.get(d).copied().unwrap_or(0),
            match_count,
        })
        .collect()
}
