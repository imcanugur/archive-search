use std::collections::HashMap;

use half::f16;
use rayon::prelude::*;

use crate::index::bitset::Bitset;

pub const EMBED_DIM: usize = 64;

#[derive(Debug, Default)]
pub struct VectorIndex {
    dim: usize,
    file_ids: Vec<u32>,
    id_to_row: HashMap<u32, usize>,
    data: Vec<f16>,
}

impl VectorIndex {
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            file_ids: Vec::new(),
            id_to_row: HashMap::new(),
            data: Vec::new(),
        }
    }

    pub fn push(&mut self, file_id: u32, vec_f32: &[f32]) {
        if vec_f32.len() != self.dim {
            return;
        }
        let row = self.file_ids.len();
        self.file_ids.push(file_id);
        self.id_to_row.insert(file_id, row);
        let norm = l2_norm(vec_f32);
        let inv = if norm > 1e-6 { 1.0 / norm } else { 0.0 };
        for &v in vec_f32 {
            self.data.push(f16::from_f32(v * inv));
        }
    }

    pub fn len(&self) -> usize {
        self.file_ids.len()
    }

    pub fn memory_bytes(&self) -> usize {
        self.data.len() * std::mem::size_of::<f16>() + self.file_ids.len() * 4
    }

    pub fn file_id_at(&self, row: usize) -> Option<u32> {
        self.file_ids.get(row).copied()
    }

    pub fn row_slice(&self, row: usize) -> Option<&[f16]> {
        let start = row.checked_mul(self.dim)?;
        let end = start.checked_add(self.dim)?;
        self.data.get(start..end)
    }

    pub fn vector_for_file(&self, file_id: u32) -> Option<&[f16]> {
        let &row = self.id_to_row.get(&file_id)?;
        self.row_slice(row)
    }

    pub fn search_top_k(
        &self,
        query: &[f16],
        top_k: usize,
        min_score: f32,
    ) -> Vec<(u32, f32)> {
        if query.len() != self.dim || self.file_ids.is_empty() {
            return Vec::new();
        }
        let mut scored: Vec<(u32, f32)> = (0..self.file_ids.len())
            .into_par_iter()
            .filter_map(|row| {
                let slice = self.row_slice(row)?;
                let score = dot_f16(query, slice);
                (score >= min_score)
                    .then(|| (self.file_ids.get(row).copied().unwrap_or(0), score))
            })
            .collect();
        scored.sort_unstable_by(|a, b| b.1.total_cmp(&a.1));
        scored.truncate(top_k);
        scored
    }

    pub fn match_bitset(&self, query: &[f16], min_score: f32, total_files: usize) -> Bitset {
        let mut bits = Bitset::new_empty(total_files);
        for (fid, _) in self.search_top_k(query, self.len(), min_score) {
            bits.set(fid as usize);
        }
        bits
    }
}

pub fn dot_f16(a: &[f16], b: &[f16]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| x.to_f32() * y.to_f32())
        .sum()
}

fn l2_norm(v: &[f32]) -> f32 {
    v.iter().map(|&x| x * x).sum::<f32>().sqrt()
}
