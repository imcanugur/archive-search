use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use exif::{In, Reader, Tag};
use ffmpeg_next as ff;

use crate::index::MediaKind;

#[derive(Debug, Clone, Default)]
pub struct MediaMeta {
    pub camera: String,
    pub year: u16,
    pub width: u32,
    pub height: u32,
    pub duration_ms: u32,
    pub codec: String,
}

impl MediaMeta {
    pub fn format_summary(&self) -> String {
        let mut parts = Vec::new();
        if !self.camera.is_empty() {
            parts.push(format!("Cam: {}", self.camera));
        }
        if self.year > 0 {
            parts.push(format!("Year: {}", self.year));
        }
        if self.width > 0 && self.height > 0 {
            parts.push(format!("{}x{}", self.width, self.height));
        }
        if self.duration_ms > 0 {
            let secs = self.duration_ms / 1000;
            parts.push(format!("{}:{:02}", secs / 60, secs % 60));
        }
        if !self.codec.is_empty() {
            parts.push(self.codec.clone());
        }
        parts.join("  |  ")
    }
}

pub fn read_media_meta(path: &Path, kind: MediaKind) -> MediaMeta {
    match kind {
        MediaKind::Image => read_image_exif(path).unwrap_or_default(),
        MediaKind::Video => read_video_header(path).unwrap_or_default(),
        MediaKind::Other => MediaMeta::default(),
    }
}

pub fn extract_year_from_name(name: &str) -> u16 {
    let bytes = name.as_bytes();
    if bytes.len() < 4 {
        return 0;
    }
    for i in 0..=bytes.len() - 4 {
        let Some(slice) = bytes.get(i..i + 4) else {
            continue;
        };
        let year = slice
            .iter()
            .all(u8::is_ascii_digit)
            .then(|| std::str::from_utf8(slice).ok())
            .flatten()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(0);
        if (1990..=2035).contains(&year) {
            return year;
        }
    }
    0
}

fn read_image_exif(path: &Path) -> Option<MediaMeta> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let exif = Reader::new().read_from_container(&mut reader).ok()?;

    let mut meta = MediaMeta::default();
    if let Some(f) = exif.get_field(Tag::Model, In::PRIMARY) {
        meta.camera = f
            .display_value()
            .to_string()
            .trim_matches('"')
            .trim()
            .to_owned();
    } else if let Some(f) = exif.get_field(Tag::Make, In::PRIMARY) {
        meta.camera = f
            .display_value()
            .to_string()
            .trim_matches('"')
            .trim()
            .to_owned();
    }
    if let Some(f) = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
        let s = f.display_value().to_string();
        meta.year = extract_year_from_name(&s);
    }
    if let Some(f) = exif.get_field(Tag::PixelXDimension, In::PRIMARY) {
        meta.width = f.value.get_uint(0).unwrap_or(0);
    }
    if let Some(f) = exif.get_field(Tag::PixelYDimension, In::PRIMARY) {
        meta.height = f.value.get_uint(0).unwrap_or(0);
    }
    Some(meta)
}

fn read_video_header(path: &Path) -> Option<MediaMeta> {
    let _ = ff::init();
    let mut opts = ff::Dictionary::new();
    opts.set("probesize", "65536");
    opts.set("analyzeduration", "100000");
    let ictx = ff::format::input_with_dictionary(path, opts).ok()?;
    let stream = ictx.streams().best(ff::media::Type::Video)?;
    let params = stream.parameters();
    let ctx = ff::codec::context::Context::from_parameters(params).ok()?;
    let dec = ctx.decoder().video().ok()?;

    let dur_us = ictx.duration().max(0);
    let duration_ms = u32::try_from(dur_us / 1000).unwrap_or(u32::MAX);
    Some(MediaMeta {
        camera: String::new(),
        year: 0,
        width: dec.width(),
        height: dec.height(),
        duration_ms,
        codec: format!("{:?}", dec.id()),
    })
}
