// Squeeze Alerts — pure detection engine + persistent settings.
//
// A "squeeze" here is detected from a stream of (symbol, timestamp,
// price, volume) ticks. For each symbol the engine maintains:
//   * Rolling price baseline (oldest tick within `window_seconds`).
//   * Rolling cumulative volume vs the average daily volume input.
// A squeeze fires when BOTH thresholds are crossed:
//   price_change_pct >= price_threshold_pct (default 5%)
//   volume_multiplier >= volume_threshold (default 2× ADV)
// Plus a per-symbol cooldown so a sustained squeeze doesn't re-alert
// on every tick.

const TOKEN_DELIM = /[\s,]+/;
const SETTINGS_KEY = 'tv-squeeze-alerts-v1';

export const DEFAULT_SETTINGS = {
    price_threshold_pct: 0.05,    // 5%
    volume_threshold:    2.0,      // 2× ADV
    window_seconds:      300,      // 5-minute lookback
    cooldown_seconds:    60,       // re-alert no sooner than 1 min later
    bell_enabled:        true,
    tts_enabled:         true,
    sound_volume:        0.30,
    use_alarm_for_critical: true,  // alarm chime when pct ≥ 2× threshold
    watchlist:           [],       // empty = all symbols qualify
};

// ── Tick parsing ──────────────────────────────────────────────────

// One tick per line: `symbol timestamp_seconds price volume`.
// timestamp_seconds is a Unix epoch seconds value (integer or decimal).
export function parseTickBlob(text) {
    const ticks = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { ticks, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.trim();
        if (!s || s.startsWith('#')) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 4) {
            errors.push({ line_no: i + 1, raw, message: `expected 4 tokens (symbol ts_sec price volume), got ${parts.length}` });
            continue;
        }
        const symbol = parts[0].toUpperCase();
        const ts = Number(parts[1]);
        const price = Number(parts[2]);
        const volume = Number(parts[3]);
        if (!/^[A-Z0-9._-]+$/.test(symbol)) {
            errors.push({ line_no: i + 1, raw, message: `bad symbol "${parts[0]}"` });
            continue;
        }
        if (!Number.isFinite(ts) || ts < 0) {
            errors.push({ line_no: i + 1, raw, message: `timestamp must be ≥ 0` });
            continue;
        }
        if (!Number.isFinite(price) || price <= 0) {
            errors.push({ line_no: i + 1, raw, message: `price must be > 0` });
            continue;
        }
        if (!Number.isFinite(volume) || volume < 0) {
            errors.push({ line_no: i + 1, raw, message: `volume must be ≥ 0` });
            continue;
        }
        ticks.push({ symbol, ts, price, volume });
    }
    return { ticks, errors };
}

// One ADV per line: `symbol adv`. ADV in shares.
export function parseAdvBlob(text) {
    const adv = {};
    const errors = [];
    if (typeof text !== 'string') {
        return { adv, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.trim();
        if (!s || s.startsWith('#')) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 2) {
            errors.push({ line_no: i + 1, raw, message: `expected 2 tokens (symbol adv), got ${parts.length}` });
            continue;
        }
        const symbol = parts[0].toUpperCase();
        const v = Number(parts[1]);
        if (!/^[A-Z0-9._-]+$/.test(symbol)) {
            errors.push({ line_no: i + 1, raw, message: `bad symbol "${parts[0]}"` });
            continue;
        }
        if (!Number.isFinite(v) || v <= 0) {
            errors.push({ line_no: i + 1, raw, message: `adv must be > 0` });
            continue;
        }
        adv[symbol] = v;
    }
    return { adv, errors };
}

// ── Settings persistence ─────────────────────────────────────────

export function loadSettings(storage = globalThis.localStorage) {
    if (!storage) return { ...DEFAULT_SETTINGS };
    try {
        const s = storage.getItem(SETTINGS_KEY);
        if (!s) return { ...DEFAULT_SETTINGS };
        const parsed = JSON.parse(s);
        return migrateSettings(parsed);
    } catch {
        return { ...DEFAULT_SETTINGS };
    }
}

export function saveSettings(settings, storage = globalThis.localStorage) {
    if (!storage) return false;
    try {
        storage.setItem(SETTINGS_KEY, JSON.stringify(settings));
        return true;
    } catch { return false; }
}

export function migrateSettings(raw) {
    if (!raw || typeof raw !== 'object') return { ...DEFAULT_SETTINGS };
    const out = { ...DEFAULT_SETTINGS };
    for (const k of Object.keys(DEFAULT_SETTINGS)) {
        if (k in raw) {
            const v = raw[k];
            const def = DEFAULT_SETTINGS[k];
            if (typeof def === 'boolean' && typeof v === 'boolean') out[k] = v;
            else if (typeof def === 'number' && Number.isFinite(v) && v >= 0) out[k] = v;
            else if (Array.isArray(def) && Array.isArray(v)) out[k] = v.filter(x => typeof x === 'string');
        }
    }
    return out;
}

// ── Watchlist gating ─────────────────────────────────────────────

// Empty watchlist = all symbols qualify. Non-empty = only listed
// symbols (case-insensitive comparison).
export function isWatched(symbol, watchlist) {
    if (!Array.isArray(watchlist) || watchlist.length === 0) return true;
    const u = String(symbol || '').toUpperCase();
    return watchlist.some(w => String(w).toUpperCase() === u);
}

