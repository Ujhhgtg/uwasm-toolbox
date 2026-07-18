/**
 * common.js — shared WASM initialisation and small utilities
 */

let _wasm = null;

/** Initialise the WASM module once and cache it. */
export async function initWasm() {
    if (_wasm) return _wasm;
    const mod = await import("./pkg/uwasm_toolbox.js");
    await mod.default();
    _wasm = mod;
    return _wasm;
}

/** Returns the cached WASM module (call initWasm first). */
export function wasm() {
    return _wasm;
}

// ── File / Blob utilities ──────────────────────────────────────────

/** Trigger a browser download for `bytes` with the given filename. */
export function downloadBytes(
    bytes,
    filename,
    mime = "application/octet-stream"
) {
    const blob = new Blob([bytes], { type: mime });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    setTimeout(() => {
        URL.revokeObjectURL(url);
        a.remove();
    }, 1000);
}

/** Read a File as a Uint8Array. */
export function readFileBytes(file) {
    return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(new Uint8Array(reader.result));
        reader.onerror = () => reject(reader.error);
        reader.readAsArrayBuffer(file);
    });
}

/** Replace a file's extension. */
export function replaceExt(filename, newExt) {
    return filename.replace(/\.[^.]+$/, "") + "." + newExt;
}

/** Format bytes as human-readable string. */
export function fmtBytes(n) {
    if (n < 1024) return n + " B";
    if (n < 1024 * 1024) return (n / 1024).toFixed(1) + " KB";
    return (n / 1024 / 1024).toFixed(2) + " MB";
}

/** Format milliseconds as m:ss. */
export function fmtDuration(ms) {
    const s = Math.round(ms / 1000);
    return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}

// ── Drop-zone helper ──────────────────────────────────────────────

/**
 * Wire up a drop zone element.
 *
 * @param {HTMLElement} zone
 * @param {function(FileList)} onFiles
 */
export function setupDropZone(zone, onFiles) {
    zone.addEventListener("dragover", (e) => {
        e.preventDefault();
        zone.classList.add("drag-over");
    });
    zone.addEventListener("dragleave", () =>
        zone.classList.remove("drag-over")
    );
    zone.addEventListener("drop", (e) => {
        e.preventDefault();
        zone.classList.remove("drag-over");
        if (e.dataTransfer.files.length) onFiles(e.dataTransfer.files);
    });
}

/**
 * Replace a drop zone's prompt with a scrollable list of selected files.
 *
 * Requires the drop zone to contain:
 *   <div class="dz-prompt">…icon/label/hint…</div>
 *   <div class="dz-list hidden"></div>
 *
 * The hidden file <input> inside the zone is temporarily removed from the
 * pointer-event flow so random clicks don't reopen the picker.
 *
 * @param {HTMLElement}  zone      The .drop-zone element
 * @param {File[]}       files     Array of File objects to list
 * @param {object}       [opts]
 * @param {function}     [opts.onAdd]  If provided, an "Add" button is shown
 *                                     that calls this function when clicked.
 *                                     Omit for folder mode (no button).
 */
export function showFileList(zone, files, { onAdd } = {}) {
    const prompt = zone.querySelector(".dz-prompt");
    const list = zone.querySelector(".dz-list");
    const input = zone.querySelector('input[type="file"]');
    if (!prompt || !list) return;

    // Hide the invisible input overlay so clicks on the list aren't swallowed
    if (input) input.style.display = "none";

    prompt.classList.add("hidden");
    list.classList.remove("hidden");

    const escHtml = (s) =>
        String(s)
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;");

    const btnHtml = onAdd ? `<button class="dz-change-btn">+ Add</button>` : "";

    list.innerHTML = `
    <div class="dz-list-header">
      <span>${files.length} file${files.length !== 1 ? "s" : ""} selected</span>
      ${btnHtml}
    </div>
    <ul class="dz-file-list">
      ${files
          .map(
              (f) => `
        <li class="dz-file-row">
          <span class="dz-file-name">${escHtml(f.name)}</span>
          <span class="dz-file-size">${fmtBytes(f.size)}</span>
        </li>`
          )
          .join("")}
    </ul>`;

    if (onAdd) {
        list.querySelector(".dz-change-btn").addEventListener("click", onAdd);
    }
}

