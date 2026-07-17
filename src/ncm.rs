use std::io::{Cursor, Read, Seek, SeekFrom};

use aes::Aes128Dec;
use aes::cipher::{BlockCipherDecrypt, KeyInit};
use base64::Engine;
use serde::Serialize;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const CORE_KEY: [u8; 16] = [
    0x68, 0x7A, 0x48, 0x52, 0x41, 0x6D, 0x73, 0x6F, 0x35, 0x6B, 0x49, 0x6E, 0x62, 0x61, 0x78, 0x57,
];

const MODIFY_KEY: [u8; 16] = [
    0x23, 0x31, 0x34, 0x6C, 0x6A, 0x6B, 0x5F, 0x21, 0x5C, 0x5D, 0x26, 0x30, 0x55, 0x3C, 0x27, 0x28,
];

const PNG_HEADER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

// ---------------------------------------------------------------------------
// Public output types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct SongMeta {
    pub name: String,
    pub album: String,
    pub artist: String,
    pub bitrate: i64,
    pub duration: i64,
    pub format: String,
}

#[derive(Debug)]
pub struct NcmDecoded {
    pub audio: Vec<u8>,
    /// "flac" or "mp3"
    pub format: String,
    pub metadata: Option<SongMeta>,
    /// Raw cover image bytes (JPEG or PNG), if embedded in the .ncm file.
    pub cover: Vec<u8>,
    /// Remote album art URL from the metadata, used when no cover is embedded.
    pub album_pic_url: String,
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn read_exact<R: Read>(reader: &mut R, size: usize) -> std::io::Result<Vec<u8>> {
    let mut buf = vec![0u8; size];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

/// AES-128-ECB decrypt with PKCS7 unpadding.
fn aes_ecb_decrypt(key: &[u8; 16], data: &[u8]) -> Vec<u8> {
    let cipher = Aes128Dec::new(key.into());
    let mut buf = data.to_vec();
    for chunk in buf.chunks_exact_mut(16) {
        cipher.decrypt_block(chunk.try_into().unwrap());
    }
    // PKCS7 unpadding
    let pad_len = *buf.last().unwrap_or(&0) as usize;
    if pad_len > 0 && pad_len <= 16 {
        buf.truncate(buf.len() - pad_len);
    }
    buf
}

/// Build the 256-byte key box from decrypted key material.
fn build_key_box(key: &[u8]) -> [u8; 256] {
    let mut box_: [u8; 256] = std::array::from_fn(|i| i as u8);
    let mut last: u8 = 0;
    let mut key_offset = 0;
    for i in 0..256usize {
        let swap = box_[i];
        let c = swap.wrapping_add(last).wrapping_add(key[key_offset]);
        key_offset = (key_offset + 1) % key.len();
        box_[i] = box_[c as usize];
        box_[c as usize] = swap;
        last = c;
    }
    box_
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Decrypt a raw .ncm file given as a byte slice.
///
/// Returns the decrypted audio bytes, detected format, parsed metadata, and
/// embedded cover art (if any).
pub fn decode(data: &[u8]) -> Result<NcmDecoded, String> {
    let mut cur = Cursor::new(data);

    // --- Validate magic header: "NETCMADF" ---
    let magic1 = read_exact(&mut cur, 4).map_err(|e| e.to_string())?;
    if u32::from_le_bytes(magic1[..4].try_into().unwrap()) != 0x4E455443 {
        return Err("not a .ncm file (bad magic 1)".into());
    }
    let magic2 = read_exact(&mut cur, 4).map_err(|e| e.to_string())?;
    if u32::from_le_bytes(magic2[..4].try_into().unwrap()) != 0x4D414446 {
        return Err("not a .ncm file (bad magic 2)".into());
    }

    // --- Skip 2-byte version ---
    cur.seek(SeekFrom::Current(2)).map_err(|e| e.to_string())?;

    // --- Encrypted key ---
    let key_len_raw = read_exact(&mut cur, 4).map_err(|e| e.to_string())?;
    let key_len = u32::from_le_bytes(key_len_raw[..4].try_into().unwrap()) as usize;

    let mut key_data = read_exact(&mut cur, key_len).map_err(|e| e.to_string())?;
    for b in &mut key_data {
        *b ^= 0x64;
    }

    let decrypted_key = aes_ecb_decrypt(&CORE_KEY, &key_data);
    let key_box = build_key_box(&decrypted_key[17..]);

    // --- Metadata ---
    let meta_len_raw = read_exact(&mut cur, 4).map_err(|e| e.to_string())?;
    let meta_len = u32::from_le_bytes(meta_len_raw[..4].try_into().unwrap()) as usize;

    let (metadata, album_pic_url) = if meta_len > 0 {
        let mut modify_data = read_exact(&mut cur, meta_len).map_err(|e| e.to_string())?;
        for b in &mut modify_data {
            *b ^= 0x63;
        }

        // Strip "163 key(Don't modify):" prefix (22 bytes)
        let b64_input = std::str::from_utf8(&modify_data[22..])
            .map_err(|_| "invalid metadata UTF-8 after XOR")?;

        let modify_out = base64::engine::general_purpose::STANDARD
            .decode(b64_input.as_bytes())
            .map_err(|e| e.to_string())?;

        let modify_decrypt = aes_ecb_decrypt(&MODIFY_KEY, &modify_out);

        // Strip "music:" prefix (6 bytes)
        let meta_json = std::str::from_utf8(&modify_decrypt[6..])
            .map_err(|_| "invalid metadata JSON UTF-8")?;

        let v: Value = serde_json::from_str(meta_json).map_err(|e| e.to_string())?;

        let name = v.get("musicName").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let album = v.get("album").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let artist = v.get("artist")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|a| {
                        a.as_array()
                            .and_then(|inner| inner.first())
                            .and_then(|f| f.as_str())
                    })
                    .collect::<Vec<_>>()
                    .join(" / ")
            })
            .unwrap_or_default();

        let bitrate = v.get("bitrate").and_then(|v| v.as_i64()).unwrap_or(0);
        let duration = v.get("duration").and_then(|v| v.as_i64()).unwrap_or(0);
        let format = v.get("format").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let pic_url = v.get("albumPic").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let meta = SongMeta { name, album, artist, bitrate, duration, format };
        (Some(meta), pic_url)
    } else {
        (None, String::new())
    };

