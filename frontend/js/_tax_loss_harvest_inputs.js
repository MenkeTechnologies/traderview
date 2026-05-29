// Tax-loss harvest helpers shared by view + vitest.
//
// Backend body: { losers, recent_buys, today (YYYY-MM-DD),
//   realized_loss_ytd (Decimal-as-string), mtm_elected (bool) }.
//   losers: [{ symbol, qty, avg_cost, current_price }]
//   recent_buys: [{ symbol, executed_at: YYYY-MM-DD }]
// Returns: { candidates: HarvestCandidate[], total_available_loss,
//   safe_harvest_loss }.
//
// HarvestCandidate: { symbol, qty, unrealized_loss, wash_sale_risk,
//   exceeds_3k_cap, note }.
//
// Local mirror reproduces the wash-sale detection, $3k cap, running
// realized-loss accumulation, sort-by-loss-size, and totals.

const TOKEN_DELIM = /[\s,]+/;

// "<symbol> <qty> <avg_cost> <current_price>" per line.
export function parseLoserBlob(text) {
    const losers = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { losers, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = stripComment(raw).trim();
        if (!s) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 4) {
            errors.push({ line_no: i + 1, raw, message: `expected 4 tokens (symbol qty cost price), got ${parts.length}` });
            continue;
        }
        const sym = parts[0].toUpperCase();
        const qty = Number(parts[1]);
        const cost = Number(parts[2]);
        const price = Number(parts[3]);
        if (![qty, cost, price].every(Number.isFinite)) {
            errors.push({ line_no: i + 1, raw, message: 'tokens must be finite numbers' });
            continue;
        }
        if (qty <= 0)   { errors.push({ line_no: i + 1, raw, message: 'qty must be > 0' });   continue; }
        if (cost <= 0)  { errors.push({ line_no: i + 1, raw, message: 'avg_cost must be > 0' });  continue; }
        if (price < 0)  { errors.push({ line_no: i + 1, raw, message: 'current_price must be ≥ 0' }); continue; }
        losers.push({ symbol: sym, qty, avg_cost: cost, current_price: price });
    }
    return { losers, errors };
}

// "<symbol> <YYYY-MM-DD>" per line.
export function parseRecentBuyBlob(text) {
    const buys = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { buys, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = stripComment(raw).trim();
        if (!s) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 2) {
            errors.push({ line_no: i + 1, raw, message: `expected 2 tokens (symbol YYYY-MM-DD), got ${parts.length}` });
            continue;
        }
        const sym = parts[0].toUpperCase();
        const date = parts[1];
        if (!isValidDate(date)) {
            errors.push({ line_no: i + 1, raw, message: 'executed_at must be YYYY-MM-DD' });
            continue;
        }
        buys.push({ symbol: sym, executed_at: date });
    }
    return { buys, errors };
}

function stripComment(raw) {
    const i = raw.indexOf('#');
    return i >= 0 ? raw.slice(0, i) : raw;
}

export function isValidDate(s) {
    if (typeof s !== 'string' || !/^\d{4}-\d{2}-\d{2}$/.test(s)) return false;
    const [y, m, d] = s.split('-').map(Number);
    const dt = new Date(Date.UTC(y, m - 1, d));
    return dt.getUTCFullYear() === y
        && dt.getUTCMonth() === m - 1
        && dt.getUTCDate() === d;
}

// Whole-days distance between two YYYY-MM-DD strings. Sign = b - a.
// Used to mirror chrono::Duration::num_days().abs().
export function daysBetween(a, b) {
    if (!isValidDate(a) || !isValidDate(b)) return Infinity;
    const da = Date.parse(a + 'T00:00:00Z');
    const db = Date.parse(b + 'T00:00:00Z');
    return Math.round((db - da) / 86_400_000);
}

export function validateInputs(losers, recent_buys, today, realized_loss_ytd, mtm_elected) {
    if (!Array.isArray(losers))      return 'losers must be an array';
    if (!Array.isArray(recent_buys)) return 'recent_buys must be an array';
    if (!isValidDate(today))         return 'today must be YYYY-MM-DD';
    if (!Number.isFinite(realized_loss_ytd)) return 'realized_loss_ytd must be finite';
    if (typeof mtm_elected !== 'boolean')   return 'mtm_elected must be boolean';
    return null;
}

export function buildBody(losers, recent_buys, today, realized_loss_ytd, mtm_elected) {
    return {
        losers: losers.map(l => ({
            symbol: l.symbol,
            qty: String(l.qty),
            avg_cost: String(l.avg_cost),
            current_price: String(l.current_price),
        })),
        recent_buys: recent_buys.map(b => ({
            symbol: b.symbol,
            executed_at: b.executed_at,
        })),
        today,
        realized_loss_ytd: String(realized_loss_ytd),
        mtm_elected,
    };
}

// Decimal-string-or-number → number for chart math.
export function dec(v) {
    if (v == null) return 0;
    if (typeof v === 'number') return v;
    const n = Number(v);
    return Number.isFinite(n) ? n : 0;
}

