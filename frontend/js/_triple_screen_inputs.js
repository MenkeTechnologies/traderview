// Triple Screen (Elder) helpers shared by view + vitest.
//
// Backend body shape (flat TripleScreenInput, no wrapper):
//   { weekly_trend, daily_oscillator_value, oversold_threshold,
//     overbought_threshold, intraday_breakout_up, intraday_breakout_down }
// Response: { verdict: "buy"|"sell"|"wait" }.

const VALID_TRENDS = new Set(['up', 'down', 'neutral']);

export function validateInputs(p) {
    if (!VALID_TRENDS.has(p.weekly_trend)) return 'weekly_trend must be up/down/neutral';
    if (!Number.isFinite(p.daily_oscillator_value)) return 'daily_oscillator_value must be finite';
    if (!Number.isFinite(p.oversold_threshold)) return 'oversold_threshold must be finite';
    if (!Number.isFinite(p.overbought_threshold)) return 'overbought_threshold must be finite';
    if (p.overbought_threshold <= p.oversold_threshold)
        return 'overbought_threshold must be > oversold_threshold';
    if (typeof p.intraday_breakout_up !== 'boolean')   return 'intraday_breakout_up must be boolean';
    if (typeof p.intraday_breakout_down !== 'boolean') return 'intraday_breakout_down must be boolean';
    return null;
}

export function buildBody(p) {
    return {
        weekly_trend: p.weekly_trend,
        daily_oscillator_value: p.daily_oscillator_value,
        oversold_threshold: p.oversold_threshold,
        overbought_threshold: p.overbought_threshold,
        intraday_breakout_up: p.intraday_breakout_up,
        intraday_breakout_down: p.intraday_breakout_down,
    };
}

// Pure-JS mirror of the backend's evaluate(). Used both as a local
// pre-flight (so the verdict can render before the network round trip
// returns) and as a parity check.
export function localEvaluate(p) {
    if (p.weekly_trend === 'up') {
        if (p.daily_oscillator_value < p.oversold_threshold && p.intraday_breakout_up) return 'buy';
        return 'wait';
    }
    if (p.weekly_trend === 'down') {
        if (p.daily_oscillator_value > p.overbought_threshold && p.intraday_breakout_down) return 'sell';
        return 'wait';
    }
    return 'wait';   // neutral always waits
}

// Per-stage gate result. Used by the cascade-visualizer + explanation
// card to surface WHY a Wait verdict was emitted (which screens failed).
export function stageResults(p) {
    const trendUp   = p.weekly_trend === 'up';
    const trendDown = p.weekly_trend === 'down';
    const oversold   = Number.isFinite(p.daily_oscillator_value) && p.daily_oscillator_value < p.oversold_threshold;
    const overbought = Number.isFinite(p.daily_oscillator_value) && p.daily_oscillator_value > p.overbought_threshold;
    // The "intermediate" gate is direction-aware: in an up-trend we want
    // the daily oscillator OVERSOLD (pullback against tide); in a
    // down-trend we want OVERBOUGHT (rally against tide).
    const intermediatePass =
        (trendUp && oversold) ||
        (trendDown && overbought);
    const intradayPass =
        (trendUp && p.intraday_breakout_up) ||
        (trendDown && p.intraday_breakout_down);
    return {
        longTide: {
            label: 'Long-tide (weekly trend)',
            pass: trendUp || trendDown,
            detail: trendUp ? 'UP — long bias' :
                    trendDown ? 'DOWN — short bias' :
                    'NEUTRAL — Triple Screen system stands aside',
        },
        intermediate: {
            label: 'Intermediate-wave (daily oscillator)',
            pass: intermediatePass,
            detail: trendUp
                ? (oversold
                    ? `${p.daily_oscillator_value} < ${p.oversold_threshold} oversold — pullback entry zone`
                    : `${p.daily_oscillator_value} ≥ ${p.oversold_threshold} — no pullback yet`)
                : trendDown
                    ? (overbought
                        ? `${p.daily_oscillator_value} > ${p.overbought_threshold} overbought — rally entry zone`
                        : `${p.daily_oscillator_value} ≤ ${p.overbought_threshold} — no rally yet`)
                    : 'no tide direction to anchor pullback against',
        },
        shortRipple: {
            label: 'Short-ripple (intraday breakout)',
            pass: intradayPass,
            detail: trendUp
                ? (p.intraday_breakout_up
                    ? 'intraday breakout UP — pulls long trigger'
                    : 'no intraday breakout up yet')
                : trendDown
                    ? (p.intraday_breakout_down
                        ? 'intraday breakout DOWN — pulls short trigger'
                        : 'no intraday breakout down yet')
                    : 'no tide to anchor breakout direction',
        },
    };
}

const VERDICT_BADGES = {
    buy:  { label: 'BUY',  cls: 'pos', hint: 'all 3 screens aligned long — pull trigger' },
    sell: { label: 'SELL', cls: 'neg', hint: 'all 3 screens aligned short — pull trigger' },
    wait: { label: 'WAIT', cls: '',    hint: 'at least 1 screen mismatched — stay flat' },
};
export function verdictBadge(v) { return VERDICT_BADGES[v] || { label: String(v || '—'), cls: '', hint: '' }; }

// Preset bundles for the demo buttons — one per Verdict-distinguishing path.
export function makeDemoData(kind) {
    const base = { oversold_threshold: 30, overbought_threshold: 70 };
    switch (kind) {
        case 'buy':
            return {
                ...base,
                weekly_trend: 'up',
                daily_oscillator_value: 25,     // oversold
                intraday_breakout_up: true,
                intraday_breakout_down: false,
            };
        case 'sell':
            return {
                ...base,
                weekly_trend: 'down',
                daily_oscillator_value: 75,     // overbought
                intraday_breakout_up: false,
                intraday_breakout_down: true,
            };
        case 'wait-no-pullback':
            return {
                ...base,
                weekly_trend: 'up',
                daily_oscillator_value: 50,     // not oversold
                intraday_breakout_up: true,
                intraday_breakout_down: false,
            };
        case 'wait-no-breakout':
            return {
                ...base,
                weekly_trend: 'up',
                daily_oscillator_value: 25,
                intraday_breakout_up: false,    // no trigger
                intraday_breakout_down: false,
            };
        case 'wait-neutral-tide':
            return {
                ...base,
                weekly_trend: 'neutral',
                daily_oscillator_value: 25,
                intraday_breakout_up: true,
                intraday_breakout_down: false,
            };
        default:
            return {
                ...base,
                weekly_trend: 'neutral',
                daily_oscillator_value: 50,
                intraday_breakout_up: false,
                intraday_breakout_down: false,
            };
    }
}

export function fmtN(v, d = 1) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}
