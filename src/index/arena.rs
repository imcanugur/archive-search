use crate::error::AppError;
use crate::index::Span32;

#[derive(Debug, Default)]
pub struct ByteArena {
    bytes: Vec<u8>,
}

impl ByteArena {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(cap),
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn push_str(&mut self, text: &str) -> Result<Span32, AppError> {
        let offset = u32::try_from(self.bytes.len()).map_err(|_| AppError::ArenaOverflow)?;
        let len = u32::try_from(text.len()).map_err(|_| AppError::ArenaOverflow)?;
        let _end = offset.checked_add(len).ok_or(AppError::ArenaOverflow)?;
        self.bytes.extend_from_slice(text.as_bytes());
        Ok(Span32 { offset, len })
    }

    pub fn get_bytes(&self, span: Span32) -> &[u8] {
        let start = span.offset as usize;
        let end = start.saturating_add(span.len as usize);
        self.bytes.get(start..end).unwrap_or(&[])
    }

    pub fn get_str(&self, span: Span32) -> &str {
        std::str::from_utf8(self.get_bytes(span)).unwrap_or("")
    }

    pub fn len_bytes(&self) -> usize {
        self.bytes.len()
    }
}

pub fn fold_into(input: &str, out: &mut String) {
    out.clear();
    out.reserve(input.len());
    for ch in input.chars() {
        match ch {
            'A'..='Z' => out.push(ch.to_ascii_lowercase()),
            'İ' | 'ı' => out.push('i'),
            'Ş' | 'ş' => out.push('s'),
            'Ğ' | 'ğ' => out.push('g'),
            'Ü' | 'ü' => out.push('u'),
            'Ö' | 'ö' => out.push('o'),
            'Ç' | 'ç' => out.push('c'),
            _ if ch.is_ascii() => out.push(ch),
            _ => {
                for lower in ch.to_lowercase() {
                    out.push(lower);
                }
            }
        }
    }
}

pub fn fold_string(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    fold_into(input, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::{fold_string, ByteArena};

    #[test]
    fn folds_turkish_and_ascii_text() {
        assert_eq!(
            fold_string("İSTANBUL_Fotoğraf_ŞĞÜÖÇ"),
            "istanbul_fotograf_sguoc"
        );
        assert_eq!(fold_string("TATİL_2024.MP4"), "tatil_2024.mp4");
    }

    #[test]
    fn arena_stores_and_reads_slices() {
        let mut arena = ByteArena::with_capacity(64);
        let span = arena.push_str("antalya.jpg");
        assert!(span.is_ok());
        if let Ok(s) = span {
            assert_eq!(arena.get_str(s), "antalya.jpg");
        }
    }
}