const THREE_K = 3000;

// Mirror of crates/traderview-core/src/tax_loss_harvest.rs::suggest.
// Same sort order, same wash-sale window, same $3k cap accumulation.
// Notes are i18n-key driven (not the verbatim Rust string) so the view
// can translate them.
export function localSuggest(losers, recent_buys, today, realized_loss_ytd, mtm_elected) {
    const candidates = [];
    let runningLoss = realized_loss_ytd;
    // Only genuine losers (price < cost), sorted by loss size DESC.
    const sorted = (losers || [])
        .filter(l => l.current_price < l.avg_cost)
        .map(l => ({ ...l, unrealized_loss: (l.avg_cost - l.current_price) * l.qty }))
        .sort((a, b) => b.unrealized_loss - a.unrealized_loss);
    for (const l of sorted) {
        const washRisk = (recent_buys || []).some(b =>
            b.symbol === l.symbol && Math.abs(daysBetween(b.executed_at, today)) <= 30);
        runningLoss += l.unrealized_loss;
        const exceedsCap = !mtm_elected && runningLoss > THREE_K;
        candidates.push({
            symbol: l.symbol,
            qty: l.qty,
            unrealized_loss: l.unrealized_loss,
            wash_sale_risk: washRisk,
            exceeds_3k_cap: exceedsCap,
            running_loss: runningLoss,
            note_key: washRisk ? 'view.tax_loss_harvest.note.wash_sale'
                     : exceedsCap ? 'view.tax_loss_harvest.note.exceeds_3k'
                     : 'view.tax_loss_harvest.note.safe',
        });
    }
    const total = candidates.reduce((s, c) => s + c.unrealized_loss, 0);
    const safe  = candidates.filter(c => !c.wash_sale_risk)
                            .reduce((s, c) => s + c.unrealized_loss, 0);
    return {
        candidates,
        total_available_loss: total,
        safe_harvest_loss: safe,
    };
}

// Summary verdict for the headline card.
export function harvestBadge(safeLoss) {
    if (!Number.isFinite(safeLoss)) return { key: 'view.tax_loss_harvest.badge.unknown', cls: '' };
    if (safeLoss <= 0)              return { key: 'view.tax_loss_harvest.badge.no_harvest', cls: '' };
    if (safeLoss < 500)             return { key: 'view.tax_loss_harvest.badge.marginal',    cls: '' };
    if (safeLoss < 3000)            return { key: 'view.tax_loss_harvest.badge.useful',      cls: 'pos' };
    return { key: 'view.tax_loss_harvest.badge.significant', cls: 'pos' };
}

// 5 demo presets exercising every Rust branch.
export function makeDemoLosers(kind = 'mixed') {
    switch (kind) {
        case 'mixed':
            return [
                { symbol: 'AAPL', qty: 100,  avg_cost: 150, current_price: 140 },  // $1k
                { symbol: 'TSLA', qty: 10,   avg_cost: 300, current_price: 250 },  // $500
                { symbol: 'NVDA', qty: 50,   avg_cost: 500, current_price: 480 },  // $1k
            ];
        case 'wash-sale':
            return [{ symbol: 'AAPL', qty: 100, avg_cost: 150, current_price: 140 }];
        case 'exceeds-3k':
            return [
                { symbol: 'BIG',  qty: 1000, avg_cost: 50, current_price: 30 },    // $20k
            ];
        case 'winners-only':
            return [{ symbol: 'WIN', qty: 100, avg_cost: 100, current_price: 150 }];
        case 'big-three':
            return [
                { symbol: 'BIG',  qty: 1000, avg_cost: 50, current_price: 30 },
                { symbol: 'MID',  qty: 100,  avg_cost: 50, current_price: 40 },
                { symbol: 'TINY', qty: 10,   avg_cost: 50, current_price: 48 },
            ];
        default:
            return makeDemoLosers('mixed');
    }
}

export function makeDemoRecentBuys(kind = 'mixed', today) {
    const tdy = today || todayIso();
    switch (kind) {
        case 'wash-sale': {
            // Buy 10 days before today → triggers wash.
            const d = new Date(tdy + 'T00:00:00Z');
            d.setUTCDate(d.getUTCDate() - 10);
            return [{ symbol: 'AAPL', executed_at: isoDate(d) }];
        }
        case 'mixed':
            return [];
        case 'exceeds-3k':
            return [];
        case 'winners-only':
            return [];
        case 'big-three':
            return [];
        default:
            return [];
    }
}

function isoDate(d) {
    const y = d.getUTCFullYear();
    const m = String(d.getUTCMonth() + 1).padStart(2, '0');
    const dd = String(d.getUTCDate()).padStart(2, '0');
    return `${y}-${m}-${dd}`;
}

export function todayIso() {
    return isoDate(new Date());
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

export function fmtBool(b) {
    return b ? '✓' : '·';
}
