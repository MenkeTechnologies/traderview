// QBI § 199A 20% Deduction Calculator — pass-through business income deduction.
// Traders with TTS (Trader Tax Status) election get QBI; standard investors don't.
// Computation is server-side via /calc/qbi-deduction (tested core); this view
// supplies the year-specific thresholds and renders the result.
//
// Verified taxable-income thresholds (web-checked):
//   2024  single 191,950 / MFJ 383,900   phase-out end 241,950 / 483,900  (±50k/100k)
//   2025  single 197,300 / MFJ 394,600   phase-out end 247,300 / 494,600  (±50k/100k)
//   2026  single 201,750 / MFJ 403,500   phase-out end 276,750 / 553,500  (OBBBA ±75k/150k)
// The 2025 OBBBA made § 199A permanent (no sunset) and added a $400 minimum
// deduction for active QBI ≥ $1,000 beginning 2026.

import { esc } from '../util.js';
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LIMITS = {
    2024: { single_t: 191_950, single_end: 241_950, mfj_t: 383_900, mfj_end: 483_900, min_ded: 0 },
    2025: { single_t: 197_300, single_end: 247_300, mfj_t: 394_600, mfj_end: 494_600, min_ded: 0 },
    2026: { single_t: 201_750, single_end: 276_750, mfj_t: 403_500, mfj_end: 553_500, min_ded: 400 },
};

const LATEST_YEAR = Math.max(...Object.keys(LIMITS).map(Number));

const REGIME = {
    below: 'view.qbi.regime.below',
    phased_out: 'view.qbi.regime.phased_out',
    above_wage_limited: 'view.qbi.regime.above_wage_limited',
    phasing_sstb: 'view.qbi.regime.phasing_sstb',
    phasing_non_sstb: 'view.qbi.regime.phasing_non_sstb',
};

let state = {
    year: LATEST_YEAR,
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
            qualify. Trading is a "specified service trade or business" (SSTB) — the deduction
            phases out and disappears at high income (2026: single $276,750 / MFJ $553,500).
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
                    <input type="number" step="0.01" name="qbi_income" value="${state.qbi_income}"></label>
                <label><span data-i18n="view.qbi.label.total_taxable">Total taxable income</span>
                    <input type="number" step="0.01" name="total_taxable_income" value="${state.total_taxable_income}"></label>
                <label><span data-i18n="view.qbi.label.w2_wages">W-2 wages paid</span>
                    <input type="number" step="0.01" name="w2_wages_paid" value="${state.w2_wages_paid}"></label>
                <label><span data-i18n="view.qbi.label.property">Qualified property basis</span>
                    <input type="number" step="0.01" name="qualified_property_basis" value="${state.qualified_property_basis}"></label>
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
        renderOutput(tok);
    });
    renderOutput(tok);
}

function regimeFor(status, isSstb) {
    if (status === 'below_threshold') return 'below';
    if (status === 'fully_phased_in') return isSstb ? 'phased_out' : 'above_wage_limited';
    return isSstb ? 'phasing_sstb' : 'phasing_non_sstb';
}

async function renderOutput(tok) {
    const el = document.getElementById('qbi-output');
    if (!el) return;
    const limits = LIMITS[state.year] || LIMITS[LATEST_YEAR];
    const threshold = state.filing === 'mfj' ? limits.mfj_t : limits.single_t;
    const end = state.filing === 'mfj' ? limits.mfj_end : limits.single_end;
    const ti = state.total_taxable_income;

    let r;
    try {
        r = await api.calcQbiDeduction({
            qbi_usd: state.qbi_income,
            taxable_income_usd: ti,
            net_capital_gain_usd: 0,
            filing_status: state.filing === 'mfj' ? 'married_joint' : 'single',
            w2_wages_usd: state.w2_wages_paid,
            ubia_usd: state.qualified_property_basis,
            is_sstb: state.is_sstb,
            rate_pct: 20,
            threshold_usd: threshold,
            phase_in_usd: end - threshold,
            min_deduction_usd: limits.min_ded,
            min_qbi_floor_usd: 1000,
        });
    } catch (err) {
        showToast(err.message || t('view.qbi.toast.error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;

    const regime = regimeFor(r.phase_in_status, state.is_sstb);
    const taxSavings = r.deduction_usd * 0.24;  // assume 24% marginal
    const fmt0 = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.qbi.h2.result">Deduction</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.qbi.card.deduction">QBI deduction</div>
                    <div class="value">$${fmt0(r.deduction_usd)}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.qbi.card.tentative">Tentative (20% QBI)</div>
                    <div class="value">$${fmt0(r.tentative_deduction_usd)}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.qbi.card.ti_cap">Taxable income cap (20%)</div>
                    <div class="value">$${fmt0(r.overall_limit_usd)}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.qbi.card.tax_savings">Tax savings @ 24%</div>
                    <div class="value">$${fmt0(taxSavings)}</div>
                </div>
            </div>
            <p style="margin-top:10px">
                <strong>${esc(t(REGIME[regime]))}</strong>
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
                        <td>$${end.toLocaleString()}</td></tr>
                    <tr><td data-i18n="view.qbi.row.your_income">Your taxable income</td>
                        <td>$${ti.toLocaleString()}</td></tr>
                    <tr><td data-i18n="view.qbi.row.position">Position</td>
                        <td>${ti < threshold
                            ? esc(t('view.qbi.position.below'))
                            : ti >= end
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
                SSTB-limited at high income. The 2025 OBBBA made § 199A permanent (no sunset) and
                added a $400 minimum deduction for active QBI of at least $1,000 starting in 2026.
            </p>
        </div>
    `;
    // The recompute path re-injects markup, so re-translate the new subtree.
    applyUiI18n(el);
}
