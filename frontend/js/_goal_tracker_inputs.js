// Goal Tracker helpers shared by view + vitest.
//
// Backend body shape: { goals: { period_start_equity, target_pct_return,
//   max_dd_pct, period_start (YYYY-MM-DD), period_end (YYYY-MM-DD) },
//   equity_history: f64[], today: YYYY-MM-DD }.
//
// Returns ProgressReport with current/peak equity, DD %, target progress,
// on-pace enum, annualized run-rate extrapolation, kill-switch flag.

import { parseFloatBlob } from './_paste_parser.js';

export function parseEquity(text) {
    return parseFloatBlob(text, { nonNegative: true });
}

const DATE_RE = /^\d{4}-\d{2}-\d{2}$/;

export function validateInputs(p) {
    if (!Number.isFinite(p.period_start_equity) || p.period_start_equity <= 0)
        return t('view.goal_tracker.validate.period_start_equity');
    if (!Number.isFinite(p.target_pct_return))
        return t('view.goal_tracker.validate.target_pct_return');
    if (!Number.isFinite(p.max_dd_pct) || p.max_dd_pct < 0 || p.max_dd_pct > 1)
        return t('view.goal_tracker.validate.max_dd_pct');
    if (!DATE_RE.test(p.period_start)) return t('view.goal_tracker.validate.period_start');
    if (!DATE_RE.test(p.period_end))   return t('view.goal_tracker.validate.period_end');
    if (!DATE_RE.test(p.today))         return t('view.goal_tracker.validate.today');
    if (p.period_end <= p.period_start) return t('view.goal_tracker.validate.period_end_after');
    if (!Array.isArray(p.equity) || p.equity.length === 0) return t('view.goal_tracker.validate.need_equity');
    if (!p.equity.every(v => Number.isFinite(v) && v > 0))
        return t('view.goal_tracker.validate.equity_positive');
    return null;
}

export function buildBody(p) {
    return {
        goals: {
            period_start_equity: p.period_start_equity,
            target_pct_return:   p.target_pct_return,
            max_dd_pct:          p.max_dd_pct,
            period_start:        p.period_start,
            period_end:          p.period_end,
        },
        equity_history: p.equity,
        today:          p.today,
    };
}

// Pure-JS mirror of the backend evaluator. Used for instant pre-flight
// + parity check. Date math uses Date.parse on YYYY-MM-DD which always
// parses as UTC midnight, so day-count differences are stable across
// timezones.
export function localEvaluate(p) {
    const out = {
        current_equity: 0, peak_equity: 0, current_dd_pct: 0,
        current_pct_return: 0, target_pct_return: p.target_pct_return,
        pct_of_target: 0, days_elapsed: 0, days_total: 0,
        annualized_pace: 0, kill_switch_breached: false, on_pace: 'out_of_period',
    };
    if (!Array.isArray(p.equity) || p.equity.length === 0) return out;
    const current = p.equity[p.equity.length - 1];
    const peak = Math.max(...p.equity);
    out.current_equity = current;
    out.peak_equity = peak;
    out.current_dd_pct = peak > 0 ? Math.max(0, (peak - current) / peak) : 0;
    if (p.period_start_equity > 0) {
        out.current_pct_return = (current - p.period_start_equity) / p.period_start_equity;
        if (p.target_pct_return > 0) {
            out.pct_of_target = out.current_pct_return / p.target_pct_return;
        }
    }
    out.kill_switch_breached = out.current_dd_pct > p.max_dd_pct;
    const startMs = Date.parse(p.period_start);
    const endMs   = Date.parse(p.period_end);
    const todayMs = Date.parse(p.today);
    const MS_DAY = 86_400_000;
    const totalDays = Math.max(1, Math.round((endMs - startMs) / MS_DAY));
    const elapsed = Math.round((todayMs - startMs) / MS_DAY);
    out.days_total = totalDays;
    out.days_elapsed = elapsed;
    if (elapsed <= 0 || elapsed > totalDays) {
        out.on_pace = 'out_of_period';
        return out;
    }
    const pctPeriodElapsed = elapsed / totalDays;
    out.annualized_pace = elapsed > 0 ? out.current_pct_return * 365 / elapsed : 0;
    const targetToday = p.target_pct_return * pctPeriodElapsed;
    const buffer = 0.10 * Math.abs(p.target_pct_return);
    if (out.current_pct_return > targetToday + buffer) out.on_pace = 'ahead_of_pace';
    else if (out.current_pct_return < targetToday - buffer) out.on_pace = 'behind_pace';
    else out.on_pace = 'on_pace';
    return out;
}

import { t } from './i18n.js';

const PACE_BADGES = {
    ahead_of_pace: { key: 'ahead', cls: 'pos' },
    on_pace:       { key: 'on_pace', cls: 'pos' },
    behind_pace:   { key: 'behind', cls: 'neg' },
    out_of_period: { key: 'out', cls: '' },
};

export function paceBadge(p) {
    const b = PACE_BADGES[p];
    if (!b) return { label: String(p || '—'), cls: '', hint: '' };
    return {
        label: t(`view.goal_tracker.pace.${b.key}.label`),
        cls: b.cls,
        hint: t(`view.goal_tracker.pace.${b.key}.hint`),
    };
}

// 5 demo presets matching each on_pace state + kill-switch breach.
// Today is anchored to 2026-06-30 (a known mid-year date) so the date math
// produces deterministic days_elapsed regardless of when this is run.
export function makeDemoData(kind = 'on-pace') {
    const base = {
        period_start_equity: 100_000,
        target_pct_return:   0.30,    // 30% annual target
        max_dd_pct:          0.10,    // 10% max DD
        period_start:        '2026-01-01',
        period_end:          '2026-12-31',
        today:               '2026-06-30',
    };
    switch (kind) {
        case 'on-pace':
            // Mid-year, 15% return (proportional target for 30% annual at half-year).
            return { ...base, equity: [100_000, 105_000, 110_000, 115_000] };
        case 'ahead':
            return { ...base, equity: [100_000, 120_000, 130_000] };
        case 'behind':
            return { ...base, equity: [100_000, 102_000] };
        case 'kill-switch':
            // Peak 120k, current 100k → DD 16.7% > 10% limit.
            return { ...base, equity: [100_000, 120_000, 100_000] };
        case 'out-of-period':
            // today is before period_start.
            return { ...base, today: '2025-12-31', equity: [100_000, 110_000] };
        default:
            return { ...base, equity: [100_000] };
    }
}

export function todayIso() {
    return new Date().toISOString().slice(0, 10);
}

export function fmtUSD(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-$' : '$';
    return sign + Math.abs(v).toFixed(0);
}

export function fmtPct(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    const sign = v >= 0 ? '+' : '';
    return sign + (v * 100).toFixed(d) + '%';
}
