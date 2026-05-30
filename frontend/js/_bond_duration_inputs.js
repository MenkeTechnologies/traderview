// Bond duration helpers.
//
// Backend body: { cash_flows: [{time_years, amount}, ...], ytm: f64,
//   compounding_per_year: usize }.
// Returns: { price, macaulay_duration, modified_duration, yield_to_maturity }.
//
// Formulas:
//   factor_t = (1 + ytm/m)^(t*m)
//   pv_t     = amount / factor_t
//   price    = Σ pv_t
//   Macaulay = Σ (t × pv_t) / price
//   Modified = Macaulay / (1 + ytm/m)
//
// Quick estimator: ΔP/P ≈ -ModDur × Δy.

import { t as tr } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;

// "<time_years> <amount>" per line.
export function parseCashFlowBlob(text) {
    const cash_flows = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { cash_flows, errors: [{ line_no: 0, raw: '', message: tr('common.parse.input_must_be_string') }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = stripComment(raw).trim();
        if (!s) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 2) {
            errors.push({ line_no: i + 1, raw, message: `expected 2 tokens (time_years amount), got ${parts.length}` });
            continue;
        }
        const t = Number(parts[0]);
        const a = Number(parts[1]);
        if (!Number.isFinite(t) || !Number.isFinite(a)) {
            errors.push({ line_no: i + 1, raw, message: t('common.parse.tokens_must_be_finite') });
            continue;
        }
        if (t <= 0) {
            errors.push({ line_no: i + 1, raw, message: 'time_years must be > 0' });
            continue;
        }
        cash_flows.push({ time_years: t, amount: a });
    }
    return { cash_flows, errors };
}

function stripComment(raw) {
    const i = raw.indexOf('#');
    return i >= 0 ? raw.slice(0, i) : raw;
}

export function validateInputs(cash_flows, ytm, compounding_per_year) {
    if (!Array.isArray(cash_flows)) return tr('view.bond_duration.validate.cash_flows_array');
    if (cash_flows.length === 0)    return tr('view.bond_duration.validate.cash_flows_empty');
    if (!Number.isFinite(ytm))      return tr('view.bond_duration.validate.ytm_finite');
    if (!Number.isInteger(compounding_per_year) || compounding_per_year < 1)
        return tr('view.bond_duration.validate.compounding');
    return null;
}

export function buildBody(cash_flows, ytm, compounding_per_year) {
    return {
        cash_flows: cash_flows.map(c => ({ time_years: c.time_years, amount: c.amount })),
        ytm, compounding_per_year,
    };
}

// Pure-JS mirror of crates/traderview-core/src/bond_duration.rs::compute.
// Same degenerate guards (1+ytm/m ≤ 0 → default report; non-finite factor
// skipped per-CF; non-positive price → default).
export function localCompute(cash_flows, ytm, compounding_per_year) {
    const out = {
        price: 0, macaulay_duration: 0, modified_duration: 0,
        yield_to_maturity: ytm,
    };
    if (!Array.isArray(cash_flows) || cash_flows.length === 0) return out;
    const m = Math.max(1, compounding_per_year);
    const onePlus = 1 + ytm / m;
    if (!Number.isFinite(onePlus) || onePlus <= 0) return out;
    let price = 0, weightedTime = 0;
    for (const cf of cash_flows) {
        const factor = Math.pow(onePlus, cf.time_years * m);
        if (factor === 0 || !Number.isFinite(factor)) continue;
        const pv = cf.amount / factor;
        price += pv;
        weightedTime += cf.time_years * pv;
    }
    if (price <= 0 || !Number.isFinite(price)) return out;
    out.price = price;
    out.macaulay_duration = weightedTime / price;
    out.modified_duration = out.macaulay_duration / onePlus;
    return out;
}

// Mirror of price_change_pct() — ΔP/P ≈ -ModDur × Δy.
export function priceChangePct(modified_duration, yield_change_bps) {
    if (!Number.isFinite(modified_duration) || !Number.isFinite(yield_change_bps)) return 0;
    return -modified_duration * (yield_change_bps / 10_000);
}

