// Trade-Plan Checklist helpers shared by view + vitest.
//
// Backend body shape: { plan: { thesis, entry_price, stop_price?,
//   target_price?, risk_dollars, account_equity, is_long },
//   config: { min_thesis_words, min_r_multiple, max_risk_pct_per_trade } }.
// Returns ChecklistReport with gates[] + all_passed + computed_r_multiple + risk_pct.

export const DEFAULT_CONFIG = {
    min_thesis_words: 10,
    min_r_multiple: 1.5,
    max_risk_pct_per_trade: 0.02,
};

export function validateInputs(p, c) {
    if (typeof p.thesis !== 'string') return 'thesis must be a string (can be empty)';
    if (!Number.isFinite(p.entry_price) || p.entry_price <= 0) return 'entry_price must be > 0';
    if (p.stop_price != null && (!Number.isFinite(p.stop_price) || p.stop_price <= 0))
        return 'stop_price must be > 0 when set';
    if (p.target_price != null && (!Number.isFinite(p.target_price) || p.target_price <= 0))
        return 'target_price must be > 0 when set';
    if (!Number.isFinite(p.risk_dollars) || p.risk_dollars < 0) return 'risk_dollars must be ≥ 0';
    if (!Number.isFinite(p.account_equity) || p.account_equity <= 0) return 'account_equity must be > 0';
    if (typeof p.is_long !== 'boolean') return 'is_long must be boolean';
    if (!Number.isInteger(c.min_thesis_words) || c.min_thesis_words < 0)
        return 'min_thesis_words must be non-negative integer';
    if (!Number.isFinite(c.min_r_multiple) || c.min_r_multiple < 0) return 'min_r_multiple must be ≥ 0';
    if (!Number.isFinite(c.max_risk_pct_per_trade) || c.max_risk_pct_per_trade < 0 || c.max_risk_pct_per_trade > 1)
        return 'max_risk_pct_per_trade must be in [0, 1] (decimal — 0.02 = 2%)';
    return null;
}

export function buildBody(p, c) {
    return {
        plan: {
            thesis: p.thesis,
            entry_price: p.entry_price,
            stop_price: p.stop_price,
            target_price: p.target_price,
            risk_dollars: p.risk_dollars,
            account_equity: p.account_equity,
            is_long: p.is_long,
        },
        config: { ...c },
    };
}

// Pure-JS mirror of the backend evaluator. Used for instant pre-flight
// + parity check. Emits the same gate names + reasons as backend.
export function localEvaluate(plan, cfg) {
    const gates = [];
    const emit = (gate, passed, reason) => gates.push({ gate, passed, reason });
    const words = (plan.thesis || '').trim().split(/\s+/).filter(Boolean).length;
    emit('thesis_present', words >= cfg.min_thesis_words,
        `${words} words (minimum ${cfg.min_thesis_words})`);
    const hasStop   = plan.stop_price   != null;
    const hasTarget = plan.target_price != null;
    emit('stop_loss_set', hasStop,
        hasStop ? 'stop is set' : 'no stop loss defined — naked trade');
    emit('target_set', hasTarget,
        hasTarget ? 'target is set' : 'no target — exit discipline missing');
    let computedR = null;
    if (hasStop && hasTarget) {
        const risk = Math.abs(plan.entry_price - plan.stop_price);
        const reward = Math.abs(plan.target_price - plan.entry_price);
        const r = risk > 0 ? reward / risk : 0;
        computedR = r;
        emit('r_multiple_meets_minimum', r >= cfg.min_r_multiple,
            `R = ${r.toFixed(2)} (min ${cfg.min_r_multiple.toFixed(2)})`);
        const targetOk = plan.is_long
            ? plan.target_price > plan.entry_price
            : plan.target_price < plan.entry_price;
        emit('target_in_direction', targetOk,
            targetOk ? 'target on profitable side of entry'
                     : 'target on WRONG side of entry — direction bug');
        const stopOk = plan.is_long
            ? plan.stop_price < plan.entry_price
            : plan.stop_price > plan.entry_price;
        emit('stop_in_direction', stopOk,
            stopOk ? 'stop on loss side of entry'
                   : 'stop on WRONG side of entry');
    }
    const riskPct = plan.account_equity > 0 ? plan.risk_dollars / plan.account_equity : 0;
    emit('risk_within_max', riskPct <= cfg.max_risk_pct_per_trade,
        `risking ${(riskPct * 100).toFixed(2)}% (max ${(cfg.max_risk_pct_per_trade * 100).toFixed(2)}%)`);
    return {
        gates,
        all_passed: gates.every(g => g.passed),
        computed_r_multiple: computedR,
        risk_pct: riskPct,
    };
}

const GATE_LABELS = {
    thesis_present:           'Thesis present (≥N words)',
    stop_loss_set:            'Stop loss set',
    target_set:               'Target set',
    r_multiple_meets_minimum: 'R-multiple ≥ minimum',
    target_in_direction:      'Target on profit side',
    stop_in_direction:        'Stop on loss side',
    risk_within_max:          'Risk within max %',
};

export function gateLabel(g) {
    return GATE_LABELS[g] || String(g || '—');
}

// 5 demo presets for the gate-pass-vs-fail outcomes.
export function makeDemoData(kind = 'good') {
    const baseLong = {
        thesis: 'Breakout above prior month high on heavy volume with sector confirmation and IBD-style cup and handle pattern completion.',
        entry_price: 100, stop_price: 98, target_price: 106,
        risk_dollars: 200, account_equity: 50_000, is_long: true,
    };
    switch (kind) {
        case 'good':           return { ...baseLong };
        case 'no-stop':        return { ...baseLong, stop_price: null };
        case 'weak-r':         return { ...baseLong, target_price: 102 };     // 1R only
        case 'oversize':       return { ...baseLong, risk_dollars: 2_000 };   // 4% > 2%
        case 'wrong-target':   return { ...baseLong, target_price: 95 };       // long with target below entry
        case 'short-trade':    return {
            ...baseLong, entry_price: 100, stop_price: 102, target_price: 94, is_long: false,
        };
        case 'no-thesis':      return { ...baseLong, thesis: 'yolo' };
        default:               return { ...baseLong };
    }
}

export function fmtPct(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtR(v) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(2) + 'R';
}

export function gateCls(passed) {
    return passed ? 'pos' : 'neg';
}

export function gateIcon(passed) {
    return passed ? '✓' : '×';
}
