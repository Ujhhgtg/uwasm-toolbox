//! Telegram animated sticker (.tgs) conversion.
//!
//! TGS files are gzip-compressed Lottie JSON. The conversion pipeline:
//!
//! 1. Decompress gzip (or pass through plain JSON) → Lottie JSON string.
//! 2. Parse with `rasterlottie` (pure-Rust Lottie rasterizer, tiny-skia
//!    backend — no C dependencies, no system libs).
//! 3. Compute frame schedule: clamp target FPS ≤ source FPS, then
//!    uniformly decimate with `step = round(source_fps / target_fps)`.
//!    Optionally cap total frames via `max_frames` with another uniform
//!    subsampling pass.
//! 4. Render each frame to RGBA via rasterlottie's `prepared.render_frame`.
//! 5. Encode as animated GIF (via `gif` crate) or lossless animated WebP
//!    (VP8L via `image-webp`, with a custom RIFF/VP8X/ANIM/ANMF muxer
//!    since the `image` crate's encoder doesn't support animation).
//!
//! All operations are synchronous and in-memory.

use flate2::read::GzDecoder;
use std::io::Read;

use crate::{clog, cwarn};
use rasterlottie::{
    Animation, RenderConfig, Renderer, Rgba8, SupportProfile, analyze_animation_with_profile,
};

// ---------------------------------------------------------------------------
// TGS decompression
// ---------------------------------------------------------------------------

/// Decompress a .tgs file and return the Lottie JSON string.
/// Accepts gzip-compressed or plain UTF-8 JSON.
pub fn decompress(data: &[u8]) -> Result<String, String> {
    if data.starts_with(&[0x1F, 0x8B]) {
        let mut decoder = GzDecoder::new(data);
        let mut json = String::new();
        decoder
            .read_to_string(&mut json)
            .map_err(|e| format!("gzip decompress failed: {e}"))?;
        Ok(json)
    } else {
        std::str::from_utf8(data)
            .map(|s| s.to_owned())
            .map_err(|_| "not a valid gzip stream or UTF-8 JSON".to_string())
    }
}

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

