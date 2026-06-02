// CRUT — Charitable Remainder Unitrust (IRC § 664(d)(2)).
// Donate appreciated asset → trust pays you fixed % of FMV each year for life or term.
// Remainder to charity. Income tax deduction = PV of remainder. No immediate cap gains.
// Min payout 5%, max 50%. Remainder must be ≥ 10% of initial FMV (10% test).
// Variants: standard (annual revalue), NICRUT (net income), NIMCRUT (makeup), FLIP.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const MIN_PAYOUT_PCT = 0.05;
const MAX_PAYOUT_PCT = 0.50;
const MIN_REMAINDER_PCT = 0.10;

let state = {
    asset_fmv: 0,
    asset_basis: 0,
    payout_pct: 0.05,
    term_years: 0,
    is_lifetime: true,
    grantor_age: 60,
    spouse_age: 60,
    is_joint_life: false,
    section_7520_rate: 0.0520,
    growth_rate: 0.07,
    crut_type: 'standard',
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

export async function renderCrut(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.crut.h1.title">// CRUT — CHARITABLE REMAINDER UNITRUST</span></h1>
        <p class="muted small" data-i18n="view.crut.hint.intro">
            Donate appreciated asset → trust pays you <strong>fixed % of FMV revalued annually</strong>
            for life or up to 20-yr term. Remainder to charity. Income tax deduction = PV of
            charitable remainder. <strong>Min payout 5%, max 50%, remainder ≥ 10% test</strong>.
            Variants: standard (annual revalue), NICRUT (net income), NIMCRUT (makeup),
            FLIP-CRUT (triggers from NICRUT → standard). No immediate cap gains; tax stretched
            across distributions per 4-tier ordering rule.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.crut.h2.inputs">Inputs</h2>
            <form id="crut-form" class="inline-form">
                <label><span data-i18n="view.crut.label.fmv">Asset FMV ($)</span>
                    <input type="number" step="10000" name="asset_fmv" value="${state.asset_fmv}"></label>
                <label><span data-i18n="view.crut.label.basis">Asset basis ($)</span>
                    <input type="number" step="1000" name="asset_basis" value="${state.asset_basis}"></label>
                <label><span data-i18n="view.crut.label.payout_pct">Annual payout %</span>
                    <input type="number" step="0.005" name="payout_pct" value="${state.payout_pct}"></label>
                <label><span data-i18n="view.crut.label.is_lifetime">Lifetime CRUT?</span>
                    <input type="checkbox" name="is_lifetime" ${state.is_lifetime ? 'checked' : ''}></label>
                <label><span data-i18n="view.crut.label.term_years">Term (years, if not lifetime)</span>
                    <input type="number" step="1" min="0" max="20" name="term_years" value="${state.term_years}"></label>
                <label><span data-i18n="view.crut.label.grantor_age">Grantor age</span>
                    <input type="number" step="1" name="grantor_age" value="${state.grantor_age}"></label>
                <label><span data-i18n="view.crut.label.is_joint">Joint life?</span>
                    <input type="checkbox" name="is_joint_life" ${state.is_joint_life ? 'checked' : ''}></label>
                <label><span data-i18n="view.crut.label.spouse_age">Spouse age</span>
                    <input type="number" step="1" name="spouse_age" value="${state.spouse_age}"></label>
                <label><span data-i18n="view.crut.label.7520">§ 7520 rate (current month)</span>
                    <input type="number" step="0.0001" name="section_7520_rate" value="${state.section_7520_rate}"></label>
                <label><span data-i18n="view.crut.label.growth">Expected trust growth</span>
                    <input type="number" step="0.01" name="growth_rate" value="${state.growth_rate}"></label>
                <label><span data-i18n="view.crut.label.type">Variant</span>
                    <select name="crut_type">
                        <option value="standard" ${state.crut_type === 'standard' ? 'selected' : ''}>Standard</option>
                        <option value="nicrut" ${state.crut_type === 'nicrut' ? 'selected' : ''}>NICRUT (net income)</option>
                        <option value="nimcrut" ${state.crut_type === 'nimcrut' ? 'selected' : ''}>NIMCRUT (with makeup)</option>
                        <option value="flip" ${state.crut_type === 'flip' ? 'selected' : ''}>FLIP CRUT</option>
                    </select>
                </label>
                <label><span data-i18n="view.crut.label.marginal">Marginal rate %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <label><span data-i18n="view.crut.label.ltcg">LTCG rate %</span>
                    <input type="number" step="0.01" name="ltcg_rate" value="${state.ltcg_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.crut.btn.compute">Compute</button>
            </form>
        </div>
        <div id="crut-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.crut.h2.tier_ordering">4-tier income ordering for distributions</h2>
            <ol class="muted small">
                <li data-i18n="view.crut.tier.ordinary">Tier 1: ordinary income (interest, dividends, ord biz)</li>
                <li data-i18n="view.crut.tier.cap_gain">Tier 2: capital gain (ST first, then LT — including pre-funding embedded gain)</li>
                <li data-i18n="view.crut.tier.other">Tier 3: other income (tax-exempt, etc.)</li>
                <li data-i18n="view.crut.tier.corpus">Tier 4: tax-free return of corpus</li>
            </ol>
        </div>
    `;
    document.getElementById('crut-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.asset_fmv = Number(fd.get('asset_fmv')) || 0;
        state.asset_basis = Number(fd.get('asset_basis')) || 0;
        state.payout_pct = Math.max(MIN_PAYOUT_PCT, Math.min(MAX_PAYOUT_PCT, Number(fd.get('payout_pct')) || MIN_PAYOUT_PCT));
        state.is_lifetime = !!fd.get('is_lifetime');
        state.term_years = Math.max(0, Math.min(20, Number(fd.get('term_years')) || 0));
        state.grantor_age = Number(fd.get('grantor_age')) || 60;
        state.is_joint_life = !!fd.get('is_joint_life');
        state.spouse_age = Number(fd.get('spouse_age')) || 60;
        state.section_7520_rate = Number(fd.get('section_7520_rate')) || 0.05;
        state.growth_rate = Number(fd.get('growth_rate')) || 0.07;
        state.crut_type = fd.get('crut_type');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.37;
        state.ltcg_rate = Number(fd.get('ltcg_rate')) || 0.20;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('crut-output');
    if (!el) return;
    const effTerm = state.is_lifetime
        ? (state.is_joint_life ? Math.max(lifeExp(state.grantor_age), lifeExp(state.spouse_age)) + 2 : lifeExp(state.grantor_age))
        : state.term_years;
    const remainderFactor = Math.pow((1 - state.payout_pct) * (1 + state.growth_rate) / (1 + state.section_7520_rate), effTerm);
    const charitableRemainder = state.asset_fmv * remainderFactor;
    const passesTenPct = (charitableRemainder / state.asset_fmv) >= MIN_REMAINDER_PCT;
    const deduction = charitableRemainder;
    const ltcgAvoided = (state.asset_fmv - state.asset_basis) * state.ltcg_rate;
    const incomeTaxSavings = deduction * state.marginal_rate;
    // Year-by-year income
    let principal = state.asset_fmv;
    const incomeYears = [];
    let totalIncome = 0;
    for (let y = 1; y <= Math.min(20, effTerm); y++) {
        principal = principal * (1 + state.growth_rate);
        const payout = principal * state.payout_pct;
        principal -= payout;
        incomeYears.push({ y, payout, principal });
        totalIncome += payout;
    }
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.crut.h2.result">CRUT outcome</h2>
            <div class="cards">
                <div class="card ${passesTenPct ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.crut.card.passes">Passes 10% remainder test</div>
                    <div class="value">${passesTenPct ? esc(t('view.crut.status.yes')) : esc(t('view.crut.status.no'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.crut.card.deduction">Charitable deduction (PV)</div>
                    <div class="value">$${deduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.crut.card.income_savings">Income tax savings (Y1)</div>
                    <div class="value">$${incomeTaxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.crut.card.ltcg_avoided">LTCG tax avoided at contribution</div>
                    <div class="value">$${ltcgAvoided.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.crut.card.total_income">Total income over term</div>
                    <div class="value">$${totalIncome.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.crut.card.eff_term">Effective term (yrs)</div>
                    <div class="value">${effTerm.toFixed(1)}</div>
                </div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.crut.h2.year_table">First 20 yrs payout</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.crut.th.year">Year</th>
                    <th data-i18n="view.crut.th.payout">Payout</th>
                    <th data-i18n="view.crut.th.balance">End balance</th>
                </tr></thead>
                <tbody>${incomeYears.map(r => `
                    <tr>
                        <td>${r.y}</td>
                        <td class="pos">$${r.payout.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td>$${r.principal.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        </div>
    `;
}
