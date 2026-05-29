// Kelly + Dynamic Kelly helpers shared by view + vitest.
//
// Static Kelly body: { win_rate, payoff_ratio }.
//   Returns: { full_kelly, half_kelly, quarter_kelly, recommended_f, note }.
//
// Dynamic Kelly body: { trade_pnls: f64[], window: usize }.
//   Returns: DynamicKellyPoint[] with
//     { window_win_rate, window_payoff_ratio?, kelly_fraction?, half_kelly_fraction? }.
//   Pre-warmup indices return all-default points (fields are 0 / null).

import { t } from './i18n.js';

// ── Static Kelly ─────────────────────────────────────────────────

export function validateStaticInputs(winRate, payoffRatio) {
    if (!Number.isFinite(winRate))     return t('view.kelly.validate.win_rate_finite');
    if (winRate < 0 || winRate > 1)    return t('view.kelly.validate.win_rate_range');
    if (!Number.isFinite(payoffRatio)) return t('view.kelly.validate.payoff_finite');
    if (payoffRatio <= 0)              return t('view.kelly.validate.payoff_positive');
    return null;
}

export function buildStaticBody(winRate, payoffRatio) {
    return { win_rate: winRate, payoff_ratio: payoffRatio };
}

// Mirror of crates/traderview-core/src/kelly.rs::compute. Same fields,
// same note thresholds.
export function localComputeStatic(winRate, payoffRatio) {
    const out = {
        full_kelly: 0, half_kelly: 0, quarter_kelly: 0,
        recommended_f: 0, note: '',
    };
    const p = Math.min(1, Math.max(0, winRate));
    const b = payoffRatio;
    if (!Number.isFinite(b) || b <= 0) {
        out.note = 'payoff_ratio must be > 0 — no win to size against';
        return out;
    }
    const q = 1 - p;
    const full = (b * p - q) / b;
    out.full_kelly = full;
    out.half_kelly = full / 2;
    out.quarter_kelly = full / 4;
    out.recommended_f = Math.max(0, out.half_kelly);
    if (full < 0) {
        out.note = `No edge: p × b = ${(p * b).toFixed(3)} < q = ${q.toFixed(3)}. Don't trade.`;
    } else if (full < 0.01) {
        out.note = 'Edge is tiny (< 1% Kelly). Position sizes will be tiny too.';
    } else if (full > 0.50) {
        out.note = 'Edge is very large (full-Kelly > 50%). Half-Kelly recommended due to extreme drawdown risk.';
    } else {
        out.note = `Half-Kelly = ${(out.recommended_f * 100).toFixed(2)}% of bankroll per trade.`;
    }
    return out;
}

// Convenience: derive (win_rate, payoff_ratio) from a list of trade
// pnls. Skips zero-pnl trades from the win/loss buckets (matches
// dynamic-Kelly convention).
export function pnlsToStaticInput(trade_pnls) {
    const out = { win_rate: 0, payoff_ratio: 0, wins: 0, losses: 0, scratches: 0 };
    if (!Array.isArray(trade_pnls) || trade_pnls.length === 0) return out;
    let winSum = 0, lossSum = 0;
    for (const p of trade_pnls) {
        if (!Number.isFinite(p)) continue;
        if (p > 0)      { out.wins++;     winSum  += p; }
        else if (p < 0) { out.losses++;   lossSum += -p; }
        else              out.scratches++;
    }
    const total = out.wins + out.losses;
    out.win_rate = total > 0 ? out.wins / total : 0;
    if (out.wins > 0 && out.losses > 0) {
        out.payoff_ratio = (winSum / out.wins) / (lossSum / out.losses);
    }
    return out;
}

// ── Dynamic Kelly ────────────────────────────────────────────────

export function validateDynamicInputs(pnls, window) {
    if (!Array.isArray(pnls) || pnls.length === 0) return t('view.kelly.validate.need_pnl');
    if (!Number.isInteger(window) || window <= 0) return t('view.kelly.validate.window_positive');
    if (window > pnls.length) return t('view.kelly.validate.window_exceeds', { window, len: pnls.length });
    return null;
}

export function buildDynamicBody(pnls, window) {
    return { trade_pnls: pnls, window };
}

