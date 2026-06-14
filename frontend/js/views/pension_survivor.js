// Pension survivor election — single-life vs joint-and-survivor, the cost of
// survivor protection, and the pension-max (single-life + life insurance)
// comparison, via /calc/pension-survivor. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const FIELDS = [
    ['single_life_monthly_usd', 'Single-life monthly ($)', 3000],
    ['joint_survivor_monthly_usd', 'Joint & survivor monthly ($)', 2600],
    ['survivor_pct', 'Survivor continues (%)', 50],
    ['life_insurance_premium_monthly_usd', 'Life insurance premium ($/mo, 0 = skip)', 0],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 });
const pct = (n) => Number(n).toFixed(1) + '%';
const VIEW = 'pension-survivor';
let lastReport = null;
let lastBody = null;

export async function renderPensionSurvivor(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.psv.h1.title">// PENSION SURVIVOR ELECTION</span></h1>
        <p class="muted small" data-i18n="view.psv.hint.intro">
            A pension forces a choice at retirement: single-life pays the most but stops at the
            retiree's death, or joint-and-survivor pays less now but continues (often reduced)
            to the surviving spouse. The reduction is the price of survivor protection. The
            pension-max alternative: take the higher single-life payment and buy life insurance
            for the spouse — if the payment minus the premium still beats the J&S amount, you
            get more income now and protect the survivor. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.psv.h2.inputs">The election</h2>
            <form id="psv-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.psv.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="psv-tools" class="ce-toolbar"></div>
        <div id="psv-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#psv-form');
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
            const r = await api.calcPensionSurvivor(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.psv.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#psv-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'pension-survivor.csv' });
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['monthly_reduction_usd', r.monthly_reduction_usd],
        ['reduction_pct', r.reduction_pct],
        ['survivor_monthly_benefit_usd', r.survivor_monthly_benefit_usd],
        ['pension_max_net_monthly_usd', r.pension_max_net_monthly_usd],
        ['pension_max_better', r.pension_max_better],
    ];
}

function renderResult(mount, r) {
    const el = mount.querySelector('#psv-result');
    const pmCls = r.pension_max_better ? 'pos' : '';
    // Survivor benefit vs pension-max net monthly — the election comparison.
    const chart = enh.svgBarChart([
        { label: 'Survivor', value: r.survivor_monthly_benefit_usd },
        { label: 'Pension-max', value: r.pension_max_net_monthly_usd },
    ]);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.psv.h2.result">The trade-off</h2>
            <div class="cards">
                <div class="card neg"><div class="label" data-i18n="view.psv.card.reduction">Cost of survivor protection</div>
                    <div class="value neg">${money(r.monthly_reduction_usd)}<span class="muted">/mo</span></div></div>
                <div class="card"><div class="label" data-i18n="view.psv.card.reduction_pct">Reduction</div>
                    <div class="value">${pct(r.reduction_pct)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.psv.card.survivor">Survivor benefit</div>
                    <div class="value pos">${money(r.survivor_monthly_benefit_usd)}<span class="muted">/mo</span></div></div>
                <div class="card ${pmCls}"><div class="label" data-i18n="view.psv.card.pensionmax">Pension-max net</div>
                    <div class="value ${pmCls}">${money(r.pension_max_net_monthly_usd)}<span class="muted">/mo</span></div></div>
            </div>
            ${chart}
            <table class="data-table">
                <thead><tr><th data-i18n="view.psv.col.line">Line</th><th data-i18n="view.psv.col.amount">Amount</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.psv.row.annual">Annual income given up</td><td>${money(r.annual_reduction_usd)}</td></tr>
                    <tr><td data-i18n="view.psv.row.pmnet">Pension-max net monthly</td><td>${money(r.pension_max_net_monthly_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.psv.row.verdict">Pension-max beats J&S?</td>
                        <td class="${pmCls}">${r.pension_max_better ? t('view.psv.yes') : t('view.psv.no')}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
