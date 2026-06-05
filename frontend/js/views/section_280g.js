// IRC § 280G — Golden Parachute 20% Excise Tax.
// Change-of-control comp ≥ 3× "base amount" (5-yr avg W-2): excess = "excess parachute payment".
// 20% EXCISE on recipient + DISALLOWED deduction to corp on excess.
// Applies to "disqualified individuals" — officers, > 1% shareholders, highly comp (top 1%).
// Section 280G(b)(5) cleansing: shareholder vote (75%+) cleans excess from G.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const SAFE_HARBOR_MULTIPLE = 3;
const EXCISE_RATE = 0.20;
const SHAREHOLDER_CLEANSING_THRESHOLD = 0.75;

let state = {
    is_disqualified_individual: true,
    base_amount_5yr_avg_w2: 0,
    parachute_payment_total: 0,
    public_company: true,
    shareholder_vote_cleared: false,
    is_small_business_corp: false,
    corp_marginal_rate: 0.21,
    individual_marginal_rate: 0.37,
};

export async function renderSection280g(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s280g.h1.title">// § 280G GOLDEN PARACHUTE EXCISE</span></h1>
        <p class="muted small" data-i18n="view.s280g.hint.intro">
            Change-of-control comp ≥ <strong>3× base amount</strong> (5-yr avg Box 1 W-2):
            entire amount over <strong>1× base</strong> is "excess parachute payment".
            <strong>20% excise on recipient</strong> + <strong>DISALLOWED deduction</strong> to
            corp on excess. Applies to "disqualified individuals" — officers, > 1% shareholders,
            highly compensated (top 1%). <strong>Private cos: § 280G(b)(5) cleansing</strong> by
            75%+ shareholder vote.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s280g.h2.inputs">Inputs</h2>
            <form id="s280g-form" class="inline-form">
                <label><span data-i18n="view.s280g.label.disqualified">Disqualified individual?</span>
                    <input type="checkbox" name="is_disqualified_individual" ${state.is_disqualified_individual ? 'checked' : ''}></label>
                <label><span data-i18n="view.s280g.label.base">5-yr W-2 base amount ($)</span>
                    <input type="number" step="0.01" name="base_amount_5yr_avg_w2" value="${state.base_amount_5yr_avg_w2}"></label>
                <label><span data-i18n="view.s280g.label.parachute">Parachute payment total ($)</span>
                    <input type="number" step="0.01" name="parachute_payment_total" value="${state.parachute_payment_total}"></label>
                <label><span data-i18n="view.s280g.label.public">Public company?</span>
                    <input type="checkbox" name="public_company" ${state.public_company ? 'checked' : ''}></label>
                <label><span data-i18n="view.s280g.label.cleansed">Shareholder vote ≥75% cleansing?</span>
                    <input type="checkbox" name="shareholder_vote_cleared" ${state.shareholder_vote_cleared ? 'checked' : ''}></label>
                <label><span data-i18n="view.s280g.label.smb">Small Business Corp (no public stock)?</span>
                    <input type="checkbox" name="is_small_business_corp" ${state.is_small_business_corp ? 'checked' : ''}></label>
                <label><span data-i18n="view.s280g.label.corp_rate">Corp marginal rate</span>
                    <input type="number" step="0.01" name="corp_marginal_rate" value="${state.corp_marginal_rate}"></label>
                <label><span data-i18n="view.s280g.label.indiv_rate">Individual marginal rate</span>
                    <input type="number" step="0.01" name="individual_marginal_rate" value="${state.individual_marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s280g.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s280g-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s280g.h2.cleansing">Private company cleansing process</h2>
            <ul class="muted small">
                <li data-i18n="view.s280g.clean.smb">Must qualify as Small Business Corp (no public stock)</li>
                <li data-i18n="view.s280g.clean.adequate">Adequate disclosure of all material facts to shareholders</li>
                <li data-i18n="view.s280g.clean.75pct">Vote ≥ 75% of OUTSTANDING shares (not just present)</li>
                <li data-i18n="view.s280g.clean.exclusions">Disqualified persons + their relatives + control entities EXCLUDED from vote</li>
                <li data-i18n="view.s280g.clean.timing">Vote BEFORE payment + binding right to refuse</li>
                <li data-i18n="view.s280g.clean.tax_savings">If cleansed: no 20% excise + corp gets full deduction</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s280g.h2.workarounds">Workarounds</h2>
            <ul class="muted small">
                <li data-i18n="view.s280g.work.gross_up">Cap on parachute at 2.99× base (avoid 3× threshold)</li>
                <li data-i18n="view.s280g.work.best_after_tax">"Best-after-tax" cutback clause: cap at lower of 2.99× or threshold that maximizes net</li>
                <li data-i18n="view.s280g.work.reasonable">Allocate to reasonable post-acquisition comp / non-compete (not parachute)</li>
                <li data-i18n="view.s280g.work.vested">Pre-existing vested benefits = not parachute (must predate change of control)</li>
                <li data-i18n="view.s280g.work.cleansing_vote">Private co: shareholder cleansing vote (75% threshold)</li>
                <li data-i18n="view.s280g.work.gross_up_pay">Gross-up payment: corp pays excise tax — but gross-up itself is parachute</li>
                <li data-i18n="view.s280g.work.severance_unwind">Restructure severance to spread over post-CoC employment</li>
            </ul>
        </div>
    `;
    document.getElementById('s280g-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.is_disqualified_individual = !!fd.get('is_disqualified_individual');
        state.base_amount_5yr_avg_w2 = Number(fd.get('base_amount_5yr_avg_w2')) || 0;
        state.parachute_payment_total = Number(fd.get('parachute_payment_total')) || 0;
        state.public_company = !!fd.get('public_company');
        state.shareholder_vote_cleared = !!fd.get('shareholder_vote_cleared');
        state.is_small_business_corp = !!fd.get('is_small_business_corp');
        state.corp_marginal_rate = Number(fd.get('corp_marginal_rate')) || 0.21;
        state.individual_marginal_rate = Number(fd.get('individual_marginal_rate')) || 0.37;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s280g-output');
    if (!el) return;
    const safeHarborCap = state.base_amount_5yr_avg_w2 * SAFE_HARBOR_MULTIPLE;
    const triggers280G = state.is_disqualified_individual && state.parachute_payment_total >= safeHarborCap;
    const cleansed = !state.public_company && state.is_small_business_corp && state.shareholder_vote_cleared;
    const excessParachute = (triggers280G && !cleansed)
        ? state.parachute_payment_total - state.base_amount_5yr_avg_w2
        : 0;
    const excise = excessParachute * EXCISE_RATE;
    const deductionLost = excessParachute * state.corp_marginal_rate;
    const totalCost = excise + deductionLost;
    const netToIndividual = state.parachute_payment_total - state.parachute_payment_total * state.individual_marginal_rate - excise;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s280g.h2.result">§ 280G outcome</h2>
            <div class="cards">
                <div class="card ${triggers280G ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s280g.card.triggers">Triggers § 280G?</div>
                    <div class="value">${triggers280G ? esc(t('view.s280g.status.yes')) : esc(t('view.s280g.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s280g.card.safe_harbor">3× safe harbor</div>
                    <div class="value">$${safeHarborCap.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${excessParachute > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s280g.card.excess">Excess parachute payment</div>
                    <div class="value">$${excessParachute.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${cleansed ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s280g.card.cleansed">Cleansed by vote?</div>
                    <div class="value">${cleansed ? esc(t('view.s280g.status.yes')) : esc(t('view.s280g.status.no'))}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s280g.card.excise">20% excise tax</div>
                    <div class="value">$${excise.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s280g.card.deduction_lost">Corp deduction lost</div>
                    <div class="value">$${deductionLost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s280g.card.total_cost">Total economic cost</div>
                    <div class="value">$${totalCost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s280g.card.net_to_individual">Net to individual (after taxes)</div>
                    <div class="value">$${netToIndividual.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
