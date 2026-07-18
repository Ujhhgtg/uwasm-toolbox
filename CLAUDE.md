# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & dev

```bash
# Build WASM + copy output to www/pkg/
./build.sh                         # wasm-pack build --target web --out-dir www/pkg --release

# Fast compile-check without full wasm-pack overhead
cargo check --target wasm32-unknown-unknown

# Serve the frontend locally
python -m http.server -d www 8080  # open http://localhost:8080
```

There are no tests. CI (`deploy-gh-pages.yml`) runs `./build.sh && rm -f www/pkg/.gitignore` on every push to `master` and deploys `www/` to the `gh-pages` branch via `JamesIves/github-pages-deploy-action`. The `.gitignore` removal is required because `wasm-pack` writes a `pkg/.gitignore` that would otherwise cause the deploy action's `git add` to skip all WASM artefacts.

## Architecture

### Rust → WASM (`src/`)

The crate is a pure `cdylib`. All public surface is in `src/lib.rs`; the two tool modules are `src/ncm.rs` and `src/tgs.rs`.

**`ncm_convert(data: &[u8]) -> Promise<NcmResult>`** (async)

1. `ncm::decode` — validates the `NETCMADF` magic, AES-128-ECB decrypts the key box with `CORE_KEY`, decrypts metadata JSON with `MODIFY_KEY`, stream-decrypts the audio using the RC4-like key-box cipher, returns raw audio + `SongMeta` + embedded cover + `album_pic_url`.
2. `ncm::apply_metadata_async` — fetches remote cover art via `reqwest::get` (browser `fetch` on WASM) if no cover was embedded, then writes title/artist/album/cover into the audio using `lofty`'s `Probe::new(BufReader::new(Cursor::new(...)))` + `save_to` API (fully in-memory, no filesystem).

**`tgs_convert(data, fps, w, h, max_frames, frame_start, frame_end, format) -> Vec<u8>`** (sync)

1. `tgs::decompress` — detects gzip by magic bytes (`0x1F 0x8B`) and decompresses, or passes through plain UTF-8 JSON.
2. Computes a frame schedule (step = `round(source_fps / target_fps)`, uniform subsampling for `max_frames`).
3. `rasterlottie::Renderer::default().prepare(&anim)` — compiles the animation once; `prepared.render_frame(f, config)` renders each frame to `RasterFrame { pixels: Vec<u8> }` (RGBA8, `tiny-skia` backend, pure Rust, no C deps).
4. GIF: `gif::Frame::from_rgba_speed` + `DisposalMethod::Background` (prevents frame accumulation).
5. WebP: `image::codecs::webp::WebPEncoder::new_lossless` per frame (VP8L via `image-webp`, pure Rust), then `mux_animated_webp` assembles the RIFF/VP8X/ANIM/ANMF container manually.

**Logging** (`src/log.rs`): `clog!`, `cwarn!`, `cerror!` macros call `web_sys::console::log_1/warn_1/error_1`. Use `use crate::{clog, cwarn, cerror};` in submodules.

### Frontend (`www/`)

**No bundler.** The HTML pages import ES modules directly. `wasm-pack --target web` emits an ES module in `www/pkg/`.

**Worker architecture**: every conversion is dispatched to a `WorkerPool` (in `common.js`). Each worker (`www/worker.js`) owns its own WASM instance, initialised with `import init, { ... } from './pkg/uwasm_toolbox.js'`. Files are dispatched concurrently with `Promise.all`; the pool size is `min(fileCount, navigator.hardwareConcurrency || 4)`. `ArrayBuffer`s are transferred (not copied) between main thread and workers.

**Message protocol** (`worker.js`):

- `{ id, type: 'ncm', data: ArrayBuffer }` → `{ id, audio, format, metadata_json, cover, cover_mime }`
- `{ id, type: 'tgs', data, fps, width, height, maxFrames, frameStart, frameEnd, format }` → `{ id, output: ArrayBuffer, format }`

**Shared utilities** (`common.js`): `WorkerPool`, `buildZip` (stored ZIP, no compression, CRC-32 computed in JS), drop-zone helpers (`showFileList`/`resetDropZone`), `setStatus`, `downloadBytes`, `fmtBytes`.

### Adding a new tool

1. Add Rust logic in a new `src/whatever.rs`, expose it in `src/lib.rs` with `#[wasm_bindgen]`.
2. Handle it in `www/worker.js` (`if (type === 'whatever') { ... }`).
3. Create `www/whatever.html` following the existing page structure (WorkerPool dispatch, result cards, `buildZip` for Download all).
4. Add a card to `www/index.html`.
