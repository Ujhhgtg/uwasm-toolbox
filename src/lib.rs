mod ncm;
mod tgs;

use wasm_bindgen::prelude::*;

// ---------------------------------------------------------------------------
// NCM
// ---------------------------------------------------------------------------

/// Result object returned by `ncm_convert`.
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
    /// Raw decrypted audio bytes (FLAC or MP3).
    #[wasm_bindgen(getter)]
    pub fn audio(&self) -> Vec<u8> {
        self.audio.clone()
    }

    /// `"flac"` or `"mp3"`.
    #[wasm_bindgen(getter)]
    pub fn format(&self) -> String {
        self.format.clone()
    }

    /// JSON string with keys: name, album, artist, bitrate, duration, format.
    /// Empty string if the file contains no metadata.
    #[wasm_bindgen(getter)]
    pub fn metadata_json(&self) -> String {
        self.metadata_json.clone()
    }

    /// Embedded cover image bytes (JPEG or PNG). Empty if none.
    #[wasm_bindgen(getter)]
    pub fn cover(&self) -> Vec<u8> {
        self.cover.clone()
    }

    /// MIME type of the cover image (`"image/jpeg"` or `"image/png"`).
    #[wasm_bindgen(getter)]
    pub fn cover_mime(&self) -> String {
        self.cover_mime.clone()
    }
}

/// Decrypt a `.ncm` file.
///
/// Returns an `NcmResult` on success, or throws a JS string error.
#[wasm_bindgen]
pub fn ncm_convert(data: &[u8]) -> Result<NcmResult, JsValue> {
    let decoded = ncm::decode(data).map_err(|e| JsValue::from_str(&e))?;

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
        audio: decoded.audio,
        format: decoded.format,
        metadata_json,
        cover: decoded.cover,
        cover_mime,
    })
}

// ---------------------------------------------------------------------------
// TGS
// ---------------------------------------------------------------------------

/// Decompress a `.tgs` file (gzip'd Lottie JSON) and return the JSON string.
///
/// The returned string should be passed to `lottie-web` for frame rendering.
#[wasm_bindgen]
pub fn tgs_decompress(data: &[u8]) -> Result<String, JsValue> {
    tgs::decompress(data).map_err(|e| JsValue::from_str(&e))
}

/// Encode a sequence of RGBA frames into an animated GIF.
///
/// `frames_rgba` — flat buffer: `frame_count × height × width × 4` bytes (RGBA order).
/// `delay_cs`    — per-frame delay in centiseconds (e.g. `7` ≈ 15 fps).
///
/// Returns the GIF file as a byte vector, or throws a JS string error.
#[wasm_bindgen]
pub fn tgs_encode_gif(
    frames_rgba: &[u8],
    width: u32,
    height: u32,
    frame_count: u32,
    delay_cs: u16,
) -> Result<Vec<u8>, JsValue> {
    tgs::encode_gif(frames_rgba, width, height, frame_count, delay_cs)
        .map_err(|e| JsValue::from_str(&e))
}