/**
 * Restore a drop zone to its empty/prompt state.
 * @param {HTMLElement} zone
 */
export function resetDropZone(zone) {
    const prompt = zone.querySelector(".dz-prompt");
    const list = zone.querySelector(".dz-list");
    const input = zone.querySelector('input[type="file"]');
    if (prompt) prompt.classList.remove("hidden");
    if (list) {
        list.classList.add("hidden");
        list.innerHTML = "";
    }
    if (input) input.style.display = "";
}

// ── ZIP builder ───────────────────────────────────────────────────

/**
 * Build a non-compressed (stored) ZIP archive.
 * @param {{ name: string, bytes: Uint8Array }[]} files
 * @returns {Uint8Array}
 */
export function buildZip(files) {
    const T = new Uint32Array(256);
    for (let i = 0; i < 256; i++) {
        let c = i;
        for (let j = 0; j < 8; j++)
            c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
        T[i] = c;
    }
    function crc32(data) {
        let c = 0xffffffff;
        for (const b of data) c = T[(c ^ b) & 0xff] ^ (c >>> 8);
        return (c ^ 0xffffffff) >>> 0;
    }

    const enc = new TextEncoder();
    const entries = [];
    const local = [];
    let dataOff = 0;

    for (const { name, bytes } of files) {
        const nb = enc.encode(name);
        const data =
            bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
        const crc = crc32(data);
        const sz = data.length;

        // Local file header (30 + filename)
        const lh = new DataView(new ArrayBuffer(30 + nb.length));
        let p = 0;
        lh.setUint32(p, 0x04034b50, true);
        p += 4;
        lh.setUint16(p, 20, true);
        p += 2; // version needed
        lh.setUint16(p, 0, true);
        p += 2; // flags
        lh.setUint16(p, 0, true);
        p += 2; // compression: stored
        lh.setUint16(p, 0, true);
        p += 2; // mod time
        lh.setUint16(p, 0, true);
        p += 2; // mod date
        lh.setUint32(p, crc, true);
        p += 4;
        lh.setUint32(p, sz, true);
        p += 4; // compressed size
        lh.setUint32(p, sz, true);
        p += 4; // uncompressed size
        lh.setUint16(p, nb.length, true);
        p += 2;
        lh.setUint16(p, 0, true);
        p += 2; // extra length
        new Uint8Array(lh.buffer, 30).set(nb);

        entries.push({ nb, crc, sz, localOff: dataOff });
        local.push(new Uint8Array(lh.buffer), data);
        dataOff += 30 + nb.length + sz;
    }

    // Central directory
    const cd = [];
    let cdSz = 0;
    for (const { nb, crc, sz, localOff } of entries) {
        const h = new DataView(new ArrayBuffer(46 + nb.length));
        let p = 0;
        h.setUint32(p, 0x02014b50, true);
        p += 4;
        h.setUint16(p, 20, true);
        p += 2; // made by
        h.setUint16(p, 20, true);
        p += 2; // needed
        h.setUint16(p, 0, true);
        p += 2; // flags
        h.setUint16(p, 0, true);
        p += 2; // compression
        h.setUint16(p, 0, true);
        p += 2; // mod time
        h.setUint16(p, 0, true);
        p += 2; // mod date
        h.setUint32(p, crc, true);
        p += 4;
        h.setUint32(p, sz, true);
        p += 4;
        h.setUint32(p, sz, true);
        p += 4;
        h.setUint16(p, nb.length, true);
        p += 2;
        h.setUint16(p, 0, true);
        p += 2; // extra
        h.setUint16(p, 0, true);
        p += 2; // comment
        h.setUint16(p, 0, true);
        p += 2; // disk start
        h.setUint16(p, 0, true);
        p += 2; // internal attrs
        h.setUint32(p, 0, true);
        p += 4; // external attrs
        h.setUint32(p, localOff, true);
        p += 4;
        new Uint8Array(h.buffer, 46).set(nb);
        cd.push(new Uint8Array(h.buffer));
        cdSz += 46 + nb.length;
    }

    // End of central directory (22 bytes)
    const eocd = new DataView(new ArrayBuffer(22));
    let p = 0;
    eocd.setUint32(p, 0x06054b50, true);
    p += 4;
    eocd.setUint16(p, 0, true);
    p += 2;
    eocd.setUint16(p, 0, true);
    p += 2;
    eocd.setUint16(p, entries.length, true);
    p += 2;
    eocd.setUint16(p, entries.length, true);
    p += 2;
    eocd.setUint32(p, cdSz, true);
    p += 4;
    eocd.setUint32(p, dataOff, true);
    p += 4;
    eocd.setUint16(p, 0, true);

    const all = [...local, ...cd, new Uint8Array(eocd.buffer)];
    const total = all.reduce((n, a) => n + a.length, 0);
    const zip = new Uint8Array(total);
    let off = 0;
    for (const a of all) {
        zip.set(a, off);
        off += a.length;
    }
    return zip;
}

