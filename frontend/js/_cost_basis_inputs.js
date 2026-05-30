// Cost-basis lot accounting helpers.
//
// Backend body: { lots: CostLot[], qty_to_close, price_per_share, method }.
// CostLot = { lot_id, acquired: 'YYYY-MM-DD', qty, cost_per_share } —
// Decimals as strings on the wire.
// LotMethod: 'fifo' | 'lifo' | 'hifo' | 'lofo' (snake_case enum).
// Returns: { closes: ClosingEntry[], total_realized, qty_remaining_to_close }.
//
// Sort order:
//   FIFO → acquired ASC
//   LIFO → acquired DESC
//   HIFO → cost_per_share DESC (close highest-cost first → min gain)
//   LOFO → cost_per_share ASC  (close lowest-cost first  → max gain)

import { t as tr } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;
export const METHODS = ['fifo', 'lifo', 'hifo', 'lofo'];

// "<lot_id> <YYYY-MM-DD> <qty> <cost_per_share>" per line.
export function parseLotBlob(text) {
    const lots = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { lots, errors: [{ line_no: 0, raw: '', message: t('common.parse.input_must_be_string') }] };
    }
    const lines = text.split(/\r?\n/);
    const seen = new Set();
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = stripComment(raw).trim();
        if (!s) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 4) {
            errors.push({ line_no: i + 1, raw, message: `expected 4 tokens (lot_id YYYY-MM-DD qty cost_per_share), got ${parts.length}` });
            continue;
        }
        const lot_id = parts[0];
        const acquired = parts[1];
        const qty = Number(parts[2]);
        const cost = Number(parts[3]);
        if (!isValidDate(acquired)) {
            errors.push({ line_no: i + 1, raw, message: 'acquired must be YYYY-MM-DD' });
            continue;
        }
        if (!Number.isFinite(qty) || qty <= 0) {
            errors.push({ line_no: i + 1, raw, message: 'qty must be > 0' });
            continue;
        }
        if (!Number.isFinite(cost) || cost < 0) {
            errors.push({ line_no: i + 1, raw, message: 'cost_per_share must be ≥ 0' });
            continue;
        }
        if (seen.has(lot_id)) {
            errors.push({ line_no: i + 1, raw, message: `duplicate lot_id "${lot_id}"` });
            continue;
        }
        seen.add(lot_id);
        lots.push({ lot_id, acquired, qty, cost_per_share: cost });
    }
    return { lots, errors };
}

function stripComment(raw) {
    const i = raw.indexOf('#');
    return i >= 0 ? raw.slice(0, i) : raw;
}

export function isValidDate(s) {
    if (typeof s !== 'string' || !/^\d{4}-\d{2}-\d{2}$/.test(s)) return false;
    const [y, m, d] = s.split('-').map(Number);
    const dt = new Date(Date.UTC(y, m - 1, d));
    return dt.getUTCFullYear() === y && dt.getUTCMonth() === m - 1 && dt.getUTCDate() === d;
}

export function validateInputs(lots, qty_to_close, price_per_share, method) {
    if (!Array.isArray(lots))                          return tr('view.cost_basis.validate.lots_array');
    if (!Number.isFinite(qty_to_close))                return tr('view.cost_basis.validate.qty_finite');
    if (qty_to_close < 0)                              return tr('view.cost_basis.validate.qty_negative');
    if (!Number.isFinite(price_per_share))             return tr('view.cost_basis.validate.price_finite');
    if (price_per_share < 0)                           return tr('view.cost_basis.validate.price_negative');
    if (!METHODS.includes(method))                     return tr('view.cost_basis.validate.method', { list: METHODS.join(', ') });
    return null;
}

export function buildBody(lots, qty_to_close, price_per_share, method) {
    return {
        lots: lots.map(l => ({
            lot_id: l.lot_id, acquired: l.acquired,
            qty: String(l.qty), cost_per_share: String(l.cost_per_share),
        })),
        qty_to_close:    String(qty_to_close),
        price_per_share: String(price_per_share),
        method,
    };
}

export function dec(v) {
    if (v == null) return 0;
    if (typeof v === 'number') return v;
    const n = Number(v);
    return Number.isFinite(n) ? n : 0;
}

// Pure-JS mirror of crates/traderview-core/src/cost_basis.rs::close.
// Same sort order per method; close-min(remaining, lot.qty) per lot; stops
// when remaining ≤ 0.
export function localClose(lots, qty_to_close, price_per_share, method) {
    const out = {
        closes: [],
        total_realized: 0,
        qty_remaining_to_close: qty_to_close,
    };
    if (!Array.isArray(lots) || lots.length === 0 || qty_to_close <= 0) return out;
    const sorted = [...lots];
    sorted.sort((a, b) => {
        switch (method) {
            case 'fifo': return cmpDate(a.acquired, b.acquired);
            case 'lifo': return cmpDate(b.acquired, a.acquired);
            case 'hifo': return b.cost_per_share - a.cost_per_share;
            case 'lofo': return a.cost_per_share - b.cost_per_share;
            default:     return 0;
        }
    });
    let remaining = qty_to_close;
    let total = 0;
    for (const lot of sorted) {
        if (remaining <= 0) break;
        const take = Math.min(remaining, lot.qty);
        const realizedPerShare = price_per_share - lot.cost_per_share;
        const realizedTotal = realizedPerShare * take;
        out.closes.push({
            lot_id: lot.lot_id,
            qty_closed: take,
            cost_per_share: lot.cost_per_share,
            realized_per_share: realizedPerShare,
            realized_total: realizedTotal,
        });
        total += realizedTotal;
        remaining -= take;
    }
    out.total_realized = total;
    out.qty_remaining_to_close = remaining;
    return out;
}

