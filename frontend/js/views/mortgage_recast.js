// Mortgage recast — re-amortize the balance after a lump-sum payment, same
// term, lower payment, via /calc/mortgage-recast. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const VIEW = 'mortgage-recast';
let lastReport = null;
let lastBody = null;

export async function renderMortgageRecast(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.recast.h1.title">// MORTGAGE RECAST</span></h1>
        <p class="muted small" data-i18n="view.recast.hint.intro">
            A recast re-amortizes your loan after a lump-sum principal payment, keeping the same
            rate and remaining term — so the monthly payment drops (unlike extra payments, which
            shorten the term instead). Enter the balance, rate, months left, and the lump sum.
            Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.recast.h2.inputs">The loan</h2>
            <form id="recast-form" class="inline-form">
                <label><span data-i18n="view.recast.label.balance">Current balance ($)</span>
                    <input type="number" step="0.01" min="0" name="current_balance_usd" value="300000" required></label>
                <label><span data-i18n="view.recast.label.rate">Annual rate (%)</span>
                    <input type="number" step="0.001" min="0" name="annual_rate_pct" value="6" required></label>
                <label><span data-i18n="view.recast.label.term">Remaining term (months)</span>
                    <input type="number" step="1" min="1" name="remaining_term_months" value="360" required></label>
                <label><span data-i18n="view.recast.label.lump">Lump-sum payment ($)</span>
                    <input type="number" step="0.01" min="0" name="lump_sum_usd" value="50000" required></label>
                <label><span data-i18n="view.recast.label.fee">Recast fee ($)</span>
                    <input type="number" step="0.01" min="0" name="recast_fee_usd" value="250"></label>
            </form>
            <div id="recast-tools" class="ce-toolbar"></div>
            <button type="button" id="recast-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="recast-sens" class="ce-sens"></div>
        </div>
        <div id="recast-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#recast-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        return {
            current_balance_usd: Number(fd.get('current_balance_usd')) || 0,
            annual_rate_pct: Number(fd.get('annual_rate_pct')) || 0,
            remaining_term_months: Number(fd.get('remaining_term_months')) || 0,
            lump_sum_usd: Number(fd.get('lump_sum_usd')) || 0,
            recast_fee_usd: Number(fd.get('recast_fee_usd')) || 0,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcMortgageRecast(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.recast.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#recast-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'mortgage-recast.csv' });
    mount.querySelector('#recast-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['new_payment_usd', r.new_payment_usd],
        ['monthly_savings_usd', r.monthly_savings_usd],
        ['interest_saved_usd', r.interest_saved_usd],
        ['new_balance_usd', r.new_balance_usd],
        ['net_interest_saved_usd', r.net_interest_saved_usd],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#recast-result');
    // Line chart: monthly savings as the lump sum sweeps 0 -> 2x current.
    const base = body.lump_sum_usd || 50000;
    const xs = enh.linspace(0, base * 2, 13);
    const pts = await Promise.all(xs.map(async (l) => {
        const rr = await api.calcMortgageRecast({ ...body, lump_sum_usd: l });
        return { x: l / 1000, y: rr ? rr.monthly_savings_usd : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'lump $k', ylabel: 'savings $/mo' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.recast.h2.result">After the recast</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.recast.card.newpayment">New payment</div>
                    <div class="value pos">${money(r.new_payment_usd)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.recast.card.savings">Monthly savings</div>
                    <div class="value pos">${money(r.monthly_savings_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.recast.card.intsaved">Interest saved</div>
                    <div class="value">${money(r.interest_saved_usd)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.recast.row.oldpayment">Old payment</td><td>${money(r.old_payment_usd)}</td></tr>
                    <tr><td data-i18n="view.recast.row.newbalance">New balance</td><td>${money(r.new_balance_usd)}</td></tr>
                    <tr><td data-i18n="view.recast.row.newpayment">New payment</td><td>${money(r.new_payment_usd)}</td></tr>
                    <tr><td data-i18n="view.recast.row.oldint">Old total interest</td><td>${money(r.old_total_interest_usd)}</td></tr>
                    <tr><td data-i18n="view.recast.row.newint">New total interest</td><td>${money(r.new_total_interest_usd)}</td></tr>
                    <tr><td data-i18n="view.recast.row.netsaved">Net interest saved (after fee)</td><td>${money(r.net_interest_saved_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.recast.row.intsaved">Interest saved</td><td>${money(r.interest_saved_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#recast-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: lump sum 0 -> 2x current; y: annual rate 3% -> 9%. Output: monthly savings.
    const l = base.lump_sum_usd || 50000;
    const xVals = enh.linspace(0, l * 2, 5);
    const yVals = enh.linspace(3, 9, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'lump_sum_usd', yKey: 'annual_rate_pct', xVals, yVals, compute: (b) => api.calcMortgageRecast(b), pick: (r) => (r ? r.monthly_savings_usd : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells, fmt: (v) => (v == null ? '—' : '$' + v.toFixed(0)), xfmt: (v) => '$' + Math.round(v / 1000) + 'k', yfmt: (v) => v.toFixed(1) + '%', xName: t('view.recast.label.lump') || 'Lump', yName: t('view.recast.label.rate') || 'Rate' });
}
