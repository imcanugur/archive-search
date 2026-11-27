pub mod arena;
pub mod bitset;
pub mod meta;
pub mod persistence;
pub mod query;
mod query_parse;
mod scan_helpers;
pub mod scanner;
pub mod search;
mod stat_backfill;
pub mod store;
mod store_read;
mod tree_filter;
pub mod worker;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct Span32 {
    pub offset: u32,
    pub len: u32,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub enum MediaKind {
    Image,
    Video,
    Other,
}

impl MediaKind {
    pub fn from_ext(ext: &str) -> Self {
        match ext {
            "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp" | "tif" | "tiff" | "heic"
            | "heif" | "avif" | "raw" | "cr2" | "cr3" | "nef" | "arw" | "dng" => Self::Image,
            "mp4" | "mkv" | "mov" | "avi" | "webm" | "m4v" | "wmv" | "flv" | "mts"
            | "m2ts" | "3gp" => Self::Video,
            _ => Self::Other,
        }
    }

    pub fn badge(self) -> &'static str {
        match self {
            Self::Image => "IMG",
            Self::Video => "VID",
            Self::Other => "FILE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibleDir {
    pub dir_id: u32,
    pub depth: u16,
    pub match_count: u32,
}
