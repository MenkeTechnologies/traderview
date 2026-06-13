// Spousal IRA — per-spouse contribution limits (with catch-up) and whether
// the couple's combined earned income covers both, via /calc/spousal-ira.
// Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const NUM = [
    ['combined_earned_income_usd', 'Combined earned income ($)', 80000],
    ['working_spouse_contribution_usd', 'Working spouse contribution ($)', 7500],
    ['nonworking_spouse_contribution_usd', 'Non-working spouse contribution ($)', 7500],
    ['base_limit_usd', 'Per-person base limit ($, 0 = default)', 0],
    ['catch_up_usd', 'Catch-up (50+) ($, 0 = default)', 0],
];
const CHECKS = [
    ['working_spouse_50plus', 'Working spouse 50+'],
    ['nonworking_spouse_50plus', 'Non-working spouse 50+'],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });

export async function renderSpousalIra(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.sira.h1.title">// SPOUSAL IRA</span></h1>
        <p class="muted small" data-i18n="view.sira.hint.intro">
            Normally an IRA contribution needs earned income. The spousal-IRA rule lets a
            married couple filing jointly fund an IRA for a spouse with little or no income, as
            long as their combined earned income covers both contributions. Each spouse is
            still capped at the annual limit ($7,500 for 2026, +$1,100 catch-up at 50+).
            Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.sira.h2.inputs">The couple</h2>
            <form id="sira-form" class="inline-form">
                ${NUM.map(([key, label, def]) => `
                    <label><span data-i18n="view.sira.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
                ${CHECKS.map(([key, label]) => `
                    <label><input type="checkbox" name="${key}"> <span data-i18n="view.sira.label.${key}">${label}</span></label>
                `).join('')}
            </form>
        </div>
        <div id="sira-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#sira-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of NUM) body[key] = Number(fd.get(key)) || 0;
        for (const [key] of CHECKS) body[key] = fd.get(key) != null;
        try {
            const r = await api.calcSpousalIra(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.sira.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#sira-result');
    const okCls = r.income_sufficient ? 'pos' : 'neg';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.sira.h2.result">Eligibility</h2>
            <div class="cards">
                <div class="card ${okCls}"><div class="label" data-i18n="view.sira.card.status">Income covers both?</div>
                    <div class="value ${okCls}">${r.income_sufficient ? t('view.sira.yes') : t('view.sira.no')}</div></div>
                <div class="card"><div class="label" data-i18n="view.sira.card.combined">Combined contribution</div>
                    <div class="value">${money(r.combined_contribution_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.sira.card.max">Max combined</div>
                    <div class="value">${money(r.max_combined_usd)}</div></div>
            </div>
            ${r.income_sufficient ? '' : `<p class="muted small neg" data-i18n="view.sira.warn.income">Combined earned income is below the contributions — total IRA contributions can't exceed earned income.</p>`}
            <table class="data-table">
                <thead><tr>
                    <th data-i18n="view.sira.col.spouse">Spouse</th>
                    <th data-i18n="view.sira.col.limit">Limit</th>
                    <th data-i18n="view.sira.col.allowed">Allowed</th>
                </tr></thead>
                <tbody>
                    <tr><td data-i18n="view.sira.row.working">Working</td><td>${money(r.working_spouse_limit_usd)}</td><td>${money(r.working_allowed_usd)}</td></tr>
                    <tr><td data-i18n="view.sira.row.nonworking">Non-working</td><td>${money(r.nonworking_spouse_limit_usd)}</td><td>${money(r.nonworking_allowed_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
