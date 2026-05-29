// Browser-side error reporter — sends every uncaught error / unhandled
// promise rejection / failed API call to /api/client-errors so it lands in
// the server log alongside Rust traces. Auth-less by design: boot errors
// fire before the token is available, and the backend binds to localhost
// only.

import { BoundedQueue } from './bounded_queue.js';

let inflight = 0;
const MAX_INFLIGHT = 4;     // don't fan-out forever if the network is dead
const MAX_QUEUE = 200;      // bound queue so a console-error loop can't OOM us
const queue = new BoundedQueue(MAX_QUEUE);

function send(payload) {
    // Always include the current view so we can correlate.
    payload.view = (location.hash || '#?').slice(1);
    payload.href = location.href;
    payload.ua = navigator.userAgent;
    // Periodically rewrite the outgoing payload's `extra` to surface that we
    // lost frames — without spamming about the drop itself.
    if (queue.dropped > 0 && queue.dropped % 50 === 1) {
        payload.extra = Object.assign({}, payload.extra || {}, {
            dropped_total: queue.dropped,
        });
    }
    queue.push(payload);
    drain();
}

function endpoint() {
    // After initApi(), window.__tvApiBase is the real backend
    // (http://127.0.0.1:<port> in desktop, '' in web). Until then we queue.
    const base = window.__tvApiBase;
    if (base === undefined) return null;  // not yet initialized — queue
    return `${base}/api/client-errors`;
}

function drain() {
    const url = endpoint();
    if (!url && url !== '') return; // not initialized yet; keep queued
    while (inflight < MAX_INFLIGHT && queue.length) {
        const body = queue.shift();
        inflight++;
        // No sendBeacon — it throws SecurityError when called from a custom
        // scheme (tauri://localhost) to a different origin (http://127.0.0.1).
        // Plain fetch with keepalive works in all WebKit contexts and is
        // already allowed by our CSP connect-src list.
        try {
            fetch(url, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(body),
                keepalive: true,
            })
                .catch(() => {})
                .finally(() => { inflight--; drain(); });
        } catch (_) {
            // Synchronous SecurityError / CSP block — drop and continue so
            // the reporter itself never crashes the app.
            inflight--;
        }
    }
}

// Drain whenever initApi finishes (it sets window.__tvApiBase).
const __drainPoll = setInterval(() => {
    if (window.__tvApiBase !== undefined) {
        clearInterval(__drainPoll);
        drain();
    }
}, 250);

window.addEventListener('error', (e) => {
    send({
        kind: 'error',
        message: e.message || (e.error && e.error.message) || 'window error',
        stack: (e.error && e.error.stack) || '',
        src: e.filename || '',
        line: e.lineno | 0,
        col: e.colno | 0,
    });
});

window.addEventListener('unhandledrejection', (e) => {
    const r = e.reason;
    const message = (r && (r.message || String(r))) || 'unhandled rejection';
    const stack = (r && r.stack) || '';
    send({ kind: 'unhandledrejection', message, stack });
});

export function reportApiFail(method, path, status, body) {
    send({
        kind: 'api-fail',
        message: `${method} ${path} → ${status}`,
        extra: { method, path, status, body: String(body).slice(0, 1024) },
    });
    // Fan out to any UI listener (e.g. a global toast handler). Detail is
    // intentionally minimal — listeners that want the body can read the
    // server log instead.
    try {
        window.dispatchEvent(new CustomEvent('tv:api-error', {
            detail: { method, path, status },
        }));
    } catch (_) { /* SSR / no DOM */ }
}

// Bring console.error into the same pipeline so plain `console.error()`
// calls show up in the server log too. Original behavior preserved.
const _err = console.error.bind(console);
console.error = function (...args) {
    try {
        send({
            kind: 'log',
            message: args.map(a =>
                (a && a.stack) ? a.stack
              : (typeof a === 'string') ? a
              : JSON.stringify(a)
            ).join(' '),
        });
    } catch (_) {}
    _err(...args);
};
