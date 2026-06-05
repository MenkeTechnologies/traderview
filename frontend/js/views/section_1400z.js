// IRC § 1400Z-2 — Qualified Opportunity Zone Investment.
// Defer capital gain by investing in QOF within 180 days of realization.
// Tier 1: defer until earlier of QOF sale or Dec 31 2026.
// Tier 2: 5-yr hold → 10% basis step-up (15% if pre-2020 + 7-yr hold).
// Tier 3: 10-yr hold → BASIS STEP-UP TO FMV → exclude appreciation on QOF investment.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    deferred_gain: 0,
    invested_in_qof: 0,
    investment_year: 2023,
    years_held: 0,
    qof_fmv_at_sale: 0,
    eligible_gain_180_days: false,
    qof_compliance_test: true,
    qoz_property_test: true,
    direct_qoz_business: false,
    days_since_realization: 0,
};

export async function renderSection1400z(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1400z.h1.title">// § 1400Z-2 OPPORTUNITY ZONES</span></h1>
        <p class="muted small" data-i18n="view.s1400z.hint.intro">
            <strong>Tier 1 — Deferral:</strong> defer capital gain by investing in QOF within <strong>180 days</strong>
            of realization → defer until earlier of QOF sale or <strong>Dec 31 2026</strong>. <strong>Tier 2 — Step-up:</strong>
            5-yr hold → <strong>10% basis step-up</strong> (15% if pre-2020 + 7-yr hold). <strong>Tier 3 — Exclusion:</strong>
            10-yr hold → BASIS STEP-UP TO FMV → <strong>EXCLUDE APPRECIATION</strong> on QOF investment.
            QOF: 90%+ of assets in QOZ Property. Form 8996 (QOF) + Form 8997 (investor). Designation through 2028.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1400z.h2.inputs">Inputs</h2>
            <form id="s1400z-form" class="inline-form">
                <label><span data-i18n="view.s1400z.label.gain">Deferred capital gain ($)</span>
                    <input type="number" step="0.01" name="deferred_gain" value="${state.deferred_gain}"></label>
                <label><span data-i18n="view.s1400z.label.invested">Invested in QOF ($)</span>
                    <input type="number" step="0.01" name="invested_in_qof" value="${state.invested_in_qof}"></label>
                <label><span data-i18n="view.s1400z.label.year">Investment year</span>
                    <input type="number" step="1" name="investment_year" value="${state.investment_year}"></label>
                <label><span data-i18n="view.s1400z.label.years_held">Years held</span>
                    <input type="number" step="0.5" name="years_held" value="${state.years_held}"></label>
                <label><span data-i18n="view.s1400z.label.fmv">QOF FMV at sale (year 10+) ($)</span>
                    <input type="number" step="0.01" name="qof_fmv_at_sale" value="${state.qof_fmv_at_sale}"></label>
                <label><span data-i18n="view.s1400z.label.eligible">Eligible gain (capital, 180-day rule)?</span>
                    <input type="checkbox" name="eligible_gain_180_days" ${state.eligible_gain_180_days ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1400z.label.qof">QOF passes 90%+ asset test?</span>
                    <input type="checkbox" name="qof_compliance_test" ${state.qof_compliance_test ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1400z.label.qoz_prop">QOZ Property qualifies?</span>
                    <input type="checkbox" name="qoz_property_test" ${state.qoz_property_test ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1400z.label.direct">Direct QOZ business interest?</span>
                    <input type="checkbox" name="direct_qoz_business" ${state.direct_qoz_business ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1400z.label.days">Days since gain realization</span>
                    <input type="number" step="1" name="days_since_realization" value="${state.days_since_realization}"></label>
                <button class="primary" type="submit" data-i18n="view.s1400z.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1400z-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1400z.h2.eligible_gain">Eligible deferred gain</h2>
            <ul class="muted small">
                <li data-i18n="view.s1400z.elig.capital">Capital gain only (short OR long term)</li>
                <li data-i18n="view.s1400z.elig.s1231">§ 1231 net gain (after netting at year-end)</li>
                <li data-i18n="view.s1400z.elig.pass_through">Pass-through K-1 gain — partner / S-shareholder reinvests</li>
                <li data-i18n="view.s1400z.elig.180_day">180-day window: starts at realization (for individual) or partnership year-end</li>
                <li data-i18n="view.s1400z.elig.not_unrealized">Unrealized gain NOT eligible — must be realized first</li>
                <li data-i18n="view.s1400z.elig.not_ordinary">Ordinary income gain NOT eligible (recapture, inventory, etc.)</li>
                <li data-i18n="view.s1400z.elig.related_party">Related-party sale: § 1400Z-2(e)(2) prohibits</li>
                <li data-i18n="view.s1400z.elig.partial">Can defer ANY portion (not all-or-nothing)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1400z.h2.tier_benefits">Three-tier benefits</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s1400z.th.tier">Tier</th>
                    <th data-i18n="view.s1400z.th.hold">Hold required</th>
                    <th data-i18n="view.s1400z.th.benefit">Benefit</th>
                </tr></thead>
                <tbody>
                    <tr><td>1 — Deferral</td><td>Invest within 180 days</td><td>Defer original gain until QOF sale OR Dec 31 2026</td></tr>
                    <tr><td>2 — 10% step-up</td><td>5 years</td><td>10% of deferred gain forgiven (15% if pre-2020 + 7-yr hold)</td></tr>
                    <tr><td>3 — Exclusion</td><td>10 years</td><td>QOF FMV step-up to FMV → exclude ALL appreciation since investment</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1400z.h2.qof_compliance">QOF compliance requirements</h2>
            <ul class="muted small">
                <li data-i18n="view.s1400z.comp.90pct">Hold 90%+ assets in QOZ Property (semiannual test, average of 2 dates)</li>
                <li data-i18n="view.s1400z.comp.qoz_property">QOZ Property: QOZ Business Property OR equity in QOZ Business OR partnership int</li>
                <li data-i18n="view.s1400z.comp.qoz_business">QOZ Business: 70% of tangible property in QOZ + 50% of gross income from QOZ + &lt; 5% nonqualified financial property</li>
                <li data-i18n="view.s1400z.comp.original_use">Original use in QOZ — OR substantial improvement (double basis within 30 months)</li>
                <li data-i18n="view.s1400z.comp.sin_business">Sin businesses excluded: golf, country clubs, massage parlors, hot tubs, sun tan, racetracks, gambling, liquor stores</li>
                <li data-i18n="view.s1400z.comp.5_pct_finance">&lt; 5% Nonqualified financial property (excludes working capital ≤ 31 months under 'safe harbor')</li>
                <li data-i18n="view.s1400z.comp.86_month">31-month "working capital safe harbor" for QOZ Business — written plan + schedule</li>
                <li data-i18n="view.s1400z.comp.statutory_sunset">Designation expires Dec 31 2028; 10-yr benefit until ~Dec 31 2047</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1400z.h2.exit_strategies">Exit strategies + tax events</h2>
            <ul class="muted small">
                <li data-i18n="view.s1400z.exit.dec_2026">Dec 31 2026: forced inclusion event — pay deferred gain tax</li>
                <li data-i18n="view.s1400z.exit.qof_sale">Earlier QOF sale: inclusion at sale date</li>
                <li data-i18n="view.s1400z.exit.10yr_step_up">10-yr+ hold: full exclusion on QOF appreciation (not original deferred gain)</li>
                <li data-i18n="view.s1400z.exit.partial_disposition">Partial QOF interest disposition: pro-rata inclusion</li>
                <li data-i18n="view.s1400z.exit.death_basis">Death of investor: § 1014 basis step-up may eliminate deferred gain</li>
                <li data-i18n="view.s1400z.exit.gift">Gift to family: inclusion event (taxable to donor)</li>
                <li data-i18n="view.s1400z.exit.s1031_no">§ 1031 within QOF not allowed (TCJA removed personal property)</li>
                <li data-i18n="view.s1400z.exit.qof_fail">QOF fails 90% test: monthly penalty $10K min (Section 6700-style)</li>
            </ul>
        </div>
    `;
    document.getElementById('s1400z-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.deferred_gain = Number(fd.get('deferred_gain')) || 0;
        state.invested_in_qof = Number(fd.get('invested_in_qof')) || 0;
        state.investment_year = Number(fd.get('investment_year')) || 0;
        state.years_held = Number(fd.get('years_held')) || 0;
        state.qof_fmv_at_sale = Number(fd.get('qof_fmv_at_sale')) || 0;
        state.eligible_gain_180_days = !!fd.get('eligible_gain_180_days');
        state.qof_compliance_test = !!fd.get('qof_compliance_test');
        state.qoz_property_test = !!fd.get('qoz_property_test');
        state.direct_qoz_business = !!fd.get('direct_qoz_business');
        state.days_since_realization = Number(fd.get('days_since_realization')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1400z-output');
    if (!el) return;
    const within180 = state.days_since_realization <= 180;
    const deferralEligible = within180 && state.eligible_gain_180_days && state.qof_compliance_test;
    const investedGain = Math.min(state.invested_in_qof, state.deferred_gain);
    let basisStepUp = 0;
    if (state.years_held >= 7 && state.investment_year < 2020) basisStepUp = 0.15;
    else if (state.years_held >= 5 && state.investment_year < 2022) basisStepUp = 0.10;
    const forgivenGain = investedGain * basisStepUp;
    const eventualGain = Math.max(0, investedGain - forgivenGain);
    const eventualTax = eventualGain * 0.20;
    const appreciationOnQOF = Math.max(0, state.qof_fmv_at_sale - state.invested_in_qof);
    const tier3Excluded = state.years_held >= 10 ? appreciationOnQOF : 0;
    const tier3TaxSaved = tier3Excluded * 0.20;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1400z.h2.result">§ 1400Z-2 computation</h2>
            <div class="cards">
                <div class="card ${deferralEligible ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s1400z.card.eligible">Deferral eligible?</div>
                    <div class="value">${deferralEligible ? esc(t('view.s1400z.status.yes')) : esc(t('view.s1400z.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1400z.card.invested_gain">Invested gain amount</div>
                    <div class="value">$${investedGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1400z.card.forgiven">Forgiven (step-up ${(basisStepUp * 100).toFixed(0)}%)</div>
                    <div class="value">$${forgivenGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1400z.card.eventual">Eventual gain (Dec 31 2026)</div>
                    <div class="value">$${eventualGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1400z.card.eventual_tax">Tax on eventual gain (20%)</div>
                    <div class="value">$${eventualTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1400z.card.qof_apprec">QOF appreciation</div>
                    <div class="value">$${appreciationOnQOF.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1400z.card.tier3_excluded">Tier 3 excluded (10-yr)</div>
                    <div class="value">$${tier3Excluded.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1400z.card.tier3_savings">Tier 3 tax saved (20%)</div>
                    <div class="value">$${tier3TaxSaved.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.years_held >= 10 ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s1400z.tier3_note">
                    Tier 3 unlocked: 10-yr hold provides BASIS STEP-UP TO FMV on QOF interest sale.
                    Full appreciation excluded — including LTCG from QOF business operations.
                    Combined with Tier 1 deferral + Tier 2 step-up, this is the largest tax shelter available.
                </p>
            ` : ''}
        </div>
    `;
}