// Pure-JS mirror of dynamic_kelly::compute. Same window-warmup behavior,
// same finite/Inf filtering, same clamp(-1, 1).
export function localComputeDynamic(trade_pnls, window) {
    const out = [];
    if (!Array.isArray(trade_pnls) || window === 0) return out;
    for (let i = 0; i < trade_pnls.length; i++) {
        if (i + 1 < window) {
            out.push({ window_win_rate: 0, window_payoff_ratio: null,
                       kelly_fraction: null, half_kelly_fraction: null });
            continue;
        }
        const w = trade_pnls.slice(i + 1 - window, i + 1);
        const wins   = w.filter(p => Number.isFinite(p) && p > 0);
        const losses = w.filter(p => Number.isFinite(p) && p < 0).map(p => -p);
        const wr = wins.length / window;
        let payoff = null;
        if (losses.length > 0 && wins.length === 0)      payoff = 0;
        else if (losses.length > 0 && wins.length > 0) {
            const avgWin  = wins.reduce((a, b) => a + b, 0) / wins.length;
            const avgLoss = losses.reduce((a, b) => a + b, 0) / losses.length;
            if (avgLoss > 0) {
                const p = avgWin / avgLoss;
                if (Number.isFinite(p)) payoff = p;
            }
        }
        let kelly = null;
        if (payoff != null) {
            if (payoff === 0) kelly = -1;
            else {
                const q = 1 - wr;
                let k = (payoff * wr - q) / payoff;
                if (k > 1) k = 1; if (k < -1) k = -1;
                kelly = k;
            }
        }
        const halfKelly = kelly == null ? null : Math.max(0, kelly / 2);
        out.push({
            window_win_rate: wr,
            window_payoff_ratio: payoff,
            kelly_fraction: kelly,
            half_kelly_fraction: halfKelly,
        });
    }
    return out;
}

// Parse PnL blob (CSV/whitespace/newline mix, %-suffix not supported here
// — Kelly works in absolute dollar PnL terms).
export function parsePnlBlob(text) {
    const pnls = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { pnls, errors: [{ line: 0, message: 'expected string input' }] };
    }
    const cleaned = text.replace(/#[^\n]*/g, ' ');
    const tokens = cleaned.split(/[\s,]+/).map(t => t.trim()).filter(Boolean);
    tokens.forEach((tok, i) => {
        const n = Number(tok);
        if (!Number.isFinite(n)) {
            errors.push({ line: i + 1, message: `"${tok}" is not finite` });
        } else {
            pnls.push(n);
        }
    });
    return { pnls, errors };
}

// 5 demo presets.
export function makeDemoPnls(kind = 'positive-edge') {
    switch (kind) {
        case 'positive-edge': {
            // 60% wr × $200, 40% × $100 → payoff 2, Kelly 0.4.
            const out = [];
            for (let i = 0; i < 30; i++) {
                out.push(i % 5 === 4 || i % 5 === 3 ? -100 : 200);
            }
            return out;
        }
        case 'negative-edge': {
            // 30% wr × $100, 70% × $100 → Kelly -0.4.
            const out = [];
            for (let i = 0; i < 30; i++) {
                out.push(i % 10 < 3 ? 100 : -100);
            }
            return out;
        }
        case 'break-even': {
            return Array.from({ length: 30 }, (_, i) => i % 2 === 0 ? 100 : -100);
        }
        case 'extreme-edge': {
            // 90% wr × 500, 10% × $100 → Kelly ≈ 0.88.
            return Array.from({ length: 30 }, (_, i) => i % 10 === 0 ? -100 : 500);
        }
        case 'regime-switch': {
            // 30 bars losers, 30 bars winners. Dynamic Kelly should swing.
            return [
                ...Array(30).fill(-100),
                ...Array(30).fill(200),
            ];
        }
        default:
            return makeDemoPnls('positive-edge');
    }
}

// ── Presentation ─────────────────────────────────────────────────

const SIZE_KEYS = ['no_trade', 'tiny', 'moderate', 'aggressive'];
const SIZE_CLS = { no_trade: 'neg', tiny: '', moderate: 'pos', aggressive: 'pos' };
function _sizeBadge(key) {
    return {
        label: t(`view.kelly.size.${key}.label`),
        cls: SIZE_CLS[key],
        hint: t(`view.kelly.size.${key}.hint`),
    };
}

export function sizeBadge(fullKelly) {
    if (!Number.isFinite(fullKelly)) return _sizeBadge('no_trade');
    if (fullKelly < 0)               return _sizeBadge('no_trade');
    if (fullKelly < 0.01)            return _sizeBadge('tiny');
    if (fullKelly > 0.50)            return _sizeBadge('aggressive');
    return _sizeBadge('moderate');
}

export function fmtPct(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtNum(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
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