    // --- Skip 5-byte gap ---
    cur.seek(SeekFrom::Current(5)).map_err(|e| e.to_string())?;

    // --- Cover frame ---
    let cf_len_raw = read_exact(&mut cur, 4).map_err(|e| e.to_string())?;
    let cf_total = u32::from_le_bytes(cf_len_raw[..4].try_into().unwrap()) as i64;

    let cd_len_raw = read_exact(&mut cur, 4).map_err(|e| e.to_string())?;
    let cd_len = u32::from_le_bytes(cd_len_raw[..4].try_into().unwrap()) as usize;

    let cover = if cd_len > 0 {
        read_exact(&mut cur, cd_len).map_err(|e| e.to_string())?
    } else {
        Vec::new()
    };

    // Skip remaining cover frame padding
    let remaining = cf_total - cd_len as i64;
    if remaining > 0 {
        cur.seek(SeekFrom::Current(remaining)).map_err(|e| e.to_string())?;
    }

    // --- Stream-decrypt audio ---
    let mut audio = Vec::with_capacity(data.len());
    let mut buffer = vec![0u8; 0x8000];
    let mut format_str = String::from("flac");
    let mut format_detected = false;

    loop {
        let n = cur.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }

        for i in 0..n {
            let j = (i + 1) & 0xff;
            let idx = (key_box[j] as usize
                + key_box[(key_box[j] as usize + j) & 0xff] as usize)
                & 0xff;
            buffer[i] ^= key_box[idx];
        }

        if !format_detected {
            if n >= 3 && buffer[0] == 0x49 && buffer[1] == 0x44 && buffer[2] == 0x33 {
                format_str = "mp3".to_string();
            }
            format_detected = true;
        }

        audio.extend_from_slice(&buffer[..n]);
    }

    // Metadata embedding happens async (may need to fetch remote cover art).
    // Return raw audio here; lib.rs calls apply_metadata_async after decode.
    Ok(NcmDecoded { audio, format: format_str, metadata, cover, album_pic_url })
}

/// MIME type of the embedded cover image, derived from its header bytes.
pub fn cover_mime(cover: &[u8]) -> &'static str {
    if cover.starts_with(&PNG_HEADER) { "image/png" } else { "image/jpeg" }
}

// ---------------------------------------------------------------------------
// Metadata writing
// ---------------------------------------------------------------------------

/// Embed title, artist, album, and cover art into the decrypted audio bytes.
///
/// If no cover is embedded in the `.ncm` file and `pic_url` is non-empty,
/// fetches the image from the NetEase CDN before writing tags (same
/// behaviour as ncmx's `fix_metadata(true)`).
///
/// Best-effort: returns the original bytes unchanged on any failure.
pub async fn apply_metadata_async(
    audio: Vec<u8>,
    meta: Option<&SongMeta>,
    embedded_cover: &[u8],
    pic_url: &str,
) -> Vec<u8> {
    let Some(meta) = meta else { return audio };

    // Resolve cover: prefer embedded, fall back to remote fetch
    let cover: Vec<u8> = if !embedded_cover.is_empty() {
        embedded_cover.to_vec()
    } else if !pic_url.is_empty() {
        fetch_bytes(pic_url).await.unwrap_or_default()
    } else {
        Vec::new()
    };

    match try_apply_metadata(&audio, meta, &cover) {
        Ok(tagged) => tagged,
        Err(_) => audio,
    }
}

/// Fetch raw bytes from a URL (async, uses browser fetch on WASM).
async fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    let resp = reqwest::get(url)
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| e.to_string())
}

fn try_apply_metadata(audio: &[u8], meta: &SongMeta, cover: &[u8]) -> Result<Vec<u8>, String> {
    use lofty::config::WriteOptions;
    use lofty::file::AudioFile;
    use lofty::picture::{MimeType, Picture, PictureType};
    use lofty::prelude::*;
    use lofty::probe::Probe;

    let read_cur = std::io::BufReader::new(Cursor::new(audio));
    let mut tagged = Probe::new(read_cur)
        .guess_file_type()
        .map_err(|e| e.to_string())?
        .read()
        .map_err(|e| e.to_string())?;

    let tag = tagged.primary_tag_mut().ok_or("no primary tag")?;

    if !meta.name.is_empty()   { tag.set_title(meta.name.clone()); }
    if !meta.artist.is_empty() { tag.set_artist(meta.artist.clone()); }
    if !meta.album.is_empty()  { tag.set_album(meta.album.clone()); }

    if !cover.is_empty() {
        let mime = if cover.starts_with(&PNG_HEADER) { MimeType::Png } else { MimeType::Jpeg };
        let picture = Picture::unchecked(cover.to_vec())
            .pic_type(PictureType::CoverFront)
            .mime_type(mime)
            .description("Cover")
            .build();
        tag.set_picture(0, picture);
    }

    let mut write_cur = Cursor::new(audio.to_vec());
    tagged
        .save_to(&mut write_cur, WriteOptions::default())
        .map_err(|e| e.to_string())?;
    Ok(write_cur.into_inner())
}