// ── Web Worker pool ───────────────────────────────────────────────

/**
 * A fixed-size pool of Web Workers that share a message-based queue.
 *
 * Each worker runs worker.js, which hosts its own WASM instance.
 * Idle workers are assigned tasks immediately; busy workers queue
 * tasks until a slot opens. Results are delivered as Promises.
 *
 * @example
 * const pool = new WorkerPool(Math.min(files.length, navigator.hardwareConcurrency || 4));
 * const result = await pool.run({ type: 'tgs', data: buf, ... }, [buf]);
 * pool.terminate();
 */
export class WorkerPool {
    constructor(size = 1) {
        this._pending = new Map(); // id → { resolve, reject }
        this._queue = []; // tasks waiting for a free worker
        this._nextId = 0;
        // Append the cache-bust version token (set by clearWasmCache) so the
        // browser treats a fresh token as a new URL and re-fetches worker.js.
        const _v = localStorage.getItem("wasm_cache_v") || "";
        const workerUrl = new URL(
            "./worker.js" + (_v ? "?v=" + _v : ""),
            import.meta.url
        );
        this._workers = Array.from({ length: size }, () => {
            const w = new Worker(workerUrl, { type: "module" });
            w._idle = true;
            w.onmessage = ({ data }) => this._recv(w, data);
            w.onerror = (e) => this._recvError(w, e);
            return w;
        });
    }

    _recv(worker, data) {
        worker._idle = true;
        this._drain();
        const { id, error, ...payload } = data;
        const handlers = this._pending.get(id);
        if (!handlers) return;
        this._pending.delete(id);
        if (error) handlers.reject(new Error(error));
        else handlers.resolve(payload);
    }

    _recvError(worker, e) {
        // Worker crashed — reject the first pending task assigned to it
        // (we can't know the id, so we reject the oldest pending)
        worker._idle = true;
        this._drain();
        const [id, handlers] = this._pending.entries().next().value ?? [];
        if (handlers) {
            this._pending.delete(id);
            handlers.reject(new Error(`Worker error: ${e.message}`));
        }
    }

    _drain() {
        if (!this._queue.length) return;
        const w = this._workers.find((w) => w._idle);
        if (!w) return;
        const { id, msg, transfer, resolve, reject } = this._queue.shift();
        this._dispatch(w, id, msg, transfer, resolve, reject);
    }

