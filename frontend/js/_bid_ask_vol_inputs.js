// Bid/Ask Volume Ratio helpers.
//
// Backend body: { bars: Bar[], period: usize }
//   where Bar = { bid_volume, ask_volume }
// Returns: (number|null)[]  — rolling Σ bid / Σ ask over `period` bars.
//
// > 1.5 → strong sell pressure; < 0.67 → strong buy pressure; ≈1 balanced.

export const DEFAULT_PERIOD = 60;
export const MIN_PERIOD = 2;
export const MAX_PERIOD = 1000;

export const DEFAULT_INPUTS = {
    bars: [],
    period: DEFAULT_PERIOD,
};

export function validateInputs(input) {
    if (!Array.isArray(input.bars))                      return 'bars must be an array';
    if (!Number.isInteger(input.period))                 return 'period must be an integer';
    if (input.period < MIN_PERIOD || input.period > MAX_PERIOD)
                                                          return `period must be in [${MIN_PERIOD}, ${MAX_PERIOD}]`;
    if (input.bars.length < input.period)                return `need at least period (${input.period}) bars`;
    for (let i = 0; i < input.bars.length; i++) {
        const b = input.bars[i];
        if (!b)                                            return `bars[${i}] missing`;
        if (typeof b.bid_volume !== 'number' || typeof b.ask_volume !== 'number')
                                                            return `bars[${i}] bid_volume / ask_volume must be numbers`;
        if (!Number.isFinite(b.bid_volume) || !Number.isFinite(b.ask_volume))
                                                            return `bars[${i}] not finite`;
        if (b.bid_volume < 0 || b.ask_volume < 0)          return `bars[${i}] volumes must be ≥ 0`;
    }
    return null;
}

export function buildBody(input) {
    return {
        bars: input.bars.map(b => ({ bid_volume: b.bid_volume, ask_volume: b.ask_volume })),
        period: input.period,
    };
}

// Pure-JS mirror of crates/traderview-core/src/bid_ask_volume_ratio.rs::compute.
export function localCompute(bars, period) {
    const n = bars.length;
    const out = new Array(n).fill(null);
    if (period < 2 || n < period) return out;
    for (const b of bars) {
        if (!Number.isFinite(b.bid_volume) || !Number.isFinite(b.ask_volume)
            || b.bid_volume < 0 || b.ask_volume < 0) return out;
    }
    let bid_sum = 0, ask_sum = 0;
    for (let i = 0; i < period; i++) {
        bid_sum += bars[i].bid_volume;
        ask_sum += bars[i].ask_volume;
    }
    if (ask_sum > 0) out[period - 1] = bid_sum / ask_sum;
    for (let i = period; i < n; i++) {
        bid_sum += bars[i].bid_volume - bars[i - period].bid_volume;
        ask_sum += bars[i].ask_volume - bars[i - period].ask_volume;
        if (ask_sum > 0) out[i] = bid_sum / ask_sum;
    }
    return out;
}

// Parse "bid_vol ask_vol" 2-token-per-line blob.
export function parseBarsBlob(blob) {
    const out = { bars: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: 'input must be a string' });
        return out;
    }
    const lines = blob.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.split('#')[0].trim();
        if (!s) continue;
        const parts = s.split(/[\s,]+/).filter(Boolean);
        if (parts.length !== 2) {
            out.errors.push({ line_no: i + 1, message: `expected 2 tokens (bid_volume ask_volume), got ${parts.length}` });
            continue;
        }
        const bv = Number(parts[0].replace(/[\$,kKmMbB]/g, ''));
        const av = Number(parts[1].replace(/[\$,kKmMbB]/g, ''));
        if (!Number.isFinite(bv) || !Number.isFinite(av) || bv < 0 || av < 0) {
            out.errors.push({ line_no: i + 1, message: `bid/ask volumes must be ≥ 0 finite` });
            continue;
        }
        out.bars.push({ bid_volume: bv, ask_volume: av });
    }
    return out;
}

export function barsToBlob(bars) {
    return bars.map(b => `${b.bid_volume} ${b.ask_volume}`).join('\n');
}

// Flow verdict on most recent ratio per Lee-Ready / order-flow convention.
export function flowBadge(ratio_last) {
    if (ratio_last == null || !Number.isFinite(ratio_last)) {
        return { key: 'view.bavr.flow.unknown', cls: '' };
    }
    if (ratio_last > 3)     return { key: 'view.bavr.flow.heavy_sell',   cls: 'neg' };
    if (ratio_last > 1.5)   return { key: 'view.bavr.flow.sell_pressure', cls: 'neg' };
    if (ratio_last > 1.1)   return { key: 'view.bavr.flow.sell_tilt',    cls: 'neg' };
    if (ratio_last > 0.9)   return { key: 'view.bavr.flow.balanced',     cls: '' };
    if (ratio_last > 0.67)  return { key: 'view.bavr.flow.buy_tilt',     cls: 'pos' };
    if (ratio_last > 0.33)  return { key: 'view.bavr.flow.buy_pressure', cls: 'pos' };
    return { key: 'view.bavr.flow.heavy_buy', cls: 'pos' };
}

