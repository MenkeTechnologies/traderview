// VPIN (Volume-synchronized Probability of Informed Trading) helpers
// shared by view + vitest.
//
// Backend body shape: { ticks: [{price, volume}, ...], config: {
// volume_per_bucket, window_buckets, return_window } }.

import { t as tr } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;

// Parses a two-column textarea: "price<sep>volume" per line. Returns
// `{ ticks, errors }`. Comment lines (#) and blank lines are skipped.
// Each line must have exactly two tokens; anything else is reported.
export function parseTickBlob(text) {
    const ticks = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { ticks, errors: [{ line_no: 0, raw: '', message: t('common.parse.input_must_be_string') }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const stripped = raw.trim();
        if (!stripped || stripped.startsWith('#')) continue;
        const parts = stripped.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 2) {
            errors.push({ line_no: i + 1, raw, message: `expected 2 tokens (price volume), got ${parts.length}` });
            continue;
        }
        const price = Number(parts[0]);
        const volume = Number(parts[1]);
        if (!Number.isFinite(price) || price <= 0) {
            errors.push({ line_no: i + 1, raw, message: `bad price "${parts[0]}"` });
            continue;
        }
        if (!Number.isFinite(volume) || volume < 0) {
            errors.push({ line_no: i + 1, raw, message: `bad volume "${parts[1]}"` });
            continue;
        }
        ticks.push({ price, volume });
    }
    return { ticks, errors };
}

export function validateInputs(ticks, config) {
    if (!Array.isArray(ticks) || ticks.length < 10)
        return tr('view.vpin.validate.ticks_min');
    if (!Number.isFinite(config.volume_per_bucket) || config.volume_per_bucket <= 0)
        return tr('view.vpin.validate.vpb');
    if (!Number.isInteger(config.window_buckets) || config.window_buckets < 1 || config.window_buckets > 2000)
        return tr('view.vpin.validate.window_buckets');
    if (!Number.isInteger(config.return_window) || config.return_window < 2 || config.return_window > 10_000)
        return tr('view.vpin.validate.return_window');
    const totalVol = ticks.reduce((a, t) => a + (t.volume || 0), 0);
    const expectedBuckets = totalVol / config.volume_per_bucket;
    if (expectedBuckets < 1)
        return tr('view.vpin.validate.vpb_too_large', { totalVol });
    return null;
}

export function buildBody(ticks, config) {
    return { ticks, config };
}

// Extracts the finished-VPIN series (drops null/None entries from the
// warmup window) for charting. Returns parallel `bucketIdx` + `vpin`
// arrays — the bucket index matters because the warmup buckets at the
// start are skipped.
export function extractFinishedVpin(report) {
    const xs = [], ys = [];
    if (!report || !Array.isArray(report.vpin)) return { xs, ys };
    for (let i = 0; i < report.vpin.length; i++) {
        const v = report.vpin[i];
        if (Number.isFinite(v)) {
            xs.push(i);
            ys.push(v);
        }
    }
    return { xs, ys };
}

// Computes a few summary scalars for the cards.
export function summarize(report, toxicThreshold = 0.5) {
    if (!report) return null;
    const finished = (report.vpin || []).filter(Number.isFinite);
    const nBuckets = report.vpin ? report.vpin.length : 0;
    const maxVpin = finished.length ? Math.max(...finished) : NaN;
    const avgVpin = finished.length ? finished.reduce((a, b) => a + b, 0) / finished.length : NaN;
    const toxicCount = Array.isArray(report.toxic_buckets) ? report.toxic_buckets.length : 0;
    const toxicPct = nBuckets > 0 ? toxicCount / nBuckets : 0;
    const totalBuy  = (report.bucket_buy_volume  || []).reduce((a, b) => a + b, 0);
    const totalSell = (report.bucket_sell_volume || []).reduce((a, b) => a + b, 0);
    const buySellSkew = (totalBuy + totalSell) > 0 ? (totalBuy - totalSell) / (totalBuy + totalSell) : 0;
    void toxicThreshold;
    return { nBuckets, maxVpin, avgVpin, toxicCount, toxicPct, totalBuy, totalSell, buySellSkew };
}

// Synthesizes a demo tick stream — used by the "Demo data" button so
// users can see the view with reasonable shapes before pasting their own.
// Mixes a benign random-walk regime with a toxic burst near the end.
export function makeDemoTicks(n = 1500, seed = 1) {
    let s = seed;
    const rand = () => { s = (s * 1664525 + 1013904223) | 0; return ((s >>> 0) / 0xffffffff); };
    const ticks = [];
    let price = 100;
    for (let i = 0; i < n; i++) {
        const inToxic = i > n * 0.75;
        const drift = inToxic ? 0.0008 : 0;
        const vol = inToxic ? 0.003 : 0.0008;
        const r = drift + vol * (rand() * 2 - 1);
        price = Math.max(0.01, price * Math.exp(r));
        const v = inToxic ? Math.round(800 + rand() * 1200) : Math.round(200 + rand() * 400);
        ticks.push({ price: Number(price.toFixed(2)), volume: v });
    }
    return ticks;
}

export function fmtN(v, digits = 3) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(digits);
}

export function fmtPct(v) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(1) + '%';
}
