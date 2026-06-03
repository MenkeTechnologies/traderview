// State Tax Estimator — top 10 states by trader-relevance.
// Compares effective state tax burden across states given income level.
// Surfaces which states would save you $ if relocating.

import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

// Simplified marginal-rate brackets (2024 — Single filer).
const STATES = [
    { code: 'FL', name: 'Florida',         brackets: [], note: 'No state income tax. Best for traders.' },
    { code: 'TX', name: 'Texas',           brackets: [], note: 'No state income tax.' },
    { code: 'WA', name: 'Washington',      brackets: [[Infinity, 0]],
        note: 'No income tax, but 7% cap-gains tax > $250k.' },
    { code: 'NV', name: 'Nevada',          brackets: [], note: 'No state income tax.' },
    { code: 'TN', name: 'Tennessee',       brackets: [], note: 'No income tax since 2021 (Hall Tax repealed).' },
    { code: 'SD', name: 'South Dakota',    brackets: [], note: 'No state income tax.' },
    { code: 'WY', name: 'Wyoming',         brackets: [], note: 'No state income tax.' },
    { code: 'AK', name: 'Alaska',          brackets: [], note: 'No state income tax.' },
    { code: 'NH', name: 'New Hampshire',   brackets: [[Infinity, 0.03]],
        note: '3% on interest + dividends only (sunsetting by 2027).' },
    { code: 'NY', name: 'New York',        brackets: [
        [8_500,  0.04], [11_700, 0.045], [13_900, 0.0525], [80_650, 0.055],
        [215_400,0.06], [1_077_550,0.0685], [5_000_000,0.0965], [Infinity,0.103],
    ], note: 'NYC adds 3-4% local tax.' },
    { code: 'CA', name: 'California',      brackets: [
        [10_412, 0.01], [24_684, 0.02], [38_959, 0.04], [54_081, 0.06],
        [68_350, 0.08], [349_137,0.093], [418_961,0.103], [698_271,0.113], [Infinity,0.123],
    ], note: 'Highest top rate in the US (13.3% > $1M with mental-health add).' },
    { code: 'NJ', name: 'New Jersey',      brackets: [
        [20_000, 0.014], [35_000, 0.0175], [40_000, 0.035], [75_000, 0.0553],
        [500_000, 0.0637], [1_000_000, 0.0897], [Infinity, 0.1075],
    ], note: '' },
    { code: 'MA', name: 'Massachusetts',   brackets: [[1_000_000, 0.05], [Infinity, 0.09]],
        note: 'Flat 5% + 4% surcharge > $1M (Millionaire Tax).' },
    { code: 'OR', name: 'Oregon',          brackets: [
        [4_300, 0.0475], [10_750, 0.0675], [125_000, 0.0875], [Infinity, 0.099],
    ], note: '' },
    { code: 'IL', name: 'Illinois',        brackets: [[Infinity, 0.0495]],
        note: 'Flat 4.95%.' },
    { code: 'CO', name: 'Colorado',        brackets: [[Infinity, 0.044]],
        note: 'Flat 4.4%.' },
    { code: 'AZ', name: 'Arizona',         brackets: [[Infinity, 0.025]],
        note: 'Flat 2.5%.' },
];

let state = { income: 200_000 };

export async function renderStateTax(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.statetax.h1.title">// STATE TAX ESTIMATOR</span></h1>
        <p class="muted small" data-i18n="view.statetax.hint.intro">
            Compare effective state tax burden across states at your income level.
            Trader-relocation hot list (FL, TX, WA, NV, TN) at top. NY / CA / NJ
            at the bottom — relocating could save 5-13% of your gross.
        </p>
        <div class="chart-panel">
            <form id="st-form" class="inline-form">
                <label><span data-i18n="view.statetax.label.income">Annual income ($)</span>
                    <input type="number" step="1000" name="income" value="${state.income}"></label>
                <button class="primary" type="submit" data-i18n="view.statetax.btn.recompute">Recompute</button>
            </form>
        </div>
        <div id="st-table" class="chart-panel"></div>
    `;
    document.getElementById('st-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.income = Number(fd.get('income')) || 0;
        render();
    });
    render();
}

function computeTax(income, brackets) {
    let owe = 0;
    let lastCap = 0;
    for (const [cap, rate] of brackets) {
        const slice = Math.max(0, Math.min(income, cap) - lastCap);
        owe += slice * rate;
        if (income <= cap) break;
        lastCap = cap;
    }
    return owe;
}

function render() {
    const el = document.getElementById('st-table');
    if (!el) return;
    const rows = STATES.map(s => {
        const tax = computeTax(state.income, s.brackets);
        const effectiveRate = state.income > 0 ? tax / state.income : 0;
        return { ...s, tax, effectiveRate };
    });
    rows.sort((a, b) => a.tax - b.tax);
    const cheapest = rows[0];
    el.innerHTML = `
        <h2 data-i18n="view.statetax.h2.results">Estimated state income tax</h2>
        <p class="muted small">
            <span data-i18n="view.statetax.summary.cheapest">Cheapest:</span> <strong>${esc(cheapest.name)}</strong>
            ($${cheapest.tax.toLocaleString(undefined, { maximumFractionDigits: 0 })})
        </p>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.statetax.th.state">State</th>
                <th data-i18n="view.statetax.th.tax">Tax</th>
                <th data-i18n="view.statetax.th.effective">Effective rate</th>
                <th data-i18n="view.statetax.th.vs_cheapest">Vs. cheapest</th>
                <th data-i18n="view.statetax.th.note">Note</th>
            </tr></thead>
            <tbody>${rows.map(r => {
                const diff = r.tax - cheapest.tax;
                const cls = r.tax === 0 ? 'pos' : r.effectiveRate >= 0.09 ? 'neg' : '';
                return `<tr>
                    <td><strong>${esc(r.name)}</strong> <span class="muted">(${esc(r.code)})</span></td>
                    <td class="${cls}">$${r.tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="${cls}">${(r.effectiveRate * 100).toFixed(2)}%</td>
                    <td class="${diff > 0 ? 'neg' : ''}">${diff > 0 ? '+$' + diff.toLocaleString(undefined, { maximumFractionDigits: 0 }) : '—'}</td>
                    <td class="muted small">${esc(r.note)}</td>
                </tr>`;
            }).join('')}</tbody>
        </table>
        <p class="muted small" data-i18n="view.statetax.disclaimer" style="margin-top:10px">
            Single-filer ordinary income only. Excludes local (NYC, San Francisco, Denver),
            property, sales tax. Federal still applies. Capital gains taxed differently in
            some states (WA 7% > $250k). Confirm domicile rules before relocating (NY claws
            back former residents aggressively).
        </p>
    `;
}
