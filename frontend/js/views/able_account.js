// ABLE Account § 529A — Achieving a Better Life Experience.
// Tax-advantaged savings for people with disabilities. Disability onset
// must be before age 26 (raised to 46 starting 2026 per SECURE 2.0).
// $18,000/yr contribution cap (2024). ABLE-to-Work: employed beneficiaries
// can contribute their earnings up to $14,580 extra (Federal poverty line).
// $100,000 ABLE balance excluded from SSI asset test.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const ABLE_LIMITS = {
    2024: { annual: 18_000, able_to_work: 14_580 },
    2025: { annual: 19_000, able_to_work: 15_060 },
    2026: { annual: 19_000, able_to_work: 15_500 },
};
const SSI_ASSET_EXCLUSION = 100_000;
const DISABILITY_AGE_PRE_SECURE = 26;
const DISABILITY_AGE_POST_SECURE = 46;

let state = {
    year: new Date().getFullYear(),
    beneficiary_age: 30,
    disability_onset_age: 18,
    annual_contribution: 18_000,
    employed: false,
    earned_income: 30_000,
    current_balance: 50_000,
    receives_ssi: true,
};

export async function renderAbleAccount(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.able.h1.title">// ABLE ACCOUNT § 529A</span></h1>
        <p class="muted small" data-i18n="view.able.hint.intro">
            Tax-advantaged savings for individuals with disabilities. <strong>Disability
            onset before age 26</strong> (raised to 46 starting 2026 per SECURE 2.0).
            $18k/yr cap (2024). ABLE-to-Work adds up to $14,580 extra from employed
            beneficiary's own earnings. $100k balance EXCLUDED from SSI asset test.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.able.h2.inputs">Inputs</h2>
            <form id="able-form" class="inline-form">
                <label><span data-i18n="view.able.label.year">Year</span>
                    <input type="number" step="1" name="year" value="${state.year}"></label>
                <label><span data-i18n="view.able.label.beneficiary_age">Beneficiary age</span>
                    <input type="number" step="1" name="beneficiary_age" value="${state.beneficiary_age}"></label>
                <label><span data-i18n="view.able.label.disability_onset_age">Disability onset age</span>
                    <input type="number" step="1" name="disability_onset_age" value="${state.disability_onset_age}"></label>
                <label><span data-i18n="view.able.label.annual_contribution">Annual contribution ($)</span>
                    <input type="number" step="500" name="annual_contribution" value="${state.annual_contribution}"></label>
                <label><span data-i18n="view.able.label.employed">Beneficiary employed?</span>
                    <input type="checkbox" name="employed" ${state.employed ? 'checked' : ''}></label>
                <label><span data-i18n="view.able.label.earned_income">Beneficiary earned income ($)</span>
                    <input type="number" step="1000" name="earned_income" value="${state.earned_income}"></label>
                <label><span data-i18n="view.able.label.current_balance">Current ABLE balance ($)</span>
                    <input type="number" step="1000" name="current_balance" value="${state.current_balance}"></label>
                <label><span data-i18n="view.able.label.receives_ssi">Receives SSI?</span>
                    <input type="checkbox" name="receives_ssi" ${state.receives_ssi ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.able.btn.compute">Compute</button>
            </form>
        </div>
        <div id="able-output"></div>
    `;
    document.getElementById('able-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.year = Number(fd.get('year'));
        state.beneficiary_age = Number(fd.get('beneficiary_age'));
        state.disability_onset_age = Number(fd.get('disability_onset_age'));
        state.annual_contribution = Number(fd.get('annual_contribution')) || 0;
        state.employed = !!fd.get('employed');
        state.earned_income = Number(fd.get('earned_income')) || 0;
        state.current_balance = Number(fd.get('current_balance')) || 0;
        state.receives_ssi = !!fd.get('receives_ssi');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('able-output');
    if (!el) return;
    const limits = ABLE_LIMITS[state.year] || ABLE_LIMITS[2024];
    const onsetThreshold = state.year >= 2026 ? DISABILITY_AGE_POST_SECURE : DISABILITY_AGE_PRE_SECURE;
    const eligible = state.disability_onset_age < onsetThreshold;

    const regularCap = limits.annual;
    const atwCap = state.employed
        ? Math.min(state.earned_income, limits.able_to_work)
        : 0;
    const totalCap = regularCap + atwCap;
    const overContrib = Math.max(0, state.annual_contribution - totalCap);

    const ssiAssetExposed = state.receives_ssi
        ? Math.max(0, state.current_balance - SSI_ASSET_EXCLUSION) : 0;
    const ssiAssetExcluded = state.receives_ssi
        ? Math.min(state.current_balance, SSI_ASSET_EXCLUSION) : state.current_balance;
    const cls = eligible ? 'pos' : 'neg';
    el.innerHTML = `
        <div class="chart-panel ${cls}">
            <h2 data-i18n="view.able.h2.eligibility">Eligibility</h2>
            <div class="cards">
                <div class="card ${eligible ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.able.card.eligible">Eligible?</div>
                    <div class="value">${eligible ? esc(t('view.able.status.yes')) : esc(t('view.able.status.no_age'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.able.card.onset_threshold">Onset age threshold</div>
                    <div class="value">${onsetThreshold}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.able.card.onset_actual">Your onset age</div>
                    <div class="value">${state.disability_onset_age}</div>
                </div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.able.h2.contribution">${state.year} contribution caps</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.able.card.regular_cap">Regular cap</div>
                    <div class="value">$${regularCap.toLocaleString()}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.able.card.atw_cap">ABLE-to-Work additional</div>
                    <div class="value">$${atwCap.toLocaleString()}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.able.card.total_cap">Total cap</div>
                    <div class="value">$${totalCap.toLocaleString()}</div>
                </div>
                ${overContrib > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.able.card.over_contrib">Over-contribution (6% excise)</div>
                        <div class="value">$${overContrib.toLocaleString()}</div>
                    </div>
                ` : ''}
            </div>
        </div>
        ${state.receives_ssi ? `
            <div class="chart-panel">
                <h2 data-i18n="view.able.h2.ssi_impact">SSI asset test</h2>
                <div class="cards">
                    <div class="card pos">
                        <div class="label" data-i18n="view.able.card.ssi_excluded">Excluded from asset test</div>
                        <div class="value">$${ssiAssetExcluded.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                    <div class="card ${ssiAssetExposed > 0 ? 'neg' : 'pos'}">
                        <div class="label" data-i18n="view.able.card.ssi_exposed">Counted against $2k limit</div>
                        <div class="value">$${ssiAssetExposed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                </div>
            </div>
        ` : ''}
        <div class="chart-panel">
            <h2 data-i18n="view.able.h2.benefits">ABLE benefits</h2>
            <ul class="muted small">
                <li data-i18n="view.able.benefit.tax_free">Tax-free growth + tax-free withdrawals for qualified disability expenses (housing, transportation, education, employment, health, financial management)</li>
                <li data-i18n="view.able.benefit.ssi">First $100k excluded from SSI asset test ($2k normally cuts off SSI)</li>
                <li data-i18n="view.able.benefit.medicaid">Full balance excluded from Medicaid asset test (in most states)</li>
                <li data-i18n="view.able.benefit.saver_credit">Contributions qualify for Saver's Credit if income low enough</li>
                <li data-i18n="view.able.benefit.529_rollover">SECURE 2.0: 529 plans can be rolled over to ABLE accounts (family member)</li>
                <li data-i18n="view.able.benefit.atw_2026">ABLE Age Adjustment Act: onset-age threshold raises 26 → 46 starting 2026</li>
            </ul>
            <p class="muted small" data-i18n="view.able.providers">
                Providers vary by state. Top: Ohio STABLE, Tennessee ABLE TN, Virginia ABLEnow.
                Most allow out-of-state residents. Compare fees + investment options.
            </p>
        </div>
    `;
}
