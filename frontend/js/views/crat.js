// CRAT — Charitable Remainder Annuity Trust (IRC § 664(d)(1)).
// Pays grantor (or named) FIXED dollar annuity (not %) each year. Min 5%, max 50% of initial FMV.
// Remainder ≥ 10% test on inception. 5% probability-of-exhaustion test killed many post-2016.
// All four corners frozen at funding — no future revaluation. Great if asset is bond / income asset.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const MIN_PAYOUT_PCT = 0.05;
const MAX_PAYOUT_PCT = 0.50;
const MIN_REMAINDER_PCT = 0.10;
const PROBABILITY_EXHAUSTION_THRESHOLD = 0.05;

let state = {
    asset_fmv: 0,
    asset_basis: 0,
    annuity_pct_of_fmv: 0.05,
    term_years: 0,
    is_lifetime: true,
    grantor_age: 65,
    section_7520_rate: 0.0520,
    expected_growth: 0.05,
    marginal_rate: 0.37,
    ltcg_rate: 0.20,
};

const LIFE_EXPECTANCY = {
    50: 33.1, 55: 28.6, 60: 24.2, 65: 20.0, 70: 16.0,
    75: 12.4, 80: 9.1, 85: 6.4, 90: 4.4, 95: 2.9,
};
function lifeExp(age) {
    const a = Math.max(50, Math.min(95, Math.round(age / 5) * 5));
    return LIFE_EXPECTANCY[a] || 20;
}

