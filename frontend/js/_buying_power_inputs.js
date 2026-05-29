// Buying-power calculator helpers shared by view + vitest.
//
// Backend body: BpInput { account_type: 'cash'|'reg_t'|'portfolio_margin',
//   equity (Decimal-as-string), is_pdt, is_day_trade,
//   share_price (Decimal-as-string) }.
// Returns: BpReport { max_notional, max_shares, leverage,
//   initial_requirement_pct, note }.
//
// Match-order mirrors crates/traderview-core/src/buying_power.rs::compute:
//   1. Cash → 1×
//   2. RegT + pdt_qualified → 4× (PDT WINS OVER sub-$5 check)
//   3. RegT + share_price < $5 → 1× (penny-stock fence)
//   4. RegT → 2×
//   5. PortfolioMargin + pdt_qualified → 6×
//   6. PortfolioMargin → 3×

export const ACCOUNT_TYPES = ['cash', 'reg_t', 'portfolio_margin'];
export const PDT_MIN_EQUITY = 25_000;

export const DEFAULT_INPUTS = {
    account_type: 'reg_t',
    equity: 30_000,
    is_pdt: false,
    is_day_trade: false,
    share_price: 50,
};

export function validateInputs(input) {
    if (!ACCOUNT_TYPES.includes(input.account_type))
        return `account_type must be one of ${ACCOUNT_TYPES.join(', ')}`;
    if (!Number.isFinite(input.equity)) return 'equity must be finite';
    if (input.equity < 0)               return 'equity must be ≥ 0';
    if (typeof input.is_pdt !== 'boolean')        return 'is_pdt must be boolean';
    if (typeof input.is_day_trade !== 'boolean')  return 'is_day_trade must be boolean';
    if (!Number.isFinite(input.share_price))      return 'share_price must be finite';
    if (input.share_price < 0)                    return 'share_price must be ≥ 0';
    return null;
}

export function buildBody(input) {
    return {
        account_type: input.account_type,
        equity: String(input.equity),
        is_pdt: input.is_pdt,
        is_day_trade: input.is_day_trade,
        share_price: String(input.share_price),
    };
}

export function dec(v) {
    if (v == null) return 0;
    if (typeof v === 'number') return v;
    const n = Number(v);
    return Number.isFinite(n) ? n : 0;
}

// Local mirror — returns the same {leverage, note_key, initial_req_pct,
// max_notional, max_shares}. note is keyed (not the raw English string)
// so the view can translate.
export function localCompute(input) {
    const initial_req_pct = input.share_price < 5 ? 1.00 : 0.50;
    const pdt_qualified = input.is_pdt
                       && input.equity >= PDT_MIN_EQUITY
                       && input.is_day_trade;
    let leverage, note_key;
    if (input.account_type === 'cash') {
        leverage = 1.0; note_key = 'view.buying_power.note.cash';
    } else if (input.account_type === 'reg_t' && pdt_qualified) {
        leverage = 4.0; note_key = 'view.buying_power.note.pdt';
    } else if (input.account_type === 'reg_t' && input.share_price < 5) {
        leverage = 1.0; note_key = 'view.buying_power.note.sub5';
    } else if (input.account_type === 'reg_t') {
        leverage = 2.0; note_key = 'view.buying_power.note.regt';
    } else if (input.account_type === 'portfolio_margin' && pdt_qualified) {
        leverage = 6.0; note_key = 'view.buying_power.note.pm_pdt';
    } else { // portfolio_margin
        leverage = 3.0; note_key = 'view.buying_power.note.pm';
    }
    const max_notional = input.equity * leverage;
    const max_shares   = input.share_price > 0 ? max_notional / input.share_price : 0;
    return {
        max_notional, max_shares, leverage,
        initial_requirement_pct: initial_req_pct,
        note_key,
    };
}

// Leverage tier badge.
export function leverageBadge(leverage) {
    if (!Number.isFinite(leverage)) return { key: 'view.buying_power.badge.unknown', cls: '' };
    if (leverage >= 6) return { key: 'view.buying_power.badge.extreme', cls: 'neg' };
    if (leverage >= 4) return { key: 'view.buying_power.badge.high',    cls: 'neg' };
    if (leverage >= 2) return { key: 'view.buying_power.badge.moderate', cls: '' };
    return { key: 'view.buying_power.badge.none', cls: 'pos' };
}

// PDT eligibility status (for the UI helper card).
export function pdtStatusKey(input) {
    if (!input.is_pdt) return 'view.buying_power.pdt.not_flagged';
    if (input.equity < PDT_MIN_EQUITY) return 'view.buying_power.pdt.below_25k';
    if (!input.is_day_trade) return 'view.buying_power.pdt.overnight';
    return 'view.buying_power.pdt.active';
}

// Demo presets — one per branch in the Rust match + corner cases.
export function makeDemoInput(kind = 'reg-t-overnight') {
    switch (kind) {
        case 'cash':
            return { account_type: 'cash', equity: 10_000, is_pdt: false, is_day_trade: false, share_price: 50 };
        case 'reg-t-overnight':
            return { account_type: 'reg_t', equity: 10_000, is_pdt: false, is_day_trade: false, share_price: 50 };
        case 'pdt-day-trade':
            return { account_type: 'reg_t', equity: 30_000, is_pdt: true, is_day_trade: true, share_price: 50 };
        case 'pdt-below-25k':
            return { account_type: 'reg_t', equity: 20_000, is_pdt: true, is_day_trade: true, share_price: 50 };
        case 'pdt-overnight':
            // PDT flagged but holding overnight → loses 4× multiplier.
            return { account_type: 'reg_t', equity: 30_000, is_pdt: true, is_day_trade: false, share_price: 50 };
        case 'sub-5':
            return { account_type: 'reg_t', equity: 10_000, is_pdt: false, is_day_trade: false, share_price: 3 };
        case 'pdt-sub-5':
            // Corner case: PDT match arm fires BEFORE the sub-$5 arm in Rust,
            // so a qualified PDT day-trader gets 4× on a penny stock too.
            return { account_type: 'reg_t', equity: 30_000, is_pdt: true, is_day_trade: true, share_price: 3 };
        case 'portfolio-margin':
            return { account_type: 'portfolio_margin', equity: 100_000, is_pdt: false, is_day_trade: false, share_price: 100 };
        case 'pm-pdt-day':
            return { account_type: 'portfolio_margin', equity: 100_000, is_pdt: true, is_day_trade: true, share_price: 100 };
        case 'zero-price':
            return { account_type: 'cash', equity: 10_000, is_pdt: false, is_day_trade: false, share_price: 0 };
        default:
            return makeDemoInput('reg-t-overnight');
    }
}

export function fmtUSD(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-' : '';
    return sign + '$' + Math.abs(v).toFixed(d);
}

export function fmtNum(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtX(v, d = 1) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d) + '×';
}

export function fmtPct(v, d = 0) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}
