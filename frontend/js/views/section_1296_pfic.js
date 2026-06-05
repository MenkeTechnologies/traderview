// IRC § 1296 PFIC Mark-to-Market Election (Form 8621).
// PFIC = foreign corp where ≥ 75% income passive OR ≥ 50% assets generate passive income.
// Default § 1291 regime: ordinary income on excess distributions + interest charge (compounding).
// § 1296 MTM election: annual ordinary income/loss on gain/loss; clean accounting.
// § 1295 QEF election: pass through PFIC income annually. All require Form 8621 annually.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    holding_value: 0,
    cost_basis: 0,
    holding_years: 5,
    annual_distribution: 0,
    election: 'default_1291',
    annual_growth_rate: 0.08,
    afr_rate: 0.05,
    marginal_rate: 0.37,
    ltcg_rate: 0.20,
};

export async function renderSection1296Pfic(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.pfic.h1.title">// § 1296 PFIC MTM ELECTION (FORM 8621)</span></h1>
        <p class="muted small" data-i18n="view.pfic.hint.intro">
            PFIC = foreign corp where <strong>≥ 75% income passive OR ≥ 50% assets generate
            passive</strong>. Default <strong>§ 1291 regime:</strong> excess distributions
            (&gt; 125% prior 3-yr avg) ordinary income + interest charge (compounding back to
            year 1). <strong>§ 1296 MTM:</strong> annual ordinary income on gain (loss limited).
            <strong>§ 1295 QEF:</strong> pass-through annual income, preserves cap gain character.
            <strong>Form 8621 annually</strong>. Common trap: foreign mutual funds, ETFs (e.g.
            European UCITS).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.pfic.h2.inputs">Inputs</h2>
            <form id="pfic-form" class="inline-form">
                <label><span data-i18n="view.pfic.label.value">Holding current value ($)</span>
                    <input type="number" step="0.01" name="holding_value" value="${state.holding_value}"></label>
                <label><span data-i18n="view.pfic.label.basis">Cost basis ($)</span>
                    <input type="number" step="0.01" name="cost_basis" value="${state.cost_basis}"></label>
                <label><span data-i18n="view.pfic.label.years">Years held</span>
                    <input type="number" step="1" name="holding_years" value="${state.holding_years}"></label>
                <label><span data-i18n="view.pfic.label.distribution">Annual distribution ($)</span>
                    <input type="number" step="0.01" name="annual_distribution" value="${state.annual_distribution}"></label>
                <label><span data-i18n="view.pfic.label.election">Election</span>
                    <select name="election">
                        <option value="default_1291" ${state.election === 'default_1291' ? 'selected' : ''}>Default § 1291 (deferred + interest charge)</option>
                        <option value="mtm_1296" ${state.election === 'mtm_1296' ? 'selected' : ''}>§ 1296 Mark-to-Market</option>
                        <option value="qef_1295" ${state.election === 'qef_1295' ? 'selected' : ''}>§ 1295 Qualified Electing Fund</option>
                    </select>
                </label>
                <label><span data-i18n="view.pfic.label.growth">Expected annual growth %</span>
                    <input type="number" step="0.01" name="annual_growth_rate" value="${state.annual_growth_rate}"></label>
                <label><span data-i18n="view.pfic.label.afr">AFR (interest charge rate)</span>
                    <input type="number" step="0.001" name="afr_rate" value="${state.afr_rate}"></label>
                <label><span data-i18n="view.pfic.label.marginal">Marginal ordinary %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <label><span data-i18n="view.pfic.label.ltcg">LTCG %</span>
                    <input type="number" step="0.01" name="ltcg_rate" value="${state.ltcg_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.pfic.btn.compute">Compute</button>
            </form>
        </div>
        <div id="pfic-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.pfic.h2.elections_compare">Election comparison</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.pfic.th.feature">Feature</th>
                    <th>§ 1291 (default)</th>
                    <th>§ 1296 MTM</th>
                    <th>§ 1295 QEF</th>
                </tr></thead>
                <tbody>
                    <tr><td data-i18n="view.pfic.row.character">Character</td><td>Ordinary on excess + interest</td><td>Annual ordinary</td><td>Annual ordinary + LTCG flow-through</td></tr>
                    <tr><td data-i18n="view.pfic.row.timing">Timing</td><td>Deferred (with cost)</td><td>Annual mark-to-market</td><td>Annual flow-through</td></tr>
                    <tr><td data-i18n="view.pfic.row.cap_gain">LTCG preservation</td><td>NO</td><td>NO (always ordinary)</td><td>YES (flow-through)</td></tr>
                    <tr><td data-i18n="view.pfic.row.requires_info">Requires fund info</td><td>NO</td><td>Tradable on est. market</td><td>YES (PFIC Annual Info Stmt)</td></tr>
                    <tr><td data-i18n="view.pfic.row.purging">Purging election</td><td>—</td><td>Available</td><td>Available</td></tr>
                    <tr><td data-i18n="view.pfic.row.best_for">Best for</td><td>Worst — avoid</td><td>Liquid foreign stocks / ETFs</td><td>QEF-providing private funds</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.pfic.h2.de_minimis">§ 1298(b)(7) De Minimis</h2>
            <p class="muted small" data-i18n="view.pfic.de_minimis.body">
                If aggregate PFIC value &lt; $25,000 ($50k MFJ) at year-end AND no excess
                distributions: <strong>annual Form 8621 NOT required</strong>. However, MUST
                track + report sale gain. Below-threshold PFIC stocks still subject to default
                § 1291 regime on disposition.
            </p>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.pfic.h2.common_pfics">Common PFIC traps</h2>
            <ul class="muted small">
                <li data-i18n="view.pfic.trap.foreign_mutual">European UCITS, foreign mutual funds + ETFs (Vanguard / iShares foreign-domiciled)</li>
                <li data-i18n="view.pfic.trap.foreign_etf">Canadian / UK / Irish / Luxembourg-domiciled ETFs</li>
                <li data-i18n="view.pfic.trap.foreign_pension">Foreign pension plans (UK SIPP, Israeli Keren Pension)</li>
                <li data-i18n="view.pfic.trap.foreign_corp">Foreign holding cos / private investment cos</li>
                <li data-i18n="view.pfic.trap.startup">Foreign tech startup pre-revenue (no active business) → PFIC</li>
                <li data-i18n="view.pfic.trap.adr_safe">ADRs of foreign companies usually SAFE (active operating business)</li>
            </ul>
        </div>
    `;
    document.getElementById('pfic-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.holding_value = Number(fd.get('holding_value')) || 0;
        state.cost_basis = Number(fd.get('cost_basis')) || 0;
        state.holding_years = Number(fd.get('holding_years')) || 0;
        state.annual_distribution = Number(fd.get('annual_distribution')) || 0;
        state.election = fd.get('election');
        state.annual_growth_rate = Number(fd.get('annual_growth_rate')) || 0.08;
        state.afr_rate = Number(fd.get('afr_rate')) || 0.05;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.37;
        state.ltcg_rate = Number(fd.get('ltcg_rate')) || 0.20;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('pfic-output');
    if (!el) return;
    const gain = Math.max(0, state.holding_value - state.cost_basis);
    let immediateTax = 0, interestCharge = 0, futureTax = 0, character = '';
    if (state.election === 'default_1291') {
        // Pretend disposition. Allocate gain across holding period.
        const gainPerYear = gain / state.holding_years;
        const taxPerYear = gainPerYear * state.marginal_rate;
        // Interest charge: simple compound
        let cum = 0;
        for (let y = 1; y <= state.holding_years; y++) {
            cum += taxPerYear * Math.pow(1 + state.afr_rate, state.holding_years - y);
        }
        interestCharge = Math.max(0, cum - (taxPerYear * state.holding_years));
        immediateTax = taxPerYear * state.holding_years;
        character = t('view.pfic.char.ordinary');
    } else if (state.election === 'mtm_1296') {
        immediateTax = gain * state.marginal_rate;
        character = t('view.pfic.char.ordinary');
    } else {
        // QEF — assume same gain as long-term capital gain flow-through
        immediateTax = gain * state.ltcg_rate;
        character = t('view.pfic.char.cap_gain');
    }
    const totalCost = immediateTax + interestCharge;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.pfic.h2.result">PFIC tax outcome</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.pfic.card.gain">Embedded gain</div>
                    <div class="value">$${gain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.pfic.card.character">Character</div>
                    <div class="value">${esc(character)}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.pfic.card.tax_base">Tax on gain</div>
                    <div class="value">$${immediateTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${interestCharge > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.pfic.card.interest_charge">Interest charge (§ 1291)</div>
                        <div class="value">$${interestCharge.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                <div class="card neg">
                    <div class="label" data-i18n="view.pfic.card.total_cost">Total tax cost</div>
                    <div class="value">$${totalCost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
