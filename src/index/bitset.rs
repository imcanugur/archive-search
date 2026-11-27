#[derive(Debug, Clone)]
pub struct Bitset {
    words: Vec<u64>,
    len: usize,
}

impl Bitset {
    pub fn new_empty(len: usize) -> Self {
        let word_count = len.div_ceil(64);
        Self {
            words: vec![0u64; word_count],
            len,
        }
    }

    pub fn from_words(words: Vec<u64>, len: usize) -> Self {
        Self { words, len }
    }

    pub fn set(&mut self, idx: usize) {
        if idx >= self.len {
            return;
        }
        if let Some(word) = self.words.get_mut(idx >> 6) {
            *word |= 1u64 << (idx & 63);
        }
    }

    pub fn get(&self, idx: usize) -> bool {
        if idx >= self.len {
            return false;
        }
        self.words
            .get(idx >> 6)
            .is_some_and(|w| (w & (1u64 << (idx & 63))) != 0)
    }

    pub fn and_with(&mut self, other: &Self) {
        for (dst, src) in self.words.iter_mut().zip(other.words.iter()) {
            *dst &= *src;
        }
    }

    pub fn count_ones(&self) -> u32 {
        self.words.iter().map(|w| w.count_ones()).sum()
    }

    pub fn collect_indices(&self) -> Vec<u32> {
        let mut out = Vec::with_capacity(self.count_ones() as usize);
        for (word_idx, &word) in self.words.iter().enumerate() {
            let mut bits = word;
            let base = (word_idx as u32).saturating_mul(64);
            while bits != 0 {
                let tz = bits.trailing_zeros();
                let idx = base.saturating_add(tz);
                if (idx as usize) < self.len {
                    out.push(idx);
                }
                bits &= bits - 1;
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::Bitset;

    #[test]
    fn bitset_and_and_collect() {
        let mut a = Bitset::new_empty(130);
        let mut b = Bitset::new_empty(130);
        a.set(0);
        a.set(64);
        a.set(129);
        b.set(64);
        b.set(129);
        a.and_with(&b);
        assert!(a.get(64));
        assert!(!a.get(0));
        assert_eq!(a.collect_indices(), vec![64, 129]);
    }
}
