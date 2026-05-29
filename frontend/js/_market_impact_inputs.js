// Market Impact helpers shared by view + vitest.
//
// Backend body shape: { trades: [{qty, adv, slippage_bps}, ...],
// spike_bps: f64 }.

const TOKEN_DELIM = /[\s,]+/;

// Parses three-token-per-line trades: "qty adv slippage_bps". Lines
// starting with # and blanks skipped. Per-line errors are tagged.
// Validation: qty > 0, adv > 0, slippage_bps finite (signed OK — a
// favorable fill is a negative bps).
export function parseTradeBlob(text) {
    const trades = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { trades, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.trim();
        if (!s || s.startsWith('#')) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 3) {
            errors.push({ line_no: i + 1, raw, message: `expected 3 tokens (qty adv slip_bps), got ${parts.length}` });
            continue;
        }
        const qty = Number(parts[0]);
        const adv = Number(parts[1]);
        const slip = Number(parts[2]);
        if (!Number.isFinite(qty) || qty <= 0) {
            errors.push({ line_no: i + 1, raw, message: `qty must be > 0` });
            continue;
        }
        if (!Number.isFinite(adv) || adv <= 0) {
            errors.push({ line_no: i + 1, raw, message: `adv must be > 0` });
            continue;
        }
        if (!Number.isFinite(slip)) {
            errors.push({ line_no: i + 1, raw, message: `slippage_bps must be finite` });
            continue;
        }
        trades.push({ qty, adv, slippage_bps: slip });
    }
    return { trades, errors };
}

export function validateInputs(trades, spikeBps) {
    if (!Array.isArray(trades) || trades.length < 5)
        return 'need at least 5 trades';
    if (!Number.isFinite(spikeBps) || spikeBps <= 0)
        return 'spike_bps must be > 0';
    return null;
}

export function buildBody(trades, spikeBps) {
    return { trades, spike_bps: spikeBps };
}

// Computes per-bucket participation % from raw trades for the histogram
// (separate from backend slippage stats — useful for the "where am I
// trading?" volume view).
export const BUCKET_LABELS = [
    '< 0.1% ADV',
    '0.1-0.5% ADV',
    '0.5-1% ADV',
    '1-5% ADV',
    '5-10% ADV',
    '> 10% ADV',
];

const BUCKET_CAPS = [0.001, 0.005, 0.01, 0.05, 0.10, Infinity];

export function bucketIndex(participationPct) {
    for (let i = 0; i < BUCKET_CAPS.length; i++) {
        if (participationPct <= BUCKET_CAPS[i]) return i;
    }
    return BUCKET_CAPS.length - 1;
}

// Aggregates raw trade participations into bucket counts for the
// "where my trades land" histogram. Mirrors backend's binning logic
// so view labels stay consistent.
export function bucketParticipations(trades) {
    const counts = new Array(BUCKET_LABELS.length).fill(0);
    for (const t of trades) {
        if (!Number.isFinite(t.qty) || !Number.isFinite(t.adv) || t.adv <= 0) continue;
        const i = bucketIndex(t.qty / t.adv);
        counts[i]++;
    }
    return counts;
}

// Synthesizes a deterministic trade stream that *clearly* exhibits the
// slippage cliff: tiny trades have ~2 bps cost, big trades (>5% ADV)
// blow out to 80+ bps. Used by the "Demo" button.
export function makeDemoTrades(seed = 1) {
    let s = seed;
    const rand = () => { s = (s * 1664525 + 1013904223) | 0; return ((s >>> 0) / 0xffffffff); };
    const adv = 5_000_000;  // 5M-share daily volume — a mid-cap
    const trades = [];
    for (let i = 0; i < 400; i++) {
        // Vary qty across all participation bands.
        const r = rand();
        let qty;
        if      (r < 0.40) qty = Math.round(500   + rand() * 4500);        // < 0.1%
        else if (r < 0.65) qty = Math.round(5000  + rand() * 20_000);      // 0.1-0.5%
        else if (r < 0.80) qty = Math.round(25_000 + rand() * 25_000);     // 0.5-1%
        else if (r < 0.92) qty = Math.round(50_000 + rand() * 200_000);    // 1-5%
        else if (r < 0.98) qty = Math.round(250_000 + rand() * 250_000);   // 5-10%
        else               qty = Math.round(500_000 + rand() * 500_000);   // > 10%
        const pct = qty / adv;
        // Quadratic + noise — flat ~2 bps below 0.5% ADV, then square-root above.
        const baseline = 2 + 8 * Math.sqrt(Math.max(0, pct - 0.005)) * 100;
        const slip = baseline + (rand() - 0.5) * 4;
        trades.push({ qty, adv, slippage_bps: Number(slip.toFixed(2)) });
    }
    return trades;
}

export function fmtBps(v) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(1) + ' bps';
}

export function fmtN(v) {
    if (!Number.isFinite(v)) return '—';
    return v.toLocaleString('en-US');
}
