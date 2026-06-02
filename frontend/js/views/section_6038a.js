// IRC § 6038A Form 5472 — 25% Foreign-Owned U.S. Corp / Disregarded Entity.
// Required when (1) US corp with 25%+ foreign ownership, OR (2) foreign-owned single-member
// LLC since 2017. Reports related-party transactions in 11 categories.
// Penalty: $25,000 per failure (was $10k pre-2018), plus $25,000/month after 90 days.
// Common trap: foreign-owned US LLC holding US real estate.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const PENALTY_PER_FAILURE = 25_000;
const ADDITIONAL_PER_MONTH = 25_000;
const MAX_PENALTY = 250_000;

let state = {
    is_us_corp: true,
    is_disregarded_llc: false,
    foreign_ownership_pct: 0,
    has_related_party_transactions: false,
    total_related_party_value: 0,
    years_unfiled: 0,
};

export async function renderSection6038a(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6038a.h1.title">// § 6038A FORM 5472 FOREIGN-OWNED US</span></h1>
        <p class="muted small" data-i18n="view.s6038a.hint.intro">
            Required when <strong>(1) US corp with 25%+ foreign ownership</strong>, OR
            <strong>(2) foreign-owned single-member LLC</strong> (since 2017 disregarded entity
            rule). Reports related-party transactions in 11 categories. <strong>Penalty: $25,000
            per failure</strong> (was $10k pre-2018) + $25k/month after 90 days. Common trap:
            foreign-owned US LLC holding US real estate. <strong>Pro-forma 1120 + Form 5472
            required even for disregarded entity.</strong>
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6038a.h2.inputs">Inputs</h2>
            <form id="s6038a-form" class="inline-form">
                <label><span data-i18n="view.s6038a.label.us_corp">US C-corp?</span>
                    <input type="checkbox" name="is_us_corp" ${state.is_us_corp ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6038a.label.llc">Foreign-owned disregarded LLC?</span>
                    <input type="checkbox" name="is_disregarded_llc" ${state.is_disregarded_llc ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6038a.label.foreign_pct">Foreign ownership %</span>
                    <input type="number" step="0.01" name="foreign_ownership_pct" value="${state.foreign_ownership_pct}"></label>
                <label><span data-i18n="view.s6038a.label.has_transactions">Related-party transactions?</span>
                    <input type="checkbox" name="has_related_party_transactions" ${state.has_related_party_transactions ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6038a.label.value">Total transaction value ($)</span>
                    <input type="number" step="1000" name="total_related_party_value" value="${state.total_related_party_value}"></label>
                <label><span data-i18n="view.s6038a.label.years_unfiled">Years unfiled</span>
                    <input type="number" step="1" name="years_unfiled" value="${state.years_unfiled}"></label>
                <button class="primary" type="submit" data-i18n="view.s6038a.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6038a-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6038a.h2.categories">11 Reportable transaction categories</h2>
            <ul class="muted small">
                <li data-i18n="view.s6038a.cat.sales">Sales of stock-in-trade</li>
                <li data-i18n="view.s6038a.cat.rents">Rents / royalties paid</li>
                <li data-i18n="view.s6038a.cat.intangibles">Intangible property transferred</li>
                <li data-i18n="view.s6038a.cat.services">Services performed</li>
                <li data-i18n="view.s6038a.cat.tangible">Tangible property bought / sold</li>
                <li data-i18n="view.s6038a.cat.cooperative">Cost sharing arrangements</li>
                <li data-i18n="view.s6038a.cat.commissions">Commissions / brokerage fees</li>
                <li data-i18n="view.s6038a.cat.interest">Interest received / paid</li>
                <li data-i18n="view.s6038a.cat.premiums">Insurance premiums</li>
                <li data-i18n="view.s6038a.cat.amounts">Amounts loaned / borrowed</li>
                <li data-i18n="view.s6038a.cat.other">Other dispositions</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6038a.h2.related_party">Related party = ?</h2>
            <ul class="muted small">
                <li data-i18n="view.s6038a.rp.direct_25">Any 25% direct or indirect foreign shareholder</li>
                <li data-i18n="view.s6038a.rp.related_to_25">Any party related to 25% foreign shareholder under § 318 attribution</li>
                <li data-i18n="view.s6038a.rp.controlled">Any other foreign person controlled by 25% shareholder</li>
                <li data-i18n="view.s6038a.rp.cousins">Foreign brother-sister corps under common ownership</li>
                <li data-i18n="view.s6038a.rp.commonality">U.S. + foreign affiliates under common control</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6038a.h2.disregarded_llc">Disregarded LLC trap</h2>
            <p class="muted small" data-i18n="view.s6038a.dis.body">
                Pre-2017: foreign-owned SMLLC = no US tax / filing if no ETB. Treas. Reg.
                <strong>§ 301.7701-2 (final 2017)</strong> treats disregarded entity as
                separate corp for § 6038A reporting. Now required to file <strong>pro-forma
                1120 + Form 5472</strong>. Doing nothing = $25k+ penalties per year.
            </p>
        </div>
    `;
    document.getElementById('s6038a-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.is_us_corp = !!fd.get('is_us_corp');
        state.is_disregarded_llc = !!fd.get('is_disregarded_llc');
        state.foreign_ownership_pct = Number(fd.get('foreign_ownership_pct')) || 0;
        state.has_related_party_transactions = !!fd.get('has_related_party_transactions');
        state.total_related_party_value = Number(fd.get('total_related_party_value')) || 0;
        state.years_unfiled = Number(fd.get('years_unfiled')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6038a-output');
    if (!el) return;
    const isUsReportingCorp = (state.is_us_corp && state.foreign_ownership_pct >= 25)
        || (state.is_disregarded_llc && state.foreign_ownership_pct >= 25);
    const requiresFiling = isUsReportingCorp && state.has_related_party_transactions;
    const annualPenalty = Math.min(PENALTY_PER_FAILURE, MAX_PENALTY);
    const totalPenalty = requiresFiling ? state.years_unfiled * annualPenalty : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6038a.h2.result">Filing requirement</h2>
            <div class="cards">
                <div class="card ${isUsReportingCorp ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6038a.card.is_reporting">US reporting corp?</div>
                    <div class="value">${isUsReportingCorp ? esc(t('view.s6038a.status.yes')) : esc(t('view.s6038a.status.no'))}</div>
                </div>
                <div class="card ${requiresFiling ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6038a.card.requires">Form 5472 required?</div>
                    <div class="value">${requiresFiling ? esc(t('view.s6038a.status.yes')) : esc(t('view.s6038a.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6038a.card.annual_penalty">Annual penalty</div>
                    <div class="value">$${annualPenalty.toLocaleString()}</div>
                </div>
                <div class="card ${totalPenalty > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6038a.card.total_penalty">Total penalty exposure</div>
                    <div class="value">$${totalPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.is_disregarded_llc ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s6038a.warning.dis">
                    Foreign-owned disregarded LLC (since 2017): MUST file pro-forma 1120 with
                    Form 5472 attached. Common trap with foreign-owned single-member LLCs holding
                    US real estate. Penalties stack year-over-year. No estate-tax / income filing
                    history? Still required. File via "Form 5472 only" return paperwork.
                </p>
            ` : ''}
        </div>
    `;
}
