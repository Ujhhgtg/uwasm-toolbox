/* tslint:disable */
/* eslint-disable */

/**
 * WASM-bindgen return type wrapping a decrypted NetEase Cloud Music file.
 *
 * Contains the decoded audio bytes, detected format, parsed metadata as JSON,
 * and any embedded cover art. All fields are clone-out getters — the WASM
 * boundary copies `Vec<u8>` rather than transferring the buffer.
 */
export class NcmResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    readonly audio: Uint8Array;
    readonly cover: Uint8Array;
    readonly cover_mime: string;
    readonly format: string;
    readonly metadata_json: string;
}

/**
 * Decrypt a `.ncm` file and apply metadata tags (title, artist, album, cover).
 *
 * Wraps the synchronous `ncm::decode` (AES-128-ECB + key-box stream
 * cipher) with async metadata application that may fetch remote cover art
 * via `reqwest::get` (browser `fetch` on WASM).
 *
 * Returns an `NcmResult` struct whose getters are callable from JS.
 */
export function ncm_convert(data: Uint8Array): Promise<NcmResult>;

/**
 * Convert a Telegram .tgs sticker (gzip-compressed Lottie JSON) to an
 * animated GIF or lossless WebP.
 *
 * # Parameters (passed from JS)
 * - `data`: raw .tgs file bytes (gzip or plain JSON)
 * - `fps`: target frame rate, clamped to ≤ source animation FPS
 * - `width`, `height`: output canvas size (maintains aspect ratio via
 *   uniform scaling; one dimension may be smaller)
 * - `max_frames`: uniform subsample cap (0 = unlimited)
 * - `frame_start`, `frame_end`: inclusive/exclusive range (0 = use animation
 *   defaults)
 * - `format`: `"gif"` or `"webp"`
 */
export function tgs_convert(data: Uint8Array, fps: number, width: number, height: number, max_frames: number, frame_start: number, frame_end: number, format: string): Uint8Array;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_ncmresult_free: (a: number, b: number) => void;
    readonly ncm_convert: (a: number, b: number) => any;
    readonly ncmresult_audio: (a: number) => [number, number];
    readonly ncmresult_cover: (a: number) => [number, number];
    readonly ncmresult_cover_mime: (a: number) => [number, number];
    readonly ncmresult_format: (a: number) => [number, number];
    readonly ncmresult_metadata_json: (a: number) => [number, number];
    readonly tgs_convert: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => [number, number, number, number];
    readonly wasm_bindgen_52a40a1bd5563bc2___convert__closures_____invoke___wasm_bindgen_52a40a1bd5563bc2___JsValue__core_9b3796e30d99ddb7___result__Result_____wasm_bindgen_52a40a1bd5563bc2___JsError___true_: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen_52a40a1bd5563bc2___convert__closures_____invoke___js_sys_7b008e29d3e68904___Function_fn_wasm_bindgen_52a40a1bd5563bc2___JsValue_____wasm_bindgen_52a40a1bd5563bc2___sys__Undefined___js_sys_7b008e29d3e68904___Function_fn_wasm_bindgen_52a40a1bd5563bc2___JsValue_____wasm_bindgen_52a40a1bd5563bc2___sys__Undefined_______true_: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen_52a40a1bd5563bc2___convert__closures_____invoke_______true_: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_destroy_closure: (a: number, b: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
