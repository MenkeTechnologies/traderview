// Take-home pay — gross paycheck to net, per period and per year, with precise
// FICA and effective income-tax rates, via /calc/take-home-paycheck. Live.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%';

export async function renderTakeHomePaycheck(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.takehome.h1.title">// TAKE-HOME PAY</span></h1>
        <p class="muted small" data-i18n="view.takehome.hint.intro">
            Turn a gross paycheck into net. FICA is exact — Social Security 6.2% up to the wage
            base, Medicare 1.45% on all wages plus 0.9% over the threshold. Federal and state
            withholding depend on your W-4, so enter them as effective rates on taxable wages.
            Pre-tax deductions (401k, HSA) cut the income-tax base but not FICA. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.takehome.h2.inputs">The paycheck</h2>
            <form id="takehome-form" class="inline-form">
                <label><span data-i18n="view.takehome.label.gross">Gross / period ($)</span>
                    <input type="number" step="0.01" min="0" name="gross_per_period" value="5000" required></label>
                <label><span data-i18n="view.takehome.label.periods">Periods / year</span>
                    <select name="periods_per_year">
                        <option value="52" data-i18n="view.takehome.periods.52">Weekly (52)</option>
                        <option value="26" data-i18n="view.takehome.periods.26">Biweekly (26)</option>
                        <option value="24" selected data-i18n="view.takehome.periods.24">Semimonthly (24)</option>
                        <option value="12" data-i18n="view.takehome.periods.12">Monthly (12)</option>
                    </select></label>
                <label><span data-i18n="view.takehome.label.federal">Federal effective rate (%)</span>
                    <input type="number" step="0.1" min="0" name="federal_rate_pct" value="12" required></label>
                <label><span data-i18n="view.takehome.label.state">State effective rate (%)</span>
                    <input type="number" step="0.1" min="0" name="state_rate_pct" value="5" required></label>
                <label><span data-i18n="view.takehome.label.pretax">Pre-tax / period ($)</span>
                    <input type="number" step="0.01" min="0" name="pre_tax_per_period" value="500"></label>
                <label><span data-i18n="view.takehome.label.posttax">Post-tax / period ($)</span>
                    <input type="number" step="0.01" min="0" name="post_tax_per_period" value="0"></label>
                <label><span data-i18n="view.takehome.label.ssbase">SS wage base ($)</span>
                    <input type="number" step="1" min="0" name="ss_wage_base" value="184500"></label>
                <label><span data-i18n="view.takehome.label.addlthresh">Add'l Medicare threshold ($)</span>
                    <input type="number" step="1" min="0" name="addl_medicare_threshold" value="200000"></label>
            </form>
        </div>
        <div id="takehome-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#takehome-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            gross_per_period: Number(fd.get('gross_per_period')) || 0,
            periods_per_year: Number(fd.get('periods_per_year')) || 0,
            federal_rate_pct: Number(fd.get('federal_rate_pct')) || 0,
            state_rate_pct: Number(fd.get('state_rate_pct')) || 0,
            pre_tax_per_period: Number(fd.get('pre_tax_per_period')) || 0,
            post_tax_per_period: Number(fd.get('post_tax_per_period')) || 0,
            ss_wage_base: Number(fd.get('ss_wage_base')) || 0,
            addl_medicare_threshold: Number(fd.get('addl_medicare_threshold')) || 0,
        };
        try {
            const r = await api.calcTakeHomePaycheck(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.takehome.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#takehome-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.takehome.h2.result">Your net</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.takehome.card.net">Take-home / period</div>
                    <div class="value pos">${money(r.take_home_per_period)}</div></div>
                <div class="card"><div class="label" data-i18n="view.takehome.card.netannual">Take-home / year</div>
                    <div class="value">${money(r.take_home_annual)}</div></div>
                <div class="card"><div class="label" data-i18n="view.takehome.card.effrate">Effective tax rate</div>
                    <div class="value">${pct(r.effective_tax_rate_pct)}</div></div>
            </div>
            <table class="data-table">
                <thead><tr><th data-i18n="view.takehome.col.item">Annual</th><th data-i18n="view.takehome.col.amount">Amount</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.takehome.row.gross">Gross</td><td>${money(r.gross_annual)}</td></tr>
                    <tr><td data-i18n="view.takehome.row.pretax">Pre-tax deductions</td><td>${money(r.pre_tax_annual)}</td></tr>
                    <tr><td data-i18n="view.takehome.row.taxable">Taxable wages</td><td>${money(r.taxable_annual)}</td></tr>
                    <tr><td data-i18n="view.takehome.row.federal">Federal income tax</td><td>${money(r.federal_annual)}</td></tr>
                    <tr><td data-i18n="view.takehome.row.state">State income tax</td><td>${money(r.state_annual)}</td></tr>
                    <tr><td data-i18n="view.takehome.row.ss">Social Security</td><td>${money(r.social_security_annual)}</td></tr>
                    <tr><td data-i18n="view.takehome.row.medicare">Medicare</td><td>${money(r.medicare_annual)}</td></tr>
                    <tr><td data-i18n="view.takehome.row.posttax">Post-tax deductions</td><td>${money(r.post_tax_annual)}</td></tr>
                    <tr><td data-i18n="view.takehome.row.totaltax">Total tax</td><td>${money(r.total_tax_annual)}</td></tr>
                    <tr class="emph"><td>${t('view.takehome.row.takehome')} (${pct(r.take_home_pct)})</td><td>${money(r.take_home_annual)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
