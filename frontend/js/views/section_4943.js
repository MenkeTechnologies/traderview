// IRC § 4943 — Private Foundation Excess Business Holdings.
// PF + disqualified persons may hold up to 20% of voting stock of business enterprise
// (35% if 3rd-party effective control). 5-yr divestiture period for gifts / bequests.
// 10% initial excise + 200% if not corrected. SECURE 2.0 Excise: 100% solely-owned biz holdings.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const PERMITTED_PCT_STANDARD = 0.20;
const PERMITTED_PCT_THIRD_PARTY = 0.35;
const DE_MINIMIS_PCT = 0.02;
const INITIAL_EXCISE = 0.10;
const SECOND_EXCISE = 2.00;

let state = {
    business_total_voting_stock_value: 0,
    pf_holdings_value: 0,
    disqualified_persons_holdings_value: 0,
    third_party_effective_control: false,
    is_program_related_investment: false,
    months_since_acquisition: 0,
    is_gift_or_bequest: false,
    years_uncorrected: 0,
};

export async function renderSection4943(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s4943.h1.title">// § 4943 PF EXCESS BUSINESS HOLDINGS</span></h1>
        <p class="muted small" data-i18n="view.s4943.hint.intro">
            PF + disqualified persons may hold up to <strong>20% of voting stock</strong>
            (<strong>35% if 3rd-party effective control</strong>). De minimis: PF alone ≤ 2%
            always permitted. <strong>5-yr divestiture period</strong> for gifts / bequests
            (extendable). <strong>10% initial excise</strong> on excess + <strong>200%</strong> if
            not corrected within taxable period. <strong>SECURE 2.0:</strong> 100% solely-owned
            biz holdings can qualify as Program-Related Investment (PRI).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s4943.h2.inputs">Inputs</h2>
            <form id="s4943-form" class="inline-form">
                <label><span data-i18n="view.s4943.label.business">Business total voting stock value ($)</span>
                    <input type="number" step="0.01" name="business_total_voting_stock_value" value="${state.business_total_voting_stock_value}"></label>
                <label><span data-i18n="view.s4943.label.pf">PF's holdings value ($)</span>
                    <input type="number" step="0.01" name="pf_holdings_value" value="${state.pf_holdings_value}"></label>
                <label><span data-i18n="view.s4943.label.dq">Disqualified persons' holdings ($)</span>
                    <input type="number" step="0.01" name="disqualified_persons_holdings_value" value="${state.disqualified_persons_holdings_value}"></label>
                <label><span data-i18n="view.s4943.label.third_party">3rd-party effective control?</span>
                    <input type="checkbox" name="third_party_effective_control" ${state.third_party_effective_control ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4943.label.pri">Program-Related Investment (PRI)?</span>
                    <input type="checkbox" name="is_program_related_investment" ${state.is_program_related_investment ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4943.label.months">Months since acquisition</span>
                    <input type="number" step="1" name="months_since_acquisition" value="${state.months_since_acquisition}"></label>
                <label><span data-i18n="view.s4943.label.gift">Acquired by gift / bequest?</span>
                    <input type="checkbox" name="is_gift_or_bequest" ${state.is_gift_or_bequest ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4943.label.years_uncorr">Years uncorrected</span>
                    <input type="number" step="1" name="years_uncorrected" value="${state.years_uncorrected}"></label>
                <button class="primary" type="submit" data-i18n="view.s4943.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s4943-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4943.h2.exceptions">Exceptions + safe harbors</h2>
            <ul class="muted small">
                <li data-i18n="view.s4943.ex.de_minimis">De minimis: PF alone ≤ 2% always allowed regardless of aggregate</li>
                <li data-i18n="view.s4943.ex.pri">Program-Related Investments (§ 4944(c)): exempt from § 4943</li>
                <li data-i18n="view.s4943.ex.functionally_related">Functionally related business (substantially-related charitable purpose)</li>
                <li data-i18n="view.s4943.ex.passive_income">Passive income business (rents, royalties, etc.) — different rules</li>
                <li data-i18n="view.s4943.ex.secure_2_0">SECURE 2.0 Newman exception: solely-owned biz qualifies as PRI if all profits charity</li>
                <li data-i18n="view.s4943.ex.farming">Agricultural ag-cooperative / pooled processing exemptions</li>
                <li data-i18n="view.s4943.ex.divestiture">5-yr divestiture window (extendable 5 yrs by IRS)</li>
                <li data-i18n="view.s4943.ex.unrelated_business">Unrelated trade or business: not exempt; subject to UBTI</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4943.h2.disposal_strategies">Disposal strategies during divestiture window</h2>
            <ul class="muted small">
                <li data-i18n="view.s4943.disp.sell_market">Sell to non-DP in arm's-length market transaction</li>
                <li data-i18n="view.s4943.disp.redeem">Corporate redemption (DP-owned must avoid § 4941 self-dealing)</li>
                <li data-i18n="view.s4943.disp.charitable">Donate to public charity</li>
                <li data-i18n="view.s4943.disp.pri_conversion">Re-characterize as PRI through governance change (Newman 2022)</li>
                <li data-i18n="view.s4943.disp.spin_off">Spin off the excess holdings to operating subsidiary</li>
                <li data-i18n="view.s4943.disp.extend">Request 5-yr extension (only one extension allowed)</li>
                <li data-i18n="view.s4943.disp.section_4943_c4">§ 4943(c)(4) — Grandfather rule for pre-1969 holdings</li>
            </ul>
        </div>
    `;
    document.getElementById('s4943-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.business_total_voting_stock_value = Number(fd.get('business_total_voting_stock_value')) || 0;
        state.pf_holdings_value = Number(fd.get('pf_holdings_value')) || 0;
        state.disqualified_persons_holdings_value = Number(fd.get('disqualified_persons_holdings_value')) || 0;
        state.third_party_effective_control = !!fd.get('third_party_effective_control');
        state.is_program_related_investment = !!fd.get('is_program_related_investment');
        state.months_since_acquisition = Number(fd.get('months_since_acquisition')) || 0;
        state.is_gift_or_bequest = !!fd.get('is_gift_or_bequest');
        state.years_uncorrected = Number(fd.get('years_uncorrected')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s4943-output');
    if (!el) return;
    if (state.is_program_related_investment) {
        el.innerHTML = `<div class="chart-panel"><p class="muted small pos" data-i18n="view.s4943.note.pri">Program-Related Investment exempt from § 4943.</p></div>`;
        return;
    }
    const pfPct = state.business_total_voting_stock_value > 0 ? state.pf_holdings_value / state.business_total_voting_stock_value : 0;
    const dqPct = state.business_total_voting_stock_value > 0 ? state.disqualified_persons_holdings_value / state.business_total_voting_stock_value : 0;
    const aggregatePct = pfPct + dqPct;
    const permittedPct = state.third_party_effective_control ? PERMITTED_PCT_THIRD_PARTY : PERMITTED_PCT_STANDARD;
    const deMinimisQualifies = pfPct <= DE_MINIMIS_PCT;
    const inDivestiturePeriod = state.is_gift_or_bequest && state.months_since_acquisition < 60;
    const excessPct = Math.max(0, aggregatePct - permittedPct);
    const excessValue = state.business_total_voting_stock_value * excessPct;
    const exciseInitial = !deMinimisQualifies && !inDivestiturePeriod ? excessValue * INITIAL_EXCISE * Math.max(1, state.years_uncorrected) : 0;
    const exciseSecond = state.years_uncorrected >= 2 ? excessValue * SECOND_EXCISE : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s4943.h2.result">Holdings analysis</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s4943.card.pf_pct">PF voting %</div>
                    <div class="value">${(pfPct * 100).toFixed(1)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s4943.card.dq_pct">DP voting %</div>
                    <div class="value">${(dqPct * 100).toFixed(1)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s4943.card.aggregate">Aggregate</div>
                    <div class="value">${(aggregatePct * 100).toFixed(1)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s4943.card.permitted">Permitted</div>
                    <div class="value">${(permittedPct * 100).toFixed(0)}%</div>
                </div>
                <div class="card ${deMinimisQualifies ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s4943.card.de_minimis">De minimis (PF ≤ 2%)</div>
                    <div class="value">${deMinimisQualifies ? esc(t('view.s4943.status.yes')) : esc(t('view.s4943.status.no'))}</div>
                </div>
                <div class="card ${inDivestiturePeriod ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s4943.card.divestiture">5-yr divestiture window</div>
                    <div class="value">${inDivestiturePeriod ? esc(t('view.s4943.status.yes')) : esc(t('view.s4943.status.no'))}</div>
                </div>
                <div class="card ${excessValue > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s4943.card.excess">Excess value</div>
                    <div class="value">$${excessValue.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s4943.card.excise_initial">Initial 10% excise</div>
                    <div class="value">$${exciseInitial.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${exciseSecond > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s4943.card.excise_200">200% SECOND excise</div>
                        <div class="value">$${exciseSecond.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
            </div>
        </div>
    `;
}
