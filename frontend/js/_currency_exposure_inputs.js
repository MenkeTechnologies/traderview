import { t } from './i18n.js';
// Currency-exposure aggregator helpers.
//
// Backend body: { positions: ForeignPosition[], home_currency: string,
//   fx_to_home: { CCY: rate } }.
// ForeignPosition = { symbol, currency, notional_native }.
// Returns: { home_currency, total_gross_home, total_net_home,
//   buckets: CurrencyBucket[], overweight_currencies: string[] }.
// CurrencyBucket = { currency, gross_native, net_native, gross_home,
//   net_home, position_count, pct_of_total }.
//
// Rules: home currency rate = 1.0; missing fx → 0; overweight > 25% of
// home-gross AND ccy ≠ home; buckets sorted by gross_home DESC.

const TOKEN_DELIM = /[\s,]+/;

// "<symbol> <CCY> <notional>" per line. Notional signed (negative = short).
export function parsePositionBlob(text) {
    const positions = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { positions, errors: [{ line_no: 0, raw: '', message: t('common.parse.input_must_be_string') }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = stripComment(raw).trim();
        if (!s) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 3) {
            errors.push({ line_no: i + 1, raw, message: `expected 3 tokens (SYMBOL CCY notional), got ${parts.length}` });
            continue;
        }
        const sym = parts[0].toUpperCase();
        const ccy = parts[1].toUpperCase();
        const n   = Number(parts[2]);
        if (!Number.isFinite(n)) {
            errors.push({ line_no: i + 1, raw, message: t('view.currency_exposure.parse.notional_finite') });
            continue;
        }
        if (!/^[A-Z]{2,5}$/.test(ccy)) {
            errors.push({ line_no: i + 1, raw, message: t('view.currency_exposure.parse.currency_alpha') });
            continue;
        }
        positions.push({ symbol: sym, currency: ccy, notional_native: n });
    }
    return { positions, errors };
}

