// VWAP Slippage helpers shared by view + vitest.
//
// Backend body shape: { side: "long"|"short", fill_price: Decimal-string,
// bars: [{typical: Decimal-string, volume: Decimal-string}, ...] }.
// Backend returns a tagged-enum: { kind: "computed", ... } or
// { kind: "empty", reason: "..." }.

const TOKEN_DELIM = /[\s,]+/;

// Two-token-per-line "typical volume". Typical = (high+low+close)/3; the
// view leaves it to the caller to pre-compute so the input format mirrors
// what an OMS or TCA pipeline would already have on hand.
export function parseBarBlob(text) {
    const bars = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { bars, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.trim();
        if (!s || s.startsWith('#')) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 2) {
            errors.push({ line_no: i + 1, raw, message: `expected 2 tokens (typical volume), got ${parts.length}` });
            continue;
        }
        const typical = Number(parts[0]);
        const volume = Number(parts[1]);
        if (!Number.isFinite(typical) || typical <= 0) {
            errors.push({ line_no: i + 1, raw, message: `typical price must be > 0` });
            continue;
        }
        if (!Number.isFinite(volume) || volume < 0) {
            errors.push({ line_no: i + 1, raw, message: `volume must be ≥ 0` });
            continue;
        }
        bars.push({ typical, volume });
    }
    return { bars, errors };
}

export function validateInputs(side, fillPrice, bars) {
    if (side !== 'long' && side !== 'short') return 'side must be long or short';
    if (!Number.isFinite(fillPrice) || fillPrice <= 0) return 'fill_price must be > 0';
    if (!Array.isArray(bars) || bars.length === 0) return 'need at least 1 bar';
    const totalVol = bars.reduce((a, b) => a + (b.volume || 0), 0);
    if (totalVol <= 0) return 'total bar volume must be > 0';
    return null;
}

export function buildBody(side, fillPrice, bars) {
    // Backend deserializes Decimals from strings; pre-format here so the
    // wire payload uses the canonical type the rust_decimal crate expects.
    return {
        side,
        fill_price: String(fillPrice),
        bars: bars.map(b => ({
            typical: String(b.typical),
            volume:  String(b.volume),
        })),
    };
}

// Computes VWAP locally for sanity-checking + chart rendering before the
// backend round-trip lands. Mirrors backend's Σ(typical·volume) / Σ(volume).
export function localVwap(bars) {
    if (!Array.isArray(bars) || bars.length === 0) return NaN;
    let num = 0, den = 0;
    for (const b of bars) {
        if (!Number.isFinite(b.typical) || !Number.isFinite(b.volume) || b.volume < 0) continue;
        num += b.typical * b.volume;
        den += b.volume;
    }
    return den > 0 ? num / den : NaN;
}

// Per-bar rolling VWAP series for the chart. Bar i's value = VWAP over
// bars[0..=i]. Lets the trader visually trace where VWAP was at any point
// in the open window.
export function rollingVwap(bars) {
    const out = [];
    if (!Array.isArray(bars)) return out;
    let num = 0, den = 0;
    for (const b of bars) {
        if (!Number.isFinite(b.typical) || !Number.isFinite(b.volume) || b.volume < 0) {
            out.push(null);
            continue;
        }
        num += b.typical * b.volume;
        den += b.volume;
        out.push(den > 0 ? num / den : null);
    }
    return out;
}

// Unwraps the backend's tagged enum into `{ ok, result?, reason? }`. The
// view treats `ok=false` as a soft error (renders the reason) rather
// than a hard crash.
export function unwrapResponse(resp) {
    if (!resp || typeof resp !== 'object') return { ok: false, reason: 'malformed response' };
    if (resp.kind === 'computed') return { ok: true, result: resp };
    if (resp.kind === 'empty')    return { ok: false, reason: resp.reason || 'empty' };
    return { ok: false, reason: 'unknown response kind' };
}

// Backend Decimal scalars come back as strings; normalize for display +
// math without forcing Decimal arithmetic in the frontend.
export function decToNum(v) {
    if (v == null) return NaN;
    if (typeof v === 'number') return v;
    const n = Number(v);
    return Number.isFinite(n) ? n : NaN;
}

// Deterministic demo: 200-bar typical/volume series where the trader's
// fill beats VWAP. Designed to land beat_vwap=true with non-trivial bps.
export function makeDemoData(seed = 1) {
    let s = seed;
    const rand = () => { s = (s * 1664525 + 1013904223) | 0; return ((s >>> 0) / 0xffffffff); };
    const bars = new Array(200);
    let typical = 100;
    for (let i = 0; i < 200; i++) {
        typical = Math.max(0.01, typical + (rand() - 0.45) * 0.08);
        const volume = Math.round(500 + rand() * 4500);
        bars[i] = { typical: Number(typical.toFixed(4)), volume };
    }
    // Long trade — fill at a deliberate ~12 bps discount to the average typical.
    const avgTypical = bars.reduce((a, b) => a + b.typical, 0) / bars.length;
    const fill_price = Number((avgTypical * 0.9988).toFixed(4));
    return { side: 'long', fill_price, bars };
}

export function fmtN(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtBps(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v >= 0 ? '+' : '';
    return sign + v.toFixed(1) + ' bps';
}

export function fmtVol(v) {
    if (!Number.isFinite(v)) return '—';
    return v.toLocaleString('en-US');
}
