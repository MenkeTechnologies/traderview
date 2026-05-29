// Daily-loss kill-switch helpers shared by view + vitest.
//
// Backend body shape: { today_pnl: Decimal-string,
//   config: { max_daily_loss_dollars, max_daily_loss_pct,
//             account_equity, warning_threshold,
//             cut_size_threshold, kill_threshold } } — all Decimal.
// Returns LossLimitReport with state, pct_of_limit, note.

export function validateInputs(p) {
    if (!Number.isFinite(p.today_pnl)) return 'today_pnl must be finite';
    if (!Number.isFinite(p.max_daily_loss_dollars) || p.max_daily_loss_dollars < 0)
        return 'max_daily_loss_dollars must be ≥ 0';
    if (!Number.isFinite(p.max_daily_loss_pct) || p.max_daily_loss_pct < 0 || p.max_daily_loss_pct > 1)
        return 'max_daily_loss_pct must be in [0, 1] (decimal — 0.02 = 2%)';
    if (!Number.isFinite(p.account_equity) || p.account_equity <= 0)
        return 'account_equity must be > 0';
    for (const k of ['warning_threshold', 'cut_size_threshold', 'kill_threshold']) {
        if (!Number.isFinite(p[k]) || p[k] < 0 || p[k] > 5)
            return `${k} must be in [0, 5] (typical 0.5 / 0.75 / 1.0)`;
    }
    if (!(p.warning_threshold < p.cut_size_threshold && p.cut_size_threshold <= p.kill_threshold))
        return 'thresholds must satisfy warning < cut_size ≤ kill';
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

const STATE_BADGES = {
    ok:          { label: 'OK',          cls: 'pos', hint: 'within limits — trade normal size' },
    warning:     { label: 'WARNING',     cls: '',    hint: '≥50% of daily loss budget used — review trades' },
    cut_size:    { label: 'CUT SIZE',    cls: 'neg', hint: 'half-size positions only — preserve remaining budget' },
    kill_switch: { label: 'KILL SWITCH', cls: 'neg', hint: 'stop trading — daily loss limit hit' },
};

export function stateBadge(s) { return STATE_BADGES[s] || { label: String(s || '—'), cls: '', hint: '' }; }

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
