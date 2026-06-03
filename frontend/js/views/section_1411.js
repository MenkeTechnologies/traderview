// IRC § 1411 — Net Investment Income Tax (3.8% surtax).
// Triggered when MAGI > $200k single / $250k MFJ / $125k MFS.
// Tax base = lesser of (1) net investment income OR (2) MAGI over threshold.
// Investment income: interest, dividends, capital gains, rental, passive K-1.
// EXCLUDED: § 475(f) MTM ordinary, active S-corp / partnership earnings, qualified plans.

import { currentViewToken, viewIsCurrent } from '../app.js';

const THRESHOLDS = {
    single: 200_000,
    hoh: 200_000,
    mfj: 250_000,
    mfs: 125_000,
};
const NIIT_RATE = 0.038;

let state = {
    filing_status: 'single',
    magi: 0,
    interest: 0,
    dividends: 0,
    capital_gains: 0,
    rental_income: 0,
    passive_kone_income: 0,
    investment_expenses: 0,
    mtm_475f_ordinary: 0,
    active_business_income: 0,
    retirement_distributions: 0,
};

export async function renderSection1411(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1411.h1.title">// § 1411 NIIT 3.8% SURTAX</span></h1>
        <p class="muted small" data-i18n="view.s1411.hint.intro">
            3.8% surtax on net investment income when <strong>MAGI &gt; $200k single / $250k MFJ
            / $125k MFS</strong>. Tax base = LESSER of (1) net investment income OR
            (2) MAGI excess over threshold. <strong>§ 475(f) MTM ordinary EXCLUDED</strong>
            (huge trader benefit). Active S-corp earnings EXCLUDED. Retirement distributions EXCLUDED.
            Reported on Form 8960. Not indexed for inflation.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1411.h2.inputs">Inputs</h2>
            <form id="s1411-form" class="inline-form">
                <label><span data-i18n="view.s1411.label.filing">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filing_status === 'single' ? 'selected' : ''}>Single</option>
                        <option value="hoh" ${state.filing_status === 'hoh' ? 'selected' : ''}>HoH</option>
                        <option value="mfj" ${state.filing_status === 'mfj' ? 'selected' : ''}>MFJ</option>
                        <option value="mfs" ${state.filing_status === 'mfs' ? 'selected' : ''}>MFS</option>
                    </select>
                </label>
                <label><span data-i18n="view.s1411.label.magi">MAGI ($)</span>
                    <input type="number" step="1000" name="magi" value="${state.magi}"></label>
                <hr style="grid-column:1/-1">
                <label><span data-i18n="view.s1411.label.interest">Interest income ($)</span>
                    <input type="number" step="100" name="interest" value="${state.interest}"></label>
                <label><span data-i18n="view.s1411.label.dividends">Dividends (qual + ord) ($)</span>
                    <input type="number" step="100" name="dividends" value="${state.dividends}"></label>
                <label><span data-i18n="view.s1411.label.cap_gains">Capital gains ($)</span>
                    <input type="number" step="100" name="capital_gains" value="${state.capital_gains}"></label>
                <label><span data-i18n="view.s1411.label.rental">Rental income (net) ($)</span>
                    <input type="number" step="100" name="rental_income" value="${state.rental_income}"></label>
                <label><span data-i18n="view.s1411.label.passive_k1">Passive K-1 income ($)</span>
                    <input type="number" step="100" name="passive_kone_income" value="${state.passive_kone_income}"></label>
                <label><span data-i18n="view.s1411.label.expenses">Investment expenses ($)</span>
                    <input type="number" step="100" name="investment_expenses" value="${state.investment_expenses}"></label>
                <hr style="grid-column:1/-1">
                <label><span data-i18n="view.s1411.label.mtm">§ 475(f) MTM ordinary (EXCLUDED) ($)</span>
                    <input type="number" step="100" name="mtm_475f_ordinary" value="${state.mtm_475f_ordinary}"></label>
                <label><span data-i18n="view.s1411.label.active_biz">Active S-corp / SE earnings (EXCLUDED) ($)</span>
                    <input type="number" step="100" name="active_business_income" value="${state.active_business_income}"></label>
                <label><span data-i18n="view.s1411.label.retirement">Retirement distributions (EXCLUDED) ($)</span>
                    <input type="number" step="100" name="retirement_distributions" value="${state.retirement_distributions}"></label>
                <button class="primary" type="submit" data-i18n="view.s1411.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1411-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1411.h2.exclusions">Income types EXCLUDED from NIIT</h2>
            <ul class="muted small">
                <li data-i18n="view.s1411.excl.wages">Wages + W-2 compensation</li>
                <li data-i18n="view.s1411.excl.self_emp">Self-employment income (active)</li>
                <li data-i18n="view.s1411.excl.active_kone">Active S-corp distributions / K-1 if material participation</li>
                <li data-i18n="view.s1411.excl.mtm">§ 475(f) MTM ordinary income — trader-specific advantage</li>
                <li data-i18n="view.s1411.excl.retirement">Roth + 401(k) + IRA distributions</li>
                <li data-i18n="view.s1411.excl.muni">Tax-exempt municipal bond interest</li>
                <li data-i18n="view.s1411.excl.gain_residence">§ 121 home sale excluded gain</li>
                <li data-i18n="view.s1411.excl.like_kind">§ 1031 deferred like-kind exchange gain</li>
                <li data-i18n="view.s1411.excl.section_988">§ 988 forex ordinary if held as trader</li>
                <li data-i18n="view.s1411.excl.alimony">Alimony / child support</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1411.h2.planning">Planning moves</h2>
            <ul class="muted small">
                <li data-i18n="view.s1411.plan.475f">§ 475(f) MTM election if TTS-qualified — escape NIIT entirely on trading</li>
                <li data-i18n="view.s1411.plan.material">Establish material participation in passive activities — recharacterize as active</li>
                <li data-i18n="view.s1411.plan.bunching">Bunch investment income into low-MAGI years</li>
                <li data-i18n="view.s1411.plan.roth">Roth conversions — distributions exempt forever after qualifying</li>
                <li data-i18n="view.s1411.plan.muni">Shift to municipal bonds — interest excluded</li>
                <li data-i18n="view.s1411.plan.qof">§ 1400Z Opportunity Zone investment — defer + reduce gain</li>
            </ul>
        </div>
    `;
    document.getElementById('s1411-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.filing_status = fd.get('filing_status');
        state.magi = Number(fd.get('magi')) || 0;
        state.interest = Number(fd.get('interest')) || 0;
        state.dividends = Number(fd.get('dividends')) || 0;
        state.capital_gains = Number(fd.get('capital_gains')) || 0;
        state.rental_income = Number(fd.get('rental_income')) || 0;
        state.passive_kone_income = Number(fd.get('passive_kone_income')) || 0;
        state.investment_expenses = Number(fd.get('investment_expenses')) || 0;
        state.mtm_475f_ordinary = Number(fd.get('mtm_475f_ordinary')) || 0;
        state.active_business_income = Number(fd.get('active_business_income')) || 0;
        state.retirement_distributions = Number(fd.get('retirement_distributions')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1411-output');
    if (!el) return;
    const threshold = THRESHOLDS[state.filing_status];
    const grossNII = state.interest + state.dividends + state.capital_gains
        + state.rental_income + state.passive_kone_income;
    const netNII = Math.max(0, grossNII - state.investment_expenses);
    const magiExcess = Math.max(0, state.magi - threshold);
    const taxBase = Math.min(netNII, magiExcess);
    const niit = taxBase * NIIT_RATE;
    const wouldOweOnAll = grossNII * NIIT_RATE;
    const mtmSaving = state.mtm_475f_ordinary * NIIT_RATE;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1411.h2.result">NIIT calculation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s1411.card.threshold">Threshold</div>
                    <div class="value">$${threshold.toLocaleString()}</div>
                </div>
                <div class="card ${magiExcess > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s1411.card.magi_excess">MAGI excess</div>
                    <div class="value">$${magiExcess.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1411.card.gross_nii">Gross investment income</div>
                    <div class="value">$${grossNII.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1411.card.net_nii">Net investment income</div>
                    <div class="value">$${netNII.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1411.card.tax_base">Tax base (lesser of)</div>
                    <div class="value">$${taxBase.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1411.card.niit_owed">NIIT owed (3.8%)</div>
                    <div class="value">$${niit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1411.card.mtm_saved">§ 475(f) MTM saves</div>
                    <div class="value">$${mtmSaving.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
