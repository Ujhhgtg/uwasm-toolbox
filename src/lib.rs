pub mod log;
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

#[wasm_bindgen]
pub async fn ncm_convert(data: &[u8]) -> Result<NcmResult, JsValue> {
    clog!("ncm_convert: {} bytes", data.len());

    let decoded = ncm::decode(data).map_err(|e| {
        cerror!("ncm decode failed: {e}");
        JsValue::from_str(&e)
    })?;

    clog!(
        "ncm decode ok: format={} cover={} bytes pic_url={}",
        decoded.format,
        decoded.cover.len(),
        if decoded.album_pic_url.is_empty() { "(none)" } else { &decoded.album_pic_url }
    );

    let audio = ncm::apply_metadata_async(
        decoded.audio,
        decoded.metadata.as_ref(),
        &decoded.cover,
        &decoded.album_pic_url,
    )
    .await;

    clog!("ncm metadata applied: audio={} bytes", audio.len());

    let metadata_json = match &decoded.metadata {
        Some(m) => serde_json::to_string(m).unwrap_or_default(),
        None => String::new(),
    };
    let cover_mime = if decoded.cover.is_empty() {
        String::new()
    } else {
        ncm::cover_mime(&decoded.cover).to_string()
    };

    Ok(NcmResult { audio, format: decoded.format, metadata_json, cover: decoded.cover, cover_mime })
}

// ---------------------------------------------------------------------------
// TGS
// ---------------------------------------------------------------------------

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
    clog!("tgs_convert: {} bytes  fps={fps}  size={width}x{height}  format={format}", data.len());

    let json = tgs::decompress(data).map_err(|e| {
        cerror!("tgs decompress failed: {e}");
        JsValue::from_str(&e)
    })?;

    clog!("tgs decompressed: {} chars of Lottie JSON", json.len());

    let opts = tgs::ConvertOptions { fps, width, height, max_frames, frame_start, frame_end };
    let result = tgs::convert(&json, &opts, format).map_err(|e| {
        cerror!("tgs convert failed: {e}");
        JsValue::from_str(&e)
    })?;

    clog!("tgs_convert done: {} bytes of {format}", result.len());
    Ok(result)
}
