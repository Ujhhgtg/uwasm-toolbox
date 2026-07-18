/**
 * common.js — shared WASM initialisation and small utilities
 */

let _wasm = null;

/** Initialise the WASM module once and cache it. */
export async function initWasm() {
  if (_wasm) return _wasm;
  const mod = await import('./pkg/uwasm_toolbox.js');
  await mod.default();
  _wasm = mod;
  return _wasm;
}

/** Returns the cached WASM module (call initWasm first). */
export function wasm() { return _wasm; }

// ── File / Blob utilities ──────────────────────────────────────────

/** Trigger a browser download for `bytes` with the given filename. */
export function downloadBytes(bytes, filename, mime = 'application/octet-stream') {
  const blob = new Blob([bytes], { type: mime });
  const url  = URL.createObjectURL(blob);
  const a    = document.createElement('a');
  a.href     = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  setTimeout(() => { URL.revokeObjectURL(url); a.remove(); }, 1000);
}

/** Read a File as a Uint8Array. */
export function readFileBytes(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload  = () => resolve(new Uint8Array(reader.result));
    reader.onerror = ()  => reject(reader.error);
    reader.readAsArrayBuffer(file);
  });
}

/** Replace a file's extension. */
export function replaceExt(filename, newExt) {
  return filename.replace(/\.[^.]+$/, '') + '.' + newExt;
}

/** Format bytes as human-readable string. */
export function fmtBytes(n) {
  if (n < 1024) return n + ' B';
  if (n < 1024 * 1024) return (n / 1024).toFixed(1) + ' KB';
  return (n / 1024 / 1024).toFixed(2) + ' MB';
}

/** Format milliseconds as m:ss. */
export function fmtDuration(ms) {
  const s = Math.round(ms / 1000);
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
}

// ── Drop-zone helper ──────────────────────────────────────────────

/**
 * Wire up a drop zone element.
 *
 * @param {HTMLElement} zone
 * @param {function(FileList)} onFiles
 */
export function setupDropZone(zone, onFiles) {
  zone.addEventListener('dragover', e => {
    e.preventDefault();
    zone.classList.add('drag-over');
  });
  zone.addEventListener('dragleave', () => zone.classList.remove('drag-over'));
  zone.addEventListener('drop', e => {
    e.preventDefault();
    zone.classList.remove('drag-over');
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
  const prompt = zone.querySelector('.dz-prompt');
  const list   = zone.querySelector('.dz-list');
  const input  = zone.querySelector('input[type="file"]');
  if (!prompt || !list) return;

  // Hide the invisible input overlay so clicks on the list aren't swallowed
  if (input) input.style.display = 'none';

  prompt.classList.add('hidden');
  list.classList.remove('hidden');

  const escHtml = s => String(s)
    .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

  const btnHtml = onAdd
    ? `<button class="dz-change-btn">+ Add</button>`
    : '';

  list.innerHTML = `
    <div class="dz-list-header">
      <span>${files.length} file${files.length !== 1 ? 's' : ''} selected</span>
      ${btnHtml}
    </div>
    <ul class="dz-file-list">
      ${files.map(f => `
        <li class="dz-file-row">
          <span class="dz-file-name">${escHtml(f.name)}</span>
          <span class="dz-file-size">${fmtBytes(f.size)}</span>
        </li>`).join('')}
    </ul>`;

  if (onAdd) {
    list.querySelector('.dz-change-btn').addEventListener('click', onAdd);
  }
}

/**
 * Restore a drop zone to its empty/prompt state.
 * @param {HTMLElement} zone
 */
export function resetDropZone(zone) {
  const prompt = zone.querySelector('.dz-prompt');
  const list   = zone.querySelector('.dz-list');
  const input  = zone.querySelector('input[type="file"]');
  if (prompt) prompt.classList.remove('hidden');
  if (list)   { list.classList.add('hidden'); list.innerHTML = ''; }
  if (input)  input.style.display = '';
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
    this._queue   = [];        // tasks waiting for a free worker
    this._nextId  = 0;
    // Resolve the worker URL relative to this module
    const workerUrl = new URL('./worker.js', import.meta.url);
    this._workers = Array.from({ length: size }, () => {
      const w  = new Worker(workerUrl, { type: 'module' });
      w._idle  = true;
      w.onmessage = ({ data }) => this._recv(w, data);
      w.onerror   = (e)       => this._recvError(w, e);
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
    else       handlers.resolve(payload);
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
    const w = this._workers.find(w => w._idle);
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
      const w  = this._workers.find(w => w._idle);
      if (w) this._dispatch(w, id, msg, transfer, resolve, reject);
      else   this._queue.push({ id, msg, transfer, resolve, reject });
    });
  }

  /** Terminate all workers immediately. */
  terminate() { this._workers.forEach(w => w.terminate()); }
}


/**
 * Update a `.status-bar` element.
 * @param {HTMLElement} el
 * @param {'idle'|'busy'|'done'|'failed'} state
 * @param {string} message
 */
export function setStatus(el, state, message) {
  el.className = `status-bar ${state}`;
  const spinner = state === 'busy'
    ? '<span class="spinner"></span>'
    : state === 'done'   ? '✓'
    : state === 'failed' ? '✕'
    : '○';
  el.innerHTML = `${spinner} <span>${message}</span>`;
}