// Build a standard plain-vanilla coupon bond:
//   coupons at every period (m/year) for `maturity_years`, plus par at maturity.
//   Coupon $ per period = par × coupon_rate / m.
export function buildCouponBond(par, coupon_rate, maturity_years, compounding_per_year) {
    const cfs = [];
    if (par <= 0 || maturity_years <= 0 || compounding_per_year < 1) return cfs;
    const m = Math.max(1, compounding_per_year);
    const periodsTotal = Math.round(maturity_years * m);
    const couponPerPeriod = par * coupon_rate / m;
    for (let k = 1; k <= periodsTotal; k++) {
        const t = k / m;
        const amount = couponPerPeriod + (k === periodsTotal ? par : 0);
        cfs.push({ time_years: t, amount });
    }
    return cfs;
}

// Duration badge by Macaulay years (rate-sensitivity tier).
const BADGES = {
    cash:        { key: 'view.bond_duration.badge.cash',        cls: 'pos' },
    short:       { key: 'view.bond_duration.badge.short',       cls: 'pos' },
    intermediate: { key: 'view.bond_duration.badge.intermediate', cls: '' },
    long:        { key: 'view.bond_duration.badge.long',        cls: 'neg' },
    ultra:       { key: 'view.bond_duration.badge.ultra',       cls: 'neg' },
    unknown:     { key: 'view.bond_duration.badge.unknown',     cls: '' },
};

export function durationBadge(macaulay) {
    if (!Number.isFinite(macaulay)) return BADGES.unknown;
    if (macaulay < 1)  return BADGES.cash;
    if (macaulay < 3)  return BADGES.short;
    if (macaulay < 7)  return BADGES.intermediate;
    if (macaulay < 12) return BADGES.long;
    return BADGES.ultra;
}

// Standard ±Δy sensitivity grid (in bps).
export const SENSITIVITY_BPS = [-200, -100, -50, -25, -10, 10, 25, 50, 100, 200];

// 6 demo presets covering common bond profiles.
export function makeDemoConfig(kind = 'treasury-5yr-coupon') {
    switch (kind) {
        case 'zero-5yr':
            // Zero-coupon $100 at year 5, YTM 4%, annual.
            return { cash_flows: buildCouponBond(100, 0, 5, 1).filter(c => c.time_years === 5)
                                  .concat([]), ytm: 0.04, compounding_per_year: 1 };
        case 'treasury-5yr-coupon':
            // 5% annual coupon, $100 par, 5-year, YTM 5% → prices at par.
            return { cash_flows: buildCouponBond(100, 0.05, 5, 1), ytm: 0.05, compounding_per_year: 1 };
        case 'treasury-10yr-semi':
            // 4% semi-annual coupon, 10-year, YTM 5% → discount bond.
            return { cash_flows: buildCouponBond(1000, 0.04, 10, 2), ytm: 0.05, compounding_per_year: 2 };
        case 'treasury-30yr-semi':
            // 4.5% semi-annual coupon, 30-year, YTM 4.5% → at par.
            return { cash_flows: buildCouponBond(1000, 0.045, 30, 2), ytm: 0.045, compounding_per_year: 2 };
        case 'corporate-7yr-high-coupon':
            // 8% semi-annual coupon, 7-year, YTM 6% → premium.
            return { cash_flows: buildCouponBond(1000, 0.08, 7, 2), ytm: 0.06, compounding_per_year: 2 };
        case 'tips-zero-2yr':
            // Zero-coupon 2-year, YTM 1.5% (low-rate environment).
            return { cash_flows: buildCouponBond(100, 0, 2, 2).filter(c => c.time_years === 2)
                                  .concat([]), ytm: 0.015, compounding_per_year: 2 };
        default:
            return makeDemoConfig('treasury-5yr-coupon');
    }
}

export function fmtUSD(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-' : '';
    return sign + '$' + Math.abs(v).toFixed(d);
}

export function fmtPctSigned(v, d = 3) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + (v * 100).toFixed(d) + '%';
}

export function fmtPct(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtYears(v, d = 3) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d) + ' yr';
}

export function fmtBpsSigned(v) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v + ' bps';
}
