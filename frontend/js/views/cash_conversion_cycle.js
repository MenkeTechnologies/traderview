// Cash conversion cycle — DSO + DIO − DPO and the operating cycle, the days
// cash is tied up between paying suppliers and collecting from customers, via
// /calc/cash-conversion-cycle. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const FIELDS = [
    ['accounts_receivable_usd', 'Accounts receivable ($)', 50000],
    ['annual_revenue_usd', 'Annual revenue ($)', 365000],
    ['inventory_usd', 'Inventory ($)', 30000],
    ['annual_cogs_usd', 'Annual COGS ($)', 219000],
    ['accounts_payable_usd', 'Accounts payable ($)', 24000],
    ['period_days', 'Period (days)', 365],
];

const days = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 1 }) + ' days';
const VIEW = 'cash-conversion-cycle';
let lastReport = null;
let lastBody = null;

export async function renderCashConversionCycle(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ccc.h1.title">// CASH CONVERSION CYCLE</span></h1>
        <p class="muted small" data-i18n="view.ccc.hint.intro">
            How many days a dollar is tied up in operations. DSO is how long customers take
            to pay (receivables ÷ revenue × period); DIO is how long inventory sits (inventory
            ÷ COGS × period); DPO is how long you take to pay suppliers (payables ÷ COGS ×
            period). CCC = DSO + DIO − DPO. A negative cycle is excellent — you collect before
            you pay, so growth funds itself. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ccc.h2.inputs">The balance sheet</h2>
            <form id="ccc-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.ccc.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
            <div id="ccc-tools" class="ce-toolbar"></div>
            <button type="button" id="ccc-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="ccc-sens" class="ce-sens"></div>
        </div>
        <div id="ccc-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ccc-form');
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
            const r = await api.calcCashConversionCycle(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.ccc.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#ccc-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'cash-conversion-cycle.csv' });
    mount.querySelector('#ccc-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['cash_conversion_cycle_days', r.cash_conversion_cycle_days],
        ['operating_cycle_days', r.operating_cycle_days],
        ['dso_days', r.dso_days],
        ['dio_days', r.dio_days],
        ['dpo_days', r.dpo_days],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#ccc-result');
    const cccCls = r.self_financing ? 'pos' : r.cash_conversion_cycle_days > 0 ? 'neg' : '';
    // Line chart: CCC as accounts payable sweeps 0 → 2× (paying slower shortens the cycle).
    const base = body.accounts_payable_usd || 24000;
    const xs = enh.linspace(0, base * 2, 13);
    const pts = await Promise.all(xs.map(async (ap) => {
        const rr = await api.calcCashConversionCycle({ ...body, accounts_payable_usd: ap });
        return { x: ap / 1000, y: rr ? rr.cash_conversion_cycle_days : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'AP $k', ylabel: 'CCC days' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.ccc.h2.result">The cycle</h2>
            <div class="cards">
                <div class="card ${cccCls}"><div class="label" data-i18n="view.ccc.card.ccc">Cash conversion cycle</div>
                    <div class="value ${cccCls}">${days(r.cash_conversion_cycle_days)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ccc.card.operating">Operating cycle</div>
                    <div class="value">${days(r.operating_cycle_days)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ccc.card.dso">DSO (collect)</div>
                    <div class="value">${days(r.dso_days)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ccc.card.dio">DIO (inventory)</div>
                    <div class="value">${days(r.dio_days)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ccc.card.dpo">DPO (pay)</div>
                    <div class="value">${days(r.dpo_days)}</div></div>
            </div>
            ${chart}
            ${r.self_financing ? `<p class="muted small pos" data-i18n="view.ccc.note.self">Negative cycle — suppliers finance your operations; growth is self-funding.</p>` : ''}
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#ccc-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: receivables 0.5× → 1.5×; y: payables 0.5× → 2×. Output: CCC days (lower is better → negate for shading).
    const ar = base.accounts_receivable_usd || 50000;
    const ap = base.accounts_payable_usd || 24000;
    const xVals = enh.linspace(ar * 0.5, ar * 1.5, 5);
    const yVals = enh.linspace(ap * 0.5, ap * 2, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'accounts_receivable_usd', yKey: 'accounts_payable_usd', xVals, yVals, compute: (b) => api.calcCashConversionCycle(b), pick: (r) => (r ? r.cash_conversion_cycle_days : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells: cells.map((row) => row.map((v) => (v == null ? null : -v))), fmt: (v) => (v == null ? '—' : (-v).toFixed(0) + 'd'), xfmt: (v) => '$' + Math.round(v / 1000) + 'k', yfmt: (v) => '$' + Math.round(v / 1000) + 'k', xName: t('view.ccc.label.accounts_receivable_usd') || 'AR', yName: t('view.ccc.label.accounts_payable_usd') || 'AP' });
}
