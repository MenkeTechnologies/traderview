// IRC § 444 — Election of Tax Year Other Than Required Year.
// Partnerships, S-corps, Personal Service Corps (PSC) generally must use CALENDAR year.
// § 444 election allows up to 3-month deferral (e.g., FY ending Sep 30).
// "Cost" of deferral: § 7519 required payment (treated as deposit, returns to taxpayer at termination).
// PSCs: alternative minimum salary requirement under § 280H instead of required payment.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    entity_type: 'partnership',
    desired_year_end_month: 9,
    months_deferred: 3,
    deferral_period_taxable_income: 0,
    highest_rate_partner: 37,
    is_psc: false,
    psc_min_salary_paid: 0,
    psc_owner_compensation_pct: 100,
    s7519_required_payment_balance: 0,
    short_period_first_yr: false,
    election_year: 2024,
    automatic_change_allowed: true,
    natural_business_year_25pct: false,
};

export async function renderSection444(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s444.h1.title">// § 444 FISCAL YEAR ELECTION</span></h1>
        <p class="muted small" data-i18n="view.s444.hint.intro">
            Partnerships, S-corps, and Personal Service Corps (PSCs) <strong>generally must use CALENDAR
            year</strong> (§ 441). <strong>§ 444 election:</strong> up to <strong>3-month deferral</strong>
            (e.g., FY ending Sep 30, Oct 31, Nov 30). <strong>Cost:</strong> § 7519 required payment
            (refundable deposit at termination). <strong>PSCs:</strong> alternative § 280H min salary
            requirement INSTEAD of § 7519. <strong>Natural business year</strong> 25%-test alternative:
            no § 7519 required if 25%+ of gross receipts in last 2 months. <strong>Form 8716</strong>.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s444.h2.inputs">Inputs</h2>
            <form id="s444-form" class="inline-form">
                <label><span data-i18n="view.s444.label.entity">Entity type</span>
                    <select name="entity_type">
                        <option value="partnership" ${state.entity_type === 'partnership' ? 'selected' : ''}>Partnership</option>
                        <option value="s_corp" ${state.entity_type === 's_corp' ? 'selected' : ''}>S-Corp</option>
                        <option value="psc" ${state.entity_type === 'psc' ? 'selected' : ''}>Personal Service Corp (PSC)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s444.label.month">Desired year-end month (1-12)</span>
                    <input type="number" step="1" name="desired_year_end_month" value="${state.desired_year_end_month}"></label>
                <label><span data-i18n="view.s444.label.deferred">Months deferred from required yr</span>
                    <input type="number" step="1" name="months_deferred" value="${state.months_deferred}"></label>
                <label><span data-i18n="view.s444.label.income">Deferral period taxable income ($)</span>
                    <input type="number" step="10000" name="deferral_period_taxable_income" value="${state.deferral_period_taxable_income}"></label>
                <label><span data-i18n="view.s444.label.rate">Highest rate partner / shareholder %</span>
                    <input type="number" step="0.1" name="highest_rate_partner" value="${state.highest_rate_partner}"></label>
                <label><span data-i18n="view.s444.label.psc">Is PSC (§ 280H instead of § 7519)?</span>
                    <input type="checkbox" name="is_psc" ${state.is_psc ? 'checked' : ''}></label>
                <label><span data-i18n="view.s444.label.psc_min">PSC min salary paid in deferral ($)</span>
                    <input type="number" step="10000" name="psc_min_salary_paid" value="${state.psc_min_salary_paid}"></label>
                <label><span data-i18n="view.s444.label.psc_comp">PSC owner comp pct of services %</span>
                    <input type="number" step="0.1" name="psc_owner_compensation_pct" value="${state.psc_owner_compensation_pct}"></label>
                <label><span data-i18n="view.s444.label.balance">§ 7519 prior required payment balance ($)</span>
                    <input type="number" step="1000" name="s7519_required_payment_balance" value="${state.s7519_required_payment_balance}"></label>
                <label><span data-i18n="view.s444.label.short">First year (short period)?</span>
                    <input type="checkbox" name="short_period_first_yr" ${state.short_period_first_yr ? 'checked' : ''}></label>
                <label><span data-i18n="view.s444.label.year">Election year</span>
                    <input type="number" step="1" name="election_year" value="${state.election_year}"></label>
                <label><span data-i18n="view.s444.label.auto">Automatic change allowed?</span>
                    <input type="checkbox" name="automatic_change_allowed" ${state.automatic_change_allowed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s444.label.natural_25">25%+ revenue in last 2 months?</span>
                    <input type="checkbox" name="natural_business_year_25pct" ${state.natural_business_year_25pct ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s444.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s444-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s444.h2.required_year">Required year per entity type</h2>
            <ul class="muted small">
                <li data-i18n="view.s444.req.partnership">Partnership: tax year that results in least deferral to partners (usually calendar)</li>
                <li data-i18n="view.s444.req.s_corp">S-corp: calendar year (§ 1378)</li>
                <li data-i18n="view.s444.req.psc">PSC: calendar year (§ 441(i))</li>
                <li data-i18n="view.s444.req.c_corp">C-corp: any year — usually 12 months</li>
                <li data-i18n="view.s444.req.trust">Trust: calendar year (§ 644)</li>
                <li data-i18n="view.s444.req.estate">Estate: any year — typically fiscal in 1st year</li>
                <li data-i18n="view.s444.req.individual">Individual: calendar year (§ 441)</li>
                <li data-i18n="view.s444.req.short_period">Initial / final year: short period return</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s444.h2.deferral">§ 444 deferral options</h2>
            <ul class="muted small">
                <li data-i18n="view.s444.def.3_months">Maximum 3-month deferral allowed (Sep 30, Oct 31, Nov 30 for calendar required)</li>
                <li data-i18n="view.s444.def.alternative_2">Alternative: shorter than 3 months OK (Oct 31, Nov 30)</li>
                <li data-i18n="view.s444.def.partnership">Partnership: must elect by 5/15 of year of election (Form 8716 by 5/15)</li>
                <li data-i18n="view.s444.def.s_corp">S-corp: within 30 days after Form 2553 election effective date</li>
                <li data-i18n="view.s444.def.psc">PSC: within 30 days after incorporation</li>
                <li data-i18n="view.s444.def.short_period_initial">Initial short period: from formation date to elected year-end</li>
                <li data-i18n="view.s444.def.natural_business">Natural Business Year (NBY) exception: no § 7519 required if 25%+ in last 2 months</li>
                <li data-i18n="view.s444.def.revocation">Revocation: Form 8716 to revert; effective next year</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s444.h2.s7519">§ 7519 required payment</h2>
            <ul class="muted small">
                <li data-i18n="view.s444.s7519.formula">Required payment = deferral period income × highest rate × (deferred months / 12)</li>
                <li data-i18n="view.s444.s7519.due">Due May 15 each year (Form 8752)</li>
                <li data-i18n="view.s444.s7519.deposit">Treated as DEPOSIT — refundable when entity terminates / changes year</li>
                <li data-i18n="view.s444.s7519.no_interest">NO interest paid on deposit (lost time value)</li>
                <li data-i18n="view.s444.s7519.recovery">Refund: filed within 2 yrs of termination event</li>
                <li data-i18n="view.s444.s7519.cumulative">Cumulative across years — adjusted based on net deferral each year</li>
                <li data-i18n="view.s444.s7519.exception_psc">NOT applicable to PSCs (use § 280H instead)</li>
                <li data-i18n="view.s444.s7519.example">Example: $1M deferral income, 37% rate, 3-month deferral = $1M × 37% × 25% = $92,500 required payment</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s444.h2.s280h">§ 280H PSC alternative</h2>
            <ul class="muted small">
                <li data-i18n="view.s444.s280h.purpose">PSC instead of § 7519: minimum salary requirement to owner-employees</li>
                <li data-i18n="view.s444.s280h.formula">Owner salary during deferral months × 365/365 × deferral pct ≥ baseline</li>
                <li data-i18n="view.s444.s280h.fail">Failure: deduction limitation on excess of bonus paid during deferral period</li>
                <li data-i18n="view.s444.s280h.policy">Policy goal: ensure PSC owners receive consistent salary throughout fiscal year</li>
                <li data-i18n="view.s444.s280h.timing">Test applied at year-end based on annual salary pattern</li>
                <li data-i18n="view.s444.s280h.distribution_alternative">Cannot satisfy by distribution alone — must be wages</li>
                <li data-i18n="view.s444.s280h.cure">Pay year-end bonus to cure shortfall</li>
                <li data-i18n="view.s444.s280h.cumulative">Annual test — no cumulative carryover</li>
            </ul>
        </div>
    `;
    document.getElementById('s444-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.entity_type = fd.get('entity_type');
        state.desired_year_end_month = Number(fd.get('desired_year_end_month')) || 0;
        state.months_deferred = Number(fd.get('months_deferred')) || 0;
        state.deferral_period_taxable_income = Number(fd.get('deferral_period_taxable_income')) || 0;
        state.highest_rate_partner = Number(fd.get('highest_rate_partner')) || 0;
        state.is_psc = !!fd.get('is_psc');
        state.psc_min_salary_paid = Number(fd.get('psc_min_salary_paid')) || 0;
        state.psc_owner_compensation_pct = Number(fd.get('psc_owner_compensation_pct')) || 0;
        state.s7519_required_payment_balance = Number(fd.get('s7519_required_payment_balance')) || 0;
        state.short_period_first_yr = !!fd.get('short_period_first_yr');
        state.election_year = Number(fd.get('election_year')) || 0;
        state.automatic_change_allowed = !!fd.get('automatic_change_allowed');
        state.natural_business_year_25pct = !!fd.get('natural_business_year_25pct');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s444-output');
    if (!el) return;
    const exempt_nby = state.natural_business_year_25pct;
    const s7519Required = !state.is_psc && state.months_deferred > 0 && !exempt_nby;
    const requiredPayment = s7519Required ?
        (state.deferral_period_taxable_income * (state.highest_rate_partner / 100) * (state.months_deferred / 12))
        : 0;
    const s280HRequired = state.is_psc;
    const valid_election = state.months_deferred <= 3;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s444.h2.result">§ 444 outcome</h2>
            <div class="cards">
                <div class="card ${valid_election ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s444.card.valid">Valid election?</div>
                    <div class="value">${valid_election ? esc(t('view.s444.status.yes')) : esc(t('view.s444.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s444.card.deferred">Months deferred</div>
                    <div class="value">${state.months_deferred}</div>
                </div>
                <div class="card ${exempt_nby ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s444.card.nby">NBY exception?</div>
                    <div class="value">${exempt_nby ? esc(t('view.s444.status.yes')) : esc(t('view.s444.status.no'))}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s444.card.s7519">§ 7519 required payment</div>
                    <div class="value">$${requiredPayment.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${s280HRequired ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s444.card.s280h">§ 280H requirement (PSC)?</div>
                    <div class="value">${s280HRequired ? esc(t('view.s444.status.yes')) : esc(t('view.s444.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s444.card.balance">Current § 7519 balance</div>
                    <div class="value">$${state.s7519_required_payment_balance.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${exempt_nby ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s444.nby_note">
                    Natural Business Year (NBY) exception: 25%+ of gross receipts in last 2 months of
                    fiscal year permits SHORT FISCAL YEAR without § 7519 required payment. Common for
                    seasonal businesses (Q4 retail, December construction). No deposit required = no time
                    value lost.
                </p>
            ` : ''}
        </div>
    `;
}
