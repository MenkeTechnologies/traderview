// Real-time event stream client.
//
// Connects to /ws once per session; reconnects with exponential backoff
// (max 30s) on close. Subscribers register via on(eventType, callback).
// Subscription happens BEFORE connection, so callbacks added during boot
// still receive every event.

const subs = new Map();    // type -> Set<fn>
let socket = null;
let backoff = 1000;
let connected = false;

export function on(type, fn) {
    if (!subs.has(type)) subs.set(type, new Set());
    subs.get(type).add(fn);
    return () => subs.get(type).delete(fn);
}

export function isConnected() { return connected; }

export function startWs() {
    if (socket) return;
    connect();
}

function connect() {
    const token = localStorage.getItem('tv-token') || '';
    // Same origin, ws:// or wss:// based on current protocol.
    const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = window.location.host || 'localhost';
    const tokenQs = token ? `?token=${encodeURIComponent(token)}` : '';
    const url = `${proto}//${host}/api/ws${tokenQs}`;

    let ws;
    try { ws = new WebSocket(url); }
    catch (e) {
        console.warn('ws: construction failed', e);
        scheduleReconnect();
        return;
    }
    socket = ws;

    ws.addEventListener('open', () => {
        connected = true;
        backoff = 1000;
        dispatch({ type: '_open' });
    });
    ws.addEventListener('close', () => {
        connected = false;
        socket = null;
        dispatch({ type: '_close' });
        scheduleReconnect();
    });
    ws.addEventListener('error', () => {
        // close handler will fire too — let it do the reconnect.
    });
    ws.addEventListener('message', (ev) => {
        let data;
        try { data = JSON.parse(ev.data); }
        catch (_) { return; }
        if (!data || typeof data.type !== 'string') return;
        dispatch(data);
    });
}

function dispatch(msg) {
    const fns = subs.get(msg.type);
    if (!fns) return;
    for (const fn of fns) {
        try { fn(msg); }
        catch (e) { console.warn('ws sub error', e); }
    }
}

function scheduleReconnect() {
    setTimeout(connect, backoff);
    backoff = Math.min(backoff * 2, 30_000);
}
