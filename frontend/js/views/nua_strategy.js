// NUA (Net Unrealized Appreciation) — IRC § 402(e)(4).
// Employer stock in 401(k) can be distributed IN-KIND in a lump-sum
// distribution. Cost basis taxed as ORDINARY income at distribution.
// Appreciation (NUA) taxed at LONG-TERM CAP GAINS when sold (even if sold
// next day). Massive win for employees with low-basis appreciated employer stock.
//
// Requires: lump-sum distribution of full 401(k) in one tax year,
// triggering event (separation/death/disability/age 59½), no rollover of
// employer stock to IRA.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    employer_stock_basis: 50_000,
    employer_stock_fmv: 500_000,
    other_401k_balance: 200_000,
    ordinary_rate: 0.32,
    lt_cap_gains_rate: 0.20,
    niit_rate: 0.038,
    held_5_years_after_dist: true,
};

export async function renderNuaStrategy(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.nua.h1.title">// NUA STRATEGY</span></h1>
        <p class="muted small" data-i18n="view.nua.hint.intro">
            <strong>NUA (Net Unrealized Appreciation):</strong> employer stock in 401(k)
            distributed in-kind. Cost basis taxed ORDINARY at distribution. Appreciation
            taxed LT cap gains when sold (even sold next day). Massive win if your
            employer stock is low-basis + highly appreciated. Compare to rollover.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.nua.h2.inputs">Inputs</h2>
            <form id="nua-form" class="inline-form">
                <label><span data-i18n="view.nua.label.basis">Employer stock cost basis ($)</span>
                    <input type="number" step="0.01" name="employer_stock_basis" value="${state.employer_stock_basis}"></label>
                <label><span data-i18n="view.nua.label.fmv">Employer stock FMV ($)</span>
                    <input type="number" step="0.01" name="employer_stock_fmv" value="${state.employer_stock_fmv}"></label>
                <label><span data-i18n="view.nua.label.other_balance">Other 401(k) balance ($)</span>
                    <input type="number" step="0.01" name="other_401k_balance" value="${state.other_401k_balance}"></label>
                <label><span data-i18n="view.nua.label.ordinary_rate">Ordinary rate %</span>
                    <input type="number" step="0.5" name="ordinary_rate" value="${(state.ordinary_rate * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.nua.label.lt_cap_gains_rate">LT cap-gains rate %</span>
                    <input type="number" step="0.5" name="lt_cap_gains_rate" value="${(state.lt_cap_gains_rate * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.nua.label.niit_rate">NIIT %</span>
                    <input type="number" step="0.1" name="niit_rate" value="${(state.niit_rate * 100).toFixed(1)}"></label>
                <button class="primary" type="submit" data-i18n="view.nua.btn.compare">Compare</button>
            </form>
        </div>
        <div id="nua-output"></div>
    `;
    document.getElementById('nua-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.employer_stock_basis = Number(fd.get('employer_stock_basis')) || 0;
        state.employer_stock_fmv = Number(fd.get('employer_stock_fmv')) || 0;
        state.other_401k_balance = Number(fd.get('other_401k_balance')) || 0;
        state.ordinary_rate = (Number(fd.get('ordinary_rate')) || 32) / 100;
        state.lt_cap_gains_rate = (Number(fd.get('lt_cap_gains_rate')) || 20) / 100;
        state.niit_rate = (Number(fd.get('niit_rate')) || 3.8) / 100;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('nua-output');
    if (!el) return;
    const nua = state.employer_stock_fmv - state.employer_stock_basis;
    const basisRatio = state.employer_stock_basis / state.employer_stock_fmv;

    // NUA strategy: basis ordinary, NUA LT cap gain
    const nuaTaxOnBasis = state.employer_stock_basis * state.ordinary_rate;
    const nuaTaxOnAppreciation = nua * (state.lt_cap_gains_rate + state.niit_rate);
    const nuaTotalTax = nuaTaxOnBasis + nuaTaxOnAppreciation;
    const nuaNetProceeds = state.employer_stock_fmv - nuaTotalTax;

    // Alternative: roll everything to IRA, eventually pull as ordinary
    const rolloverTax = state.employer_stock_fmv * state.ordinary_rate;
    const rolloverNetProceeds = state.employer_stock_fmv - rolloverTax;

    const advantage = nuaNetProceeds - rolloverNetProceeds;
    const isWin = advantage > 0;
    const cls = isWin ? 'pos' : 'neg';

    el.innerHTML = `
        <div class="chart-panel ${cls}">
            <h2 data-i18n="view.nua.h2.headline">Bottom line</h2>
            <div class="cards">
                <div class="card ${cls}">
                    <div class="label" data-i18n="view.nua.card.advantage">NUA advantage (vs rollover)</div>
                    <div class="value">$${advantage.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.nua.card.nua_amount">NUA (appreciation)</div>
                    <div class="value">$${nua.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.nua.card.basis_ratio">Basis as % of FMV</div>
                    <div class="value">${(basisRatio * 100).toFixed(1)}%</div>
                </div>
            </div>
            <p style="margin-top:10px">
                <strong>${esc(isWin ? t('view.nua.rec.use_nua') : t('view.nua.rec.roll_over'))}</strong>
            </p>
        </div>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.nua.h2.nua_strategy">NUA strategy</h2>
                <table class="trades"><tbody>
                    <tr><td data-i18n="view.nua.row.basis_ordinary">Basis taxed as ordinary</td>
                        <td>$${state.employer_stock_basis.toLocaleString()}</td></tr>
                    <tr><td data-i18n="view.nua.row.ordinary_tax">Tax on basis</td>
                        <td class="neg">$${nuaTaxOnBasis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                    <tr><td data-i18n="view.nua.row.appreciation_lt">Appreciation taxed LT cap-gains</td>
                        <td>$${nua.toLocaleString()}</td></tr>
                    <tr><td data-i18n="view.nua.row.cap_gains_tax">Tax on NUA (incl NIIT)</td>
                        <td class="neg">$${nuaTaxOnAppreciation.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                    <tr><td><strong data-i18n="view.nua.row.total_tax">Total tax</strong></td>
                        <td><strong class="neg">$${nuaTotalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</strong></td></tr>
                    <tr><td><strong data-i18n="view.nua.row.net_proceeds">Net proceeds</strong></td>
                        <td><strong class="pos">$${nuaNetProceeds.toLocaleString(undefined, { maximumFractionDigits: 0 })}</strong></td></tr>
                </tbody></table>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.nua.h2.rollover">Rollover scenario (alternative)</h2>
                <table class="trades"><tbody>
                    <tr><td data-i18n="view.nua.row.full_fmv">Full FMV → IRA</td>
                        <td>$${state.employer_stock_fmv.toLocaleString()}</td></tr>
                    <tr><td data-i18n="view.nua.row.eventual_ordinary">Eventually withdrawn as ordinary</td>
                        <td class="muted">100% taxed as ordinary</td></tr>
                    <tr><td><strong data-i18n="view.nua.row.total_tax_2">Total tax</strong></td>
                        <td><strong class="neg">$${rolloverTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</strong></td></tr>
                    <tr><td><strong data-i18n="view.nua.row.net_proceeds_2">Net proceeds</strong></td>
                        <td><strong class="pos">$${rolloverNetProceeds.toLocaleString(undefined, { maximumFractionDigits: 0 })}</strong></td></tr>
                </tbody></table>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.nua.h2.requirements">Requirements</h2>
            <ol class="muted small">
                <li data-i18n="view.nua.req.lump_sum">Lump-sum distribution: entire 401(k) balance distributed in ONE tax year</li>
                <li data-i18n="view.nua.req.triggering_event">Triggering event: separation, death, disability, age 59½</li>
                <li data-i18n="view.nua.req.in_kind">Employer stock distributed IN-KIND (don't sell inside the plan first)</li>
                <li data-i18n="view.nua.req.no_rollover">Do NOT roll employer stock to IRA — that destroys NUA treatment</li>
                <li data-i18n="view.nua.req.other_assets">Non-employer-stock assets CAN be rolled to IRA in same lump-sum</li>
            </ol>
            <p class="muted small" data-i18n="view.nua.note">
                Rule of thumb: NUA wins when basis ratio &lt; ~40% AND you'll hold stock 5+ years
                after distribution. Below 25% basis ratio, NUA almost always wins. SECURE 2.0
                considered restricting NUA but the rule survived intact.
            </p>
        </div>
    `;
}
