/**
 * worker.js — WASM runner for Web Worker threads.
 *
 * Each worker owns its own WASM instance. The main thread dispatches one file
 * per message; the worker posts back the result (or an error string).
 *
 * Message protocol
 * ----------------
 * In  (main → worker):
 *   { id, type: 'ncm', data: ArrayBuffer }
 *   { id, type: 'tgs', data: ArrayBuffer,
 *     fps, width, height, maxFrames, frameStart, frameEnd, format }
 *
 * Out (worker → main):
 *   NCM success: { id, audio: ArrayBuffer, format, metadata_json,
 *                  cover: ArrayBuffer, cover_mime }
 *   TGS success: { id, output: ArrayBuffer, format }
 *   Any error:   { id, error: <string> }
 *
 * ArrayBuffers are transferred (zero-copy) in both directions.
 */

import init, { ncm_convert, tgs_convert } from './pkg/uwasm_toolbox.js';

let ready = false;
const queue = [];

// Initialise WASM once, then flush any messages that arrived early.
(async () => {
  await init();
  ready = true;
  queue.splice(0).forEach(handle);
})();

self.onmessage = ({ data: msg }) => {
  if (!ready) { queue.push(msg); return; }
  handle(msg);
};

async function handle(msg) {
  const { id, type } = msg;
  try {
    if (type === 'ncm') {
      const r = await ncm_convert(new Uint8Array(msg.data));
      // .audio / .cover are copies of the WASM Vec<u8> — safe to transfer
      const audio = r.audio;
      const cover = r.cover;
      self.postMessage(
        { id, audio: audio.buffer, format: r.format,
          metadata_json: r.metadata_json,
          cover: cover.buffer, cover_mime: r.cover_mime },
        [audio.buffer, cover.buffer],
      );

    } else if (type === 'tgs') {
      const out = tgs_convert(
        new Uint8Array(msg.data),
        msg.fps, msg.width, msg.height,
        msg.maxFrames, msg.frameStart, msg.frameEnd, msg.format,
      );
      self.postMessage(
        { id, output: out.buffer, format: msg.format },
        [out.buffer],
      );

    } else {
      self.postMessage({ id, error: `unknown message type: ${type}` });
    }
  } catch (err) {
    self.postMessage({ id, error: String(err) });
  }
}
