/* tslint:disable */
/* eslint-disable */

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
 * Decrypt a `.ncm` file, embed metadata and cover art, then return the result.
 *
 * If no cover art is embedded in the file, attempts to fetch it from the
 * NetEase CDN via the URL stored in the metadata (uses browser `fetch`).
 */
export function ncm_convert(data: Uint8Array): Promise<NcmResult>;

/**
 * Convert a `.tgs` file to an animated GIF or lossless WebP entirely in Rust.
 *
 * Parameters
 * ----------
 * data         — raw `.tgs` bytes (gzip-compressed or plain UTF-8 Lottie JSON)
 * fps          — target output frame rate (clamped to animation's native fps)
 * width        — output width in pixels
 * height       — output height in pixels
 * max_frames   — maximum number of frames (0 = unlimited)
 * frame_start  — first source frame to include (0 = animation start)
 * frame_end    — last source frame (exclusive, 0 = animation end)
 * format       — `"gif"` or `"webp"`
 *
 * Returns the encoded file bytes, or throws a JS string error.
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
