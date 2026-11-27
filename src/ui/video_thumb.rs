use std::path::Path;
use std::sync::Once;

use ff::format::Pixel;
use ff::software::scaling::{Context as Scaler, Flags};
use ffmpeg_next as ff;

static FFMPEG_INIT: Once = Once::new();
const MAX_PACKETS: usize = 64;

pub fn decode_video_rgba(path: &Path, max_dim: u32) -> Option<(usize, usize, Vec<u8>)> {
    FFMPEG_INIT.call_once(|| {
        let _ = ff::init();
        ff::log::set_level(ff::log::Level::Quiet);
    });

    let mut opts = ff::Dictionary::new();
    opts.set("probesize", "65536");
    opts.set("analyzeduration", "100000");
    let mut ictx = ff::format::input_with_dictionary(path, opts)
        .or_else(|_| ff::format::input(path))
        .ok()?;
    let stream = ictx.streams().best(ff::media::Type::Video)?;
    let video_idx = stream.index();
    let params = stream.parameters();

    let mut ctx = ff::codec::context::Context::from_parameters(params).ok()?;
    ctx.set_threading(ff::threading::Config {
        kind: ff::threading::Type::Slice,
        count: 1,
    });
    let mut decoder = ctx.decoder().video().ok()?;

    let frame = decode_first_frame(&mut ictx, video_idx, &mut decoder)?;
    scale_to_rgba(&frame, max_dim)
}

fn decode_first_frame(
    ictx: &mut ff::format::context::Input,
    video_idx: usize,
    decoder: &mut ff::decoder::Video,
) -> Option<ff::frame::Video> {
    let mut decoded = ff::frame::Video::empty();
    let mut seen = 0usize;

    for (stream, packet) in ictx.packets() {
        if stream.index() != video_idx {
            continue;
        }
        seen += 1;
        if decoder.send_packet(&packet).is_ok() && decoder.receive_frame(&mut decoded).is_ok()
        {
            return Some(decoded);
        }
        if seen >= MAX_PACKETS {
            break;
        }
    }
    let _ = decoder.send_eof();
    decoder.receive_frame(&mut decoded).ok().map(|()| decoded)
}

fn scale_to_rgba(frame: &ff::frame::Video, max_dim: u32) -> Option<(usize, usize, Vec<u8>)> {
    let src_w = frame.width();
    let src_h = frame.height();
    if src_w == 0 || src_h == 0 {
        return None;
    }
    let (dst_w, dst_h) = fit_dimensions(src_w, src_h, max_dim);
    let mut scaler = Scaler::get(
        frame.format(),
        src_w,
        src_h,
        Pixel::RGBA,
        dst_w,
        dst_h,
        Flags::FAST_BILINEAR,
    )
    .ok()?;

    let mut rgba = ff::frame::Video::empty();
    scaler.run(frame, &mut rgba).ok()?;
    let w = dst_w as usize;
    let h = dst_h as usize;
    let row_bytes = w * 4;
    let stride = rgba.stride(0);
    let data = rgba.data(0);

    let mut out = Vec::with_capacity(row_bytes * h);
    for y in 0..h {
        let start = y * stride;
        let end = start + row_bytes;
        out.extend_from_slice(data.get(start..end)?);
    }
    Some((w, h, out))
}

fn fit_dimensions(w: u32, h: u32, max_dim: u32) -> (u32, u32) {
    if w >= h {
        let nw = max_dim.min(w).max(1);
        let nh = ((u64::from(h) * u64::from(nw)) / u64::from(w)).max(1) as u32;
        (nw, nh)
    } else {
        let nh = max_dim.min(h).max(1);
        let nw = ((u64::from(w) * u64::from(nh)) / u64::from(h)).max(1) as u32;
        (nw, nh)
    }
}
