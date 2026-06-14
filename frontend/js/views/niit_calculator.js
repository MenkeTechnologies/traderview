// Net Investment Income Tax (NIIT) — the ACA 3.8% surtax on investment income
// for high earners, on the lesser of net investment income or MAGI over the
// (2013-frozen) threshold. Computation runs server-side via /calc/niit
// (traderview-core::niit) — a faithful port of the former client-side math,
// Python-pinned and unit-tested. Class-based styling for release-WebKit.

import { api } from '../api.js';
import { applyUiI18n } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import { esc } from '../util.js';
import * as enh from '../calc_enhance.js';

const fmt = (n, d) => (n == null || !Number.isFinite(Number(n)) ? '—' : Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d }));
const VIEW = 'niit-calculator';
let lastReport = null;
let lastBody = null;

export async function renderNiitCalculator(mount, _state) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.niit_calculator.title">// NIIT · 3.8% INVESTMENT SURTAX</span></h1>
        <p class="muted small" data-i18n-html="view.niit_calculator.intro">
            Affordable Care Act's <strong>3.8% Net Investment Income Tax</strong>.
            Applies to lesser of (a) net investment income, or (b) MAGI over the
            threshold. Thresholds <strong>have NOT been indexed since 2013</strong> —
            $200k single / $250k MFJ / $125k MFS — increasingly affecting middle
            earners as nominal income drifts upward.
        </p>
        <div class="chart-panel">
            <div class="re-grid">
                <label><span class="muted small">Filing status</span>
                    <select id="niit-status">
                        <option value="single">Single</option>
                        <option value="mfj" selected>MFJ</option>
                        <option value="mfs">MFS</option>
                        <option value="hoh">HoH</option>
                    </select></label>
                <label><span class="muted small">MAGI $</span>
                    <input type="number" id="niit-magi" step="1000" min="0" value="320000"></label>
                <label><span class="muted small">Interest income $</span>
                    <input type="number" id="niit-interest" step="100" min="0" value="5000"></label>
                <label><span class="muted small">Dividends (qual + ord) $</span>
                    <input type="number" id="niit-div" step="100" min="0" value="12000"></label>
                <label><span class="muted small">Net capital gains $</span>
                    <input type="number" id="niit-gains" step="100" value="40000"></label>
                <label><span class="muted small">Rental net income $</span>
                    <input type="number" id="niit-rent" step="100" value="8000"></label>
                <label><span class="muted small">Royalties + passive biz $</span>
                    <input type="number" id="niit-passive" step="100" min="0" value="0"></label>
                <label><span class="muted small">Allocable deductions $</span>
                    <input type="number" id="niit-ded" step="100" min="0" value="0"></label>
            </div>
            <button class="btn btn-sm primary" id="niit-run">⚡ Compute</button>
            <div id="niit-tools" class="ce-toolbar"></div>
            <div id="niit-result" class="re-result"></div>
        </div>
    `;
    applyUiI18n(mount);

    const num = (id) => parseFloat(mount.querySelector(id).value) || 0;
    const readBody = () => ({
        filing_status: mount.querySelector('#niit-status').value,
        magi_usd: num('#niit-magi'),
        interest_usd: num('#niit-interest'),
        dividends_usd: num('#niit-div'),
        net_capital_gains_usd: num('#niit-gains'),
        rental_net_income_usd: num('#niit-rent'),
        royalties_passive_usd: num('#niit-passive'),
        allocable_deductions_usd: num('#niit-ded'),
    });
    const compute = async () => {
        const body = readBody();
        try {
            const r = await api.calcNiit(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body);
        } catch (err) {
            showToast(err.message || 'Could not compute the NIIT.', { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#niit-tools'), {
        viewId: VIEW, link: false, filename: 'niit.csv',
        getRows: () => reportRows(lastReport),
    });
    mount.querySelectorAll('#niit-status, #niit-magi, #niit-interest, #niit-div, #niit-gains, #niit-rent, #niit-passive, #niit-ded').forEach(el => {
        el.addEventListener('input', debounce(compute, 250));
    });
    mount.querySelector('#niit-run').addEventListener('click', compute);
    compute();
}

function reportRows(r) {
    if (!r || !r.valid) return [];
    return [
        ['metric', 'value'],
        ['threshold_usd', r.threshold_usd],
        ['magi_excess_usd', r.magi_excess_usd],
        ['net_investment_income_usd', r.net_investment_income_usd],
        ['subject_to_niit_usd', r.subject_to_niit_usd],
        ['niit_tax_usd', r.niit_tax_usd],
        ['effective_on_investment_pct', r.effective_on_investment_pct],
    ];
}

function renderResult(mount, r, body) {
    const result = mount.querySelector('#niit-result');
    if (!r.valid) {
        result.innerHTML = `<p class="muted">Enter a non-negative MAGI.</p>`;
        return;
    }
    // What's subject to NIIT: net investment income vs MAGI excess — the lesser binds.
    const chart = enh.svgBarChart([
        { label: 'Net inv', value: r.net_investment_income_usd },
        { label: 'MAGI exc', value: r.magi_excess_usd },
        { label: 'Subject', value: r.subject_to_niit_usd },
    ]);
    const note = r.magi_excess_usd <= 0
        ? '<p class="pos small"><strong>Not subject to NIIT</strong> — MAGI below threshold.</p>'
        : r.net_investment_income_usd <= 0
            ? '<p class="pos small"><strong>No net investment income</strong> — NIIT does not apply.</p>'
            : '';
    result.innerHTML = `
        <div class="cards">
            <div class="card"><div class="label">MAGI threshold</div><div class="value">$${fmt(r.threshold_usd, 0)}</div><div class="muted small">${esc(body.filing_status.toUpperCase())} (frozen 2013)</div></div>
            <div class="card"><div class="label">MAGI excess</div><div class="value ${r.magi_excess_usd > 0 ? 'neg' : 'pos'}">$${fmt(r.magi_excess_usd, 0)}</div></div>
            <div class="card"><div class="label">Net investment income</div><div class="value">$${fmt(r.net_investment_income_usd, 0)}</div></div>
            <div class="card"><div class="label">Subject to NIIT</div><div class="value">$${fmt(r.subject_to_niit_usd, 0)}</div><div class="muted small">min(net inv, MAGI excess)</div></div>
            <div class="card"><div class="label">NIIT @ 3.8%</div><div class="value neg"><strong>$${fmt(r.niit_tax_usd, 0)}</strong></div></div>
            <div class="card"><div class="label">Effective on investment $</div><div class="value">${fmt(r.effective_on_investment_pct, 2)}%</div></div>
        </div>
        ${chart}
        <h3 class="section-title">Investment income detail</h3>
        <table class="trades" data-table-key="niit-detail">
            <thead><tr><th>Component</th><th>Amount</th></tr></thead>
            <tbody>
                <tr><td>Interest</td><td>$${fmt(body.interest_usd, 0)}</td></tr>
                <tr><td>Dividends</td><td>$${fmt(body.dividends_usd, 0)}</td></tr>
                <tr><td>Net capital gains</td><td>$${fmt(body.net_capital_gains_usd, 0)}</td></tr>
                <tr><td>Rental net income</td><td>$${fmt(body.rental_net_income_usd, 0)}</td></tr>
                <tr><td>Royalties / passive biz</td><td>$${fmt(body.royalties_passive_usd, 0)}</td></tr>
                <tr><td class="muted">Less: allocable deductions</td><td class="muted">-$${fmt(body.allocable_deductions_usd, 0)}</td></tr>
                <tr><td><strong>Net investment income</strong></td><td><strong>$${fmt(r.net_investment_income_usd, 0)}</strong></td></tr>
            </tbody>
        </table>
        ${note}
    `;
}
