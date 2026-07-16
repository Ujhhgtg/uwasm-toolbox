/* tslint:disable */
/* eslint-disable */

/**
 * Result object returned by `ncm_convert`.
 */
export class NcmResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Raw decrypted audio bytes (FLAC or MP3).
     */
    readonly audio: Uint8Array;
    /**
     * Embedded cover image bytes (JPEG or PNG). Empty if none.
     */
    readonly cover: Uint8Array;
    /**
     * MIME type of the cover image (`"image/jpeg"` or `"image/png"`).
     */
    readonly cover_mime: string;
    /**
     * `"flac"` or `"mp3"`.
     */
    readonly format: string;
    /**
     * JSON string with keys: name, album, artist, bitrate, duration, format.
     * Empty string if the file contains no metadata.
     */
    readonly metadata_json: string;
}

/**
 * Decrypt a `.ncm` file.
 *
 * Returns an `NcmResult` on success, or throws a JS string error.
 */
export function ncm_convert(data: Uint8Array): NcmResult;

/**
 * Decompress a `.tgs` file (gzip'd Lottie JSON) and return the JSON string.
 *
 * The returned string should be passed to `lottie-web` for frame rendering.
 */
export function tgs_decompress(data: Uint8Array): string;

/**
 * Encode a sequence of RGBA frames into an animated GIF.
 *
 * `frames_rgba` — flat buffer: `frame_count × height × width × 4` bytes (RGBA order).
 * `delay_cs`    — per-frame delay in centiseconds (e.g. `7` ≈ 15 fps).
 *
 * Returns the GIF file as a byte vector, or throws a JS string error.
 */
export function tgs_encode_gif(frames_rgba: Uint8Array, width: number, height: number, frame_count: number, delay_cs: number): Uint8Array;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_ncmresult_free: (a: number, b: number) => void;
    readonly ncm_convert: (a: number, b: number) => [number, number, number];
    readonly ncmresult_audio: (a: number) => [number, number];
    readonly ncmresult_cover: (a: number) => [number, number];
    readonly ncmresult_cover_mime: (a: number) => [number, number];
    readonly ncmresult_format: (a: number) => [number, number];
    readonly ncmresult_metadata_json: (a: number) => [number, number];
    readonly tgs_decompress: (a: number, b: number) => [number, number, number, number];
    readonly tgs_encode_gif: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
