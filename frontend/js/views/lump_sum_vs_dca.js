// Lump-Sum vs DCA — classic Vanguard "lump-sum beats DCA ~2/3 of the time"
// comparison. Given a total to invest, a horizon, an expected return,
// and a DCA cadence: simulates terminal value of (a) all-in on day 1 vs
// (b) drip across the cadence, then reports the gap and the breakeven
// market return below which DCA pulls ahead.

import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderLumpSumVsDca(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.lump_sum_vs_dca.title">// LUMP-SUM vs DCA</span></h1>
        <p class="muted small" data-i18n-html="view.lump_sum_vs_dca.intro">
            Vanguard 2012 paper: lump-sum beats DCA <strong>~67%</strong> of the time on
            a US 60/40 portfolio, with average end-value <strong>~2.4% higher</strong>
            over a 10-year horizon — because time in market typically beats waiting,
            and DCA leaves capital in cash earning less. DCA still wins emotionally
            (lower regret variance) and statistically in down or sideways markets.
            Adjust expected return below 0%/yr to see DCA's break-even.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small" data-i18n="view.lump_sum_vs_dca.field.total">Total to invest $</span>
                    <input type="number" id="ls-total" step="1000" min="0" value="120000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.lump_sum_vs_dca.field.months">DCA over N months</span>
                    <input type="number" id="ls-months" step="1" min="2" max="120" value="12" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.lump_sum_vs_dca.field.horizon">Horizon (months)</span>
                    <input type="number" id="ls-horizon" step="1" min="2" max="600" value="120" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.lump_sum_vs_dca.field.return">Expected annual return %</span>
                    <input type="number" id="ls-return" step="0.5" min="-30" max="50" value="7" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.lump_sum_vs_dca.field.cash_rate">Cash drag rate %</span>
                    <input type="number" id="ls-cash" step="0.1" min="-5" max="10" value="4" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="ls-run" data-shortcut="r" data-i18n="view.lump_sum_vs_dca.btn.run">⚡ Compute</button>
            <div id="ls-result" style="margin-top:12px"></div>
        </div>
    `;
    mount.querySelector('#ls-run').addEventListener('click', () => runCompute(mount));
    runCompute(mount);
}

function runCompute(mount) {
    const result = mount.querySelector('#ls-result');
    const T = parseFloat(mount.querySelector('#ls-total').value) || 0;
    const N = parseInt(mount.querySelector('#ls-months').value, 10) || 0;
    const H = parseInt(mount.querySelector('#ls-horizon').value, 10) || 0;
    const r_ann = parseFloat(mount.querySelector('#ls-return').value) / 100;
    const c_ann = parseFloat(mount.querySelector('#ls-cash').value) / 100;
    if (T <= 0 || N < 2 || H < N) {
        result.innerHTML = `<p class="muted">${esc(t('view.lump_sum_vs_dca.empty.invalid') || 'Total > 0, DCA months ≥ 2, horizon ≥ DCA months.')}</p>`;
        return;
    }
    const r = Math.pow(1 + r_ann, 1/12) - 1;     // monthly market return
    const c = Math.pow(1 + c_ann, 1/12) - 1;     // monthly cash rate
    const perMonth = T / N;

    // LUMP — single investment at month 0, compounded H months at r.
    const lump_end = T * Math.pow(1 + r, H);

    // DCA — invested portion compounds at r, un-invested cash sits at c.
    // Track invested + cash each month.
    let invested = 0;     // value of already-invested portion
    let cash = T;         // cash bucket waiting to be DCA'd
    for (let m = 1; m <= H; m++) {
        // Existing invested compounds at market rate.
        invested *= (1 + r);
        // Existing cash earns the cash rate.
        cash *= (1 + c);
        // If still in DCA window, move perMonth from cash to invested.
        if (m <= N) {
            const drop = Math.min(perMonth, cash);
            cash -= drop;
            invested += drop;
        }
    }
    const dca_end = invested + cash;

    const gap = lump_end - dca_end;
    const gap_pct = (gap / dca_end) * 100;
    const winner = gap > 0 ? 'LUMP-SUM' : gap < 0 ? 'DCA' : 'TIE';
    const winnerCls = gap > 0 ? 'pos' : gap < 0 ? 'neg' : 'muted';

    // Solve for break-even market return where dca_end == lump_end.
    // Closed form is ugly; binary-search r between -30%/yr and +50%/yr.
    const breakeven = findBreakeven(T, N, H, c_ann);
    result.innerHTML = `
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
            <div class="card">
                <div class="label">Lump-sum end value</div>
                <div class="value">$${fmt(lump_end, 0)}</div>
            </div>
            <div class="card">
                <div class="label">DCA end value</div>
                <div class="value">$${fmt(dca_end, 0)}</div>
            </div>
            <div class="card">
                <div class="label">Winner</div>
                <div class="value ${winnerCls}"><strong>${winner}</strong></div>
                <div class="muted small">by $${fmt(Math.abs(gap), 0)} (${gap_pct >= 0 ? '+' : ''}${fmt(gap_pct, 2)}%)</div>
            </div>
            <div class="card">
                <div class="label">Break-even market return</div>
                <div class="value">${breakeven == null ? '—' : fmt(breakeven * 100, 2) + '%/yr'}</div>
                <div class="muted small">below this, DCA wins (cash drag less of a penalty)</div>
            </div>
        </div>
        <p class="muted small">
            Assumption: lump-sum invests <strong>$${fmt(T, 0)}</strong> at month 0 and rides
            <strong>${H}</strong> months at <strong>${fmt(r_ann * 100, 1)}%</strong>/yr.
            DCA invests <strong>$${fmt(T / N, 0)}</strong>/month for ${N} months while
            the un-invested cash earns <strong>${fmt(c_ann * 100, 1)}%</strong>/yr in a money-market.
        </p>
    `;
}

function simulateDca(T, N, H, r_ann, c_ann) {
    const r = Math.pow(1 + r_ann, 1/12) - 1;
    const c = Math.pow(1 + c_ann, 1/12) - 1;
    const perMonth = T / N;
    let invested = 0;
    let cash = T;
    for (let m = 1; m <= H; m++) {
        invested *= (1 + r);
        cash *= (1 + c);
        if (m <= N) {
            const drop = Math.min(perMonth, cash);
            cash -= drop;
            invested += drop;
        }
    }
    return invested + cash;
}

function findBreakeven(T, N, H, c_ann) {
    // Binary search for r_ann where dca_end == lump_end. Both are
    // monotonic in r_ann; lump grows faster, so above breakeven lump wins.
    let lo = -0.30, hi = 0.50;
    for (let i = 0; i < 60; i++) {
        const mid = (lo + hi) / 2;
        const lump = T * Math.pow(1 + Math.pow(1 + mid, 1/12) - 1, H);
        const dca = simulateDca(T, N, H, mid, c_ann);
        if (Math.abs(lump - dca) < 1) return mid;
        if (lump > dca) hi = mid; else lo = mid;
    }
    return (lo + hi) / 2;
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
