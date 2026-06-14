// Seller financing / carryback note — amortized payment, balloon balance,
// and the seller's interest income, via /calc/seller-financing. Live.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const FIELDS = [
    ['sale_price_usd', 'Sale price ($)', 400000],
    ['down_payment_usd', 'Down payment ($)', 80000],
    ['annual_rate_pct', 'Note rate (%)', 7],
    ['amortization_years', 'Amortization (years)', 30],
    ['balloon_years', 'Balloon (years, 0 = none)', 5],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 });
const money0 = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });
const VIEW = 'seller-financing';
let lastReport = null;
let lastBody = null;

export async function renderSellerFinancing(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.sf.h1.title">// SELLER FINANCING</span></h1>
        <p class="muted small" data-i18n="view.sf.hint.intro">
            When the seller is the bank: the buyer puts down a down payment and pays the seller
            directly on the balance (the carryback note) at a rate, amortized over a schedule —
            often with a balloon where the remaining balance is due in a few years. Shows the
            monthly payment, the balloon balance owed at that date, and the interest the seller
            earns. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.sf.h2.inputs">The deal</h2>
            <form id="sf-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.sf.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
            <div id="sf-tools" class="ce-toolbar"></div>
            <button type="button" id="sf-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="sf-sens" class="ce-sens"></div>
        </div>
        <div id="sf-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#sf-form');
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
            const r = await api.calcSellerFinancing(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.sf.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#sf-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'seller-financing.csv' });
    mount.querySelector('#sf-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['monthly_payment_usd', r.monthly_payment_usd],
        ['note_amount_usd', r.note_amount_usd],
        ['balloon_balance_usd', r.balloon_balance_usd],
        ['seller_interest_income_usd', r.seller_interest_income_usd],
        ['total_payments_usd', r.total_payments_usd],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#sf-result');
    // Line chart: monthly payment as the note rate sweeps 0 -> 12%.
    const xs = enh.linspace(0, 12, 13);
    const pts = await Promise.all(xs.map(async (rate) => {
        const rr = await api.calcSellerFinancing({ ...body, annual_rate_pct: rate });
        return { x: rate, y: rr ? rr.monthly_payment_usd : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'rate %', ylabel: 'payment $/mo' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.sf.h2.result">The note</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.sf.card.payment">Monthly payment</div>
                    <div class="value pos">${money(r.monthly_payment_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.sf.card.note">Note amount</div>
                    <div class="value">${money0(r.note_amount_usd)}</div></div>
                <div class="card ${r.has_balloon ? 'neg' : ''}"><div class="label" data-i18n="view.sf.card.balloon">Balloon balance</div>
                    <div class="value">${r.has_balloon ? money0(r.balloon_balance_usd) : t('view.sf.none')}</div></div>
                <div class="card"><div class="label" data-i18n="view.sf.card.interest">Seller interest income</div>
                    <div class="value">${money0(r.seller_interest_income_usd)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <thead><tr><th data-i18n="view.sf.col.line">Line</th><th data-i18n="view.sf.col.amount">Amount</th></tr></thead>
                <tbody>
                    <tr><td>${t('view.sf.row.payments')} ${r.has_balloon ? t('view.sf.to_balloon') : t('view.sf.full_term')}</td><td>${money0(r.total_payments_usd)}</td></tr>
                    <tr><td data-i18n="view.sf.row.balloon">Balloon balance due</td><td>${money0(r.balloon_balance_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.sf.row.interest">Seller interest income</td><td class="pos">${money0(r.seller_interest_income_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#sf-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: note rate 0 -> 12%; y: amortization 10 -> 40 years. Output: monthly payment.
    const xVals = enh.linspace(0, 12, 5);
    const yVals = enh.linspace(10, 40, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'annual_rate_pct', yKey: 'amortization_years', xVals, yVals: yVals.map(Math.round), compute: (b) => api.calcSellerFinancing(b), pick: (r) => (r ? r.monthly_payment_usd : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals: yVals.map(Math.round), cells, fmt: (v) => (v == null ? '—' : '$' + Math.round(v)), xfmt: (v) => v.toFixed(1) + '%', yfmt: (v) => v.toFixed(0) + 'y', xName: t('view.sf.label.annual_rate_pct') || 'Rate', yName: t('view.sf.label.amortization_years') || 'Amort' });
}
