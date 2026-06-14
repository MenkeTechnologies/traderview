// Sequence-of-Returns Risk — the same annual returns in four orderings (forward,
// reversed, worst-first, best-first) run against an inflation-adjusted withdrawal.
// Identical mean, very different terminal balances. Computation runs server-side
// via /calc/sequence-of-returns (traderview-core::sequence_of_returns) — a
// faithful port of the former client-side simulator, Python-pinned and tested.
// Class-based styling for release-WebKit correctness.

import { api } from '../api.js';
import { applyUiI18n } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import { esc } from '../util.js';
import * as enh from '../calc_enhance.js';

const DEFAULT_RETURNS = [
    -9.1, -11.9, -22.1, 28.7, 10.9, 4.9, 15.8, 5.5, -37.0, 26.5,
    15.1, 2.1, 16.0, 32.4, 13.7, 1.4, 12.0, 21.8, -4.4, 31.5,
    18.4, 28.7, -18.1, 26.3, 25.0,
];

const fmt = (n, d) => (n == null || !Number.isFinite(Number(n)) ? '—' : Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d }));
const VIEW = 'sequence-of-returns';
let lastReport = null;
let lastStart = 0;

export async function renderSequenceOfReturns(mount, _state) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
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
            <div class="re-grid">
                <label><span class="muted small">Starting balance $</span>
                    <input type="number" id="sr-start" step="10000" min="0" value="1000000"></label>
                <label><span class="muted small">Annual withdrawal $</span>
                    <input type="number" id="sr-wd" step="1000" min="0" value="40000"></label>
                <label><span class="muted small">Inflation adjust withdrawal %/yr</span>
                    <input type="number" id="sr-infl" step="0.1" min="0" max="20" value="3.0"></label>
                <label><span class="muted small">Returns sequence (%/yr, comma-separated)</span>
                    <input type="text" id="sr-seq" value="${DEFAULT_RETURNS.join(',')}"></label>
            </div>
            <button class="btn btn-sm primary" id="sr-run">⚡ Simulate</button>
            <div id="sr-tools" class="ce-toolbar"></div>
            <div id="sr-result" class="re-result"></div>
        </div>
    `;
    applyUiI18n(mount);

    const readBody = () => ({
        returns_pct: (mount.querySelector('#sr-seq').value || '').split(',').map(s => parseFloat(s.trim())).filter(n => Number.isFinite(n)),
        start_balance_usd: parseFloat(mount.querySelector('#sr-start').value) || 0,
        annual_withdrawal_usd: parseFloat(mount.querySelector('#sr-wd').value) || 0,
        inflation_pct: parseFloat(mount.querySelector('#sr-infl').value) || 0,
    });
    const compute = async () => {
        const body = readBody();
        if (body.returns_pct.length < 5) {
            mount.querySelector('#sr-result').innerHTML = `<p class="muted">Need at least 5 annual returns.</p>`;
            lastReport = null; return;
        }
        try {
            const r = await api.calcSequenceOfReturns(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastStart = body.start_balance_usd;
            renderResult(mount, r, body.start_balance_usd);
        } catch (err) {
            showToast(err.message || 'Could not run the simulation.', { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#sr-tools'), {
        viewId: VIEW, link: false, filename: 'sequence-of-returns.csv',
        getRows: () => reportRows(lastReport),
    });
    mount.querySelectorAll('#sr-start, #sr-wd, #sr-infl, #sr-seq').forEach(el => {
        el.addEventListener('input', debounce(compute, 250));
    });
    mount.querySelector('#sr-run').addEventListener('click', compute);
    compute();
}

function reportRows(r) {
    if (!r || !r.valid) return [];
    return [
        ['scenario', 'end_balance_usd', 'failed_at_year', 'total_withdrawn_usd'],
        ...r.scenarios.map(s => [s.name, s.end_balance_usd, s.failed_at_year == null ? '' : s.failed_at_year, s.total_withdrawn_usd]),
    ];
}

function renderResult(mount, r, start) {
    const result = mount.querySelector('#sr-result');
    if (!r.valid) { result.innerHTML = ''; return; }
    // End-balance comparison across the four orderings (failures clamped to 0 so
    // the bar scale stays readable — the card text shows the failure year).
    const chart = enh.svgBarChart(r.scenarios.map(s => ({ label: s.name.split(' ')[0], value: Math.max(0, s.end_balance_usd) })));
    const cardCls = (s) => (s.failed_at_year != null || s.end_balance_usd <= 0) ? 'neg'
        : s.name.startsWith('Best') ? 'pos' : s.name.startsWith('Worst') ? 'neg' : '';
    result.innerHTML = `
        <p class="muted small">Arithmetic mean return: <strong>${fmt(r.mean_return_pct, 2)}%</strong>/yr across ${r.years} years.</p>
        <div class="cards">
            ${r.scenarios.map(s => `<div class="card">
                <div class="label">${esc(s.name)}</div>
                <div class="value ${cardCls(s)}">${s.failed_at_year != null ? 'FAILED yr ' + s.failed_at_year : '$' + fmt(s.end_balance_usd, 0)}</div>
                <div class="muted small">Total withdrawn: $${fmt(s.total_withdrawn_usd, 0)}</div>
            </div>`).join('')}
        </div>
        ${chart}
        <h3 class="section-title">Year-by-year breakdown</h3>
        <table class="trades" data-table-key="sr-walk">
            <thead><tr>
                <th>Year</th>
                ${r.scenarios.map(s => `<th>${esc(s.name.split(' ')[0])}<br/><span class="muted small">close</span></th>`).join('')}
            </tr></thead>
            <tbody>${Array.from({ length: r.years }, (_, i) => `<tr>
                <td>${i + 1}</td>
                ${r.scenarios.map(s => {
                    const cell = s.path[i];
                    if (!cell) return `<td class="muted">—</td>`;
                    const cls = cell.close_usd <= 0 ? 'neg' : (cell.close_usd < start * 0.5 ? 'muted' : '');
                    return `<td class="${cls}"><span class="muted small">[${fmt(cell.return_pct, 1)}%]</span> $${fmt(cell.close_usd, 0)}</td>`;
                }).join('')}
            </tr>`).join('')}</tbody>
        </table>
        <p class="muted small">
            <strong>Defense:</strong> hold 1-3 years of expenses in cash + short
            bonds so you can suspend equity withdrawals during a market crash
            (the "bucket strategy"). Reduces sequence risk dramatically without
            lowering long-term return.
        </p>
    `;
}
