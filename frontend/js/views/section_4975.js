// IRC § 4975 — Prohibited Transactions for IRAs / Qualified Plans.
// 15% excise on amount involved (per year) + 100% if not corrected within taxable period.
// PT killer for self-directed IRAs investing in private deals, real estate, crypto, etc.
// IRA holder is the "disqualified person" — most PTs disqualify IRA entirely (deemed distributed).
// Roll-your-own RE / business with IRA funds = high audit risk.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const EXCISE_INITIAL = 0.15;
const EXCISE_FULL = 1.00;

let state = {
    amount_involved: 0,
    years_uncorrected: 1,
    transaction_type: 'self_dealing',
    is_ira: true,
    deemed_distribution: false,
    distribution_amount: 0,
    marginal_rate: 0.37,
};

export async function renderSection4975(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s4975.h1.title">// § 4975 PROHIBITED TRANSACTIONS</span></h1>
        <p class="muted small" data-i18n="view.s4975.hint.intro">
            <strong>15% excise on amount involved</strong> (per year) + <strong>100% if uncorrected</strong>
            within taxable period. <strong>IRA PT = automatic distribution of ENTIRE IRA</strong> on
            Jan 1 of year of PT — § 408(e)(2). Most common traps in self-directed IRAs investing
            in real estate, private business, crypto. <strong>Disqualified persons:</strong>
            IRA holder, spouse, ancestors, descendants, fiduciaries (50% owned entities).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s4975.h2.inputs">Inputs</h2>
            <form id="s4975-form" class="inline-form">
                <label><span data-i18n="view.s4975.label.kind">PT type</span>
                    <select name="transaction_type">
                        <option value="self_dealing" ${state.transaction_type === 'self_dealing' ? 'selected' : ''}>Self-dealing</option>
                        <option value="furnishing">Furnishing goods / services</option>
                        <option value="extension_credit">Extending credit (borrow from IRA)</option>
                        <option value="purchase">Buying / selling with disqualified person</option>
                        <option value="lease">Lease to / from disqualified person</option>
                        <option value="use_assets">Use of IRA assets by disqualified person</option>
                        <option value="payment_to_fiduciary">Payment to fiduciary</option>
                    </select>
                </label>
                <label><span data-i18n="view.s4975.label.amount">Amount involved ($)</span>
                    <input type="number" step="1000" name="amount_involved" value="${state.amount_involved}"></label>
                <label><span data-i18n="view.s4975.label.years">Years uncorrected</span>
                    <input type="number" step="1" name="years_uncorrected" value="${state.years_uncorrected}"></label>
                <label><span data-i18n="view.s4975.label.is_ira">IRA (vs qualified plan)?</span>
                    <input type="checkbox" name="is_ira" ${state.is_ira ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4975.label.deemed">Deemed distribution triggered?</span>
                    <input type="checkbox" name="deemed_distribution" ${state.deemed_distribution ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4975.label.dist_amount">Total IRA balance at PT date ($)</span>
                    <input type="number" step="1000" name="distribution_amount" value="${state.distribution_amount}"></label>
                <label><span data-i18n="view.s4975.label.marginal">Marginal rate</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s4975.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s4975-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4975.h2.disqualified">Disqualified persons</h2>
            <ul class="muted small">
                <li data-i18n="view.s4975.dq.you">You (the IRA holder / plan participant)</li>
                <li data-i18n="view.s4975.dq.spouse">Spouse</li>
                <li data-i18n="view.s4975.dq.lineal">Ancestors + descendants (parents, kids, grandkids) + their spouses</li>
                <li data-i18n="view.s4975.dq.50_owned">Entity 50%+ owned by combination of above</li>
                <li data-i18n="view.s4975.dq.officer">Officer / director / 10% shareholder of 50%-owned entity</li>
                <li data-i18n="view.s4975.dq.not_siblings">NOT siblings, NOT in-laws beyond spouse</li>
                <li data-i18n="view.s4975.dq.fiduciary">Plan fiduciary, anyone with discretion over IRA assets</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4975.h2.common_traps">Common SDIRA traps</h2>
            <ul class="muted small">
                <li data-i18n="view.s4975.trap.live_in">Buying real estate IRA owns and living in it (use by you)</li>
                <li data-i18n="view.s4975.trap.fix_it">Personally doing repairs on IRA-owned property (services to IRA)</li>
                <li data-i18n="view.s4975.trap.lend_to_co">Lending IRA funds to your closely-held biz</li>
                <li data-i18n="view.s4975.trap.guarantee">Personally guaranteeing IRA loan (extension of credit)</li>
                <li data-i18n="view.s4975.trap.buy_from_self">Buying property already owned personally with IRA funds</li>
                <li data-i18n="view.s4975.trap.commission">Receiving real estate commission on IRA's property purchase</li>
                <li data-i18n="view.s4975.trap.rent_to_kid">Renting IRA-owned property to your child</li>
                <li data-i18n="view.s4975.trap.checkbook_warning">"Checkbook IRA" / IRA-LLC: Swanson / Niemann ok IF structured right + no comingling</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4975.h2.exemptions">PTE exemptions</h2>
            <ul class="muted small">
                <li data-i18n="view.s4975.ex.pte_84_14">PTE 84-14 (QPAM) — independent qualified professional asset manager</li>
                <li data-i18n="view.s4975.ex.pte_75_1">PTE 75-1 — bank / broker-dealer services</li>
                <li data-i18n="view.s4975.ex.pte_77_4">PTE 77-4 — mutual fund investment by self-directed IRA</li>
                <li data-i18n="view.s4975.ex.pte_80_26">PTE 80-26 — short-term loans for ordinary operating expenses</li>
                <li data-i18n="view.s4975.ex.section_408">§ 408 ROBS (Rollover for Business Startups) C-corp structure — narrow</li>
            </ul>
        </div>
    `;
    document.getElementById('s4975-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.transaction_type = fd.get('transaction_type');
        state.amount_involved = Number(fd.get('amount_involved')) || 0;
        state.years_uncorrected = Number(fd.get('years_uncorrected')) || 1;
        state.is_ira = !!fd.get('is_ira');
        state.deemed_distribution = !!fd.get('deemed_distribution');
        state.distribution_amount = Number(fd.get('distribution_amount')) || 0;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.37;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s4975-output');
    if (!el) return;
    const exciseInitial = state.amount_involved * EXCISE_INITIAL * state.years_uncorrected;
    const exciseFull = state.years_uncorrected >= 2 ? state.amount_involved * EXCISE_FULL : 0;
    const totalExcise = exciseInitial + exciseFull;
    const deemedDistributionTax = state.is_ira && state.deemed_distribution
        ? state.distribution_amount * state.marginal_rate
        : 0;
    const earlyPenalty = state.is_ira && state.deemed_distribution
        ? state.distribution_amount * 0.10
        : 0;
    const totalCost = totalExcise + deemedDistributionTax + earlyPenalty;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s4975.h2.result">PT damage</h2>
            <div class="cards">
                <div class="card neg">
                    <div class="label" data-i18n="view.s4975.card.excise_15">15% excise (initial)</div>
                    <div class="value">$${exciseInitial.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${exciseFull > 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s4975.card.excise_100">100% excise (uncorrected)</div>
                    <div class="value">$${exciseFull.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s4975.card.total_excise">Total excise</div>
                    <div class="value">$${totalExcise.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${state.is_ira && state.deemed_distribution ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s4975.card.dist_tax">Deemed distribution tax</div>
                        <div class="value">$${deemedDistributionTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                    <div class="card neg">
                        <div class="label" data-i18n="view.s4975.card.early_penalty">10% early-withdrawal</div>
                        <div class="value">$${earlyPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                <div class="card neg">
                    <div class="label" data-i18n="view.s4975.card.total">TOTAL cost</div>
                    <div class="value">$${totalCost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
