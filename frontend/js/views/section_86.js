// IRC § 86 — Social Security Income Taxability.
// 0%, 50%, or 85% of SS benefits taxable based on Provisional Income.
// Provisional Income = AGI + tax-exempt interest + 50% of SS benefits.
// Single thresholds: $25k (50%) / $34k (85%); MFJ: $32k / $44k.
// NOT indexed to inflation since 1983/1993. Highly criticized.

import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const SINGLE_FIRST_TIER = 25_000;
const SINGLE_SECOND_TIER = 34_000;
const MFJ_FIRST_TIER = 32_000;
const MFJ_SECOND_TIER = 44_000;

let state = {
    filing_status: 'single',
    ss_benefits: 0,
    other_income_pre_ss: 0,
    tax_exempt_interest: 0,
    above_line_deductions: 0,
    marginal_rate: 0.22,
};

export async function renderSection86(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s86.h1.title">// § 86 SOCIAL SECURITY TAXABILITY</span></h1>
        <p class="muted small" data-i18n="view.s86.hint.intro">
            <strong>0%, 50%, or 85% of SS benefits taxable</strong> based on Provisional Income.
            <strong>Provisional Income</strong> = AGI + tax-exempt interest + 50% of SS benefits.
            Thresholds: <strong>$25k / $34k (single)</strong>, <strong>$32k / $44k (MFJ)</strong>.
            <strong>NOT indexed to inflation since 1983/1993</strong> — highly criticized.
            <strong>Tier 1 RRSI</strong> treated identically; Tier 2 Railroad retirement taxed
            differently (more like pension).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s86.h2.inputs">Inputs</h2>
            <form id="s86-form" class="inline-form">
                <label><span data-i18n="view.s86.label.filing">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filing_status === 'single' ? 'selected' : ''}>Single / HoH / QW</option>
                        <option value="mfj" ${state.filing_status === 'mfj' ? 'selected' : ''}>MFJ</option>
                        <option value="mfs" ${state.filing_status === 'mfs' ? 'selected' : ''}>MFS</option>
                    </select>
                </label>
                <label><span data-i18n="view.s86.label.ss">SS benefits received ($)</span>
                    <input type="number" step="100" name="ss_benefits" value="${state.ss_benefits}"></label>
                <label><span data-i18n="view.s86.label.other">Other income (pre-SS taxability) ($)</span>
                    <input type="number" step="100" name="other_income_pre_ss" value="${state.other_income_pre_ss}"></label>
                <label><span data-i18n="view.s86.label.muni">Tax-exempt interest (muni bonds) ($)</span>
                    <input type="number" step="100" name="tax_exempt_interest" value="${state.tax_exempt_interest}"></label>
                <label><span data-i18n="view.s86.label.adjustments">Above-line deductions ($)</span>
                    <input type="number" step="100" name="above_line_deductions" value="${state.above_line_deductions}"></label>
                <label><span data-i18n="view.s86.label.marginal">Marginal rate</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s86.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s86-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s86.h2.formula">§ 86 formula</h2>
            <ol class="muted small">
                <li data-i18n="view.s86.f.provisional">Provisional Income = AGI + tax-exempt interest + 50% of SS benefits</li>
                <li data-i18n="view.s86.f.first_tier">First tier: PI &gt; first threshold → 50% of excess (up to 50% of SS) taxable</li>
                <li data-i18n="view.s86.f.second_tier">Second tier: PI &gt; second threshold → 85% of excess + first tier amount; capped at 85% of SS</li>
                <li data-i18n="view.s86.f.maximum">MAXIMUM taxable: 85% of SS benefits</li>
                <li data-i18n="view.s86.f.mfs">MFS living with spouse: $0 thresholds → 85% always taxable</li>
                <li data-i18n="view.s86.f.foreign">Foreign-earned income excluded under § 911 still counts in AGI for § 86</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s86.h2.planning">Planning strategies</h2>
            <ul class="muted small">
                <li data-i18n="view.s86.p.roth_conversion">Roth conversions in low-income years before SS starts</li>
                <li data-i18n="view.s86.p.delay">Delay SS to 70 (DRC 8% each year) — taxable but at higher amount</li>
                <li data-i18n="view.s86.p.qcd">QCD ($105k 2024) reduces RMD inclusion in AGI</li>
                <li data-i18n="view.s86.p.hsa">HSA contributions reduce AGI (if eligible, ≤ 65)</li>
                <li data-i18n="view.s86.p.standard_deduction">Standard deduction doesn't reduce SS taxability (already below the line)</li>
                <li data-i18n="view.s86.p.muni_no_help">Muni bond interest INCLUDED in provisional income — doesn't help § 86</li>
                <li data-i18n="view.s86.p.no_indexing">No inflation indexing: 50 million seniors paying tax they didn't expect (1983 affected 10%, 2024 affects ~55%)</li>
                <li data-i18n="view.s86.p.spend_down_tax_deferred">Spend tax-deferred accounts before SS starts (lower AGI when SS adds in)</li>
                <li data-i18n="view.s86.p.realized_cap_gains">Manage realized capital gains around § 86 thresholds</li>
                <li data-i18n="view.s86.p.couple_strategy">Married couples: file MFJ (much higher thresholds vs MFS $0)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s86.h2.state">State taxability</h2>
            <ul class="muted small">
                <li data-i18n="view.s86.state.no_tax">No state income tax states: WA, TX, FL, NV, AK, SD, WY, TN, NH</li>
                <li data-i18n="view.s86.state.exempt">Specifically exempt SS: AL, AZ, AR, CA, DE, GA, HI, ID, IL, IN, IA (post-2023), KY, LA, ME, MD, MA, MI, MS, NV (no tax), NJ, NY, NC, OH, OK, OR, PA, SC, VA, WI</li>
                <li data-i18n="view.s86.state.partial">Partial tax: CO, CT, KS, MN, MO (phased out), MT, NE (phased out), NM, RI, UT, VT, WV</li>
                <li data-i18n="view.s86.state.full">Full tax: federal only as of 2024 in remaining states</li>
                <li data-i18n="view.s86.state.changes">Recent trend: states phasing out SS taxability (NE + UT 2024)</li>
            </ul>
        </div>
    `;
    document.getElementById('s86-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.filing_status = fd.get('filing_status');
        state.ss_benefits = Number(fd.get('ss_benefits')) || 0;
        state.other_income_pre_ss = Number(fd.get('other_income_pre_ss')) || 0;
        state.tax_exempt_interest = Number(fd.get('tax_exempt_interest')) || 0;
        state.above_line_deductions = Number(fd.get('above_line_deductions')) || 0;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.22;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s86-output');
    if (!el) return;
    const halfSs = state.ss_benefits * 0.50;
    const agiPreSs = state.other_income_pre_ss - state.above_line_deductions;
    const provisional = agiPreSs + state.tax_exempt_interest + halfSs;
    let first, second;
    if (state.filing_status === 'mfj') {
        first = MFJ_FIRST_TIER;
        second = MFJ_SECOND_TIER;
    } else if (state.filing_status === 'mfs') {
        first = 0;
        second = 0;
    } else {
        first = SINGLE_FIRST_TIER;
        second = SINGLE_SECOND_TIER;
    }
    let taxable;
    if (provisional <= first) {
        taxable = 0;
    } else if (provisional <= second) {
        taxable = Math.min((provisional - first) * 0.50, halfSs);
    } else {
        const tier1 = Math.min((second - first) * 0.50, halfSs);
        const tier2 = (provisional - second) * 0.85;
        taxable = Math.min(tier1 + tier2, state.ss_benefits * 0.85);
    }
    const taxOnSs = taxable * state.marginal_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s86.h2.result">Taxability calculation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s86.card.provisional">Provisional income</div>
                    <div class="value">$${provisional.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s86.card.first_threshold">First threshold</div>
                    <div class="value">$${first.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s86.card.second_threshold">Second threshold</div>
                    <div class="value">$${second.toLocaleString()}</div>
                </div>
                <div class="card ${taxable > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s86.card.taxable">SS benefit taxable</div>
                    <div class="value">$${taxable.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s86.card.pct">% of SS taxable</div>
                    <div class="value">${state.ss_benefits > 0 ? (taxable / state.ss_benefits * 100).toFixed(0) : 0}%</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s86.card.tax">Federal tax on SS</div>
                    <div class="value">$${taxOnSs.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