// ── Detection engine ─────────────────────────────────────────────

// Processes a tick stream chronologically and returns the list of
// squeeze events that would have fired given the settings + ADV table.
// Pure compute — same input always returns same output. Used both for
// historical replay and (later) live-feed scoring.
//
// Output event shape:
//   { symbol, ts, price_change_pct, volume_multiplier, severity }
//   severity ∈ { "normal", "critical" }
export function detectSqueezes(ticks, advMap, settings) {
    const events = [];
    if (!Array.isArray(ticks) || ticks.length === 0) return events;
    const s = { ...DEFAULT_SETTINGS, ...settings };
    // Per-symbol state: array of recent ticks within window + last alert ts.
    const state = new Map();
    // Sort defensively.
    const sorted = [...ticks].sort((a, b) => a.ts - b.ts);
    for (const t of sorted) {
        if (!isWatched(t.symbol, s.watchlist)) continue;
        const adv = advMap?.[t.symbol];
        if (!Number.isFinite(adv) || adv <= 0) continue;
        let st = state.get(t.symbol);
        if (!st) {
            st = { window: [], last_alert_ts: -Infinity };
            state.set(t.symbol, st);
        }
        // Drop ticks older than window_seconds.
        const cutoff = t.ts - s.window_seconds;
        while (st.window.length && st.window[0].ts < cutoff) st.window.shift();
        st.window.push(t);
        if (st.window.length < 2) continue;
        const oldest = st.window[0];
        const priceChange = (t.price - oldest.price) / oldest.price;
        const cumVolume = st.window.reduce((a, x) => a + (x.volume || 0), 0);
        // Volume-multiplier: cumulative window volume vs ADV scaled to the
        // window's duration (so a 5-min cum vol vs ADV gets the right ratio).
        const winSecs = Math.max(1, t.ts - oldest.ts);
        const advForWindow = adv * (winSecs / 23400);   // 6.5h trading day
        const volMult = advForWindow > 0 ? cumVolume / advForWindow : 0;
        if (priceChange < s.price_threshold_pct) continue;
        if (volMult < s.volume_threshold) continue;
        if (t.ts - st.last_alert_ts < s.cooldown_seconds) continue;
        const critical = priceChange >= s.price_threshold_pct * 2 ||
                         volMult     >= s.volume_threshold    * 2;
        events.push({
            symbol: t.symbol,
            ts: t.ts,
            price_change_pct: priceChange,
            volume_multiplier: volMult,
            severity: critical ? 'critical' : 'normal',
        });
        st.last_alert_ts = t.ts;
    }
    return events;
}

// ── Demo data ────────────────────────────────────────────────────

// Deterministic synthetic feed: 3 symbols over 30 minutes (1800s) with
// AAPL ramping +8% in the last 5 min on 4× volume, MSFT drifting
// sideways (no alert), and SMID popping +12% on 6× volume (CRITICAL).
export function makeDemoData() {
    const ticks = [];
    const baseTs = 1_700_000_000;
    // AAPL: 0-1500s flat, 1500-1800s spike +8% on heavy volume. Spike
    // volume sized so 5-min cumulative cleanly exceeds 2× the per-window-
    // prorated ADV (50M × 300/23400 ≈ 641k) — i.e., ~2.2M+ over 11 ticks.
    for (let s = 0; s < 1800; s += 30) {
        const inSpike = s >= 1500;
        const price = inSpike
            ? 150 + (s - 1500) / 300 * 12         // ramp 150 → 162 over 5 min
            : 150 + Math.sin(s / 120) * 0.4;      // gentle wiggle
        const volume = inSpike ? 200_000 : 5000;  // 11 spike ticks × 200k = 2.2M
        ticks.push({ symbol: 'AAPL', ts: baseTs + s, price: round2(price), volume });
    }
    // MSFT: just noise — should NOT alert.
    for (let s = 0; s < 1800; s += 30) {
        const price = 320 + Math.sin(s / 60) * 0.5;
        ticks.push({ symbol: 'MSFT', ts: baseTs + s, price: round2(price), volume: 3000 });
    }
    // SMID: low-cap that gaps +12% on huge relative volume — critical squeeze.
    for (let s = 0; s < 1800; s += 30) {
        const inSpike = s >= 1200;
        const price = inSpike
            ? 8 + (s - 1200) / 600 * 1.0          // ramp 8 → 9 over 10 min (= +12.5%)
            : 8 + Math.sin(s / 90) * 0.05;
        const volume = inSpike ? 20000 : 200;
        ticks.push({ symbol: 'SMID', ts: baseTs + s, price: round2(price), volume });
    }
    const adv = { AAPL: 50_000_000, MSFT: 25_000_000, SMID: 250_000 };
    return { ticks, adv };
}

function round2(v) { return Math.round(v * 100) / 100; }

// ── Formatters ───────────────────────────────────────────────────

export function fmtPct(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v >= 0 ? '+' : '';
    return sign + (v * 100).toFixed(2) + '%';
}

export function fmtMult(v) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(1) + '×';
}

export function fmtTime(ts) {
    if (!Number.isFinite(ts)) return '—';
    const d = new Date(ts * 1000);
    return d.toISOString().slice(11, 19);   // HH:MM:SS
}
