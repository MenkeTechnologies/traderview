// Daily-loss kill-switch helpers shared by view + vitest.
//
// Backend body shape: { today_pnl: Decimal-string,
//   config: { max_daily_loss_dollars, max_daily_loss_pct,
//             account_equity, warning_threshold,
//             cut_size_threshold, kill_threshold } } — all Decimal.
// Returns LossLimitReport with state, pct_of_limit, note.

export function validateInputs(p) {
    if (!Number.isFinite(p.today_pnl)) return t('view.daily_loss_limit.validate.today_pnl');
    if (!Number.isFinite(p.max_daily_loss_dollars) || p.max_daily_loss_dollars < 0)
        return t('view.daily_loss_limit.validate.max_dollars');
    if (!Number.isFinite(p.max_daily_loss_pct) || p.max_daily_loss_pct < 0 || p.max_daily_loss_pct > 1)
        return t('view.daily_loss_limit.validate.max_pct');
    if (!Number.isFinite(p.account_equity) || p.account_equity <= 0)
        return t('view.daily_loss_limit.validate.equity');
    for (const k of ['warning_threshold', 'cut_size_threshold', 'kill_threshold']) {
        if (!Number.isFinite(p[k]) || p[k] < 0 || p[k] > 5)
            return t('view.daily_loss_limit.validate.threshold_range', { field: k });
    }
    if (!(p.warning_threshold < p.cut_size_threshold && p.cut_size_threshold <= p.kill_threshold))
        return t('view.daily_loss_limit.validate.threshold_order');
    return null;
}

export function buildBody(p) {
    return {
        today_pnl: String(p.today_pnl),
        config: {
            max_daily_loss_dollars: String(p.max_daily_loss_dollars),
            max_daily_loss_pct:     String(p.max_daily_loss_pct),
            account_equity:         String(p.account_equity),
            warning_threshold:      String(p.warning_threshold),
            cut_size_threshold:     String(p.cut_size_threshold),
            kill_threshold:         String(p.kill_threshold),
        },
    };
}

// Local mirror of backend's binding_limit logic (smaller of $ cap vs
// pct cap when both > 0). Used for the explanatory math panel +
// instant pre-flight verdict before the network round-trip.
export function localBindingLimit(p) {
    const pctLimit = p.account_equity * p.max_daily_loss_pct;
    if (p.max_daily_loss_dollars > 0 && p.max_daily_loss_dollars < pctLimit) {
        return p.max_daily_loss_dollars;
    }
    return pctLimit;
}

// Returns the state local-eval would produce. Mirrors backend.
export function localEvaluate(p) {
    const loss = p.today_pnl < 0 ? -p.today_pnl : 0;
    const limit = localBindingLimit(p);
    const pct = limit > 0 ? loss / limit : 0;
    let state;
    if (pct >= p.kill_threshold)         state = 'kill_switch';
    else if (pct >= p.cut_size_threshold) state = 'cut_size';
    else if (pct >= p.warning_threshold) state = 'warning';
    else                                  state = 'ok';
    return { loss, limit, pct, state };
}

import { t } from './i18n.js';

const STATE_BADGES = {
    ok:          { key: 'ok', cls: 'pos' },
    warning:     { key: 'warning', cls: '' },
    cut_size:    { key: 'cut_size', cls: 'neg' },
    kill_switch: { key: 'kill_switch', cls: 'neg' },
};

export function stateBadge(s) {
    const b = STATE_BADGES[s];
    if (!b) return { label: String(s || '—'), cls: '', hint: '' };
    return {
        label: t(`view.daily_loss_limit.state.${b.key}.label`),
        cls: b.cls,
        hint: t(`view.daily_loss_limit.state.${b.key}.hint`),
    };
}

// Coerces backend Decimal-string scalars to JS numbers for display math.
export function decToNum(v) {
    if (v == null) return NaN;
    if (typeof v === 'number') return v;
    const n = Number(v);
    return Number.isFinite(n) ? n : NaN;
}

// Default config + 5 demo presets matching the four LossState variants.
export function makeDemoData(kind = 'cut-size') {
    const base = {
        max_daily_loss_dollars: 2000,
        max_daily_loss_pct: 0.02,
        account_equity: 100_000,
        warning_threshold: 0.50,
        cut_size_threshold: 0.75,
        kill_threshold: 1.00,
    };
    switch (kind) {
        case 'ok':           return { ...base, today_pnl: 500 };       // profit → OK
        case 'warning':      return { ...base, today_pnl: -1200 };     // 60% of $2k
        case 'cut-size':     return { ...base, today_pnl: -1600 };     // 80% of $2k
        case 'kill':         return { ...base, today_pnl: -2200 };     // over the limit
        case 'tight':        return { ...base, max_daily_loss_dollars: 0, max_daily_loss_pct: 0.005, today_pnl: -400 };  // pct binds
        default:             return { ...base, today_pnl: 0 };
    }
}

export function fmtUSD(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-$' : '$';
    return sign + Math.abs(v).toFixed(2);
}

export function fmtUSDSigned(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-$' : '+$';
    return sign + Math.abs(v).toFixed(2);
}

export function fmtPct(v, d = 1) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}
