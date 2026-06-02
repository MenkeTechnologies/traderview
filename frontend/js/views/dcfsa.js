// Dependent Care FSA — IRC § 129.
// $5,000/yr pre-tax ($2,500 MFS). Use for child care under 13 or disabled
// spouse/dep care. INTERACTS with Child & Dependent Care Credit (§ 21) —
// can't use same dollars for both. DCFSA usually wins above ~$43k AGI.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const DCFSA_MAX_SINGLE_MFJ = 5_000;
const DCFSA_MAX_MFS = 2_500;
const CDCC_MAX_QE_ONE = 3_000;
const CDCC_MAX_QE_TWO_PLUS = 6_000;
const CDCC_AGI_RATES = [
    [15_000, 0.35], [17_000, 0.34], [19_000, 0.33], [21_000, 0.32],
    [23_000, 0.31], [25_000, 0.30], [27_000, 0.29], [29_000, 0.28],
    [31_000, 0.27], [33_000, 0.26], [35_000, 0.25], [37_000, 0.24],
    [39_000, 0.23], [41_000, 0.22], [43_000, 0.21], [Infinity, 0.20],
];

let state = {
    filing: 'mfj',
    agi: 150_000,
    num_qualifying: 2,
    annual_care_cost: 12_000,
    marginal_federal: 0.32,
    state_rate: 0.05,
    fica_rate: 0.0765,
};

export async function renderDcfsa(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.dcfsa.h1.title">// DEPENDENT CARE FSA</span></h1>
        <p class="muted small" data-i18n="view.dcfsa.hint.intro">
            $5,000/yr pre-tax for child care (under 13) or disabled spouse/dependent care.
            INTERACTS with the Child &amp; Dependent Care Credit (§ 21) — can't use the
            same dollars for both. DCFSA usually wins above ~$43k AGI.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.dcfsa.h2.inputs">Inputs</h2>
            <form id="dcfsa-form" class="inline-form">
                <label><span data-i18n="view.dcfsa.label.filing">Filing</span>
                    <select name="filing">
                        <option value="mfj" ${state.filing === 'mfj' ? 'selected' : ''}>MFJ</option>
                        <option value="single" ${state.filing === 'single' ? 'selected' : ''}>Single / HoH</option>
                        <option value="mfs" ${state.filing === 'mfs' ? 'selected' : ''}>MFS</option>
                    </select>
                </label>
                <label><span data-i18n="view.dcfsa.label.agi">AGI ($)</span>
                    <input type="number" step="1000" name="agi" value="${state.agi}"></label>
                <label><span data-i18n="view.dcfsa.label.num_qualifying">Qualifying dependents</span>
                    <input type="number" step="1" name="num_qualifying" value="${state.num_qualifying}" min="0"></label>
                <label><span data-i18n="view.dcfsa.label.annual_care_cost">Annual care cost ($)</span>
                    <input type="number" step="100" name="annual_care_cost" value="${state.annual_care_cost}"></label>
                <label><span data-i18n="view.dcfsa.label.marginal_federal">Marginal federal %</span>
                    <input type="number" step="0.5" name="marginal_federal" value="${(state.marginal_federal * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.dcfsa.label.state_rate">State rate %</span>
                    <input type="number" step="0.5" name="state_rate" value="${(state.state_rate * 100).toFixed(1)}"></label>
                <button class="primary" type="submit" data-i18n="view.dcfsa.btn.recompute">Recompute</button>
            </form>
        </div>
        <div id="dcfsa-output"></div>
    `;
    document.getElementById('dcfsa-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.filing = fd.get('filing');
        state.agi = Number(fd.get('agi')) || 0;
        state.num_qualifying = Number(fd.get('num_qualifying')) || 0;
        state.annual_care_cost = Number(fd.get('annual_care_cost')) || 0;
        state.marginal_federal = (Number(fd.get('marginal_federal')) || 32) / 100;
        state.state_rate = (Number(fd.get('state_rate')) || 0) / 100;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('dcfsa-output');
    if (!el) return;
    const dcfsaCap = state.filing === 'mfs' ? DCFSA_MAX_MFS : DCFSA_MAX_SINGLE_MFJ;
    const dcfsaContrib = Math.min(dcfsaCap, state.annual_care_cost);
    const totalRate = state.marginal_federal + state.state_rate + state.fica_rate;
    const dcfsaSavings = dcfsaContrib * totalRate;

    // CDCC: % × min(QE, cap) where QE excludes DCFSA-paid portion
    const cdccCap = state.num_qualifying >= 2 ? CDCC_MAX_QE_TWO_PLUS : CDCC_MAX_QE_ONE;
    const remainingCareAfterDcfsa = Math.max(0, state.annual_care_cost - dcfsaContrib);
    const cdccQE = Math.min(remainingCareAfterDcfsa, cdccCap - dcfsaContrib);
    const cdccRate = CDCC_AGI_RATES.find(([cap, _]) => state.agi <= cap)?.[1] || 0.20;
    const cdccCredit = Math.max(0, cdccQE) * cdccRate;

    // Compare: all DCFSA vs all CDCC (no DCFSA)
    const allDcfsaSavings = Math.min(dcfsaCap, state.annual_care_cost) * totalRate;
    const allCdccSavings = Math.min(state.annual_care_cost, cdccCap) * cdccRate;
    const combinedSavings = dcfsaSavings + cdccCredit;
    const recommendation = combinedSavings > allCdccSavings
        ? t('view.dcfsa.rec.use_both')
        : t('view.dcfsa.rec.use_cdcc');

    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.dcfsa.h2.combined">Optimal: DCFSA + remaining CDCC</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.dcfsa.card.dcfsa_contrib">DCFSA contribution</div>
                    <div class="value">$${dcfsaContrib.toLocaleString()}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.dcfsa.card.dcfsa_savings">DCFSA savings</div>
                    <div class="value">$${dcfsaSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.dcfsa.card.cdcc_credit">CDCC credit (remainder)</div>
                    <div class="value">$${cdccCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.dcfsa.card.combined">Combined savings</div>
                    <div class="value">$${combinedSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.dcfsa.card.cdcc_rate">CDCC rate @ your AGI</div>
                    <div class="value">${(cdccRate * 100).toFixed(0)}%</div>
                </div>
            </div>
        </div>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.dcfsa.h2.dcfsa_only">DCFSA-only scenario</h2>
                <p>$${allDcfsaSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}
                <span class="muted">${esc(t('view.dcfsa.savings'))}</span></p>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.dcfsa.h2.cdcc_only">CDCC-only scenario</h2>
                <p>$${allCdccSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}
                <span class="muted">${esc(t('view.dcfsa.savings'))}</span></p>
            </div>
        </div>
        <div class="chart-panel pos">
            <h2 data-i18n="view.dcfsa.h2.recommendation">Recommendation</h2>
            <p><strong>${esc(recommendation)}</strong></p>
            <p class="muted small" data-i18n="view.dcfsa.rec.note">
                DCFSA contributions reduce W-2 taxable wages AND escape FICA — that's the
                7.65% FICA savings advantage. Above $43k AGI the CDCC drops to 20% which
                makes DCFSA almost always the winner at any meaningful income.
            </p>
        </div>
    `;
}
