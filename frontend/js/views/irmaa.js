// IRMAA — Medicare Part B/D income surcharge (2026). Maps a two-year-prior
// MAGI + filing status to its surcharge tier, the per-person and household
// annual cost, and the headroom to the next cliff, via /calc/irmaa.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 });
const money0 = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });

export async function renderIrmaa(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.irmaa.h1.title">// MEDICARE IRMAA SURCHARGE</span></h1>
        <p class="muted small" data-i18n="view.irmaa.hint.intro">
            High-income Medicare beneficiaries pay a surcharge on top of the standard Part B
            and Part D premiums. It's a cliff: cross a MAGI threshold by a single dollar and
            you owe the full higher tier all year — no phase-in. IRMAA is assessed per
            enrolled person on a two-year lookback, so 2026 premiums are set on your 2024
            MAGI. That's why Roth conversions, capital gains, and RMDs taken two years before
            you enroll quietly raise these premiums.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.irmaa.h2.inputs">Your income</h2>
            <form id="irmaa-form" class="inline-form">
                <label><span data-i18n="view.irmaa.label.magi">2024 Modified AGI ($)</span>
                    <input type="number" step="0.01" min="0" name="magi_usd" value="150000" required></label>
                <label><span data-i18n="view.irmaa.label.status">Filing status</span>
                    <select name="filing_status">
                        <option value="single" data-i18n="view.irmaa.status.single">Single</option>
                        <option value="married_joint" data-i18n="view.irmaa.status.joint">Married filing jointly</option>
                        <option value="married_separate" data-i18n="view.irmaa.status.separate">Married filing separately</option>
                    </select></label>
                <label data-tip="view.irmaa.tip.both"><input type="checkbox" name="both_spouses_on_medicare"> <span data-i18n="view.irmaa.label.both">Both spouses on Medicare</span></label>
                <button class="primary" type="submit" data-i18n="view.irmaa.btn.run">Find tier</button>
            </form>
        </div>
        <div id="irmaa-result"></div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#irmaa-form');
    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = {
            magi_usd: Number(fd.get('magi_usd')) || 0,
            filing_status: fd.get('filing_status'),
            both_spouses_on_medicare: fd.get('both_spouses_on_medicare') != null,
        };
        try {
            const r = await api.calcIrmaa(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.irmaa.toast.error'), { level: 'error' });
        }
    });
    form.dispatchEvent(new Event('submit'));
}

function renderResult(mount, r) {
    const el = mount.querySelector('#irmaa-result');
    const tierCls = r.tier === 0 ? 'pos' : 'neg';
    const upper = r.upper_threshold_usd == null
        ? t('view.irmaa.no_cap')
        : money0(r.upper_threshold_usd);
    const headroom = r.headroom_to_next_cliff_usd == null
        ? '—'
        : money0(r.headroom_to_next_cliff_usd);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.irmaa.h2.result">Your surcharge</h2>
            <div class="cards">
                <div class="card ${tierCls}"><div class="label" data-i18n="view.irmaa.card.tier">IRMAA tier</div>
                    <div class="value ${tierCls}">${r.tier} / 5</div></div>
                <div class="card ${r.tier === 0 ? 'pos' : 'neg'}"><div class="label" data-i18n="view.irmaa.card.annual">Annual surcharge (per person)</div>
                    <div class="value">${money0(r.annual_surcharge_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.irmaa.card.household">Household annual surcharge</div>
                    <div class="value">${money0(r.household_annual_surcharge_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.irmaa.card.band">This tier's MAGI band</div>
                    <div class="value">${money0(r.lower_threshold_usd)} – ${upper}</div></div>
                <div class="card"><div class="label" data-i18n="view.irmaa.card.headroom">Headroom to next cliff</div>
                    <div class="value">${headroom}</div></div>
            </div>
            <table class="data-table">
                <thead><tr>
                    <th data-i18n="view.irmaa.col.part">Premium part</th>
                    <th data-i18n="view.irmaa.col.monthly">Monthly</th>
                    <th data-i18n="view.irmaa.col.surcharge">Monthly surcharge</th>
                </tr></thead>
                <tbody>
                    <tr><td data-i18n="view.irmaa.row.part_b">Part B</td>
                        <td>${money(r.part_b_monthly_usd)}</td><td>${money(r.part_b_surcharge_usd)}</td></tr>
                    <tr><td data-i18n="view.irmaa.row.part_d">Part D (added to plan premium)</td>
                        <td>—</td><td>${money(r.part_d_surcharge_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.irmaa.row.total">Total surcharge</td>
                        <td>—</td><td class="${r.tier === 0 ? 'pos' : 'neg'}">${money(r.monthly_surcharge_usd)}</td></tr>
                </tbody>
            </table>
            <p class="muted small" data-i18n="view.irmaa.note">2026 premiums (based on 2024 MAGI). Standard Part B is $202.90/mo. IRMAA is charged per enrolled individual; brackets are inflation-adjusted yearly.</p>
        </div>
    `;
    applyUiI18n(el);
}
