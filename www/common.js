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

// ── Status bar helper ──────────────────────────────────────────────

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