// Trend verdict over last N populated ratios.
export function trendBadge(ratios, lookback = 10) {
    if (!Array.isArray(ratios) || ratios.length === 0) {
        return { key: 'view.bavr.trend.unknown', cls: '' };
    }
    const tail = [];
    for (let i = ratios.length - 1; i >= 0 && tail.length < lookback; i--) {
        if (ratios[i] != null && Number.isFinite(ratios[i])) tail.unshift(ratios[i]);
    }
    if (tail.length < 2) return { key: 'view.bavr.trend.unknown', cls: '' };
    const slope = tail[tail.length - 1] - tail[0];
    const range = Math.max(...tail) - Math.min(...tail);
    if (range === 0)              return { key: 'view.bavr.trend.flat', cls: '' };
    if (slope > range * 0.5)       return { key: 'view.bavr.trend.rising_sell',  cls: 'neg' };
    if (slope > range * 0.1)       return { key: 'view.bavr.trend.tilting_sell', cls: 'neg' };
    if (slope < -range * 0.5)      return { key: 'view.bavr.trend.rising_buy',   cls: 'pos' };
    if (slope < -range * 0.1)      return { key: 'view.bavr.trend.tilting_buy',  cls: 'pos' };
    return { key: 'view.bavr.trend.flat', cls: '' };
}

// Imbalance magnitude badge — |log(ratio)| — symmetric around 0.
export function imbalanceBadge(ratio_last) {
    if (ratio_last == null || !Number.isFinite(ratio_last) || ratio_last <= 0) {
        return { key: 'view.bavr.imbalance.unknown', cls: '' };
    }
    const mag = Math.abs(Math.log(ratio_last));
    if (mag < 0.10)  return { key: 'view.bavr.imbalance.symmetric', cls: 'pos' };
    if (mag < 0.40)  return { key: 'view.bavr.imbalance.mild',      cls: '' };
    if (mag < 1.10)  return { key: 'view.bavr.imbalance.strong',    cls: 'neg' };
    return { key: 'view.bavr.imbalance.extreme', cls: 'neg' };
}

export function summarizeBars(bars) {
    if (!Array.isArray(bars) || bars.length === 0) {
        return { count: 0, total_bid: NaN, total_ask: NaN, total_vol: NaN,
                 mean_bid: NaN, mean_ask: NaN, lifetime_ratio: NaN };
    }
    let bid = 0, ask = 0;
    for (const b of bars) { bid += b.bid_volume; ask += b.ask_volume; }
    return {
        count: bars.length,
        total_bid: bid,
        total_ask: ask,
        total_vol: bid + ask,
        mean_bid: bid / bars.length,
        mean_ask: ask / bars.length,
        lifetime_ratio: ask > 0 ? bid / ask : NaN,
    };
}

function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF;
    };
}

export function makeDemoInput(kind = 'balanced') {
    switch (kind) {
        case 'balanced': {
            const rand = lcg(42n);
            return {
                bars: Array.from({ length: 120 }, () => ({
                    bid_volume: 1000 + (rand() - 0.5) * 200,
                    ask_volume: 1000 + (rand() - 0.5) * 200,
                })),
                period: 60,
            };
        }
        case 'buy-pressure': {
            // ask vol > bid vol → ratio < 1
            const rand = lcg(7n);
            return {
                bars: Array.from({ length: 120 }, () => ({
                    bid_volume: 600 + (rand() - 0.5) * 100,
                    ask_volume: 1400 + (rand() - 0.5) * 100,
                })),
                period: 60,
            };
        }
        case 'sell-pressure': {
            const rand = lcg(11n);
            return {
                bars: Array.from({ length: 120 }, () => ({
                    bid_volume: 1500 + (rand() - 0.5) * 100,
                    ask_volume: 750 + (rand() - 0.5) * 100,
                })),
                period: 60,
            };
        }
        case 'shifting-buy': {
            // First half balanced, second half buy pressure (ratio decreases).
            const rand = lcg(13n);
            const bars = [];
            for (let i = 0; i < 60; i++) bars.push({
                bid_volume: 1000 + (rand() - 0.5) * 100,
                ask_volume: 1000 + (rand() - 0.5) * 100,
            });
            for (let i = 0; i < 60; i++) bars.push({
                bid_volume: 500 + (rand() - 0.5) * 100,
                ask_volume: 1500 + (rand() - 0.5) * 100,
            });
            return { bars, period: 60 };
        }
        case 'shifting-sell': {
            const rand = lcg(21n);
            const bars = [];
            for (let i = 0; i < 60; i++) bars.push({
                bid_volume: 1000 + (rand() - 0.5) * 100,
                ask_volume: 1000 + (rand() - 0.5) * 100,
            });
            for (let i = 0; i < 60; i++) bars.push({
                bid_volume: 1500 + (rand() - 0.5) * 100,
                ask_volume: 500 + (rand() - 0.5) * 100,
            });
            return { bars, period: 60 };
        }
        case 'heavy-buy': {
            // ratio ≈ 0.2 → extreme buy
            const rand = lcg(33n);
            return {
                bars: Array.from({ length: 80 }, () => ({
                    bid_volume: 200 + (rand() - 0.5) * 50,
                    ask_volume: 1000 + (rand() - 0.5) * 100,
                })),
                period: 30,
            };
        }
        case 'heavy-sell': {
            const rand = lcg(57n);
            return {
                bars: Array.from({ length: 80 }, () => ({
                    bid_volume: 5000 + (rand() - 0.5) * 200,
                    ask_volume: 1000 + (rand() - 0.5) * 100,
                })),
                period: 30,
            };
        }
        case 'short-period': {
            const rand = lcg(99n);
            return {
                bars: Array.from({ length: 40 }, () => ({
                    bid_volume: 1000 + (rand() - 0.5) * 200,
                    ask_volume: 800 + (rand() - 0.5) * 200,
                })),
                period: 10,
            };
        }
        default: return makeDemoInput('balanced');
    }
}

export function fmtNum(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    const abs = Math.abs(v);
    if (abs >= 1e9) return (v / 1e9).toFixed(2) + 'B';
    if (abs >= 1e6) return (v / 1e6).toFixed(2) + 'M';
    if (abs >= 1e3) return (v / 1e3).toFixed(2) + 'k';
    return v.toFixed(d);
}

export function fmtRatio(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtPct(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
