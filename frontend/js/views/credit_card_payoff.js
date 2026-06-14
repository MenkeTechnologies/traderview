// Credit-card minimum-payment trap — months and interest paying only the
// declining minimum vs a fixed payment, via /calc/credit-card-payoff. Live.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const months = (n) => {
    if (n == null) return '—';
    const y = Math.floor(n / 12);
    const rem = n % 12;
    return `${n} (${y}y ${rem}m)`;
};
const VIEW = 'credit-card-payoff';
let lastReport = null;
let lastBody = null;

export async function renderCreditCardPayoff(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ccpayoff.h1.title">// CREDIT CARD PAYOFF</span></h1>
        <p class="muted small" data-i18n="view.ccpayoff.hint.intro">
            The minimum-payment trap. The minimum due is the greater of a floor and a percent of the
            balance, so it shrinks as you pay down — most of each payment goes to interest, stretching
            a small balance into decades. Compare it to a fixed monthly payment. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ccpayoff.h2.inputs">The card</h2>
            <form id="ccpayoff-form" class="inline-form">
                <label><span data-i18n="view.ccpayoff.label.balance">Balance ($)</span>
                    <input type="number" step="0.01" min="0" name="balance_usd" value="5000" required></label>
                <label><span data-i18n="view.ccpayoff.label.apr">APR (%)</span>
                    <input type="number" step="0.01" min="0" name="apr_pct" value="22" required></label>
                <label><span data-i18n="view.ccpayoff.label.minpct">Minimum payment (% of balance)</span>
                    <input type="number" step="0.1" min="0" name="min_payment_pct" value="2" required></label>
                <label><span data-i18n="view.ccpayoff.label.floor">Minimum payment floor ($)</span>
                    <input type="number" step="1" min="0" name="min_payment_floor_usd" value="25"></label>
                <label><span data-i18n="view.ccpayoff.label.fixed">Fixed payment to compare ($)</span>
                    <input type="number" step="0.01" min="0" name="fixed_payment_usd" value="200"></label>
            </form>
            <div id="ccpayoff-tools" class="ce-toolbar"></div>
            <button type="button" id="ccpayoff-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="ccpayoff-sens" class="ce-sens"></div>
        </div>
        <div id="ccpayoff-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ccpayoff-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        return {
            balance_usd: Number(fd.get('balance_usd')) || 0,
            apr_pct: Number(fd.get('apr_pct')) || 0,
            min_payment_pct: Number(fd.get('min_payment_pct')) || 0,
            min_payment_floor_usd: Number(fd.get('min_payment_floor_usd')) || 0,
            fixed_payment_usd: Number(fd.get('fixed_payment_usd')) || 0,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcCreditCardPayoff(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.ccpayoff.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#ccpayoff-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'credit-card-payoff.csv' });
    mount.querySelector('#ccpayoff-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['months_minimum', r.never_pays_off ? 'never' : r.months_minimum],
        ['total_interest_minimum_usd', r.total_interest_minimum_usd],
        ['total_paid_minimum_usd', r.total_paid_minimum_usd],
        ['months_fixed', r.months_fixed == null ? '' : r.months_fixed],
        ['total_interest_fixed_usd', r.total_interest_fixed_usd == null ? '' : r.total_interest_fixed_usd],
        ['interest_saved_usd', r.interest_saved_usd == null ? '' : r.interest_saved_usd],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#ccpayoff-result');
    const minMonths = r.never_pays_off ? t('view.ccpayoff.never') : months(r.months_minimum);
    // Line chart: months to clear the balance as the fixed payment sweeps from the
    // first minimum up to 5× it (payoff time falls steeply as payment rises).
    const lo = Math.max(r.first_minimum_usd || 25, body.balance_usd * 0.01);
    const xs = enh.linspace(lo, lo * 5, 13);
    const pts = await Promise.all(xs.map(async (pay) => {
        const rr = await api.calcCreditCardPayoff({ ...body, fixed_payment_usd: pay });
        return { x: pay, y: rr && rr.months_fixed != null ? rr.months_fixed : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'payment $', ylabel: 'months' });
    const fixedRows = r.months_fixed == null ? '' : `
        <tr><td data-i18n="view.ccpayoff.row.fixedmonths">Months (fixed)</td><td>${months(r.months_fixed)}</td></tr>
        <tr><td data-i18n="view.ccpayoff.row.fixedint">Interest (fixed)</td><td>${money(r.total_interest_fixed_usd)}</td></tr>
        ${r.interest_saved_usd == null ? '' : `<tr class="pos"><td data-i18n="view.ccpayoff.row.saved">Interest saved</td><td>${money(r.interest_saved_usd)}</td></tr>`}`;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.ccpayoff.h2.result">The cost</h2>
            <div class="cards">
                <div class="card neg"><div class="label" data-i18n="view.ccpayoff.card.minmonths">Minimum-only payoff</div>
                    <div class="value neg">${minMonths}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.ccpayoff.card.minint">Interest (minimum)</div>
                    <div class="value neg">${money(r.total_interest_minimum_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ccpayoff.card.firstmin">First minimum</div>
                    <div class="value">${money(r.first_minimum_usd)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.ccpayoff.row.minmonths">Months (minimum only)</td><td>${minMonths}</td></tr>
                    <tr><td data-i18n="view.ccpayoff.row.minint">Total interest (minimum)</td><td>${money(r.total_interest_minimum_usd)}</td></tr>
                    <tr><td data-i18n="view.ccpayoff.row.minpaid">Total paid (minimum)</td><td>${money(r.total_paid_minimum_usd)}</td></tr>
                    ${fixedRows}
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#ccpayoff-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: APR 10% → 30%; y: fixed payment from first-min to 5×. Output: months to payoff (lower better → negate).
    const lo = Math.max(base.min_payment_floor_usd || 25, base.balance_usd * 0.02);
    const xVals = enh.linspace(10, 30, 5);
    const yVals = enh.linspace(lo, lo * 5, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'apr_pct', yKey: 'fixed_payment_usd', xVals, yVals, compute: (b) => api.calcCreditCardPayoff(b), pick: (r) => (r && r.months_fixed != null ? r.months_fixed : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells: cells.map((row) => row.map((v) => (v == null ? null : -v))), fmt: (v) => (v == null ? '—' : String(-v) + 'mo'), xfmt: (v) => v.toFixed(0) + '%', yfmt: (v) => '$' + Math.round(v), xName: t('view.ccpayoff.label.apr') || 'APR', yName: t('view.ccpayoff.label.fixed') || 'Payment' });
}
