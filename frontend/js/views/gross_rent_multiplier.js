// Gross rent multiplier — price ÷ gross annual rent, with vacancy/credit
// loss and other income rolled into effective gross income and an effective
// GRM, via /calc/grm. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['property_price_usd', 'Property price ($)', 600000],
    ['gross_monthly_rent_usd', 'Scheduled monthly rent ($)', 5000],
    ['vacancy_rate_pct', 'Vacancy + credit loss (%)', 5],
    ['other_income_monthly_usd', 'Other monthly income ($)', 0],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });

export async function renderGrm(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.grm.h1.title">// GROSS RENT MULTIPLIER</span></h1>
        <p class="muted small" data-i18n="view.grm.hint.intro">
            A fast way to compare rental prices: property price ÷ gross annual rent. A lower GRM
            is cheaper relative to the rent it produces. Scheduled rent overstates reality, so
            vacancy and credit loss are netted out (and other income added) into effective gross
            income — and an effective GRM on what's actually collected. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.grm.h2.inputs">The property</h2>
            <form id="grm-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.grm.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="grm-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#grm-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcGrm(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.grm.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#grm-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.grm.h2.result">The multiplier</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.grm.card.grm">Gross rent multiplier</div>
                    <div class="value">${Number(r.gross_rent_multiplier).toFixed(1)}</div></div>
                <div class="card"><div class="label" data-i18n="view.grm.card.effgrm">Effective GRM</div>
                    <div class="value">${Number(r.effective_grm).toFixed(1)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.grm.card.egi">Effective gross income</div>
                    <div class="value pos">${money(r.effective_gross_income_usd)}</div></div>
            </div>
            <table class="data-table">
                <thead><tr><th data-i18n="view.grm.col.line">Line</th><th data-i18n="view.grm.col.amount">Annual</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.grm.row.gsi">Gross scheduled income</td><td>${money(r.gross_scheduled_income_usd)}</td></tr>
                    <tr><td data-i18n="view.grm.row.vacancy">Vacancy + credit loss</td><td class="neg">-${money(r.vacancy_loss_usd)}</td></tr>
                    <tr><td data-i18n="view.grm.row.other">Other income</td><td class="pos">+${money(r.other_income_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.grm.row.egi">Effective gross income</td><td>${money(r.effective_gross_income_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
