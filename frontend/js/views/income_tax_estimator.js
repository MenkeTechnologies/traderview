// Income Tax Estimator — TY2025 federal estimate (IRS Rev. Proc. 2024-40).
// Computation runs server-side via /calc/income-tax-estimator
// (traderview-core::income_tax) — a faithful port of the former client-side
// bracket/LTCG/FICA math, Python-pinned and unit-tested. The brackets, standard
// deductions, LTCG breakpoints, and FICA thresholds live in the Rust module.
// Class-based styling for release-WebKit correctness.

import { api } from '../api.js';
import { applyUiI18n } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const fmt = (n, d) => (n == null || !Number.isFinite(Number(n)) ? '—' : Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d }));
const pct = (r) => (r == null ? '—' : Number(r).toFixed(1) + '%');
const VIEW = 'income-tax-estimator';
let lastReport = null;

export async function renderIncomeTaxEstimator(mount, _state) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.income_tax_estimator.title">// INCOME TAX ESTIMATOR · TY2025</span></h1>
        <p class="muted small" data-i18n-html="view.income_tax_estimator.intro">
            Federal estimate against TY2025 IRS-published brackets (Rev. Proc. 2024-40).
            Includes ordinary income + LTCG segregated at 0/15/20%, standard or
            itemized deduction, FICA, and Additional Medicare 0.9% over the threshold.
            Does <strong>not</strong> include state tax, AMT, NIIT, self-employment tax,
            or credits — for those, use a CPA or tax software.
        </p>
        <div class="chart-panel">
            <div class="re-grid">
                <label><span class="muted small">Filing status</span>
                    <select id="tx-status">
                        <option value="single" selected>Single</option>
                        <option value="mfj">Married filing jointly</option>
                        <option value="mfs">Married filing separately</option>
                        <option value="hoh">Head of household</option>
                    </select></label>
                <label><span class="muted small">W-2 / ordinary income $</span>
                    <input type="number" id="tx-wages" step="1000" min="0" value="180000"></label>
                <label><span class="muted small">Short-term cap gains $</span>
                    <input type="number" id="tx-stcg" step="100" value="0"></label>
                <label><span class="muted small">Long-term cap gains $</span>
                    <input type="number" id="tx-ltcg" step="100" value="0"></label>
                <label><span class="muted small">Qualified dividends $</span>
                    <input type="number" id="tx-qdiv" step="100" value="0"></label>
                <label><span class="muted small">Itemized override $ (0 = use std)</span>
                    <input type="number" id="tx-item" step="500" value="0"></label>
                <label><span class="muted small">Pre-tax 401(k) / HSA $</span>
                    <input type="number" id="tx-pretax" step="500" value="23500"></label>
            </div>
            <button class="btn btn-sm primary" id="tx-run" data-shortcut="r">⚡ Compute</button>
            <div id="tx-tools" class="ce-toolbar"></div>
            <div id="tx-result" class="re-result"></div>
        </div>
    `;
    applyUiI18n(mount);

    const num = (id) => parseFloat(mount.querySelector(id).value) || 0;
    const readBody = () => ({
        filing_status: mount.querySelector('#tx-status').value,
        wages_usd: num('#tx-wages'),
        short_term_gains_usd: num('#tx-stcg'),
        long_term_gains_usd: num('#tx-ltcg'),
        qualified_dividends_usd: num('#tx-qdiv'),
        itemized_override_usd: num('#tx-item'),
        pretax_401k_hsa_usd: num('#tx-pretax'),
    });
    const compute = async () => {
        try {
            const r = await api.calcIncomeTax(readBody());
            if (!viewIsCurrent(tok)) return;
            lastReport = r;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || 'Could not compute the estimate.', { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#tx-tools'), {
        viewId: VIEW, link: false, filename: 'income-tax-estimator.csv',
        getRows: () => reportRows(lastReport),
    });
    mount.querySelectorAll('#tx-status, #tx-wages, #tx-stcg, #tx-ltcg, #tx-qdiv, #tx-item, #tx-pretax').forEach(el => {
        el.addEventListener('input', debounce(compute, 250));
    });
    mount.querySelector('#tx-run').addEventListener('click', compute);
    compute();
}

function reportRows(r) {
    if (!r || !r.valid) return [];
    const rows = [['metric', 'value'],
        ['total_income_usd', r.total_income_usd],
        ['ordinary_tax_usd', r.ordinary_tax_usd],
        ['preferential_tax_usd', r.preferential_tax_usd],
        ['fica_total_usd', r.fica_total_usd],
        ['total_tax_usd', r.total_tax_usd],
        ['effective_rate_pct', r.effective_rate_pct],
        ['take_home_usd', r.take_home_usd],
        [], ['rate_pct', 'from_usd', 'to_usd', 'amount_usd', 'tax_usd']];
    for (const b of r.ordinary_breakdown) rows.push([b.rate_pct, b.from_usd, b.to_usd == null ? '' : b.to_usd, b.amount_usd, b.tax_usd]);
    return rows;
}

function bracketRows(rows) {
    return rows.map(row => `<tr>
        <td>${pct(row.rate_pct)}</td>
        <td class="muted">$${fmt(row.from_usd, 0)} → ${row.to_usd == null ? '∞' : '$' + fmt(row.to_usd, 0)}</td>
        <td>$${fmt(row.amount_usd, 0)}</td>
        <td><strong>$${fmt(row.tax_usd, 0)}</strong></td>
    </tr>`).join('');
}

function renderResult(mount, r) {
    const result = mount.querySelector('#tx-result');
    if (!r.valid) { result.innerHTML = ''; return; }
    // Where the total tax comes from: ordinary income tax, preferential, and FICA.
    const chart = enh.svgBarChart([
        { label: 'Ordinary', value: r.ordinary_tax_usd },
        { label: 'LTCG/QD', value: r.preferential_tax_usd },
        { label: 'FICA', value: r.fica_total_usd },
    ]);
    result.innerHTML = `
        <div class="cards">
            <div class="card"><div class="label">Total income</div><div class="value">$${fmt(r.total_income_usd, 0)}</div></div>
            <div class="card"><div class="label">Ordinary federal</div><div class="value neg">-$${fmt(r.ordinary_tax_usd, 0)}</div><div class="muted small">marginal ${pct(r.marginal_rate_pct)}</div></div>
            <div class="card"><div class="label">LTCG / qualified</div><div class="value neg">-$${fmt(r.preferential_tax_usd, 0)}</div></div>
            <div class="card"><div class="label">FICA</div><div class="value neg">-$${fmt(r.fica_total_usd, 0)}</div><div class="muted small">SS $${fmt(r.social_security_tax_usd, 0)} · Medi $${fmt(r.medicare_tax_usd, 0)}</div></div>
            <div class="card"><div class="label">Effective tax rate</div><div class="value">${pct(r.effective_rate_pct)}</div></div>
            <div class="card"><div class="label">Est. take-home</div><div class="value pos">$${fmt(r.take_home_usd, 0)}</div></div>
        </div>
        ${chart}
        <h3 class="section-title">Ordinary bracket walk</h3>
        <table class="trades" data-table-key="tx-ord">
            <thead><tr><th>Rate</th><th>Bracket range</th><th>Income in bracket</th><th>Tax in bracket</th></tr></thead>
            <tbody>${bracketRows(r.ordinary_breakdown)}</tbody>
        </table>
        ${r.ltcg_breakdown.length ? `
        <h3 class="section-title">LTCG bracket walk (stacked above ordinary)</h3>
        <table class="trades" data-table-key="tx-pref">
            <thead><tr><th>Rate</th><th>Bracket range</th><th>Income in bracket</th><th>Tax in bracket</th></tr></thead>
            <tbody>${bracketRows(r.ltcg_breakdown)}</tbody>
        </table>` : ''}
    `;
}
