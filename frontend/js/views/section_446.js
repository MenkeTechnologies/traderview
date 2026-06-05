// IRC § 446 — General Rule for Methods of Accounting.
// Two primary methods: Cash + Accrual. Hybrid permitted. Method must clearly reflect income.
// IRS may impose method if filer's method doesn't clearly reflect income.
// TCJA raised cash method threshold to $25M (now $30M 2024) gross receipts for small biz.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const SMALL_BIZ_GROSS_RECEIPTS_2024 = 30_000_000;

let state = {
    business_type: 'sole_prop',
    current_method: 'cash',
    proposed_method: 'cash',
    annual_revenue: 0,
    average_inventory: 0,
    accounts_receivable_year_end: 0,
    accounts_payable_year_end: 0,
    is_corporation: false,
    has_inventory: false,
    is_farming: false,
    is_partnership_corporate_partner: false,
    avg_gross_receipts_3yr: 0,
    marginal_rate: 0.32,
};

export async function renderSection446(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s446.h1.title">// § 446 ACCOUNTING METHODS</span></h1>
        <p class="muted small" data-i18n="view.s446.hint.intro">
            Two primary methods: <strong>Cash + Accrual</strong>. Hybrid permitted.
            <strong>Cash method generally available if avg gross receipts ≤ $30M (2024)</strong>
            and not "tax shelter". <strong>C-corps + partnerships with C-corp partner:</strong>
            generally must use accrual if &gt; $30M. <strong>Inventory:</strong> § 471 generally
            requires accrual for inventory; small biz exception allows cash basis (§ 471(c)).
            Method change via Form 3115 + § 481(a) adjustment.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s446.h2.inputs">Inputs</h2>
            <form id="s446-form" class="inline-form">
                <label><span data-i18n="view.s446.label.business_type">Business type</span>
                    <select name="business_type">
                        <option value="sole_prop" ${state.business_type === 'sole_prop' ? 'selected' : ''}>Sole prop / SMLLC</option>
                        <option value="s_corp" ${state.business_type === 's_corp' ? 'selected' : ''}>S-corp</option>
                        <option value="c_corp" ${state.business_type === 'c_corp' ? 'selected' : ''}>C-corp</option>
                        <option value="partnership" ${state.business_type === 'partnership' ? 'selected' : ''}>Partnership / LLC-multi</option>
                    </select>
                </label>
                <label><span data-i18n="view.s446.label.current">Current method</span>
                    <select name="current_method">
                        <option value="cash" ${state.current_method === 'cash' ? 'selected' : ''}>Cash</option>
                        <option value="accrual" ${state.current_method === 'accrual' ? 'selected' : ''}>Accrual</option>
                        <option value="hybrid" ${state.current_method === 'hybrid' ? 'selected' : ''}>Hybrid</option>
                    </select>
                </label>
                <label><span data-i18n="view.s446.label.proposed">Proposed method</span>
                    <select name="proposed_method">
                        <option value="cash" ${state.proposed_method === 'cash' ? 'selected' : ''}>Cash</option>
                        <option value="accrual" ${state.proposed_method === 'accrual' ? 'selected' : ''}>Accrual</option>
                        <option value="hybrid" ${state.proposed_method === 'hybrid' ? 'selected' : ''}>Hybrid</option>
                    </select>
                </label>
                <label><span data-i18n="view.s446.label.revenue">Annual revenue ($)</span>
                    <input type="number" step="0.01" name="annual_revenue" value="${state.annual_revenue}"></label>
                <label><span data-i18n="view.s446.label.inventory">Average inventory ($)</span>
                    <input type="number" step="0.01" name="average_inventory" value="${state.average_inventory}"></label>
                <label><span data-i18n="view.s446.label.ar">A/R year-end ($)</span>
                    <input type="number" step="0.01" name="accounts_receivable_year_end" value="${state.accounts_receivable_year_end}"></label>
                <label><span data-i18n="view.s446.label.ap">A/P year-end ($)</span>
                    <input type="number" step="0.01" name="accounts_payable_year_end" value="${state.accounts_payable_year_end}"></label>
                <label><span data-i18n="view.s446.label.has_inventory">Has inventory?</span>
                    <input type="checkbox" name="has_inventory" ${state.has_inventory ? 'checked' : ''}></label>
                <label><span data-i18n="view.s446.label.farming">Farming business?</span>
                    <input type="checkbox" name="is_farming" ${state.is_farming ? 'checked' : ''}></label>
                <label><span data-i18n="view.s446.label.corp_partner">Partnership with C-corp partner?</span>
                    <input type="checkbox" name="is_partnership_corporate_partner" ${state.is_partnership_corporate_partner ? 'checked' : ''}></label>
                <label><span data-i18n="view.s446.label.gross_3yr">Avg gross receipts 3-yr ($)</span>
                    <input type="number" step="0.01" name="avg_gross_receipts_3yr" value="${state.avg_gross_receipts_3yr}"></label>
                <label><span data-i18n="view.s446.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s446.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s446-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s446.h2.cash_method">Cash method characteristics</h2>
            <ul class="muted small">
                <li data-i18n="view.s446.cash.income">Income reported when CASH received</li>
                <li data-i18n="view.s446.cash.deductions">Deductions when CASH paid</li>
                <li data-i18n="view.s446.cash.simple">Simpler — matches bank account flow</li>
                <li data-i18n="view.s446.cash.constructive">Constructive receipt: available without restriction = received (even if not deposited)</li>
                <li data-i18n="view.s446.cash.year_end_planning">Year-end planning: defer income / accelerate expenses</li>
                <li data-i18n="view.s446.cash.cap_assets">Capital purchases still depreciated, not expensed</li>
                <li data-i18n="view.s446.cash.prepaid">Prepaid expenses can NOT be deducted until benefit consumed (12-month rule limited exception)</li>
                <li data-i18n="view.s446.cash.eligibility">Eligible: sole prop, partnership, S-corp; C-corp + partnership-with-C-corp if ≤ $30M</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s446.h2.accrual_method">Accrual method characteristics</h2>
            <ul class="muted small">
                <li data-i18n="view.s446.acc.income">Income when EARNED (all-events test + economic performance)</li>
                <li data-i18n="view.s446.acc.deductions">Deductions when LIABILITY fixed + amount determinable + economic performance</li>
                <li data-i18n="view.s446.acc.matches">Matches revenue with expenses</li>
                <li data-i18n="view.s446.acc.gaap">Aligns with GAAP financial reporting</li>
                <li data-i18n="view.s446.acc.complex">More complex — A/R, A/P, accruals tracked</li>
                <li data-i18n="view.s446.acc.book_tax">May reduce book / tax differences</li>
                <li data-i18n="view.s446.acc.required">Required when § 471 inventory + &gt; $30M gross receipts</li>
                <li data-i18n="view.s446.acc.tax_shelter">Tax shelter must use accrual regardless of size</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s446.h2.hybrid">Hybrid method</h2>
            <ul class="muted small">
                <li data-i18n="view.s446.hybrid.combine">Combine cash + accrual for different parts</li>
                <li data-i18n="view.s446.hybrid.common">Common: accrual for inventory + cash for everything else</li>
                <li data-i18n="view.s446.hybrid.consistency">Must use consistently year-to-year</li>
                <li data-i18n="view.s446.hybrid.clearly_reflects">IRS challenges if doesn't clearly reflect income</li>
                <li data-i18n="view.s446.hybrid.bookkeeping">Bookkeeping must support both methods</li>
                <li data-i18n="view.s446.hybrid.farming">Farming + ranching often hybrid (crop insurance + livestock)</li>
            </ul>
        </div>
    `;
    document.getElementById('s446-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.business_type = fd.get('business_type');
        state.current_method = fd.get('current_method');
        state.proposed_method = fd.get('proposed_method');
        state.annual_revenue = Number(fd.get('annual_revenue')) || 0;
        state.average_inventory = Number(fd.get('average_inventory')) || 0;
        state.accounts_receivable_year_end = Number(fd.get('accounts_receivable_year_end')) || 0;
        state.accounts_payable_year_end = Number(fd.get('accounts_payable_year_end')) || 0;
        state.has_inventory = !!fd.get('has_inventory');
        state.is_farming = !!fd.get('is_farming');
        state.is_partnership_corporate_partner = !!fd.get('is_partnership_corporate_partner');
        state.avg_gross_receipts_3yr = Number(fd.get('avg_gross_receipts_3yr')) || 0;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s446-output');
    if (!el) return;
    const smallBizEligible = state.avg_gross_receipts_3yr <= SMALL_BIZ_GROSS_RECEIPTS_2024;
    const mustUseAccrual = (state.business_type === 'c_corp' || state.is_partnership_corporate_partner) && !smallBizEligible;
    const proposedAllowed = state.proposed_method !== 'cash' || !mustUseAccrual;
    const cashToAccrualAdj = state.accounts_receivable_year_end - state.accounts_payable_year_end + state.average_inventory;
    const accrualToCashAdj = -cashToAccrualAdj;
    let s481Adjustment = 0;
    if (state.current_method === 'cash' && state.proposed_method === 'accrual') {
        s481Adjustment = cashToAccrualAdj;
    } else if (state.current_method === 'accrual' && state.proposed_method === 'cash') {
        s481Adjustment = accrualToCashAdj;
    }
    const taxImpact = Math.abs(s481Adjustment) * state.marginal_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s446.h2.result">Method analysis</h2>
            <div class="cards">
                <div class="card ${smallBizEligible ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s446.card.small_biz">Small biz eligible (≤ $30M)</div>
                    <div class="value">${smallBizEligible ? esc(t('view.s446.status.yes')) : esc(t('view.s446.status.no'))}</div>
                </div>
                <div class="card ${mustUseAccrual ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s446.card.must_accrual">Must use accrual?</div>
                    <div class="value">${mustUseAccrual ? esc(t('view.s446.status.yes')) : esc(t('view.s446.status.no'))}</div>
                </div>
                <div class="card ${proposedAllowed ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s446.card.proposed_allowed">Proposed method allowed?</div>
                    <div class="value">${proposedAllowed ? esc(t('view.s446.status.yes')) : esc(t('view.s446.status.no'))}</div>
                </div>
                ${s481Adjustment !== 0 ? `
                    <div class="card ${s481Adjustment > 0 ? 'neg' : 'pos'}">
                        <div class="label" data-i18n="view.s446.card.481_adj">§ 481(a) adjustment</div>
                        <div class="value">${s481Adjustment > 0 ? '+' : ''}$${s481Adjustment.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                    <div class="card">
                        <div class="label" data-i18n="view.s446.card.tax_impact">Total tax impact (4-yr spread)</div>
                        <div class="value">$${taxImpact.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
            </div>
        </div>
    `;
}
