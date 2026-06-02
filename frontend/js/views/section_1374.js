// IRC § 1374 — Built-In Gains Tax (BIG) on S-Corp Conversion.
// Former C-corp converting to S-corp: corporate-level tax (21%) on gain accrued PRE-conversion
// when realized within 5-year "recognition period". Limited to NUBIG (Net Unrealized Built-In Gain
// at conversion). Reduces NUBIG dollar-for-dollar.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const RECOGNITION_PERIOD_YEARS = 5;
const CORP_TAX_RATE = 0.21;

let state = {
    year_of_s_election: new Date().getFullYear() - 2,
    current_year: new Date().getFullYear(),
    nubig_at_conversion: 0,
    nubig_used: 0,
    current_year_built_in_gain: 0,
    current_year_taxable_income: 0,
    state_corp_rate: 0,
    individual_marginal_rate: 0.37,
    ltcg_rate: 0.20,
};

export async function renderSection1374(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1374.h1.title">// § 1374 S-CORP BIG TAX</span></h1>
        <p class="muted small" data-i18n="view.s1374.hint.intro">
            Former C-corp converting to S-corp: <strong>corporate-level 21% tax</strong> on gain
            accrued PRE-conversion when realized within <strong>5-year recognition period</strong>.
            Limited to <strong>NUBIG</strong> (Net Unrealized Built-In Gain at conversion).
            BIG income flows to shareholders AFTER corp tax (double tax during 5-yr window).
            Reduces NUBIG dollar-for-dollar. Goal: hold appreciated assets ≥ 5 yrs post-conversion.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1374.h2.inputs">Inputs</h2>
            <form id="s1374-form" class="inline-form">
                <label><span data-i18n="view.s1374.label.election_year">S-election year</span>
                    <input type="number" step="1" name="year_of_s_election" value="${state.year_of_s_election}"></label>
                <label><span data-i18n="view.s1374.label.current_year">Current year</span>
                    <input type="number" step="1" name="current_year" value="${state.current_year}"></label>
                <label><span data-i18n="view.s1374.label.nubig">NUBIG at conversion ($)</span>
                    <input type="number" step="10000" name="nubig_at_conversion" value="${state.nubig_at_conversion}"></label>
                <label><span data-i18n="view.s1374.label.nubig_used">NUBIG already used in prior years ($)</span>
                    <input type="number" step="10000" name="nubig_used" value="${state.nubig_used}"></label>
                <label><span data-i18n="view.s1374.label.built_in">Current year built-in gain realized ($)</span>
                    <input type="number" step="10000" name="current_year_built_in_gain" value="${state.current_year_built_in_gain}"></label>
                <label><span data-i18n="view.s1374.label.ti">Current year taxable income (if C) ($)</span>
                    <input type="number" step="10000" name="current_year_taxable_income" value="${state.current_year_taxable_income}"></label>
                <label><span data-i18n="view.s1374.label.state_rate">State corp rate</span>
                    <input type="number" step="0.01" name="state_corp_rate" value="${state.state_corp_rate}"></label>
                <label><span data-i18n="view.s1374.label.indiv_rate">Individual marginal %</span>
                    <input type="number" step="0.01" name="individual_marginal_rate" value="${state.individual_marginal_rate}"></label>
                <label><span data-i18n="view.s1374.label.ltcg">LTCG %</span>
                    <input type="number" step="0.01" name="ltcg_rate" value="${state.ltcg_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s1374.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1374-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1374.h2.planning">Planning techniques</h2>
            <ul class="muted small">
                <li data-i18n="view.s1374.plan.5_yr">Hold appreciated assets ≥ 5 yrs post-conversion → BIG period expires</li>
                <li data-i18n="view.s1374.plan.appraisal">Pre-conversion appraisal: low FMV reduces NUBIG</li>
                <li data-i18n="view.s1374.plan.allocation">Allocate gain to assets WITHOUT built-in gain</li>
                <li data-i18n="view.s1374.plan.installment">§ 453 installment sale spreads BIG over years (still within window though)</li>
                <li data-i18n="view.s1374.plan.like_kind">§ 1031 exchange defers BIG (real estate post-TCJA only)</li>
                <li data-i18n="view.s1374.plan.charitable">Charitable contribution of appreciated property eliminates BIG</li>
                <li data-i18n="view.s1374.plan.short_recog">CARES Act / Bush Tax Relief: 5-yr recognition period (down from 10-yr)</li>
                <li data-i18n="view.s1374.plan.ti_limit">BIG limited to corp-level taxable income (recognition deferred if no income)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1374.h2.related">Related S-corp gotchas</h2>
            <ul class="muted small">
                <li data-i18n="view.s1374.gotcha.passive">§ 1375 passive investment income (PII) > 25% for 3 consecutive yrs → S-status revoked + 35% tax</li>
                <li data-i18n="view.s1374.gotcha.lifo">§ 1363(d) LIFO recapture on C → S conversion (one-time pay over 4 yrs)</li>
                <li data-i18n="view.s1374.gotcha.aet">§ 532 Accumulated Earnings Tax 20% (C-corp only) on undistributed earnings</li>
                <li data-i18n="view.s1374.gotcha.phc">§ 541 Personal Holding Company 20% tax on undistributed personal holding income</li>
                <li data-i18n="view.s1374.gotcha.dpad">QBI § 199A may benefit from S vs C — analyze QBI limitations</li>
            </ul>
        </div>
    `;
    document.getElementById('s1374-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.year_of_s_election = Number(fd.get('year_of_s_election')) || new Date().getFullYear();
        state.current_year = Number(fd.get('current_year')) || new Date().getFullYear();
        state.nubig_at_conversion = Number(fd.get('nubig_at_conversion')) || 0;
        state.nubig_used = Number(fd.get('nubig_used')) || 0;
        state.current_year_built_in_gain = Number(fd.get('current_year_built_in_gain')) || 0;
        state.current_year_taxable_income = Number(fd.get('current_year_taxable_income')) || 0;
        state.state_corp_rate = Number(fd.get('state_corp_rate')) || 0;
        state.individual_marginal_rate = Number(fd.get('individual_marginal_rate')) || 0.37;
        state.ltcg_rate = Number(fd.get('ltcg_rate')) || 0.20;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1374-output');
    if (!el) return;
    const yearsSinceConversion = state.current_year - state.year_of_s_election;
    const inRecognitionPeriod = yearsSinceConversion < RECOGNITION_PERIOD_YEARS;
    const remainingNubig = Math.max(0, state.nubig_at_conversion - state.nubig_used);
    const bigSubject = inRecognitionPeriod
        ? Math.min(state.current_year_built_in_gain, remainingNubig, state.current_year_taxable_income)
        : 0;
    const corpTax = bigSubject * (CORP_TAX_RATE + state.state_corp_rate);
    const shareholderTaxAfterCorp = (state.current_year_built_in_gain - corpTax) * state.individual_marginal_rate;
    const totalTax = corpTax + shareholderTaxAfterCorp;
    const taxIfNoBig = state.current_year_built_in_gain * state.ltcg_rate;
    const extraCostFromBig = Math.max(0, totalTax - taxIfNoBig);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1374.h2.result">BIG calculation</h2>
            <div class="cards">
                <div class="card ${inRecognitionPeriod ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s1374.card.in_period">In 5-yr recognition period?</div>
                    <div class="value">${inRecognitionPeriod ? esc(t('view.s1374.status.yes')) : esc(t('view.s1374.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1374.card.years_remaining">Years until safe</div>
                    <div class="value">${Math.max(0, RECOGNITION_PERIOD_YEARS - yearsSinceConversion)}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1374.card.remaining_nubig">Remaining NUBIG</div>
                    <div class="value">$${remainingNubig.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${bigSubject > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s1374.card.subject">Subject to BIG</div>
                    <div class="value">$${bigSubject.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1374.card.corp_tax">Corp-level tax</div>
                    <div class="value">$${corpTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1374.card.shareholder">Shareholder pass-through tax</div>
                    <div class="value">$${shareholderTaxAfterCorp.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1374.card.total">Total tax (BIG path)</div>
                    <div class="value">$${totalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1374.card.extra">Extra cost from BIG</div>
                    <div class="value">$${extraCostFromBig.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
