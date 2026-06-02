// IRC § 481(a) — Accounting Method Change (Form 3115).
// When changing accounting method, § 481(a) adjustment captures the difference between old + new method.
// Positive adjustment (increases income): spread over 4 yrs (or in year of change for de minimis < $50k).
// Negative adjustment (decreases income): fully in year of change.
// Automatic (Rev. Proc. 2024-14) vs non-automatic (advance consent required).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const DE_MINIMIS_2024 = 50_000;

let state = {
    old_method: 'cash',
    new_method: 'accrual',
    change_type: 'automatic',
    section_481_adjustment: 0,
    is_positive_adjustment: true,
    year_of_change: new Date().getFullYear(),
    automatic_dcn: '',
    marginal_rate: 0.32,
    interest_rate: 0.08,
};

export async function renderSection481a(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s481.h1.title">// § 481(a) ACCOUNTING METHOD CHANGE</span></h1>
        <p class="muted small" data-i18n="view.s481.hint.intro">
            <strong>Form 3115</strong>. § 481(a) adjustment captures difference between old + new
            method. <strong>Positive (increases income):</strong> spread over 4 yrs OR year of
            change for de minimis &lt; $50k. <strong>Negative (decreases income):</strong> fully
            in year of change. <strong>Automatic (Rev. Proc. 2024-14):</strong> no IRS consent,
            file with return. <strong>Non-automatic:</strong> advance consent, $11,500 user fee,
            6-month review.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s481.h2.inputs">Inputs</h2>
            <form id="s481-form" class="inline-form">
                <label><span data-i18n="view.s481.label.old_method">Old method</span>
                    <select name="old_method">
                        <option value="cash" ${state.old_method === 'cash' ? 'selected' : ''}>Cash basis</option>
                        <option value="accrual" ${state.old_method === 'accrual' ? 'selected' : ''}>Accrual</option>
                        <option value="hybrid" ${state.old_method === 'hybrid' ? 'selected' : ''}>Hybrid</option>
                        <option value="completed_contract" ${state.old_method === 'completed_contract' ? 'selected' : ''}>Completed contract</option>
                        <option value="installment" ${state.old_method === 'installment' ? 'selected' : ''}>Installment</option>
                    </select>
                </label>
                <label><span data-i18n="view.s481.label.new_method">New method</span>
                    <select name="new_method">
                        <option value="cash" ${state.new_method === 'cash' ? 'selected' : ''}>Cash basis</option>
                        <option value="accrual" ${state.new_method === 'accrual' ? 'selected' : ''}>Accrual</option>
                        <option value="lifo" ${state.new_method === 'lifo' ? 'selected' : ''}>LIFO inventory</option>
                        <option value="fifo" ${state.new_method === 'fifo' ? 'selected' : ''}>FIFO inventory</option>
                        <option value="percentage_completion" ${state.new_method === 'percentage_completion' ? 'selected' : ''}>Percentage of completion</option>
                        <option value="economic_performance" ${state.new_method === 'economic_performance' ? 'selected' : ''}>Economic performance</option>
                    </select>
                </label>
                <label><span data-i18n="view.s481.label.change_type">Change type</span>
                    <select name="change_type">
                        <option value="automatic" ${state.change_type === 'automatic' ? 'selected' : ''}>Automatic (Rev. Proc. 2024-14)</option>
                        <option value="non_automatic" ${state.change_type === 'non_automatic' ? 'selected' : ''}>Non-automatic (advance consent)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s481.label.adjustment">§ 481(a) adjustment amount ($)</span>
                    <input type="number" step="1000" name="section_481_adjustment" value="${state.section_481_adjustment}"></label>
                <label><span data-i18n="view.s481.label.positive">Positive adjustment (increases income)?</span>
                    <input type="checkbox" name="is_positive_adjustment" ${state.is_positive_adjustment ? 'checked' : ''}></label>
                <label><span data-i18n="view.s481.label.year">Year of change</span>
                    <input type="number" step="1" name="year_of_change" value="${state.year_of_change}"></label>
                <label><span data-i18n="view.s481.label.dcn">Designated Change Number (DCN)</span>
                    <input type="text" name="automatic_dcn" value="${state.automatic_dcn}" placeholder="e.g. 233"></label>
                <label><span data-i18n="view.s481.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <label><span data-i18n="view.s481.label.interest">NPV discount rate</span>
                    <input type="number" step="0.01" name="interest_rate" value="${state.interest_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s481.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s481-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s481.h2.common_changes">Common automatic method changes (DCN)</h2>
            <ul class="muted small">
                <li data-i18n="view.s481.cc.dcn7">DCN 7: Cash to accrual</li>
                <li data-i18n="view.s481.cc.dcn33">DCN 33: Small business inventory (§ 471(c))</li>
                <li data-i18n="view.s481.cc.dcn34">DCN 34: § 263A UNICAP adoption / discontinuation</li>
                <li data-i18n="view.s481.cc.dcn123">DCN 123: § 461(h) economic performance recurring item exception</li>
                <li data-i18n="view.s481.cc.dcn190">DCN 190: § 451 advance payments OneYear deferral</li>
                <li data-i18n="view.s481.cc.dcn205">DCN 205: § 168(i)-8 partial disposition election</li>
                <li data-i18n="view.s481.cc.dcn235">DCN 235: De minimis tangible property election (§ 263(a))</li>
                <li data-i18n="view.s481.cc.dcn265">DCN 265: § 174 R&D capitalization (post-TCJA mandate)</li>
                <li data-i18n="view.s481.cc.dcn7_overall">DCN 7 overall method change automatic if &lt; $25M gross receipts</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s481.h2.process">Filing process</h2>
            <ul class="muted small">
                <li data-i18n="view.s481.proc.timing">File Form 3115 with timely-filed return (incl. extensions) for year of change</li>
                <li data-i18n="view.s481.proc.duplicate">Send duplicate copy to IRS National Office Ogden, UT (since 2024)</li>
                <li data-i18n="view.s481.proc.signature">Officer / partner / sole owner signs Form 3115</li>
                <li data-i18n="view.s481.proc.non_automatic">Non-automatic: $11,500 user fee + advance consent (6 mo review)</li>
                <li data-i18n="view.s481.proc.under_examination">Under audit / examination: Form 3115 generally NOT permitted (limited exceptions)</li>
                <li data-i18n="view.s481.proc.notice_2022_36">Notice 2022-36 disaster relief: timely filing extension</li>
                <li data-i18n="view.s481.proc.consistency">Must use new method consistently going forward</li>
            </ul>
        </div>
    `;
    document.getElementById('s481-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.old_method = fd.get('old_method');
        state.new_method = fd.get('new_method');
        state.change_type = fd.get('change_type');
        state.section_481_adjustment = Number(fd.get('section_481_adjustment')) || 0;
        state.is_positive_adjustment = !!fd.get('is_positive_adjustment');
        state.year_of_change = Number(fd.get('year_of_change')) || new Date().getFullYear();
        state.automatic_dcn = fd.get('automatic_dcn') || '';
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        state.interest_rate = Number(fd.get('interest_rate')) || 0.08;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s481-output');
    if (!el) return;
    const isDeMinimis = state.section_481_adjustment < DE_MINIMIS_2024;
    const yearOfChangeAmount = state.is_positive_adjustment
        ? (isDeMinimis ? state.section_481_adjustment : state.section_481_adjustment / 4)
        : state.section_481_adjustment;
    const yearsRemaining = state.is_positive_adjustment && !isDeMinimis ? 3 : 0;
    const remainingAmount = state.is_positive_adjustment && !isDeMinimis ? state.section_481_adjustment * 0.75 : 0;
    const totalTaxImpact = state.section_481_adjustment * state.marginal_rate;
    const yearOfChangeTax = yearOfChangeAmount * state.marginal_rate;
    // NPV with 4-yr spread vs 1-yr lump
    const lumpNPV = totalTaxImpact;
    const spreadNPV = state.section_481_adjustment / 4 * state.marginal_rate * (1 + 1/Math.pow(1+state.interest_rate, 1) + 1/Math.pow(1+state.interest_rate, 2) + 1/Math.pow(1+state.interest_rate, 3));
    const npvSavings = lumpNPV - spreadNPV;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s481.h2.result">§ 481(a) adjustment flow</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s481.card.adjustment">Total adjustment</div>
                    <div class="value">${state.is_positive_adjustment ? '+' : '-'}$${state.section_481_adjustment.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${isDeMinimis ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s481.card.de_minimis">De minimis ($50k)</div>
                    <div class="value">${isDeMinimis ? esc(t('view.s481.status.yes')) : esc(t('view.s481.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s481.card.year_change_amount">Year-of-change amount</div>
                    <div class="value">$${yearOfChangeAmount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s481.card.years_remaining">Years remaining</div>
                    <div class="value">${yearsRemaining}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s481.card.year_tax">Year-of-change tax</div>
                    <div class="value">$${yearOfChangeTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s481.card.total_tax">Total tax impact</div>
                    <div class="value">$${totalTaxImpact.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${state.is_positive_adjustment && !isDeMinimis ? `
                    <div class="card pos">
                        <div class="label" data-i18n="view.s481.card.npv_savings">NPV savings (4-yr spread)</div>
                        <div class="value">$${npvSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
            </div>
        </div>
    `;
}
