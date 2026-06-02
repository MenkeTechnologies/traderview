// IRC § 877A — Expatriation / Exit Tax (HEART Act 2008).
// "Covered expatriate" mark-to-market: deemed sale of all assets day before expatriation.
// Triggers: $2M net worth, 5-yr avg tax > $201K (2024), or failure to certify 5 years compliance.
// Exemption: $866K (2024 inflation-indexed) of net gain.
// Deferred comp + IRA + grantor trust: special rules (not MTM).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const NET_WORTH_THRESHOLD_2024 = 2_000_000;
const AVG_TAX_THRESHOLD_2024 = 201_000;
const EXIT_TAX_EXEMPTION_2024 = 866_000;

let state = {
    net_worth: 0,
    avg_5yr_income_tax: 0,
    five_year_compliance: false,
    expatriation_date: '',
    total_unrealized_gain: 0,
    deferred_comp: 0,
    ira_balance: 0,
    grantor_trust_value: 0,
    interest_in_specified_tax_deferred_account: 0,
    long_term_resident: false,
    elect_deferral: false,
};

export async function renderSection877A(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s877A.h1.title">// § 877A EXIT TAX</span></h1>
        <p class="muted small" data-i18n="view.s877A.hint.intro">
            "<strong>Covered expatriate</strong>" mark-to-market: <strong>deemed sale of all assets</strong>
            day before expatriation. <strong>Triggers (any one):</strong> $2M net worth, 5-yr avg tax
            > $201K (2024), failure to certify 5-yr compliance. <strong>Exemption:</strong> $866K (2024)
            of net gain. <strong>Special asset rules:</strong> deferred comp + IRA + grantor trust not MTM
            (withholding + future tax). <strong>Long-term resident</strong> (LPR 8 of 15 yrs) also subject.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s877A.h2.inputs">Inputs</h2>
            <form id="s877A-form" class="inline-form">
                <label><span data-i18n="view.s877A.label.net_worth">Net worth ($)</span>
                    <input type="number" step="100000" name="net_worth" value="${state.net_worth}"></label>
                <label><span data-i18n="view.s877A.label.avg_tax">5-yr avg income tax ($)</span>
                    <input type="number" step="1000" name="avg_5yr_income_tax" value="${state.avg_5yr_income_tax}"></label>
                <label><span data-i18n="view.s877A.label.compliance">5-yr compliance certified?</span>
                    <input type="checkbox" name="five_year_compliance" ${state.five_year_compliance ? 'checked' : ''}></label>
                <label><span data-i18n="view.s877A.label.date">Expatriation date</span>
                    <input type="date" name="expatriation_date" value="${state.expatriation_date}"></label>
                <label><span data-i18n="view.s877A.label.gain">Total unrealized gain (mark-to-market) ($)</span>
                    <input type="number" step="10000" name="total_unrealized_gain" value="${state.total_unrealized_gain}"></label>
                <label><span data-i18n="view.s877A.label.deferred">Deferred comp ($)</span>
                    <input type="number" step="1000" name="deferred_comp" value="${state.deferred_comp}"></label>
                <label><span data-i18n="view.s877A.label.ira">IRA balance ($)</span>
                    <input type="number" step="1000" name="ira_balance" value="${state.ira_balance}"></label>
                <label><span data-i18n="view.s877A.label.trust">Grantor trust value ($)</span>
                    <input type="number" step="1000" name="grantor_trust_value" value="${state.grantor_trust_value}"></label>
                <label><span data-i18n="view.s877A.label.specified">Specified tax-deferred account ($)</span>
                    <input type="number" step="1000" name="interest_in_specified_tax_deferred_account" value="${state.interest_in_specified_tax_deferred_account}"></label>
                <label><span data-i18n="view.s877A.label.lpr">Long-term resident (LPR 8 of 15)?</span>
                    <input type="checkbox" name="long_term_resident" ${state.long_term_resident ? 'checked' : ''}></label>
                <label><span data-i18n="view.s877A.label.defer">Elect § 877A(b) deferral?</span>
                    <input type="checkbox" name="elect_deferral" ${state.elect_deferral ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s877A.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s877A-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s877A.h2.covered_test">"Covered expatriate" test</h2>
            <ol class="muted small">
                <li data-i18n="view.s877A.cov.networth">Net worth ≥ $2M on date of expatriation</li>
                <li data-i18n="view.s877A.cov.tax">5-yr avg US income tax ≥ $201K (2024, inflation-indexed)</li>
                <li data-i18n="view.s877A.cov.compliance">Fail to certify 5-year tax compliance (Form 8854)</li>
                <li data-i18n="view.s877A.cov.any">ANY ONE triggers covered expatriate status</li>
                <li data-i18n="view.s877A.cov.dual">Dual-status exception: born outside US + dual nationality at birth + foreign country tax resident 10 of 15 yrs</li>
                <li data-i18n="view.s877A.cov.minor">Minor exception: relinquish before 18.5 + US resident &lt; 10 yrs</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s877A.h2.special_assets">Special asset rules (NOT mark-to-market)</h2>
            <ul class="muted small">
                <li data-i18n="view.s877A.spec.deferred">Deferred compensation: 30% withholding on future distributions OR W-8CE election to MTM</li>
                <li data-i18n="view.s877A.spec.ira">IRA / qualified plans: deemed distribution day before expatriation (current taxation)</li>
                <li data-i18n="view.s877A.spec.trust">Grantor trust portion: deemed distribution + applicable rate</li>
                <li data-i18n="view.s877A.spec.non_grantor">Non-grantor trust: 30% withholding on future distributions</li>
                <li data-i18n="view.s877A.spec.specified_tax_deferred">Specified tax-deferred (529, MSA, HSA): mark-to-market with inclusion in income</li>
                <li data-i18n="view.s877A.spec.eligible_deferred">Eligible deferred comp: foreign payor + irrevocable W-8CE</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s877A.h2.gift_tax">§ 2801 Gift / inheritance tax on US recipients</h2>
            <ul class="muted small">
                <li data-i18n="view.s877A.gift.recipient_tax">US recipient pays 40% gift / inheritance tax on transfers from covered expatriate</li>
                <li data-i18n="view.s877A.gift.lifetime">Lifetime gift exclusion does NOT apply</li>
                <li data-i18n="view.s877A.gift.annual">Annual exclusion ($18,000 2024) DOES apply</li>
                <li data-i18n="view.s877A.gift.form">Form 708 (proposed) — final regs pending</li>
                <li data-i18n="view.s877A.gift.permanent">Applies even decades after expatriation — permanent tax</li>
            </ul>
        </div>
    `;
    document.getElementById('s877A-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.net_worth = Number(fd.get('net_worth')) || 0;
        state.avg_5yr_income_tax = Number(fd.get('avg_5yr_income_tax')) || 0;
        state.five_year_compliance = !!fd.get('five_year_compliance');
        state.expatriation_date = fd.get('expatriation_date');
        state.total_unrealized_gain = Number(fd.get('total_unrealized_gain')) || 0;
        state.deferred_comp = Number(fd.get('deferred_comp')) || 0;
        state.ira_balance = Number(fd.get('ira_balance')) || 0;
        state.grantor_trust_value = Number(fd.get('grantor_trust_value')) || 0;
        state.interest_in_specified_tax_deferred_account = Number(fd.get('interest_in_specified_tax_deferred_account')) || 0;
        state.long_term_resident = !!fd.get('long_term_resident');
        state.elect_deferral = !!fd.get('elect_deferral');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s877A-output');
    if (!el) return;
    const networthTrigger = state.net_worth >= NET_WORTH_THRESHOLD_2024;
    const taxTrigger = state.avg_5yr_income_tax >= AVG_TAX_THRESHOLD_2024;
    const complianceTrigger = !state.five_year_compliance;
    const isCovered = networthTrigger || taxTrigger || complianceTrigger;
    const exemptGain = Math.min(state.total_unrealized_gain, EXIT_TAX_EXEMPTION_2024);
    const taxableGain = Math.max(0, state.total_unrealized_gain - EXIT_TAX_EXEMPTION_2024);
    const mtmTax = isCovered ? taxableGain * 0.238 : 0; // LTCG 20% + NIIT 3.8% = 23.8%
    const iraTax = isCovered ? state.ira_balance * 0.37 : 0; // deemed distribution at top rate
    const specifiedTaxDeferredTax = isCovered ? state.interest_in_specified_tax_deferred_account * 0.37 : 0;
    const trustTax = isCovered ? state.grantor_trust_value * 0.37 : 0;
    const totalExitTax = mtmTax + iraTax + specifiedTaxDeferredTax + trustTax;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s877A.h2.result">Exit tax computation</h2>
            <div class="cards">
                <div class="card ${isCovered ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s877A.card.covered">Covered expatriate?</div>
                    <div class="value">${isCovered ? esc(t('view.s877A.status.yes')) : esc(t('view.s877A.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s877A.card.networth_trigger">Net worth ≥ $2M</div>
                    <div class="value">${networthTrigger ? esc(t('view.s877A.status.yes')) : esc(t('view.s877A.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s877A.card.tax_trigger">5-yr avg tax ≥ $201K</div>
                    <div class="value">${taxTrigger ? esc(t('view.s877A.status.yes')) : esc(t('view.s877A.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s877A.card.compliance">5-yr compliance?</div>
                    <div class="value">${state.five_year_compliance ? esc(t('view.s877A.status.yes')) : esc(t('view.s877A.status.no'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s877A.card.exemption">Exemption ($866K 2024)</div>
                    <div class="value">$${exemptGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s877A.card.mtm">MTM tax (23.8%)</div>
                    <div class="value">$${mtmTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s877A.card.ira">IRA deemed distribution tax (37%)</div>
                    <div class="value">$${iraTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s877A.card.total">TOTAL EXIT TAX</div>
                    <div class="value">$${totalExitTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${isCovered ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s877A.cov_note">
                    Covered expatriate: Form 8854 required for year of expatriation + 10 subsequent years.
                    Deferral election § 877A(b) requires bond / adequate security + waiver of treaty benefits.
                    § 2801 inheritance tax applies to US recipients of future gifts / bequests at 40%.
                </p>
            ` : ''}
        </div>
    `;
}