// "<CCY> <rate>" per line.
export function parseFxBlob(text) {
    const fx = {};
    const errors = [];
    if (typeof text !== 'string') {
        return { fx, errors: [{ line_no: 0, raw: '', message: t('common.parse.input_must_be_string') }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = stripComment(raw).trim();
        if (!s) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 2) {
            errors.push({ line_no: i + 1, raw, message: `expected 2 tokens (CCY rate), got ${parts.length}` });
            continue;
        }
        const ccy = parts[0].toUpperCase();
        const rate = Number(parts[1]);
        if (!Number.isFinite(rate) || rate <= 0) {
            errors.push({ line_no: i + 1, raw, message: t('view.currency_exposure.parse.rate_positive') });
            continue;
        }
        fx[ccy] = rate;
    }
    return { fx, errors };
}

function stripComment(raw) {
    const i = raw.indexOf('#');
    return i >= 0 ? raw.slice(0, i) : raw;
}

export function validateInputs(positions, home, fx) {
    if (!Array.isArray(positions)) return t('common.validate.must_be_array', { field: 'positions' });
    if (typeof home !== 'string' || !/^[A-Z]{2,5}$/.test(home))
        return 'home_currency must be 2-5 uppercase alpha';
    if (typeof fx !== 'object' || fx === null) return 'fx_to_home must be an object';
    for (const [k, v] of Object.entries(fx)) {
        if (!Number.isFinite(v) || v <= 0) return `fx_to_home.${k} must be > 0`;
    }
    return null;
}

export function buildBody(positions, home, fx) {
    return {
        positions: positions.map(p => ({
            symbol: p.symbol, currency: p.currency,
            notional_native: p.notional_native,
        })),
        home_currency: home,
        fx_to_home: { ...fx },
    };
}

// Pure-JS mirror of crates/traderview-core/src/currency_exposure.rs::analyze.
// Uses Map to preserve insertion order for grouping (Rust uses BTreeMap which
// sorts alphabetically — we sort the final buckets by gross_home DESC anyway).
export function localAnalyze(positions, home, fx) {
    const out = {
        home_currency: home,
        total_gross_home: 0, total_net_home: 0,
        buckets: [], overweight_currencies: [],
    };
    if (!Array.isArray(positions) || positions.length === 0) return out;
    const byCcy = new Map();   // ccy → { gross, net, count }
    for (const p of positions) {
        if (!byCcy.has(p.currency)) byCcy.set(p.currency, { gross: 0, net: 0, count: 0 });
        const e = byCcy.get(p.currency);
        e.gross += Math.abs(p.notional_native);
        e.net   += p.notional_native;
        e.count += 1;
    }
    let totalGrossHome = 0, totalNetHome = 0;
    const rateFor = (ccy) => ccy === home ? 1.0 : (fx[ccy] ?? 0);
    for (const [ccy, e] of byCcy.entries()) {
        const rate = rateFor(ccy);
        totalGrossHome += e.gross * rate;
        totalNetHome   += e.net * rate;
    }
    out.total_gross_home = totalGrossHome;
    out.total_net_home   = totalNetHome;
    for (const [ccy, e] of byCcy.entries()) {
        const rate = rateFor(ccy);
        const gross_home = e.gross * rate;
        const net_home   = e.net * rate;
        const pct = totalGrossHome > 0 ? gross_home / totalGrossHome : 0;
        if (pct > 0.25 && ccy !== home) out.overweight_currencies.push(ccy);
        out.buckets.push({
            currency: ccy,
            gross_native: e.gross, net_native: e.net,
            gross_home, net_home,
            position_count: e.count,
            pct_of_total: pct,
        });
    }
    out.buckets.sort((a, b) => b.gross_home - a.gross_home);
    out.overweight_currencies.sort();
    return out;
}

// Concentration badge by single largest non-home currency share.
export function concentrationBadge(report, home) {
    if (!report || !Array.isArray(report.buckets)) return { key: 'view.currency_exposure.badge.unknown', cls: '' };
    const fx = report.buckets.find(b => b.currency !== home);
    if (!fx) return { key: 'view.currency_exposure.badge.no_fx', cls: 'pos' };
    if (fx.pct_of_total >= 0.50) return { key: 'view.currency_exposure.badge.concentrated', cls: 'neg' };
    if (fx.pct_of_total >= 0.25) return { key: 'view.currency_exposure.badge.tilted',       cls: 'neg' };
    if (fx.pct_of_total >= 0.10) return { key: 'view.currency_exposure.badge.diversified',  cls: '' };
    return { key: 'view.currency_exposure.badge.minimal', cls: 'pos' };
}

const PALETTE = ['#00e5ff', '#ffd84a', '#ff3860', '#23d18b', '#c678dd', '#ffa657', '#5fd0ff', '#ff7ab2'];
export function ccyColor(i) {
    if (!Number.isInteger(i) || i < 0) return '#aab';
    return PALETTE[i % PALETTE.length];
}

// Default FX rates the demos reference.
export function defaultFxRates() {
    return {
        EUR: 1.10,
        GBP: 1.27,
        JPY: 0.0064,
        CAD: 0.74,
        CHF: 1.11,
        AUD: 0.65,
    };
}

// 5 demo presets.
export function makeDemoPositions(kind = 'multi-region') {
    switch (kind) {
        case 'multi-region':
            return [
                { symbol: 'AAPL', currency: 'USD', notional_native: 30_000 },
                { symbol: 'SAP',  currency: 'EUR', notional_native: 20_000 },
                { symbol: 'HSBA', currency: 'GBP', notional_native: 10_000 },
                { symbol: 'SONY', currency: 'JPY', notional_native: 1_500_000 },
            ];
        case 'eur-concentrated':
            return [
                { symbol: 'AAPL', currency: 'USD', notional_native: 5_000 },
                { symbol: 'SAP',  currency: 'EUR', notional_native: 50_000 },
            ];
        case 'short-hedged':
            return [
                { symbol: 'SPY',     currency: 'USD', notional_native: 100_000 },
                { symbol: 'EWG',     currency: 'EUR', notional_native: -20_000 },
            ];
        case 'home-only':
            return [
                { symbol: 'SPY',  currency: 'USD', notional_native: 50_000 },
                { symbol: 'AGG',  currency: 'USD', notional_native: 30_000 },
            ];
        case 'missing-fx':
            // CAD not in default rates → exposure computes at 0 (defensive).
            return [
                { symbol: 'AAPL', currency: 'USD', notional_native: 10_000 },
                { symbol: 'RY',   currency: 'CAD', notional_native: 10_000 },
            ];
        default:
            return makeDemoPositions('multi-region');
    }
}

// FX rates for demos. CAD is intentionally omitted from missing-fx demo.
export function makeDemoFx(kind = 'multi-region') {
    if (kind === 'missing-fx') {
        const r = defaultFxRates();
        delete r.CAD;
        return r;
    }
    return defaultFxRates();
}

export function fmtUSD(v, d = 0) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-' : '';
    return sign + '$' + Math.abs(v).toFixed(d);
}

export function fmtUSDSigned(v, d = 0) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '-') + '$' + Math.abs(v).toFixed(d);
}

export function fmtPct(v, d = 1) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtNum(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtRate(v) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(4);
}
