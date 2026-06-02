// IRC § 199A — Qualified Business Income (QBI) Deduction (TCJA, sunsets 2025 absent extension).
// Up to 20% deduction on qualified business income from pass-through entities.
// Wages + UBIA limit for higher-income taxpayers ($241K single / $483K joint 2024).
// SSTB (specified service trade or business) phase-out for high-income (lawyers / doctors / consultants).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    filing_status: 'mfj',
    qbi: 0,
    w2_wages_paid: 0,
    ubia_qualified_property: 0,
    taxable_income_before_qbi: 0,
    is_sstb: false,
    is_aggregated: false,
    aggregation_count: 0,
    has_reit_dividends: false,
    qualified_reit_dividends: 0,
    has_ptp_income: false,
    qualified_ptp_income: 0,
    rental_real_estate: false,
    safe_harbor_250hr: false,
    is_specified_service: false,
    s199a_g_coop: false,
    domestic_only: true,
    is_us_trade_business: true,
};

export async function renderSection199A(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s199a.h1.title">// § 199A QBI DEDUCTION</span></h1>
        <p class="muted small" data-i18n="view.s199a.hint.intro">
            <strong>Up to 20% deduction</strong> on qualified business income from pass-through entities.
            <strong>2024 thresholds:</strong> $241,950 single / $483,900 MFJ — full phase-in $75K/$50K
            above. <strong>SSTB</strong> (specified service trade or business — health, law, accounting,
            actuarial, performing arts, consulting, athletics, financial services, brokerage, investing,
            principal asset of which is reputation/skill): COMPLETELY phased out above upper threshold.
            <strong>Non-SSTB wages + UBIA limit:</strong> greater of 50% W-2 wages OR 25% wages + 2.5%
            UBIA (Unadjusted Basis Immediately After Acquisition). <strong>Domestic only.</strong>
            <strong>Sunsets Dec 31, 2025</strong> absent TCJA extension. Aggregation election under
            § 199A(b)(1)(B).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s199a.h2.inputs">Inputs</h2>
            <form id="s199a-form" class="inline-form">
                <label><span data-i18n="view.s199a.label.status">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filing_status === 'single' ? 'selected' : ''}>Single</option>
                        <option value="mfj" ${state.filing_status === 'mfj' ? 'selected' : ''}>MFJ</option>
                        <option value="hoh" ${state.filing_status === 'hoh' ? 'selected' : ''}>HOH</option>
                        <option value="mfs" ${state.filing_status === 'mfs' ? 'selected' : ''}>MFS</option>
                    </select>
                </label>
                <label><span data-i18n="view.s199a.label.qbi">QBI ($)</span>
                    <input type="number" step="10000" name="qbi" value="${state.qbi}"></label>
                <label><span data-i18n="view.s199a.label.wages">W-2 wages paid ($)</span>
                    <input type="number" step="10000" name="w2_wages_paid" value="${state.w2_wages_paid}"></label>
                <label><span data-i18n="view.s199a.label.ubia">UBIA qualified property ($)</span>
                    <input type="number" step="10000" name="ubia_qualified_property" value="${state.ubia_qualified_property}"></label>
                <label><span data-i18n="view.s199a.label.ti">Taxable income (before QBI) ($)</span>
                    <input type="number" step="10000" name="taxable_income_before_qbi" value="${state.taxable_income_before_qbi}"></label>
                <label><span data-i18n="view.s199a.label.sstb">Is SSTB?</span>
                    <input type="checkbox" name="is_sstb" ${state.is_sstb ? 'checked' : ''}></label>
                <label><span data-i18n="view.s199a.label.aggregated">Aggregated under § 199A(b)?</span>
                    <input type="checkbox" name="is_aggregated" ${state.is_aggregated ? 'checked' : ''}></label>
                <label><span data-i18n="view.s199a.label.agg_count">Number of aggregated trades</span>
                    <input type="number" step="1" name="aggregation_count" value="${state.aggregation_count}"></label>
                <label><span data-i18n="view.s199a.label.reit_div">REIT dividends?</span>
                    <input type="checkbox" name="has_reit_dividends" ${state.has_reit_dividends ? 'checked' : ''}></label>
                <label><span data-i18n="view.s199a.label.reit_amt">REIT dividends ($)</span>
                    <input type="number" step="1000" name="qualified_reit_dividends" value="${state.qualified_reit_dividends}"></label>
                <label><span data-i18n="view.s199a.label.ptp_inc">PTP income?</span>
                    <input type="checkbox" name="has_ptp_income" ${state.has_ptp_income ? 'checked' : ''}></label>
                <label><span data-i18n="view.s199a.label.ptp_amt">PTP qualified income ($)</span>
                    <input type="number" step="1000" name="qualified_ptp_income" value="${state.qualified_ptp_income}"></label>
                <label><span data-i18n="view.s199a.label.rental">Rental real estate?</span>
                    <input type="checkbox" name="rental_real_estate" ${state.rental_real_estate ? 'checked' : ''}></label>
                <label><span data-i18n="view.s199a.label.250hr">250-hr safe harbor (Rev Proc 2019-38)?</span>
                    <input type="checkbox" name="safe_harbor_250hr" ${state.safe_harbor_250hr ? 'checked' : ''}></label>
                <label><span data-i18n="view.s199a.label.specified">Specified service?</span>
                    <input type="checkbox" name="is_specified_service" ${state.is_specified_service ? 'checked' : ''}></label>
                <label><span data-i18n="view.s199a.label.coop">§ 199A(g) coop?</span>
                    <input type="checkbox" name="s199a_g_coop" ${state.s199a_g_coop ? 'checked' : ''}></label>
                <label><span data-i18n="view.s199a.label.domestic">Domestic only?</span>
                    <input type="checkbox" name="domestic_only" ${state.domestic_only ? 'checked' : ''}></label>
                <label><span data-i18n="view.s199a.label.us_tb">US trade or business?</span>
                    <input type="checkbox" name="is_us_trade_business" ${state.is_us_trade_business ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s199a.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s199a-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s199a.h2.thresholds">2024 income thresholds</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s199a.tbl.status">Filing status</th><th data-i18n="view.s199a.tbl.lower">Lower threshold (no W2/UBIA limit + SSTB OK)</th><th data-i18n="view.s199a.tbl.upper">Upper threshold (full SSTB exclusion, full W2/UBIA limit)</th><th data-i18n="view.s199a.tbl.phaseout">Phase-out range</th></tr></thead>
                <tbody>
                    <tr><td>Single / HOH / MFS</td><td>$241,950</td><td>$291,950</td><td>$50,000</td></tr>
                    <tr><td>MFJ</td><td>$483,900</td><td>$583,900</td><td>$100,000</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s199a.h2.sstb">SSTB list (Reg § 1.199A-5(b)(2))</h2>
            <ul class="muted small">
                <li data-i18n="view.s199a.sstb.health">Health (doctors, dentists, nurses, vets, pharmacists)</li>
                <li data-i18n="view.s199a.sstb.law">Law (lawyers, paralegals, mediators)</li>
                <li data-i18n="view.s199a.sstb.accounting">Accounting (CPAs, tax preparers, bookkeepers)</li>
                <li data-i18n="view.s199a.sstb.actuarial">Actuarial science</li>
                <li data-i18n="view.s199a.sstb.performing">Performing arts (actors, musicians, directors)</li>
                <li data-i18n="view.s199a.sstb.consulting">Consulting (NOT routine business mgmt)</li>
                <li data-i18n="view.s199a.sstb.athletics">Athletics (athletes, coaches, sports broadcasters)</li>
                <li data-i18n="view.s199a.sstb.financial">Financial services (advisors, planners, agents)</li>
                <li data-i18n="view.s199a.sstb.brokerage">Brokerage services (NOT real estate)</li>
                <li data-i18n="view.s199a.sstb.investing">Investing / investment management</li>
                <li data-i18n="view.s199a.sstb.trading">Trading / dealing in securities, partnership interests, commodities</li>
                <li data-i18n="view.s199a.sstb.reputation">Principal asset of which is reputation or skill</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s199a.h2.formula">Deduction formula</h2>
            <ol class="muted small">
                <li data-i18n="view.s199a.formula.combined">Combined QBI amount = sum of (20% QBI per trade) limited per § 199A(b)(2)</li>
                <li data-i18n="view.s199a.formula.w2_ubia">W2/UBIA limit: greater of (50% × W-2) OR (25% × W-2 + 2.5% × UBIA)</li>
                <li data-i18n="view.s199a.formula.below">Below lower threshold: no W2/UBIA limit, SSTB allowed</li>
                <li data-i18n="view.s199a.formula.phaseout">In phase-in range: partial W2/UBIA limit + partial SSTB allowed</li>
                <li data-i18n="view.s199a.formula.above">Above upper threshold: full W2/UBIA limit + SSTB excluded</li>
                <li data-i18n="view.s199a.formula.reit_ptp">REIT dividends + PTP income: separate 20% category — no W2/UBIA limit</li>
                <li data-i18n="view.s199a.formula.cap">Overall cap: 20% × (taxable income - net capital gain)</li>
                <li data-i18n="view.s199a.formula.aggregation">Aggregation election: aggregate W2/UBIA across same-owner same-class businesses</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s199a.h2.qbi_inclusions">QBI inclusions + exclusions</h2>
            <ul class="muted small">
                <li data-i18n="view.s199a.qbi.included">Included: net income from US trade or business via pass-through (sole prop, partnership, S-corp)</li>
                <li data-i18n="view.s199a.qbi.excluded_wages">EXCLUDED: W-2 wages (employee comp)</li>
                <li data-i18n="view.s199a.qbi.excluded_guaranteed">EXCLUDED: § 707(c) guaranteed payments to partner</li>
                <li data-i18n="view.s199a.qbi.excluded_reasonable_comp">EXCLUDED: S-corp reasonable compensation to shareholder</li>
                <li data-i18n="view.s199a.qbi.excluded_capital">EXCLUDED: capital gain/loss + dividend income + interest income (most)</li>
                <li data-i18n="view.s199a.qbi.excluded_foreign">EXCLUDED: non-US income</li>
                <li data-i18n="view.s199a.qbi.excluded_qualified_dividends">EXCLUDED: qualified dividends</li>
                <li data-i18n="view.s199a.qbi.included_se">Included: self-employment income (Schedule C / partnership K-1)</li>
                <li data-i18n="view.s199a.qbi.included_rental">Included: rental real estate IF rises to "trade or business" level (250hr safe harbor)</li>
            </ul>
        </div>
    `;
    document.getElementById('s199a-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.filing_status = fd.get('filing_status');
        state.qbi = Number(fd.get('qbi')) || 0;
        state.w2_wages_paid = Number(fd.get('w2_wages_paid')) || 0;
        state.ubia_qualified_property = Number(fd.get('ubia_qualified_property')) || 0;
        state.taxable_income_before_qbi = Number(fd.get('taxable_income_before_qbi')) || 0;
        state.is_sstb = !!fd.get('is_sstb');
        state.is_aggregated = !!fd.get('is_aggregated');
        state.aggregation_count = Number(fd.get('aggregation_count')) || 0;
        state.has_reit_dividends = !!fd.get('has_reit_dividends');
        state.qualified_reit_dividends = Number(fd.get('qualified_reit_dividends')) || 0;
        state.has_ptp_income = !!fd.get('has_ptp_income');
        state.qualified_ptp_income = Number(fd.get('qualified_ptp_income')) || 0;
        state.rental_real_estate = !!fd.get('rental_real_estate');
        state.safe_harbor_250hr = !!fd.get('safe_harbor_250hr');
        state.is_specified_service = !!fd.get('is_specified_service');
        state.s199a_g_coop = !!fd.get('s199a_g_coop');
        state.domestic_only = !!fd.get('domestic_only');
        state.is_us_trade_business = !!fd.get('is_us_trade_business');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s199a-output');
    if (!el) return;
    const single = state.filing_status === 'single' || state.filing_status === 'hoh' || state.filing_status === 'mfs';
    const lower = single ? 241_950 : 483_900;
    const upper = single ? 291_950 : 583_900;
    const ti = state.taxable_income_before_qbi;
    let allowed_qbi = state.qbi * 0.20;
    let w2_ubia_limit = Math.max(state.w2_wages_paid * 0.5, state.w2_wages_paid * 0.25 + state.ubia_qualified_property * 0.025);
    if (ti > upper) {
        if (state.is_sstb) allowed_qbi = 0;
        else allowed_qbi = Math.min(allowed_qbi, w2_ubia_limit);
    } else if (ti > lower) {
        const phase = (ti - lower) / (upper - lower);
        if (state.is_sstb) allowed_qbi = allowed_qbi * (1 - phase);
        const w2_ubia_pct = phase;
        const limit_partial = w2_ubia_limit + (allowed_qbi - w2_ubia_limit) * (1 - w2_ubia_pct);
        allowed_qbi = Math.min(allowed_qbi, limit_partial);
    }
    const reit_ptp_dedn = 0.20 * (state.qualified_reit_dividends + state.qualified_ptp_income);
    const overall_cap = 0.20 * ti;
    const total_dedn = Math.min(allowed_qbi + reit_ptp_dedn, overall_cap);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s199a.h2.result">§ 199A deduction</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s199a.card.qbi">QBI</div><div class="value">$${state.qbi.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s199a.card.qbi_dedn">QBI deduction (20%)</div><div class="value">$${allowed_qbi.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card"><div class="label" data-i18n="view.s199a.card.reit">REIT/PTP deduction</div><div class="value">$${reit_ptp_dedn.toLocaleString()}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s199a.card.total">Total § 199A deduction</div><div class="value">$${total_dedn.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card ${state.is_sstb && ti > upper ? 'neg' : ''}"><div class="label" data-i18n="view.s199a.card.sstb">SSTB above upper?</div><div class="value">${state.is_sstb && ti > upper ? 'YES (excluded)' : 'NO'}</div></div>
            </div>
        </div>
    `;
}
