// QBI § 199A 20% Deduction Calculator — pass-through business income deduction.
// Traders with TTS (Trader Tax Status) election get QBI; standard investors don't.
// Phase-out: SSTB exclusion for service businesses begins at taxable income
// $241,950 single / $483,900 MFJ (2024), fully phased out at $291,950 / $583,900.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LIMITS = {
    2024: {
        single_threshold: 241_950,
        single_phaseout_end: 291_950,
        mfj_threshold: 483_900,
        mfj_phaseout_end: 583_900,
    },
    2025: {
        single_threshold: 250_000,
        single_phaseout_end: 300_000,
        mfj_threshold: 500_000,
        mfj_phaseout_end: 600_000,
    },
    2026: {
        single_threshold: 257_000,
        single_phaseout_end: 307_000,
        mfj_threshold: 514_000,
        mfj_phaseout_end: 614_000,
    },
};

let state = {
    year: new Date().getFullYear(),
    filing: 'single',
    qbi_income: 0,
    total_taxable_income: 0,
    w2_wages_paid: 0,
    qualified_property_basis: 0,
    is_sstb: true,  // Trading is a SSTB (Specified Service Trade or Business)
};

export async function renderQbi199A(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.qbi.h1.title">// QBI § 199A CALCULATOR</span></h1>
        <p class="muted small" data-i18n="view.qbi.hint.intro">
            IRC § 199A — 20% pass-through deduction. <strong>Day-traders with TTS election</strong>
            qualify. Trading is a "specified service trade or business" (SSTB) — deduction
            phases out + disappears at high income (single $291k / MFJ $584k, 2024).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.qbi.h2.inputs">Inputs</h2>
            <form id="qbi-form" class="inline-form">
                <label><span data-i18n="view.qbi.label.year">Year</span>
                    <select name="year">${Object.keys(LIMITS).map(y =>
                        `<option value="${y}" ${Number(y) === state.year ? 'selected' : ''}>${y}</option>`
                    ).join('')}</select>
                </label>
                <label><span data-i18n="view.qbi.label.filing">Filing status</span>
                    <select name="filing">
                        <option value="single" ${state.filing === 'single' ? 'selected' : ''}>Single</option>
                        <option value="mfj"    ${state.filing === 'mfj' ? 'selected' : ''}>Married filing jointly</option>
                    </select>
                </label>
                <label><span data-i18n="view.qbi.label.qbi_income">QBI (net business income)</span>
                    <input type="number" step="100" name="qbi_income" value="${state.qbi_income}"></label>
                <label><span data-i18n="view.qbi.label.total_taxable">Total taxable income</span>
                    <input type="number" step="100" name="total_taxable_income" value="${state.total_taxable_income}"></label>
                <label><span data-i18n="view.qbi.label.w2_wages">W-2 wages paid</span>
                    <input type="number" step="100" name="w2_wages_paid" value="${state.w2_wages_paid}"></label>
                <label><span data-i18n="view.qbi.label.property">Qualified property basis</span>
                    <input type="number" step="100" name="qualified_property_basis" value="${state.qualified_property_basis}"></label>
                <label><span data-i18n="view.qbi.label.is_sstb">SSTB? (trading = yes)</span>
                    <input type="checkbox" name="is_sstb" ${state.is_sstb ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.qbi.btn.recompute">Recompute</button>
            </form>
        </div>
        <div id="qbi-output"></div>
    `;
    document.getElementById('qbi-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.year = Number(fd.get('year'));
        state.filing = fd.get('filing');
        state.qbi_income = Number(fd.get('qbi_income')) || 0;
        state.total_taxable_income = Number(fd.get('total_taxable_income')) || 0;
        state.w2_wages_paid = Number(fd.get('w2_wages_paid')) || 0;
        state.qualified_property_basis = Number(fd.get('qualified_property_basis')) || 0;
        state.is_sstb = !!fd.get('is_sstb');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('qbi-output');
    if (!el) return;
    const limits = LIMITS[state.year] || LIMITS[2024];
    const threshold = state.filing === 'mfj' ? limits.mfj_threshold : limits.single_threshold;
    const phaseoutEnd = state.filing === 'mfj' ? limits.mfj_phaseout_end : limits.single_phaseout_end;
    const ti = state.total_taxable_income;
    const tentative = Math.max(0, state.qbi_income) * 0.20;
    // Below threshold: full deduction. Limited to 20% of (taxable income - net cap gain).
    // Simplifies here (ignores net cap gain reduction).
    const taxableIncomeCap = ti * 0.20;
    let finalDeduction = 0;
    let regime;
    if (ti <= threshold) {
        regime = 'below';
        finalDeduction = Math.min(tentative, taxableIncomeCap);
    } else if (state.is_sstb && ti >= phaseoutEnd) {
        regime = 'phased_out';
        finalDeduction = 0;
    } else if (ti >= phaseoutEnd) {
        // Above phase-out, non-SSTB: full deduction subject to W-2/UBIA limit.
        regime = 'above_wage_limited';
        const wageLimit1 = state.w2_wages_paid * 0.50;
        const wageLimit2 = state.w2_wages_paid * 0.25 + state.qualified_property_basis * 0.025;
        const wageLimit = Math.max(wageLimit1, wageLimit2);
        finalDeduction = Math.min(tentative, wageLimit, taxableIncomeCap);
    } else {
        // In phase-out range.
        const phaseoutPct = (ti - threshold) / (phaseoutEnd - threshold);
        if (state.is_sstb) {
            const reduced_qbi = state.qbi_income * (1 - phaseoutPct);
            regime = 'phasing_sstb';
            finalDeduction = Math.min(Math.max(0, reduced_qbi) * 0.20, taxableIncomeCap);
        } else {
            const wageLimit = state.w2_wages_paid * 0.50;
            const wageLimitedAmount = tentative - (tentative - Math.min(tentative, wageLimit)) * phaseoutPct;
            regime = 'phasing_non_sstb';
            finalDeduction = Math.min(wageLimitedAmount, taxableIncomeCap);
        }
    }
    const taxSavings = finalDeduction * 0.24;  // assume 24% marginal
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.qbi.h2.result">Deduction</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.qbi.card.deduction">QBI deduction</div>
                    <div class="value">$${finalDeduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.qbi.card.tentative">Tentative (20% QBI)</div>
                    <div class="value">$${tentative.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.qbi.card.ti_cap">Taxable income cap (20%)</div>
                    <div class="value">$${taxableIncomeCap.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.qbi.card.tax_savings">Tax savings @ 24%</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            <p style="margin-top:10px">
                <strong>${esc(t('view.qbi.regime.' + regime))}</strong>
            </p>
            <table class="trades" style="margin-top:10px">
                <thead><tr>
                    <th data-i18n="view.qbi.th.metric">Metric</th>
                    <th data-i18n="view.qbi.th.value">Value</th>
                </tr></thead>
                <tbody>
                    <tr><td data-i18n="view.qbi.row.threshold">${state.filing.toUpperCase()} threshold</td>
                        <td>$${threshold.toLocaleString()}</td></tr>
                    <tr><td data-i18n="view.qbi.row.phaseout_end">Phase-out end</td>
                        <td>$${phaseoutEnd.toLocaleString()}</td></tr>
                    <tr><td data-i18n="view.qbi.row.your_income">Your taxable income</td>
                        <td>$${ti.toLocaleString()}</td></tr>
                    <tr><td data-i18n="view.qbi.row.position">Position</td>
                        <td>${ti < threshold
                            ? esc(t('view.qbi.position.below'))
                            : ti >= phaseoutEnd
                                ? esc(t('view.qbi.position.above'))
                                : esc(t('view.qbi.position.in_range'))}</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.qbi.h2.notes">Notes</h2>
            <p class="muted small" data-i18n="view.qbi.notes.body">
                Traders without TTS election: NO QBI. Investment income (capital gains, dividends,
                interest) is explicitly excluded from QBI by statute. TTS election + § 475(f)
                mark-to-market converts trading P&L to ordinary income, which CAN be QBI but is
                SSTB-limited at high income. § 199A SUNSETS after 2025 unless extended.
            </p>
        </div>
    `;
}
