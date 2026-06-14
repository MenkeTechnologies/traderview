// Years to FI — savings rate, FI number (expenses / SWR), and the years for
// current savings + the annual surplus to reach it, via /calc/years-to-fi.
// Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const FIELDS = [
    ['current_savings_usd', 'Current savings ($)', 50000],
    ['annual_income_usd', 'Annual income ($)', 80000],
    ['annual_expenses_usd', 'Annual expenses ($)', 40000],
    ['annual_return_pct', 'Expected real return (%)', 5],
    ['safe_withdrawal_rate_pct', 'Safe withdrawal rate (%)', 4],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });
const VIEW = 'years-to-fi';
let lastReport = null;
let lastBody = null;

export async function renderYearsToFi(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ytf.h1.title">// YEARS TO FINANCIAL INDEPENDENCE</span></h1>
        <p class="muted small" data-i18n="view.ytf.hint.intro">
            FI is when your portfolio funds your spending indefinitely at a safe withdrawal rate
            — the FI number = annual expenses ÷ SWR (4% → 25× expenses). Starting from current
            savings and adding the income-minus-expenses gap each year (grown at your return),
            this finds the years to reach it. The dominant lever is the savings rate, far more
            than the return. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ytf.h2.inputs">Your finances</h2>
            <form id="ytf-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.ytf.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
            <div id="ytf-tools" class="ce-toolbar"></div>
            <button type="button" id="ytf-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="ytf-sens" class="ce-sens"></div>
        </div>
        <div id="ytf-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ytf-form');
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
            const r = await api.calcYearsToFi(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.ytf.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#ytf-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'years-to-fi.csv' });
    mount.querySelector('#ytf-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['years_to_fi', r.already_fi ? 0 : (r.years_to_fi == null ? '' : r.years_to_fi)],
        ['savings_rate_pct', r.savings_rate_pct],
        ['fi_number_usd', r.fi_number_usd],
        ['annual_savings_usd', r.annual_savings_usd],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#ytf-result');
    // Line chart: years to FI as annual expenses sweep 0.5x -> 1.5x (lower expenses both
    // raise the savings rate and shrink the FI number, so years fall steeply).
    const base = body.annual_expenses_usd || 40000;
    const xs = enh.linspace(base * 0.5, base * 1.5, 13);
    const pts = await Promise.all(xs.map(async (e) => {
        const rr = await api.calcYearsToFi({ ...body, annual_expenses_usd: e });
        return { x: e / 1000, y: rr && !rr.already_fi && rr.years_to_fi != null ? rr.years_to_fi : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'expenses $k', ylabel: 'years' });
    let yearsVal, yearsCls;
    if (r.already_fi) {
        yearsVal = t('view.ytf.already');
        yearsCls = 'pos';
    } else if (r.years_to_fi == null) {
        yearsVal = t('view.ytf.never');
        yearsCls = 'neg';
    } else {
        yearsVal = `${r.years_to_fi} ${t('view.ytf.years')}`;
        yearsCls = r.years_to_fi <= 15 ? 'pos' : r.years_to_fi <= 30 ? '' : 'neg';
    }
    const srCls = r.savings_rate_pct >= 50 ? 'pos' : r.savings_rate_pct >= 20 ? '' : 'neg';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.ytf.h2.result">The timeline</h2>
            <div class="cards">
                <div class="card ${yearsCls}"><div class="label" data-i18n="view.ytf.card.years">Years to FI</div>
                    <div class="value ${yearsCls}">${yearsVal}</div></div>
                <div class="card ${srCls}"><div class="label" data-i18n="view.ytf.card.rate">Savings rate</div>
                    <div class="value ${srCls}">${Number(r.savings_rate_pct).toFixed(1)}%</div></div>
                <div class="card"><div class="label" data-i18n="view.ytf.card.finumber">FI number</div>
                    <div class="value">${money(r.fi_number_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ytf.card.savings">Annual savings</div>
                    <div class="value">${money(r.annual_savings_usd)}</div></div>
            </div>
            ${chart}
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#ytf-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: annual expenses 0.5x -> 1.5x; y: real return 1% -> 10%. Output: years to FI (lower better -> negate).
    const ex = base.annual_expenses_usd || 40000;
    const xVals = enh.linspace(ex * 0.5, ex * 1.5, 5);
    const yVals = enh.linspace(1, 10, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'annual_expenses_usd', yKey: 'annual_return_pct', xVals, yVals, compute: (b) => api.calcYearsToFi(b), pick: (r) => (r && !r.already_fi && r.years_to_fi != null ? r.years_to_fi : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells: cells.map((row) => row.map((v) => (v == null ? null : -v))), fmt: (v) => (v == null ? '—' : (-v).toFixed(0) + 'y'), xfmt: (v) => '$' + Math.round(v / 1000) + 'k', yfmt: (v) => v.toFixed(0) + '%', xName: t('view.ytf.label.annual_expenses_usd') || 'Expenses', yName: t('view.ytf.label.annual_return_pct') || 'Return' });
}
