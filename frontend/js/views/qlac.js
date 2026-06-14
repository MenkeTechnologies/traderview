// QLAC — caps the premium at the SECURE 2.0 limit and shows the RMD
// reduction from excluding it from the RMD base, via /calc/qlac. Live.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const FIELDS = [
    ['account_balance_usd', 'IRA / 401(k) balance ($)', 1000000],
    ['qlac_premium_usd', 'QLAC premium ($)', 210000],
    ['premium_limit_usd', 'Premium limit ($, 0 = default)', 0],
    ['rmd_divisor', 'Uniform Lifetime divisor (age 73 = 26.5)', 26.5],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });
const VIEW = 'qlac';
let lastReport = null;
let lastBody = null;

export async function renderQlac(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.qlac.h1.title">// QLAC — RMD DEFERRAL</span></h1>
        <p class="muted small" data-i18n="view.qlac.hint.intro">
            A Qualified Longevity Annuity Contract moves a capped amount out of an IRA/401(k)
            into a deferred annuity that starts later (up to age 85). That money is excluded
            from the RMD base, lowering your required distribution (and tax) during the deferral
            years. The premium is capped by SECURE 2.0 at $210,000 for 2025–2026 (lifetime, per
            person) and can't exceed the account balance. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.qlac.h2.inputs">Your account</h2>
            <form id="qlac-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.qlac.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
            <div id="qlac-tools" class="ce-toolbar"></div>
        </div>
        <div id="qlac-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#qlac-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        return body;
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcQlac(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.qlac.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#qlac-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'qlac.csv' });
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['annual_rmd_reduction_usd', r.annual_rmd_reduction_usd],
        ['premium_allowed_usd', r.premium_allowed_usd],
        ['premium_limit_usd', r.premium_limit_usd],
        ['rmd_without_qlac_usd', r.rmd_without_qlac_usd],
        ['rmd_with_qlac_usd', r.rmd_with_qlac_usd],
    ];
}

function renderResult(mount, r) {
    const el = mount.querySelector('#qlac-result');
    // RMD without vs with the QLAC — the deferral's effect.
    const chart = enh.svgBarChart([
        { label: 'No QLAC', value: r.rmd_without_qlac_usd },
        { label: 'With QLAC', value: r.rmd_with_qlac_usd },
    ]);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.qlac.h2.result">The deferral</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.qlac.card.reduction">Annual RMD reduction</div>
                    <div class="value pos">${money(r.annual_rmd_reduction_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.qlac.card.allowed">QLAC premium allowed</div>
                    <div class="value">${money(r.premium_allowed_usd)}</div></div>
                <div class="card ${r.over_limit ? 'neg' : ''}"><div class="label" data-i18n="view.qlac.card.limit">Premium limit</div>
                    <div class="value">${money(r.premium_limit_usd)}</div></div>
            </div>
            ${chart}
            ${r.over_limit ? `<p class="muted small neg" data-i18n="view.qlac.warn.over">Requested premium exceeds the limit — capped to the maximum allowed.</p>` : ''}
            <table class="data-table">
                <thead><tr><th data-i18n="view.qlac.col.line">Line</th><th data-i18n="view.qlac.col.amount">Annual RMD</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.qlac.row.without">RMD without QLAC</td><td>${money(r.rmd_without_qlac_usd)}</td></tr>
                    <tr><td data-i18n="view.qlac.row.with">RMD with QLAC</td><td>${money(r.rmd_with_qlac_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.qlac.row.reduction">Reduction</td><td class="pos">${money(r.annual_rmd_reduction_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