pub struct ConvertOptions {
    pub fps: f32,
    pub width: u32,
    pub height: u32,
    /// 0 = unlimited
    pub max_frames: u32,
    /// inclusive start frame index (0 = animation default)
    pub frame_start: u32,
    /// exclusive end frame index (0 = animation default)
    pub frame_end: u32,
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

/// Convert a TGS/Lottie JSON string to an animated GIF or WebP.
///
/// Frame pipeline:
///   1. Parse Lottie JSON → `Animation` (rasterlottie).
///   2. Compute frame range from `opts.frame_start`/`frame_end`,
///      falling back to `anim.in_point`/`anim.out_point`.
///   3. Uniformly decimate source frames to `target_fps` using
///      `step = round(source_fps / target_fps)`.
///   4. If `max_frames` is set and the decimated count exceeds it,
///      subsample again with a uniform stride.
///   5. Render each frame to RGBA via `prepared.render_frame`.
///   6. Encode as GIF or WebP (animated).
pub fn convert(json: &str, opts: &ConvertOptions, format: &str) -> Result<Vec<u8>, String> {
    let anim = Animation::from_json_str(json).map_err(|e| format!("Lottie parse error: {e}"))?;

    let source_fps = anim.frame_rate.max(1.0);
    let anim_start = anim.in_point.floor();
    let anim_end = anim.out_point.ceil().max(anim_start + 1.0);

    crate::clog!(
        "tgs: animation {}x{}  {:.0}fps  frames {:.0}–{:.0}",
        anim.width,
        anim.height,
        source_fps,
        anim_start,
        anim_end
    );

    let range_start = if opts.frame_start > 0 {
        (opts.frame_start as f32).clamp(anim_start, anim_end)
    } else {
        anim_start
    };
    let range_end = if opts.frame_end > 0 {
        (opts.frame_end as f32).clamp(range_start, anim_end)
    } else {
        anim_end
    };

    let target_fps = opts.fps.clamp(1.0, source_fps);
    let step = (source_fps / target_fps).round().max(1.0);
    let actual_fps = source_fps / step;

    let mut frame_nums: Vec<f32> = {
        let mut v = Vec::new();
        let mut f = range_start;
        while f < range_end {
            v.push(f);
            f += step;
        }
        v
    };

    // Uniform subsampling for max_frames
    if opts.max_frames > 0 && frame_nums.len() > opts.max_frames as usize {
        let keep = opts.max_frames as usize;
        let stride = ((frame_nums.len() as f32) / (keep as f32)).ceil() as usize;
        frame_nums = frame_nums
            .into_iter()
            .step_by(stride.max(1))
            .take(keep)
            .collect();
    }

    if frame_nums.is_empty() {
        return Err("no frames to render (check frame range / max-frames)".to_string());
    }

    let anim_w = anim.width as f32;
    let anim_h = anim.height as f32;
    let scale = if anim_w > 0.0 && anim_h > 0.0 {
        (opts.width as f32 / anim_w)
            .min(opts.height as f32 / anim_h)
            .max(0.01)
    } else {
        1.0
    };

    clog!(
        "tgs: rendering {} frames at {:.1}fps  scale={:.3}  output={}x{}",
        frame_nums.len(),
        actual_fps,
        scale,
        opts.width,
        opts.height
    );

    let config = RenderConfig::new(Rgba8::TRANSPARENT, scale);

    // Warn when the animation uses features outside the strict target-corpus
    // profile (layer effects, expressions, unknown shape items like merge-paths
    // "mm").  We still attempt rendering with a lenient profile: the renderer
    // skips effects it cannot apply, falls back to static keyframe values for
    // unresolvable expressions, and ignores unknown shape items.
    let strict_report = analyze_animation_with_profile(&anim, SupportProfile::target_corpus());
    if !strict_report.is_supported() {
        cwarn!(
            "tgs: animation has {} unsupported feature(s) — rendering with best-effort approximation ({})",
            strict_report.len(),
            strict_report
        );
    }

    let lenient = SupportProfile {
        allow_effects: true,
        allow_expressions: true,
        allow_unknown_shape_items: true,
        ..SupportProfile::target_corpus()
    };

    let prepared = Renderer::new(lenient)
        .prepare(&anim)
        .map_err(|e| format!("prepare error: {e}"))?;

    let frames: Vec<(u32, u32, Vec<u8>)> = frame_nums
        .iter()
        .map(|&f| {
            let rf = prepared
                .render_frame(f, config)
                .map_err(|e| format!("render frame {f}: {e}"))?;
            Ok((rf.width, rf.height, rf.pixels))
        })
        .collect::<Result<Vec<_>, String>>()?;

    if frames.is_empty() {
        return Err("rendering produced no frames".to_string());
    }

    let (out_w, out_h) = (frames[0].0, frames[0].1);
    crate::clog!(
        "tgs: rendered {} frames at {}x{}",
        frames.len(),
        out_w,
        out_h
    );

    match format {
        "gif" => {
            let delay_cs = ((100.0 / actual_fps).round() as u16).max(1);
            encode_gif(&frames, out_w, out_h, delay_cs)
        }
        "webp" => {
            let delay_ms = ((1000.0 / actual_fps).round() as u32).max(1);
            encode_webp_anim(&frames, out_w, out_h, delay_ms)
        }
        other => Err(format!("unknown format: {other}")),
    }
}

// ---------------------------------------------------------------------------
// GIF encoder
// ---------------------------------------------------------------------------

/// Encode a sequence of RGBA frames as an animated GIF.
///
/// Dispose to background before each frame so transparent pixels
/// in frame N don't reveal frame N-1 content underneath.
fn encode_gif(
    frames: &[(u32, u32, Vec<u8>)],
    width: u32,
    height: u32,
    delay_cs: u16,
) -> Result<Vec<u8>, String> {
    let w = width as u16;
    let h = height as u16;
    let mut output = Vec::new();
    {
        let mut encoder =
            gif::Encoder::new(&mut output, w, h, &[]).map_err(|e| format!("gif init: {e}"))?;
        encoder
            .set_repeat(gif::Repeat::Infinite)
            .map_err(|e| format!("gif repeat: {e}"))?;

        for (_, _, pixels) in frames {
            let mut rgba = pixels.clone();
            let mut frame = gif::Frame::from_rgba_speed(w, h, &mut rgba, 10);
            frame.delay = delay_cs;
            // Dispose to background before each frame so transparent pixels
            // in frame N don't reveal frame N-1 content underneath.
            frame.dispose = gif::DisposalMethod::Background;
            encoder
                .write_frame(&frame)
                .map_err(|e| format!("gif write frame: {e}"))?;
        }
    }
    Ok(output)
}

// ---------------------------------------------------------------------------
// Animated WebP encoder (pure Rust, VP8L lossless)
// ---------------------------------------------------------------------------

/// Encode a sequence of RGBA frames as an animated lossless WebP.
///
/// Each frame is independently encoded as VP8L via `image::codecs::webp`,
/// then `mux_animated_webp` assembles the RIFF/VP8X/ANIM/ANMF container.
fn encode_webp_anim(
    frames: &[(u32, u32, Vec<u8>)],
    width: u32,
    height: u32,
    delay_ms: u32,
) -> Result<Vec<u8>, String> {
    // Encode every frame as a standalone lossless WebP
    let frame_webps: Vec<Vec<u8>> = frames
        .iter()
        .map(|(_, _, pixels)| encode_webp_frame(pixels, width, height))
        .collect::<Result<Vec<_>, _>>()?;

    mux_animated_webp(&frame_webps, width, height, delay_ms)
}

/// Encode one RGBA frame as a static lossless WebP file (VP8L).
///
/// Uses `image::codecs::webp::WebPEncoder::new_lossless`, which outputs
/// a standalone RIFF-WEBP file. The enclosing `mux_animated_webp` extracts
/// the VP8L chunk from this output and places it into an ANMF container.
fn encode_webp_frame(pixels: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    use image::codecs::webp::WebPEncoder;
    use image::{ColorType, ImageEncoder};

    let mut buf = Vec::new();
    WebPEncoder::new_lossless(&mut buf)
        .write_image(pixels, width, height, ColorType::Rgba8.into())
        .map_err(|e| format!("webp frame encode: {e}"))?;
    Ok(buf)
}

/// Mux a sequence of single-frame WebP files into an animated WebP.
///
/// RIFF layout:
///   RIFF <size> WEBP
///     VP8X <10 bytes>  — animation + alpha flags, canvas dims
///     ANIM <6 bytes>   — transparent bg, loop count 0 (infinite)
///     ANMF …           — one per frame
fn mux_animated_webp(
    frame_webps: &[Vec<u8>],
    width: u32,
    height: u32,
    delay_ms: u32,
) -> Result<Vec<u8>, String> {
    // ── helpers ──────────────────────────────────────────────────────

    fn u24le(n: u32) -> [u8; 3] {
        [n as u8, (n >> 8) as u8, (n >> 16) as u8]
    }

    fn riff_chunk(tag: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let size = payload.len() as u32;
        let mut out = Vec::with_capacity(8 + payload.len() + (payload.len() & 1));
        out.extend_from_slice(tag);
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(payload);
        if payload.len() & 1 == 1 {
            out.push(0x00); // RIFF word-alignment pad
        }
        out
    }

    /// Extract ALPH / VP8 / VP8L chunks from a single-frame WebP file.
    /// Skips VP8X — ANMF frame data must not contain a VP8X wrapper.
    fn extract_frame_chunks(webp: &[u8]) -> Result<Vec<u8>, String> {
        if webp.len() < 12 || &webp[0..4] != b"RIFF" || &webp[8..12] != b"WEBP" {
            return Err("encode_webp_frame returned invalid WebP".to_string());
        }
        let mut out = Vec::new();
        let mut offset = 12usize;
        while offset + 8 <= webp.len() {
            let tag = &webp[offset..offset + 4];
            let size =
                u32::from_le_bytes(webp[offset + 4..offset + 8].try_into().unwrap()) as usize;
            let padded = size + (size & 1);
            let end = (offset + 8 + padded).min(webp.len());
            if tag == b"VP8 " || tag == b"VP8L" || tag == b"ALPH" {
                out.extend_from_slice(&webp[offset..end]);
            }
            offset = offset + 8 + padded;
        }
        if out.is_empty() {
            return Err("no VP8/VP8L chunk found in frame WebP".to_string());
        }
        Ok(out)
    }

    // ── VP8X chunk (10-byte payload) ──────────────────────────────────
    // Byte 0: flags — bit 1 = animation, bit 4 = alpha
    // Bytes 1-3: reserved
    // Bytes 4-6: canvas width  − 1 (24-bit LE)
    // Bytes 7-9: canvas height − 1 (24-bit LE)
    let vp8x_flags: u8 = (1 << 1) | (1 << 4); // animation | alpha
    let mut vp8x_payload = Vec::with_capacity(10);
    vp8x_payload.push(vp8x_flags);
    vp8x_payload.extend_from_slice(&[0u8; 3]); // reserved
    vp8x_payload.extend_from_slice(&u24le(width - 1));
    vp8x_payload.extend_from_slice(&u24le(height - 1));
    let vp8x_chunk = riff_chunk(b"VP8X", &vp8x_payload);

    // ── ANIM chunk (6-byte payload) ───────────────────────────────────
    // Bytes 0-3: background colour (0 = transparent)
    // Bytes 4-5: loop count (0 = infinite)
    let anim_chunk = riff_chunk(b"ANIM", &[0u8; 6]);

    // ── ANMF chunks ───────────────────────────────────────────────────
    // Per-frame header (16 bytes before frame data):
    //   frame X/2 (24-bit LE) | frame Y/2 (24-bit LE)
    //   width−1   (24-bit LE) | height−1  (24-bit LE)
    //   duration ms (24-bit LE) | flags (1 byte)
    let mut anmf_chunks = Vec::new();
    for webp in frame_webps {
        let frame_data = extract_frame_chunks(webp)?;
        let mut anmf_payload = Vec::with_capacity(16 + frame_data.len());
        anmf_payload.extend_from_slice(&u24le(0)); // frame X / 2
        anmf_payload.extend_from_slice(&u24le(0)); // frame Y / 2
        anmf_payload.extend_from_slice(&u24le(width - 1));
        anmf_payload.extend_from_slice(&u24le(height - 1));
        anmf_payload.extend_from_slice(&u24le(delay_ms));
        // Blending method bit (6) = 1 → overwrite (do not alpha-blend onto
        // previous frame). Without this, transparent pixels reveal the
        // prior frame's content and frames accumulate visually.
        anmf_payload.push(0x02); // bits[1]=1 → no-blend/overwrite, dispose=0
        anmf_payload.extend_from_slice(&frame_data);
        anmf_chunks.push(riff_chunk(b"ANMF", &anmf_payload));
    }

    // ── Assemble RIFF container ───────────────────────────────────────
    let mut webp_body = Vec::new();
    webp_body.extend_from_slice(b"WEBP");
    webp_body.extend_from_slice(&vp8x_chunk);
    webp_body.extend_from_slice(&anim_chunk);
    for chunk in &anmf_chunks {
        webp_body.extend_from_slice(chunk);
    }

    let riff_size = webp_body.len() as u32;
    let mut out = Vec::with_capacity(8 + webp_body.len());
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&riff_size.to_le_bytes());
    out.extend_from_slice(&webp_body);
    Ok(out)
}
