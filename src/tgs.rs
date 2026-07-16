use flate2::read::GzDecoder;
use std::io::Read;

// ---------------------------------------------------------------------------
// TGS decompression
// ---------------------------------------------------------------------------

/// Decompress a .tgs file and return the Lottie JSON string.
///
/// Accepts both the standard gzip-compressed format and plain UTF-8 JSON
/// (some clients export .tgs files without compression).
pub fn decompress(data: &[u8]) -> Result<String, String> {
    // Gzip magic bytes: 0x1F 0x8B
    if data.starts_with(&[0x1F, 0x8B]) {
        let mut decoder = GzDecoder::new(data);
        let mut json = String::new();
        decoder
            .read_to_string(&mut json)
            .map_err(|e| format!("gzip decompress failed: {e}"))?;
        Ok(json)
    } else {
        // Fall back to plain UTF-8 JSON
        std::str::from_utf8(data)
            .map(|s| s.to_owned())
            .map_err(|_| "not a valid gzip stream or UTF-8 JSON".to_string())
    }
}

// ---------------------------------------------------------------------------
// GIF encoding
// ---------------------------------------------------------------------------

/// Encode a sequence of RGBA frames into an animated GIF.
///
/// `frames_rgba` is a flat buffer: `frame_count` × `height` × `width` × 4 bytes (RGBA).
/// `delay_cs` is the per-frame delay in centiseconds (100 = 1 s).
pub fn encode_gif(
    frames_rgba: &[u8],
    width: u32,
    height: u32,
    frame_count: u32,
    delay_cs: u16,
) -> Result<Vec<u8>, String> {
    let w = width as u16;
    let h = height as u16;
    let frame_bytes = (width * height * 4) as usize;

    if frames_rgba.len() != frame_bytes * frame_count as usize {
        return Err(format!(
            "buffer length mismatch: expected {} got {}",
            frame_bytes * frame_count as usize,
            frames_rgba.len()
        ));
    }

    let mut output: Vec<u8> = Vec::new();

    {
        let mut encoder = gif::Encoder::new(&mut output, w, h, &[])
            .map_err(|e| format!("gif encoder init: {e}"))?;
        encoder
            .set_repeat(gif::Repeat::Infinite)
            .map_err(|e| format!("gif set repeat: {e}"))?;

        for i in 0..frame_count as usize {
            let start = i * frame_bytes;
            let end = start + frame_bytes;
            // from_rgba_speed quantises the palette and handles transparency
            let mut rgba = frames_rgba[start..end].to_vec();
            let mut frame = gif::Frame::from_rgba_speed(w, h, &mut rgba, 10);
            frame.delay = delay_cs;
            encoder
                .write_frame(&frame)
                .map_err(|e| format!("gif write frame {i}: {e}"))?;
        }
        // encoder dropped here, releasing borrow on output
    }

    Ok(output)
}
