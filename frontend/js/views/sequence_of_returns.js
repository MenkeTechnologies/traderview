// Sequence-of-Returns Risk — Bengen 4% Rule pressure-tested.
// Two portfolios with identical AVERAGE return can have wildly
// different terminal values if one starts with bad years and the
// other with good ones — because withdrawals against a depleted
// portfolio compound the damage. This view simulates 3 scenarios
// on the same nominal-return sequence:
//   A) Forward order — historical sequence as is
//   B) Reversed order — same returns, ending first
//   C) Bad-early — sorted ascending (worst years first)
//   D) Good-early — sorted descending (best years first)
// All four end with the same arithmetic mean return but very
// different terminal values.

import { esc } from '../util.js';
import { t } from '../i18n.js';

// Default sequence: SPY annual total returns 2000-2024 (approximate).
// Captures the "lost decade" (2000-2010) ideal for sequence-risk demos.
const DEFAULT_RETURNS = [
    -9.1, -11.9, -22.1, 28.7, 10.9, 4.9, 15.8, 5.5, -37.0, 26.5,
    15.1, 2.1, 16.0, 32.4, 13.7, 1.4, 12.0, 21.8, -4.4, 31.5,
    18.4, 28.7, -18.1, 26.3, 25.0,
];

export async function renderSequenceOfReturns(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.sequence_of_returns.title">// SEQUENCE OF RETURNS RISK</span></h1>
        <p class="muted small" data-i18n-html="view.sequence_of_returns.intro">
            Two portfolios with the <strong>same arithmetic mean</strong> can have
            wildly different terminal values during withdrawal — because pulling
            from a depleted balance compounds the damage. Demonstrated below
            using a fixed return sequence in 4 orderings: actual (forward),
            reversed, worst-years-first, best-years-first. Same mean, same
            withdrawals, vastly different outcomes. Default uses SPY 2000-2024
            (the "lost decade" makes the effect dramatic).
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small">Starting balance $</span>
                    <input type="number" id="sr-start" step="10000" min="0" value="1000000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Annual withdrawal $</span>
                    <input type="number" id="sr-wd" step="1000" min="0" value="40000" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Inflation adjust withdrawal %/yr</span>
                    <input type="number" id="sr-infl" step="0.1" min="0" max="20" value="3.0" style="width:100%">
                </label>
                <label>
                    <span class="muted small">Returns sequence (%/yr, comma-separated)</span>
                    <input type="text" id="sr-seq" value="${DEFAULT_RETURNS.join(',')}" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="sr-run">⚡ Simulate</button>
            <div id="sr-result" style="margin-top:12px"></div>
        </div>
    `;
    mount.querySelectorAll('#sr-start, #sr-wd, #sr-infl, #sr-seq').forEach(el => {
        el.addEventListener('input', () => compute(mount));
    });
    mount.querySelector('#sr-run').addEventListener('click', () => compute(mount));
    compute(mount);
}

function compute(mount) {
    const start = parseFloat(mount.querySelector('#sr-start').value) || 0;
    const wd = parseFloat(mount.querySelector('#sr-wd').value) || 0;
    const infl = parseFloat(mount.querySelector('#sr-infl').value) / 100;
    const seqStr = mount.querySelector('#sr-seq').value;
    const result = mount.querySelector('#sr-result');

    const returns = seqStr.split(',').map(s => parseFloat(s.trim()) / 100).filter(n => Number.isFinite(n));
    if (returns.length < 5) {
        result.innerHTML = `<p class="muted">Need at least 5 annual returns.</p>`;
        return;
    }
    const mean = returns.reduce((s, r) => s + r, 0) / returns.length;
    const sorted = returns.slice().sort((a, b) => a - b);
    const scenarios = [
        { name: 'Forward (actual)',     seq: returns.slice(),          cls: '' },
        { name: 'Reversed',             seq: returns.slice().reverse(), cls: 'muted' },
        { name: 'Worst-years-first',    seq: sorted.slice(),            cls: 'neg' },
        { name: 'Best-years-first',     seq: sorted.slice().reverse(),  cls: 'pos' },
    ];

    const runs = scenarios.map(sc => {
        let bal = start;
        let realWd = wd;
        const path = [];
        let failedAtYear = null;
        for (let i = 0; i < sc.seq.length; i++) {
            const open = bal;
            const ret = sc.seq[i];
            bal *= (1 + ret);
            bal -= realWd;
            path.push({ year: i + 1, open, ret, wd: realWd, close: bal });
            if (bal <= 0 && failedAtYear == null) { failedAtYear = i + 1; bal = 0; }
            realWd *= (1 + infl);
        }
        return { ...sc, end: bal, path, failedAtYear, totalWd: path.reduce((s, p) => s + p.wd, 0) };
    });

    result.innerHTML = `
        <p class="muted small">Arithmetic mean return: <strong>${fmt(mean * 100, 2)}%</strong>/yr across ${returns.length} years.</p>
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:8px;margin-bottom:12px">
            ${runs.map(r => `<div class="card">
                <div class="label">${esc(r.name)}</div>
                <div class="value ${r.end <= 0 ? 'neg' : r.cls}">${r.end <= 0 ? 'FAILED yr ' + r.failedAtYear : '$' + fmt(r.end, 0)}</div>
                <div class="muted small">Total withdrawn: $${fmt(r.totalWd, 0)}</div>
            </div>`).join('')}
        </div>
        <h3 class="section-title">Year-by-year breakdown</h3>
        <table class="trades" data-table-key="sr-walk">
            <thead><tr>
                <th>Year</th>
                ${runs.map(r => `<th>${esc(r.name)}<br/><span class="muted small">close</span></th>`).join('')}
            </tr></thead>
            <tbody>${returns.map((_, i) => `<tr>
                <td>${i + 1}</td>
                ${runs.map(r => {
                    const cell = r.path[i];
                    if (!cell) return `<td class="muted">—</td>`;
                    const cls = cell.close <= 0 ? 'neg' : (cell.close < start * 0.5 ? 'muted' : '');
                    return `<td class="${cls}"><span class="muted small">[${(cell.ret*100).toFixed(1)}%]</span> $${fmt(cell.close, 0)}</td>`;
                }).join('')}
            </tr>`).join('')}</tbody>
        </table>
        <p class="muted small" style="margin-top:8px">
            <strong>Defense:</strong> hold 1-3 years of expenses in cash + short
            bonds so you can suspend equity withdrawals during a market crash
            (the "bucket strategy"). Reduces sequence risk dramatically without
            lowering long-term return.
        </p>
    `;
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