    _dispatch(w, id, msg, transfer, resolve, reject) {
        w._idle = false;
        this._pending.set(id, { resolve, reject });
        w.postMessage({ ...msg, id }, transfer);
    }

    /**
     * Submit a task. Returns a Promise that resolves with the worker's reply.
     * Pass transferable objects (ArrayBuffers) in `transfer` for zero-copy.
     */
    run(msg, transfer = []) {
        return new Promise((resolve, reject) => {
            const id = this._nextId++;
            const w = this._workers.find((w) => w._idle);
            if (w) this._dispatch(w, id, msg, transfer, resolve, reject);
            else this._queue.push({ id, msg, transfer, resolve, reject });
        });
    }

    /** Terminate all workers immediately. */
    terminate() {
        this._workers.forEach((w) => w.terminate());
    }
}

// ── WASM cache ────────────────────────────────────────────────────

/**
 * Force the browser to re-fetch the WASM binary and its JS glue on the next
 * page load by writing a new version token into localStorage.  WorkerPool
 * reads this token and appends it as `?v=<token>` to the worker URL;
 * worker.js passes the same token through to every dynamic import() and to
 * the wasm-pack init() call.  Because the browser keys its module cache on
 * the full URL string, a new token → new URLs → guaranteed cache miss.
 */
export function clearWasmCache() {
    localStorage.setItem("wasm_cache_v", String(Date.now()));
    location.reload();
}

// ── Status-bar timer ─────────────────────────────────────────────

let _timerEl = null; // element currently being timed
let _timerStart = 0; // Date.now() at conversion start
let _timerId = null; // setInterval handle

/** Format elapsed milliseconds as "0s", "45s", "1m 03s", etc. */
function fmtElapsed(ms) {
    const s = Math.floor(ms / 1000);
    if (s < 60) return `${s}s`;
    return `${Math.floor(s / 60)}m ${String(s % 60).padStart(2, "0")}s`;
}

function _stopTimer() {
    if (_timerId !== null) {
        clearInterval(_timerId);
        _timerId = null;
    }
    _timerEl = null;
}

/**
 * Update a `.status-bar` element.
 *
 * While `state === 'busy'` a live elapsed-time counter is shown on the
 * right side of the bar.  The timer starts the first time `busy` is set
 * on an element and keeps running through subsequent `busy` calls (which
 * only update the progress message).  When the state changes to anything
 * other than `busy` the timer stops and the final elapsed time is
 * appended to the message.
 *
 * @param {HTMLElement} el
 * @param {'idle'|'busy'|'done'|'failed'} state
 * @param {string} message
 */
export function setStatus(el, state, message) {
    el.className = `status-bar ${state}`;
    const spinner =
        state === "busy"
            ? '<span class="spinner"></span>'
            : state === "done"
              ? "✓"
              : state === "failed"
                ? "✕"
                : "○";

    if (state === "busy") {
        // Start a fresh timer when we first enter the busy state on this
        // element; reuse it for subsequent progress updates.
        if (_timerEl !== el) {
            _stopTimer();
            _timerStart = Date.now();
            _timerEl = el;
            _timerId = setInterval(() => {
                // Re-query the span each tick — setStatus rewrites innerHTML on
                // every progress update so we must not hold a stale reference.
                const span =
                    _timerEl && _timerEl.querySelector(".status-elapsed");
                if (span)
                    span.textContent = fmtElapsed(Date.now() - _timerStart);
            }, 1000);
        }
        el.innerHTML =
            `${spinner} <span>${message}</span>` +
            `<span class="status-elapsed">${fmtElapsed(Date.now() - _timerStart)}</span>`;
    } else {
        // Capture final elapsed before clearing the timer.
        const elapsed =
            _timerId !== null
                ? ` · ${fmtElapsed(Date.now() - _timerStart)}`
                : "";
        _stopTimer();
        el.innerHTML = `${spinner} <span>${message}${elapsed}</span>`;
    }
}
