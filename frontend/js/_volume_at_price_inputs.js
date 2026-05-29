// Volume-at-price (Volume Profile) helpers.
//
// Backend body: { bars: [{high, low, volume}, ...],
//                 num_bins: number, value_area_pct: number }
// Returns: { bins: [{center, volume}], poc_index, value_area_high,
//   value_area_low, total_volume } — empty fields if validation failed.

export const DEFAULT_NUM_BINS = 50;
export const DEFAULT_VA_PCT = 70.0;

export const DEFAULT_INPUTS = {
    bars: [],
    num_bins: DEFAULT_NUM_BINS,
    value_area_pct: DEFAULT_VA_PCT,
};

export function validateInputs(input) {
    if (!Array.isArray(input.bars))                                 return 'bars must be an array';
    for (let i = 0; i < input.bars.length; i++) {
        const b = input.bars[i];
        if (!b || typeof b !== 'object')                           return `bars[${i}] must be an object`;
        if (!Number.isFinite(b.high) || !Number.isFinite(b.low) || !Number.isFinite(b.volume))
                                                                    return `bars[${i}] has non-finite field`;
        if (b.volume < 0)                                          return `bars[${i}].volume must be ≥ 0`;
        if (b.high < b.low)                                        return `bars[${i}].high must be ≥ low`;
    }
    if (!Number.isInteger(input.num_bins) || input.num_bins < 2)    return 'num_bins must be an integer ≥ 2';
    if (!Number.isFinite(input.value_area_pct)
        || input.value_area_pct < 1 || input.value_area_pct > 99.9) return 'value_area_pct must be in [1, 99.9]';
    return null;
}

export function buildBody(input) {
    return {
        bars: input.bars.map(b => ({ high: b.high, low: b.low, volume: b.volume })),
        num_bins: input.num_bins,
        value_area_pct: input.value_area_pct,
    };
}

// Pure-JS mirror of crates/traderview-core/src/volume_at_price.rs::compute.
// Returns the same VolumeAtPriceReport shape (poc_index/value_area_*=null when empty).
export function localCompute(bars, num_bins, value_area_pct) {
    const empty = { bins: [], poc_index: null, value_area_high: null, value_area_low: null, total_volume: 0 };
    if (!Array.isArray(bars) || bars.length === 0) return empty;
    if (!Number.isInteger(num_bins) || num_bins < 2) return empty;
    if (!Number.isFinite(value_area_pct) || value_area_pct < 1 || value_area_pct > 99.9) return empty;
    for (const b of bars) {
        if (!Number.isFinite(b.high) || !Number.isFinite(b.low) || !Number.isFinite(b.volume)
            || b.volume < 0 || b.high < b.low) return empty;
    }
    let minP = Infinity, maxP = -Infinity;
    for (const b of bars) { if (b.low < minP) minP = b.low; if (b.high > maxP) maxP = b.high; }
    const bin_size = (maxP - minP) / num_bins;
    if (bin_size <= 0) return empty;
    const bin_volumes = new Array(num_bins).fill(0);
    for (const bar of bars) {
        const range = bar.high - bar.low;
        for (let i = 0; i < num_bins; i++) {
            const bin_low = minP + bin_size * i;
            const bin_high = bin_low + bin_size;
            const overlap = Math.max(0, Math.min(bar.high, bin_high) - Math.max(bar.low, bin_low));
            if (overlap > 0 && range > 0) {
                bin_volumes[i] += bar.volume * overlap / range;
            } else if (range === 0 && bar.high >= bin_low && bar.high < bin_high) {
                bin_volumes[i] += bar.volume;
            }
        }
    }
    let total_volume = 0;
    for (const v of bin_volumes) total_volume += v;
    const bins = bin_volumes.map((v, i) => ({
        center: minP + bin_size * (i + 0.5),
        volume: v,
    }));
    // Find POC: bin with highest volume.
    let poc_index = 0;
    let pocVol = -Infinity;
    for (let i = 0; i < bins.length; i++) {
        if (bins[i].volume > pocVol) { pocVol = bins[i].volume; poc_index = i; }
    }
    if (bins.length === 0) return empty;
    // Value-area expansion: grow from POC outward, preferring side with more volume.
    const target_vol = total_volume * value_area_pct / 100;
    let accum = bins[poc_index].volume;
    let lo = poc_index;
    let hi = poc_index;
    while (accum < target_vol && (lo > 0 || hi + 1 < bins.length)) {
        const lo_vol = lo > 0                ? bins[lo - 1].volume : -1;
        const hi_vol = hi + 1 < bins.length  ? bins[hi + 1].volume : -1;
        if (hi_vol >= lo_vol) {
            if (hi + 1 < bins.length) { hi++; accum += bins[hi].volume; }
            else if (lo > 0)          { lo--; accum += bins[lo].volume; }
        } else if (lo > 0) { lo--; accum += bins[lo].volume; }
        else if (hi + 1 < bins.length) { hi++; accum += bins[hi].volume; }
    }
    return {
        bins,
        poc_index,
        value_area_low:  bins[lo].center,
        value_area_high: bins[hi].center,
        total_volume,
    };
}

