// Pure helpers for the Optimal-f Position Sizer view.
//
// Ralph Vince's optimal-f: the fraction of capital to risk per trade
// that maximizes geometric growth (TWR — terminal wealth relative).
//
// Math reminder (Vince 1990):
//   For each trade i with P/L = r_i (positive = profit, negative = loss),
//   and a candidate bet fraction f ∈ (0, 1]:
//     hpr_i(f) = 1 + f · (r_i / worst_loss)
//   where worst_loss = max |r_i| over losses. The terminal wealth
//   relative is:
//     TWR(f) = Π_i hpr_i(f)
//   optimal_f = argmax_f TWR(f).
//
// We replicate this locally for the "TWR vs f" sweep chart so the user
// sees the geometric-growth curve and can eyeball how steep the falloff
// is around the optimum (often very narrow — overbetting is asymmetric
// downside risk).

import { parseFloatBlob } from './_paste_parser.js';

/** Parse the trade-P/L textarea. Each value is one trade's P/L (sign
 *  matters — positive = gain, negative = loss). */
export function parseReturns(text) {
    return parseFloatBlob(text);
}

/** Build backend payload. */
export function buildBody(returns) {
    return { returns };
}

/** Validate. Optimal-f needs at least one losing trade — without one,
 *  worst_loss = 0 and the bet-fraction is unbounded. */
export function validateInputs(returns) {
    if (!Array.isArray(returns) || returns.length < 5) {
        return 'need at least 5 trade P/Ls';
    }
    if (returns.some(x => !Number.isFinite(x))) return 'P/L series contains non-finite values';
    const hasLoser = returns.some(x => x < 0);
    if (!hasLoser) return 'need at least one losing trade (Vince formula requires worst_loss > 0)';
    return null;
}

/** Compute TWR(f) for a single f, given the trade P/Ls and the
 *  worst-loss magnitude. Returns 0 (a meaningless TWR) when any HPR
 *  would go non-positive (i.e. f is too large given the worst trade). */
export function twrAt(returns, worstLoss, f) {
    if (!(worstLoss > 0)) return 1;
    let twr = 1;
    for (const r of returns) {
        const hpr = 1 + f * (r / worstLoss);
        if (hpr <= 0) return 0;
        twr *= hpr;
    }
    return twr;
}

/** Generate a TWR(f) sweep across N points in (0, 1]. Powers the
 *  "geometric growth curve" chart. Skips f=0 (degenerate TWR=1) and
 *  starts at 1/N. */
export function twrSweep(returns, points = 101) {
    const worstLoss = Math.max(0, ...returns.map(r => r < 0 ? -r : 0));
    if (!(worstLoss > 0) || returns.length < 1) return { xs: [], ys: [] };
    const xs = new Array(points);
    const ys = new Array(points);
    for (let i = 0; i < points; i++) {
        const f = (i + 1) / points;          // (0, 1]
        xs[i] = f;
        ys[i] = twrAt(returns, worstLoss, f);
    }
    return { xs, ys };
}

/** Two-decimal percent of a fraction-of-capital number. */
export function fmtPctF(x, digits = 2) {
    if (!Number.isFinite(x)) return '—';
    return `${(x * 100).toFixed(digits)}%`;
}

/** Money-style format for absolute P/Ls (typically dollars). */
export function fmtMoney(x, digits = 2) {
    if (!Number.isFinite(x)) return '—';
    const sign = x < 0 ? '-' : '';
    return `${sign}$${Math.abs(x).toFixed(digits)}`;
}

/** Two-decimal multiplier ("1.23×"). */
export function fmtMultiple(x, digits = 2) {
    if (!Number.isFinite(x)) return '—';
    return `${x.toFixed(digits)}×`;
}
