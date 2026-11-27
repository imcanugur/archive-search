use rayon::prelude::*;

use crate::gpu::vram::{dot_f16, VectorIndex};

pub fn find_similar_files(
    index: &VectorIndex,
    target_file_id: u32,
    top_k: usize,
    min_score: f32,
) -> Vec<u32> {
    let Some(query) = index.vector_for_file(target_file_id) else {
        return Vec::new();
    };
    index
        .search_top_k(query, top_k.saturating_add(1), min_score)
        .into_iter()
        .map(|(fid, _)| fid)
        .collect()
}

pub fn find_visual_duplicates(
    index: &VectorIndex,
    threshold: f32,
    max_files: usize,
) -> Vec<u32> {
    let n = index.len();
    if n < 2 {
        return Vec::new();
    }
    let dup_flags: Vec<bool> = (0..n)
        .into_par_iter()
        .map(|i| {
            let Some(vi) = index.row_slice(i) else {
                return false;
            };
            for j in 0..n {
                if i == j {
                    continue;
                }
                if index
                    .row_slice(j)
                    .is_some_and(|vj| dot_f16(vi, vj) >= threshold)
                {
                    return true;
                }
            }
            false
        })
        .collect();

    let mut out = Vec::new();
    for (row, is_dup) in dup_flags.into_iter().enumerate() {
        if !is_dup {
            continue;
        }
        if let Some(fid) = index.file_id_at(row) {
            out.push(fid);
            if out.len() >= max_files {
                break;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{find_similar_files, find_visual_duplicates};
    use crate::gpu::vram::VectorIndex;
    use crate::index::bitset::Bitset;

    #[test]
    fn similarity_and_duplicate_detection_work() {
        let mut idx = VectorIndex::new(4);
        idx.push(10, &[1.0, 0.0, 0.0, 0.0]);
        idx.push(20, &[0.999, 0.01, 0.0, 0.0]);
        idx.push(30, &[0.0, 1.0, 0.0, 0.0]);

        let sim = find_similar_files(&idx, 10, 5, 0.9);
        assert_eq!(sim, vec![10, 20]);

        let dups = find_visual_duplicates(&idx, 0.99, 10);
        assert_eq!(dups, vec![10, 20]);

        if let Some(q) = idx.vector_for_file(10) {
            let mut gpu_bits = idx.match_bitset(q, 0.9, 40);
            let mut cpu_bits = Bitset::new_empty(40);
            cpu_bits.set(20);
            gpu_bits.and_with(&cpu_bits);
            assert_eq!(gpu_bits.collect_indices(), vec![20]);
        }
    }
}
