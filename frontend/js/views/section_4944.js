// IRC § 4944 — Private Foundation Jeopardizing Investments.
// PF managers must exercise "ordinary business care + prudence" — investments jeopardizing
// charitable purpose = excise tax. 10% initial on PF + 10% on manager (knowing).
// 25% if not corrected + 5% additional manager penalty.
// § 4944(c): Program-Related Investments (PRI) EXEMPT — primary purpose charitable + no
// significant purpose to produce income.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const INITIAL_PF_RATE = 0.10;
const INITIAL_MANAGER_RATE = 0.10;
const SECOND_PF_RATE = 0.25;
const SECOND_MANAGER_RATE = 0.05;

let state = {
    investment_amount: 0,
    investment_type: 'speculation',
    pri_qualifies: false,
    pri_primary_purpose_charitable: false,
    pri_no_significant_income_purpose: false,
    pri_no_lobbying_political: false,
    manager_knew: false,
    investment_made_pre_69: false,
    months_uncorrected: 0,
};

export async function renderSection4944(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s4944.h1.title">// § 4944 PF JEOPARDIZING INVESTMENTS</span></h1>
        <p class="muted small" data-i18n="view.s4944.hint.intro">
            PF managers must exercise <strong>ordinary business care + prudence</strong> —
            investments jeopardizing charitable purpose = excise. <strong>10% initial</strong> on
            PF + <strong>10% on manager</strong> (knowing). <strong>25% PF + 5% additional manager</strong>
            if not corrected. <strong>§ 4944(c) PRI EXEMPT</strong> — primary purpose charitable +
            no significant income purpose + no lobbying / political. Most foundations use modern
            portfolio theory.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s4944.h2.inputs">Inputs</h2>
            <form id="s4944-form" class="inline-form">
                <label><span data-i18n="view.s4944.label.amount">Investment amount ($)</span>
                    <input type="number" step="0.01" name="investment_amount" value="${state.investment_amount}"></label>
                <label><span data-i18n="view.s4944.label.type">Investment type</span>
                    <select name="investment_type">
                        <option value="speculation" ${state.investment_type === 'speculation' ? 'selected' : ''}>Speculation / high-risk option</option>
                        <option value="margin">Margin trading</option>
                        <option value="commodity">Commodity futures</option>
                        <option value="short_sale">Short sales</option>
                        <option value="warrants">Warrants / oil + gas</option>
                        <option value="undeveloped_land">Undeveloped land</option>
                        <option value="working_interest">Working interests</option>
                        <option value="single_position">Concentrated single-stock position</option>
                        <option value="balanced">Balanced portfolio (safe)</option>
                        <option value="bonds">Government / investment-grade bonds</option>
                    </select>
                </label>
                <label><span data-i18n="view.s4944.label.pri">Asserted as PRI (§ 4944(c))?</span>
                    <input type="checkbox" name="pri_qualifies" ${state.pri_qualifies ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4944.label.charitable">Primary purpose charitable?</span>
                    <input type="checkbox" name="pri_primary_purpose_charitable" ${state.pri_primary_purpose_charitable ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4944.label.no_income">No significant income purpose?</span>
                    <input type="checkbox" name="pri_no_significant_income_purpose" ${state.pri_no_significant_income_purpose ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4944.label.no_lobby">No lobbying / political?</span>
                    <input type="checkbox" name="pri_no_lobbying_political" ${state.pri_no_lobbying_political ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4944.label.knew">Manager knew it was jeopardizing?</span>
                    <input type="checkbox" name="manager_knew" ${state.manager_knew ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4944.label.pre_69">Investment made before 1969?</span>
                    <input type="checkbox" name="investment_made_pre_69" ${state.investment_made_pre_69 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4944.label.months_uncorr">Months uncorrected</span>
                    <input type="number" step="1" name="months_uncorrected" value="${state.months_uncorrected}"></label>
                <button class="primary" type="submit" data-i18n="view.s4944.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s4944-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4944.h2.pri">PRI requirements (§ 4944(c))</h2>
            <ul class="muted small">
                <li data-i18n="view.s4944.pri.primary">Primary purpose: accomplish exempt purposes (charitable / educational / scientific)</li>
                <li data-i18n="view.s4944.pri.no_income">No SIGNIFICANT purpose to produce income</li>
                <li data-i18n="view.s4944.pri.no_political">No purpose to influence legislation / political campaigns</li>
                <li data-i18n="view.s4944.pri.examples">Examples: micro-loans, low-interest community development loans, charter school bonds</li>
                <li data-i18n="view.s4944.pri.counts_payout">PRIs COUNT toward § 4942 5% minimum payout requirement</li>
                <li data-i18n="view.s4944.pri.terms_loans">Below-market loans + equity in social enterprises common PRIs</li>
                <li data-i18n="view.s4944.pri.preamble">Notice 2015-62: clarified ESG / mission-related investing</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4944.h2.prudence">"Ordinary business care + prudence" factors</h2>
            <ul class="muted small">
                <li data-i18n="view.s4944.prud.long_short">Long-term + short-term financial needs of PF</li>
                <li data-i18n="view.s4944.prud.diversification">Diversification of overall portfolio</li>
                <li data-i18n="view.s4944.prud.expected_return">Expected total return (income + appreciation)</li>
                <li data-i18n="view.s4944.prud.tax_consequences">Tax consequences of investment decisions</li>
                <li data-i18n="view.s4944.prud.economic_conditions">General economic conditions</li>
                <li data-i18n="view.s4944.prud.uniform_act">Uniform Prudent Management of Institutional Funds Act (UPMIFA) — state law overlay</li>
                <li data-i18n="view.s4944.prud.ipsweb">Investment Policy Statement (IPS) documenting strategy</li>
                <li data-i18n="view.s4944.prud.review">Periodic portfolio review by qualified advisors</li>
                <li data-i18n="view.s4944.prud.no_per_se">NO per se prohibited investments under modern prudent investor rule</li>
            </ul>
        </div>
    `;
    document.getElementById('s4944-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.investment_amount = Number(fd.get('investment_amount')) || 0;
        state.investment_type = fd.get('investment_type');
        state.pri_qualifies = !!fd.get('pri_qualifies');
        state.pri_primary_purpose_charitable = !!fd.get('pri_primary_purpose_charitable');
        state.pri_no_significant_income_purpose = !!fd.get('pri_no_significant_income_purpose');
        state.pri_no_lobbying_political = !!fd.get('pri_no_lobbying_political');
        state.manager_knew = !!fd.get('manager_knew');
        state.investment_made_pre_69 = !!fd.get('investment_made_pre_69');
        state.months_uncorrected = Number(fd.get('months_uncorrected')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s4944-output');
    if (!el) return;
    const priValid = state.pri_qualifies && state.pri_primary_purpose_charitable
        && state.pri_no_significant_income_purpose && state.pri_no_lobbying_political;
    const highRiskTypes = ['speculation', 'margin', 'commodity', 'short_sale', 'warrants', 'undeveloped_land', 'working_interest'];
    const isJeopardizing = !priValid && !state.investment_made_pre_69 && highRiskTypes.includes(state.investment_type);
    const pfInitial = isJeopardizing ? state.investment_amount * INITIAL_PF_RATE : 0;
    const managerInitial = (isJeopardizing && state.manager_knew) ? state.investment_amount * INITIAL_MANAGER_RATE : 0;
    const pfSecond = (isJeopardizing && state.months_uncorrected >= 24) ? state.investment_amount * SECOND_PF_RATE : 0;
    const managerSecond = (isJeopardizing && state.manager_knew && state.months_uncorrected >= 24) ? state.investment_amount * SECOND_MANAGER_RATE : 0;
    const totalExcise = pfInitial + managerInitial + pfSecond + managerSecond;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s4944.h2.result">§ 4944 analysis</h2>
            <div class="cards">
                <div class="card ${priValid ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s4944.card.pri_valid">PRI exempt?</div>
                    <div class="value">${priValid ? esc(t('view.s4944.status.yes')) : esc(t('view.s4944.status.no'))}</div>
                </div>
                <div class="card ${isJeopardizing ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s4944.card.jeopardizing">Jeopardizing?</div>
                    <div class="value">${isJeopardizing ? esc(t('view.s4944.status.yes')) : esc(t('view.s4944.status.no'))}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s4944.card.pf_initial">PF initial 10%</div>
                    <div class="value">$${pfInitial.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${managerInitial > 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s4944.card.manager_initial">Manager 10% (if knew)</div>
                    <div class="value">$${managerInitial.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${pfSecond > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s4944.card.pf_second">PF SECOND 25%</div>
                        <div class="value">$${pfSecond.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                <div class="card neg">
                    <div class="label" data-i18n="view.s4944.card.total">Total excise</div>
                    <div class="value">$${totalExcise.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
