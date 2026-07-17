mod ncm;
mod tgs;

use wasm_bindgen::prelude::*;

// ---------------------------------------------------------------------------
// NCM
// ---------------------------------------------------------------------------

#[wasm_bindgen]
pub struct NcmResult {
    audio: Vec<u8>,
    format: String,
    metadata_json: String,
    cover: Vec<u8>,
    cover_mime: String,
}

#[wasm_bindgen]
impl NcmResult {
    #[wasm_bindgen(getter)]
    pub fn audio(&self) -> Vec<u8> { self.audio.clone() }
    #[wasm_bindgen(getter)]
    pub fn format(&self) -> String { self.format.clone() }
    #[wasm_bindgen(getter)]
    pub fn metadata_json(&self) -> String { self.metadata_json.clone() }
    #[wasm_bindgen(getter)]
    pub fn cover(&self) -> Vec<u8> { self.cover.clone() }
    #[wasm_bindgen(getter)]
    pub fn cover_mime(&self) -> String { self.cover_mime.clone() }
}

/// Decrypt a `.ncm` file, embed metadata and cover art, then return the result.
///
/// If no cover art is embedded in the file, attempts to fetch it from the
/// NetEase CDN via the URL stored in the metadata (uses browser `fetch`).
#[wasm_bindgen]
pub async fn ncm_convert(data: &[u8]) -> Result<NcmResult, JsValue> {
    let decoded = ncm::decode(data).map_err(|e| JsValue::from_str(&e))?;

    let audio = ncm::apply_metadata_async(
        decoded.audio,
        decoded.metadata.as_ref(),
        &decoded.cover,
        &decoded.album_pic_url,
    )
    .await;

    let metadata_json = match &decoded.metadata {
        Some(m) => serde_json::to_string(m).unwrap_or_default(),
        None => String::new(),
    };

    let cover_mime = if decoded.cover.is_empty() {
        String::new()
    } else {
        ncm::cover_mime(&decoded.cover).to_string()
    };

    Ok(NcmResult {
        audio,
        format: decoded.format,
        metadata_json,
        cover: decoded.cover,
        cover_mime,
    })
}

// ---------------------------------------------------------------------------
// TGS
// ---------------------------------------------------------------------------

/// Convert a `.tgs` file to an animated GIF or lossless WebP entirely in Rust.
///
/// Parameters
/// ----------
/// data         — raw `.tgs` bytes (gzip-compressed or plain UTF-8 Lottie JSON)
/// fps          — target output frame rate (clamped to animation's native fps)
/// width        — output width in pixels
/// height       — output height in pixels
/// max_frames   — maximum number of frames (0 = unlimited)
/// frame_start  — first source frame to include (0 = animation start)
/// frame_end    — last source frame (exclusive, 0 = animation end)
/// format       — `"gif"` or `"webp"`
///
/// Returns the encoded file bytes, or throws a JS string error.
#[wasm_bindgen]
pub fn tgs_convert(
    data: &[u8],
    fps: f32,
    width: u32,
    height: u32,
    max_frames: u32,
    frame_start: u32,
    frame_end: u32,
    format: &str,
) -> Result<Vec<u8>, JsValue> {
    let json = tgs::decompress(data).map_err(|e| JsValue::from_str(&e))?;
    let opts = tgs::ConvertOptions {
        fps,
        width,
        height,
        max_frames,
        frame_start,
        frame_end,
    };
    tgs::convert(&json, &opts, format).map_err(|e| JsValue::from_str(&e))
}