export async function renderCrat(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.crat.h1.title">// CRAT — CHARITABLE REMAINDER ANNUITY TRUST</span></h1>
        <p class="muted small" data-i18n="view.crat.hint.intro">
            Pays grantor (or named beneficiary) <strong>FIXED dollar annuity</strong> (not %)
            each year. Min 5%, max 50% of <strong>initial</strong> FMV. Remainder ≥ 10% test.
            <strong>5% probability-of-exhaustion test</strong> killed many CRATs post-2016.
            All four corners frozen at funding — no future revaluation. Distinct from CRUT
            (annual revalue). Best for bond / income asset where appreciation upside not needed.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.crat.h2.inputs">Inputs</h2>
            <form id="crat-form" class="inline-form">
                <label><span data-i18n="view.crat.label.fmv">Asset FMV ($)</span>
                    <input type="number" step="0.01" name="asset_fmv" value="${state.asset_fmv}"></label>
                <label><span data-i18n="view.crat.label.basis">Asset basis ($)</span>
                    <input type="number" step="0.01" name="asset_basis" value="${state.asset_basis}"></label>
                <label><span data-i18n="view.crat.label.payout_pct">Annuity % of initial FMV</span>
                    <input type="number" step="0.005" name="annuity_pct_of_fmv" value="${state.annuity_pct_of_fmv}"></label>
                <label><span data-i18n="view.crat.label.is_lifetime">Lifetime?</span>
                    <input type="checkbox" name="is_lifetime" ${state.is_lifetime ? 'checked' : ''}></label>
                <label><span data-i18n="view.crat.label.term_years">Term (years, if not lifetime)</span>
                    <input type="number" step="1" min="0" max="20" name="term_years" value="${state.term_years}"></label>
                <label><span data-i18n="view.crat.label.grantor_age">Grantor age</span>
                    <input type="number" step="1" name="grantor_age" value="${state.grantor_age}"></label>
                <label><span data-i18n="view.crat.label.7520">§ 7520 rate (current month)</span>
                    <input type="number" step="0.0001" name="section_7520_rate" value="${state.section_7520_rate}"></label>
                <label><span data-i18n="view.crat.label.growth">Expected trust growth</span>
                    <input type="number" step="0.01" name="expected_growth" value="${state.expected_growth}"></label>
                <label><span data-i18n="view.crat.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <label><span data-i18n="view.crat.label.ltcg">LTCG %</span>
                    <input type="number" step="0.01" name="ltcg_rate" value="${state.ltcg_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.crat.btn.compute">Compute</button>
            </form>
        </div>
        <div id="crat-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.crat.h2.vs_crut">CRAT vs CRUT comparison</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.crat.th.feature">Feature</th>
                    <th>CRAT</th>
                    <th>CRUT</th>
                </tr></thead>
                <tbody>
                    <tr><td data-i18n="view.crat.row.payout_type">Payout type</td><td>Fixed $</td><td>Fixed % of revalued FMV</td></tr>
                    <tr><td data-i18n="view.crat.row.upside">Inflation upside</td><td>NO</td><td>YES</td></tr>
                    <tr><td data-i18n="view.crat.row.extra_contrib">Additional contributions</td><td>NO</td><td>YES</td></tr>
                    <tr><td data-i18n="view.crat.row.exhaustion">5% exhaustion test</td><td>APPLIES</td><td>DOES NOT</td></tr>
                    <tr><td data-i18n="view.crat.row.complexity">Simplicity</td><td>HIGH (fixed)</td><td>MEDIUM (annual revalue)</td></tr>
                    <tr><td data-i18n="view.crat.row.best_for">Best for</td><td>Bond / income asset</td><td>Growth asset</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.crat.h2.exhaustion">5% Probability of Exhaustion Test</h2>
            <p class="muted small" data-i18n="view.crat.exhaustion.body">
                Rev. Rul. 77-374: if probability that the trust will run out of money BEFORE the
                charity receives anything exceeds 5%, the trust does not qualify. Causes:
                payout rate / § 7520 rate / life expectancy combination. Modern low rates +
                long expected lifetimes have made CRATs hard to qualify.
                <strong>Notice 2008-90 + Rev. Proc. 2016-42 "qualified contingency"</strong>
                clause lets CRAT include early-termination clause if probability would exceed 5%.
            </p>
        </div>
    `;
    document.getElementById('crat-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.asset_fmv = Number(fd.get('asset_fmv')) || 0;
        state.asset_basis = Number(fd.get('asset_basis')) || 0;
        state.annuity_pct_of_fmv = Math.max(MIN_PAYOUT_PCT, Math.min(MAX_PAYOUT_PCT, Number(fd.get('annuity_pct_of_fmv')) || MIN_PAYOUT_PCT));
        state.is_lifetime = !!fd.get('is_lifetime');
        state.term_years = Math.max(0, Math.min(20, Number(fd.get('term_years')) || 0));
        state.grantor_age = Number(fd.get('grantor_age')) || 65;
        state.section_7520_rate = Number(fd.get('section_7520_rate')) || 0.05;
        state.expected_growth = Number(fd.get('expected_growth')) || 0.05;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.37;
        state.ltcg_rate = Number(fd.get('ltcg_rate')) || 0.20;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('crat-output');
    if (!el) return;
    const term = state.is_lifetime ? lifeExp(state.grantor_age) : state.term_years;
    const annuity = state.asset_fmv * state.annuity_pct_of_fmv;
    const r = state.section_7520_rate;
    // PV of annuity (ordinary annuity)
    const annuityFactor = (1 - Math.pow(1 + r, -term)) / r;
    const pvAnnuity = annuity * annuityFactor;
    const charitableRemainder = Math.max(0, state.asset_fmv - pvAnnuity);
    const passesTenPct = charitableRemainder / state.asset_fmv >= MIN_REMAINDER_PCT;
    // Simulate path
    let balance = state.asset_fmv;
    let exhaustionYear = null;
    let totalReceived = 0;
    for (let y = 1; y <= 50; y++) {
        balance = balance * (1 + state.expected_growth);
        balance -= annuity;
        totalReceived += annuity;
        if (balance <= 0 && exhaustionYear === null) {
            exhaustionYear = y;
            break;
        }
    }
    const probabilityExhaustion = exhaustionYear !== null && exhaustionYear < term ? 0.10 : 0.02;
    const passes5pct = probabilityExhaustion <= PROBABILITY_EXHAUSTION_THRESHOLD;
    const incomeTaxSavings = charitableRemainder * state.marginal_rate;
    const ltcgAvoided = (state.asset_fmv - state.asset_basis) * state.ltcg_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.crat.h2.result">CRAT outcome</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.crat.card.annuity">Annual annuity</div>
                    <div class="value">$${annuity.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.crat.card.total_received">Total annuity received</div>
                    <div class="value">$${totalReceived.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.crat.card.charitable">Charitable remainder (PV)</div>
                    <div class="value">$${charitableRemainder.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${passesTenPct ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.crat.card.passes_10">Passes 10% remainder test</div>
                    <div class="value">${passesTenPct ? esc(t('view.crat.status.yes')) : esc(t('view.crat.status.no'))}</div>
                </div>
                <div class="card ${passes5pct ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.crat.card.passes_5">Passes 5% exhaustion test</div>
                    <div class="value">${passes5pct ? esc(t('view.crat.status.yes')) : esc(t('view.crat.status.no'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.crat.card.income_savings">Income tax savings (Y1)</div>
                    <div class="value">$${incomeTaxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.crat.card.ltcg_avoided">LTCG avoided at contribution</div>
                    <div class="value">$${ltcgAvoided.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${exhaustionYear !== null ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.crat.card.exhaustion_year">Projected exhaustion</div>
                        <div class="value">Year ${exhaustionYear}</div>
                    </div>
                ` : ''}
            </div>
        </div>
    `;
}