// Parse "high low volume" per line; ignores blanks + # comments.
export function parseBarsBlob(blob) {
    const out = { bars: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: 'input must be a string' });
        return out;
    }
    const lines = blob.split('\n');
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i].split('#')[0].trim();
        if (!raw) continue;
        const toks = raw.split(/[\s,]+/).filter(t => t.length > 0);
        if (toks.length !== 3) {
            out.errors.push({ line_no: i + 1, message: 'expected 3 tokens (high low volume)' });
            continue;
        }
        const high = Number(toks[0]);
        const low = Number(toks[1]);
        const volume = Number(toks[2]);
        if (![high, low, volume].every(Number.isFinite)) {
            out.errors.push({ line_no: i + 1, message: 'non-finite token' });
            continue;
        }
        if (high < low) {
            out.errors.push({ line_no: i + 1, message: 'high < low' });
            continue;
        }
        if (volume < 0) {
            out.errors.push({ line_no: i + 1, message: 'volume must be ≥ 0' });
            continue;
        }
        out.bars.push({ high, low, volume });
    }
    return out;
}

export function barsToBlob(bars) {
    return bars.map(b => `${b.high} ${b.low} ${b.volume}`).join('\n');
}

// Range of the value area as % of total profile range.
export function valueAreaRangePct(report) {
    if (!report || report.value_area_high == null || report.value_area_low == null
        || !report.bins || report.bins.length === 0) return NaN;
    const totalRange = report.bins[report.bins.length - 1].center - report.bins[0].center;
    if (totalRange <= 0) return NaN;
    const vaRange = report.value_area_high - report.value_area_low;
    return vaRange / totalRange;
}

// Shape verdict — how the profile is distributed.
export function profileBadge(report) {
    if (!report || !report.bins || report.bins.length === 0)
        return { key: 'view.vap.badge.unknown', cls: '' };
    const r = valueAreaRangePct(report);
    if (!Number.isFinite(r))     return { key: 'view.vap.badge.unknown',     cls: '' };
    if (r < 0.3)                 return { key: 'view.vap.badge.balanced',    cls: 'pos' };
    if (r < 0.6)                 return { key: 'view.vap.badge.normal',      cls: '' };
    if (r < 0.85)                return { key: 'view.vap.badge.skewed',      cls: '' };
    return { key: 'view.vap.badge.trending', cls: 'neg' };
}

// Synthetic demos.
export function makeDemoInput(kind = 'normal-session') {
    switch (kind) {
        case 'normal-session': {
            // Bell-shaped around POC ~102.
            const bars = [];
            for (let i = 0; i < 30; i++) {
                const base = 100 + Math.sin(i * 0.3) * 1.5;
                bars.push({ high: base + 1.2, low: base - 1.2, volume: 1000 + Math.cos(i * 0.5) * 200 });
            }
            return { bars, num_bins: 50, value_area_pct: 70 };
        }
        case 'tight-balanced': {
            const bars = [];
            for (let i = 0; i < 40; i++) {
                bars.push({ high: 100.5, low: 99.5, volume: 1000 });
            }
            return { bars, num_bins: 30, value_area_pct: 70 };
        }
        case 'trending-up': {
            // Drift upward over the session.
            const bars = [];
            for (let i = 0; i < 50; i++) {
                const base = 100 + i * 0.2;
                bars.push({ high: base + 0.5, low: base - 0.5, volume: 1000 });
            }
            return { bars, num_bins: 50, value_area_pct: 70 };
        }
        case 'double-distribution': {
            // 20 bars at 100, 20 bars at 110 — 2 modes.
            const bars = [];
            for (let i = 0; i < 20; i++) bars.push({ high: 101, low: 99, volume: 1000 });
            for (let i = 0; i < 20; i++) bars.push({ high: 111, low: 109, volume: 1000 });
            return { bars, num_bins: 30, value_area_pct: 70 };
        }
        case 'spike-poc': {
            // One huge-volume bar in the middle.
            return { bars: [
                { high: 101, low: 100, volume: 500 },
                { high: 106, low: 105, volume: 50_000 },
                { high: 111, low: 110, volume: 500 },
            ], num_bins: 20, value_area_pct: 70 };
        }
        case 'narrow-va': {
            // Tight 90% VA.
            const bars = [];
            for (let i = 0; i < 30; i++) bars.push({ high: 100.2, low: 99.8, volume: 1000 });
            return { bars, num_bins: 50, value_area_pct: 90 };
        }
        case 'wide-va': {
            // 50% VA.
            const bars = [];
            for (let i = 0; i < 30; i++) bars.push({ high: 105, low: 95, volume: 1000 });
            return { bars, num_bins: 50, value_area_pct: 50 };
        }
        case 'fine-bins': {
            const bars = [];
            for (let i = 0; i < 40; i++) bars.push({ high: 100 + Math.sin(i * 0.4) * 2, low: 99 + Math.sin(i * 0.4) * 2, volume: 1000 });
            return { bars, num_bins: 100, value_area_pct: 70 };
        }
        default:
            return makeDemoInput('normal-session');
    }
}

export function fmtUSD(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return '$' + v.toFixed(d);
}

export function fmtVol(v) {
    if (!Number.isFinite(v)) return '—';
    if (Math.abs(v) >= 1e6) return (v / 1e6).toFixed(2) + 'M';
    if (Math.abs(v) >= 1e3) return (v / 1e3).toFixed(2) + 'k';
    return v.toFixed(0);
}

export function fmtInt(v) {
    if (!Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}

export function fmtPct(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}