function cmpDate(a, b) {
    if (a < b) return -1;
    if (a > b) return 1;
    return 0;
}

// Verdict-style badge by realized P&L.
export function realizedBadge(total_realized) {
    if (!Number.isFinite(total_realized)) return { key: 'view.cost_basis.badge.unknown', cls: '' };
    if (total_realized > 0)  return { key: 'view.cost_basis.badge.gain',     cls: 'neg' };  // gains = taxable
    if (total_realized < 0)  return { key: 'view.cost_basis.badge.loss',     cls: 'pos' };  // losses = offset
    return { key: 'view.cost_basis.badge.scratch', cls: '' };
}

// Suggest the tax-optimal method for the user's lots + sale price.
//   Both pure-gain AND pure-loss → HIFO. HIFO closes highest-cost lots
//   first, which minimizes the realized GAIN (cap-gains tax) AND
//   maximizes the realized LOSS magnitude (harvest more for offsets).
//   In both cases HIFO pushes realized P&L most negative.
//   Mixed → FIFO (IRS default).
export function suggestMethod(lots, qty_to_close, price_per_share) {
    if (!Array.isArray(lots) || lots.length === 0) return 'fifo';
    const allGain = lots.every(l => price_per_share > l.cost_per_share);
    const allLoss = lots.every(l => price_per_share < l.cost_per_share);
    if (allGain || allLoss) return 'hifo';
    return 'fifo';
}

// Demo presets covering every method + the test cases from Rust.
export function makeDemoLots(kind = 'classic') {
    switch (kind) {
        case 'classic':
            return [
                { lot_id: 'A', acquired: '2024-01-15', qty: 100, cost_per_share: 100 },
                { lot_id: 'B', acquired: '2024-06-10', qty: 100, cost_per_share: 150 },
                { lot_id: 'C', acquired: '2025-03-05', qty: 100, cost_per_share: 125 },
            ];
        case 'gain-only':
            // All lots below sale price → selling = gain. HIFO minimizes.
            return [
                { lot_id: 'X', acquired: '2024-01-15', qty: 50,  cost_per_share: 50 },
                { lot_id: 'Y', acquired: '2024-06-10', qty: 50,  cost_per_share: 75 },
                { lot_id: 'Z', acquired: '2025-03-05', qty: 50,  cost_per_share: 90 },
            ];
        case 'loss-only':
            // All lots above sale price → selling = loss. LOFO maximizes loss.
            return [
                { lot_id: 'X', acquired: '2024-01-15', qty: 50,  cost_per_share: 200 },
                { lot_id: 'Y', acquired: '2024-06-10', qty: 50,  cost_per_share: 250 },
                { lot_id: 'Z', acquired: '2025-03-05', qty: 50,  cost_per_share: 220 },
            ];
        case 'many-lots':
            return [
                { lot_id: 'L1', acquired: '2023-01-01', qty: 25, cost_per_share: 80 },
                { lot_id: 'L2', acquired: '2023-06-01', qty: 25, cost_per_share: 95 },
                { lot_id: 'L3', acquired: '2024-01-01', qty: 25, cost_per_share: 110 },
                { lot_id: 'L4', acquired: '2024-06-01', qty: 25, cost_per_share: 130 },
                { lot_id: 'L5', acquired: '2025-01-01', qty: 25, cost_per_share: 145 },
                { lot_id: 'L6', acquired: '2025-04-01', qty: 25, cost_per_share: 160 },
            ];
        case 'single-lot':
            return [{ lot_id: 'SOLO', acquired: '2024-01-01', qty: 200, cost_per_share: 100 }];
        case 'overclose':
            return [
                { lot_id: 'A', acquired: '2024-01-15', qty: 50, cost_per_share: 100 },
                { lot_id: 'B', acquired: '2024-06-10', qty: 50, cost_per_share: 150 },
            ];
        default:
            return makeDemoLots('classic');
    }
}

export function makeDemoQtyPrice(kind = 'classic') {
    switch (kind) {
        case 'classic':       return { qty_to_close: 100, price_per_share: 200 };
        case 'gain-only':     return { qty_to_close: 100, price_per_share: 150 };
        case 'loss-only':     return { qty_to_close: 100, price_per_share: 150 };
        case 'many-lots':     return { qty_to_close: 100, price_per_share: 120 };
        case 'single-lot':    return { qty_to_close: 50,  price_per_share: 175 };
        case 'overclose':     return { qty_to_close: 200, price_per_share: 200 };  // total 100 lots only
        default:              return { qty_to_close: 100, price_per_share: 200 };
    }
}

export function fmtUSD(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-' : '';
    return sign + '$' + Math.abs(v).toFixed(d);
}

export function fmtUSDSigned(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '-') + '$' + Math.abs(v).toFixed(d);
}

export function fmtNum(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function methodLabelKey(m) {
    return `view.cost_basis.method.${m || 'unknown'}`;
}
