// Disabled Access Credit § 44 — small businesses only.
// 50% of eligible expenditures between $250 and $10,250 → max $5,000/yr credit.
// Eligible small biz: gross receipts ≤ $1M prior year, OR ≤ 30 FTE employees.
// Combine with § 190 Barrier Removal deduction ($15k/yr).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const MIN_EXPEND = 250;
const MAX_EXPEND = 10_250;
const CREDIT_RATE = 0.50;
const MAX_CREDIT = 5_000;
const GROSS_RECEIPTS_THRESHOLD = 1_000_000;
const FTE_THRESHOLD = 30;
const BARRIER_REMOVAL_CAP = 15_000;

let state = {
    gross_receipts: 0,
    fte_employees: 0,
    eligible_expenditure: 0,
    barrier_removal_expenditure: 0,
    marginal_rate: 0.32,
};

export async function renderDisabledAccess(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.da.h1.title">// DISABLED ACCESS § 44</span></h1>
        <p class="muted small" data-i18n="view.da.hint.intro">
            <strong>Small business only.</strong> Up to $5,000 credit/year for ADA-compliance expenditures.
            Eligible: gross receipts ≤ $1M prior year OR ≤ 30 FTE employees. Credit = 50% of expenditures
            between $250 and $10,250. Combine with § 190 Barrier Removal deduction ($15,000/yr).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.da.h2.eligibility">Eligibility test</h2>
            <ul class="muted small">
                <li data-i18n="view.da.elig.receipts">Prior year gross receipts ≤ $1 million, <strong>OR</strong></li>
                <li data-i18n="view.da.elig.fte">≤ 30 full-time employees (FTE)</li>
                <li data-i18n="view.da.elig.either">Either test passes = eligible for the credit</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.da.h2.qualifying">Qualifying expenditures</h2>
            <ul class="muted small">
                <li data-i18n="view.da.qual.removal">Removal of architectural / communication barriers (ramps, restrooms, doors)</li>
                <li data-i18n="view.da.qual.interpreter">Sign language interpreters, qualified readers</li>
                <li data-i18n="view.da.qual.printed">Print materials in alternate formats (Braille, audio, large print)</li>
                <li data-i18n="view.da.qual.equipment">Adaptive equipment / modifications (computer access, hearing devices)</li>
                <li data-i18n="view.da.qual.acquisition">Acquisition or modification of equipment / devices for disabled use</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.da.h2.inputs">Inputs</h2>
            <form id="da-form" class="inline-form">
                <label><span data-i18n="view.da.label.gross_receipts">Prior year gross receipts ($)</span>
                    <input type="number" step="0.01" name="gross_receipts" value="${state.gross_receipts}"></label>
                <label><span data-i18n="view.da.label.fte">FTE employees</span>
                    <input type="number" step="1" name="fte_employees" value="${state.fte_employees}"></label>
                <label><span data-i18n="view.da.label.expenditure">Eligible access expenditure ($)</span>
                    <input type="number" step="0.01" name="eligible_expenditure" value="${state.eligible_expenditure}"></label>
                <label><span data-i18n="view.da.label.barrier">§ 190 barrier removal expenditure ($)</span>
                    <input type="number" step="0.01" name="barrier_removal_expenditure" value="${state.barrier_removal_expenditure}"></label>
                <label><span data-i18n="view.da.label.marginal">Marginal tax %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.da.btn.compute">Compute</button>
            </form>
        </div>
        <div id="da-output"></div>
    `;
    document.getElementById('da-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.gross_receipts = Number(fd.get('gross_receipts')) || 0;
        state.fte_employees = Number(fd.get('fte_employees')) || 0;
        state.eligible_expenditure = Number(fd.get('eligible_expenditure')) || 0;
        state.barrier_removal_expenditure = Number(fd.get('barrier_removal_expenditure')) || 0;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('da-output');
    if (!el) return;
    const eligible = state.gross_receipts <= GROSS_RECEIPTS_THRESHOLD || state.fte_employees <= FTE_THRESHOLD;
    const expCapped = Math.min(state.eligible_expenditure, MAX_EXPEND);
    const expAboveMin = Math.max(0, expCapped - MIN_EXPEND);
    const credit = eligible ? Math.min(expAboveMin * CREDIT_RATE, MAX_CREDIT) : 0;
    const barrierDeduction = Math.min(state.barrier_removal_expenditure, BARRIER_REMOVAL_CAP);
    const barrierTaxSavings = barrierDeduction * state.marginal_rate;
    const totalSavings = credit + barrierTaxSavings;
    const stackedExpenses = state.eligible_expenditure + state.barrier_removal_expenditure;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.da.h2.result">Calculation</h2>
            <div class="cards">
                <div class="card ${eligible ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.da.card.eligible">Eligible small business?</div>
                    <div class="value">${eligible ? esc(t('view.da.status.yes')) : esc(t('view.da.status.no'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.da.card.credit">§ 44 credit (50%, capped $5k)</div>
                    <div class="value">$${credit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.da.card.barrier_deduction">§ 190 deduction ($15k cap)</div>
                    <div class="value">$${barrierDeduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.da.card.barrier_savings">§ 190 tax savings</div>
                    <div class="value">$${barrierTaxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.da.card.total_savings">Total tax savings</div>
                    <div class="value">$${totalSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.da.card.total_outlay">Total outlay</div>
                    <div class="value">$${stackedExpenses.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.eligible_expenditure > MAX_EXPEND ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.da.warning.cap">
                    Eligible expenditure exceeds $10,250 cap — excess flows to § 190 deduction or capitalized.
                </p>
            ` : ''}
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.da.h2.stacking">Stacking § 44 + § 190</h2>
            <p class="muted" data-i18n="view.da.stacking.body">
                You CANNOT double-dip — every dollar counts for ONE provision.
                Best practice: claim § 44 credit first (50% pass-through), then apply § 190 to
                the next $15,000 of expenditures. Above that, normal § 263A capitalization.
            </p>
        </div>
    `;
}
