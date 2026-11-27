use std::sync::Arc;
use std::time::Instant;

use memchr::memmem::Finder;
use rayon::prelude::*;

use crate::index::bitset::Bitset;
use crate::index::query::ParsedQuery;
use crate::index::store::ArchiveStore;
use crate::index::tree_filter::{build_dir_allowed, collapse_dir_tree, propagate_dir_finder};
use crate::messages::SearchSnapshot;

const CHUNK_FILES: usize = 4096;
const CHUNK_WORDS: usize = CHUNK_FILES / 64;

struct SearchCtx<'a> {
    store: &'a ArchiveStore,
    query: &'a ParsedQuery,
    dir_allowed: &'a Bitset,
    finders: &'a [Finder<'a>],
    term_in_dir: &'a [Bitset],
    ext_ids: &'a [u16],
}

pub fn execute_search(
    store: &ArchiveStore,
    query: &ParsedQuery,
    gen_id: u64,
) -> SearchSnapshot {
    let start = Instant::now();
    let dir_allowed = build_dir_allowed(store, query);
    let finders: Vec<Finder<'_>> = query
        .terms
        .iter()
        .map(|t| Finder::new(t.as_bytes()))
        .collect();
    let term_in_dir: Vec<Bitset> = finders
        .iter()
        .map(|f| propagate_dir_finder(store, f))
        .collect();
    let ext_ids: Vec<u16> = query
        .ext_filters
        .iter()
        .filter_map(|ext| store.find_ext_id(ext))
        .collect();

    let matched_files = if !query.ext_filters.is_empty() && ext_ids.is_empty() {
        Vec::new()
    } else {
        let ctx = SearchCtx {
            store,
            query,
            dir_allowed: &dir_allowed,
            finders: &finders,
            term_in_dir: &term_in_dir,
            ext_ids: &ext_ids,
        };
        filter_files_parallel(&ctx)
    };

    let visible_dirs = collapse_dir_tree(store, &matched_files, query.selected_dir);
    let elapsed_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
    SearchSnapshot {
        gen_id,
        matched_files: Arc::from(matched_files),
        visible_dirs: Arc::from(visible_dirs),
        elapsed_us,
    }
}

fn filter_files_parallel(ctx: &SearchCtx<'_>) -> Vec<u32> {
    let total = ctx.store.file_count();
    let words: Vec<u64> = (0..total.div_ceil(CHUNK_FILES))
        .into_par_iter()
        .flat_map(|chunk_idx| scan_chunk(ctx, chunk_idx, total))
        .collect();
    Bitset::from_words(words, total).collect_indices()
}

fn scan_chunk(ctx: &SearchCtx<'_>, chunk_idx: usize, total: usize) -> Vec<u64> {
    let start = chunk_idx * CHUNK_FILES;
    let end = (start + CHUNK_FILES).min(total);
    let mut local = [0u64; CHUNK_WORDS];
    for idx in start..end {
        if file_matches(ctx, idx) {
            let rel = idx - start;
            if let Some(w) = local.get_mut(rel >> 6) {
                *w |= 1u64 << (rel & 63);
            }
        }
    }
    local
        .get(..(end - start).div_ceil(64))
        .unwrap_or(&[])
        .to_vec()
}

fn file_matches(ctx: &SearchCtx<'_>, idx: usize) -> bool {
    let dir_id = ctx.store.file_dir.get(idx).copied().unwrap_or(0) as usize;
    if !ctx.dir_allowed.get(dir_id) || !meta_matches(ctx, idx) {
        return false;
    }
    let fname = ctx.store.file_folded(idx as u32);
    ctx.finders.iter().enumerate().all(|(t, f)| {
        ctx.term_in_dir.get(t).is_some_and(|m| m.get(dir_id)) || f.find(fname).is_some()
    })
}

fn meta_matches(ctx: &SearchCtx<'_>, idx: usize) -> bool {
    if !ctx.ext_ids.is_empty()
        && !ctx
            .ext_ids
            .contains(&ctx.store.file_ext.get(idx).copied().unwrap_or(0))
    {
        return false;
    }
    let size = ctx.store.file_size.get(idx).copied().unwrap_or(0);
    if ctx.query.min_size.is_some_and(|m| size < m)
        || ctx.query.max_size.is_some_and(|m| size > m)
    {
        return false;
    }
    if ctx
        .query
        .year_filter
        .is_some_and(|y| ctx.store.file_year.get(idx).copied().unwrap_or(0) != y)
    {
        return false;
    }
    if !ctx.query.cam_filters.is_empty() {
        let cam_id = ctx.store.file_cam.get(idx).copied().unwrap_or(0);
        let cam = ctx.store.cam_name(cam_id).to_ascii_lowercase();
        if !ctx.query.cam_filters.iter().any(|c| cam.contains(c)) {
            return false;
        }
    }
    true
}
