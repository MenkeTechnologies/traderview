// NPV / IRR for a cash-flow series — NPV at a discount rate, IRR, profitability
// index, and simple + discounted payback, via /calc/npv-irr. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
const yrs = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }));
const idx = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 3 }));

function parseFlows(raw) {
    return String(raw || '')
        .split(/[\s,]+/)
        .map((s) => s.trim())
        .filter((s) => s.length)
        .map(Number)
        .filter((n) => Number.isFinite(n));
}

export async function renderNpvIrr(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.npvirr.h1.title">// NPV / IRR</span></h1>
        <p class="muted small" data-i18n="view.npvirr.hint.intro">
            Discounted-cash-flow analysis for a project or investment. Enter the cash flows period by
            period — the first is today (usually a negative outlay), then one per period. The NPV
            uses your discount rate; the IRR is the rate that zeroes NPV. Also shows the
            profitability index and simple/discounted payback. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.npvirr.h2.inputs">The cash flows</h2>
            <form id="npvirr-form" class="inline-form">
                <label><span data-i18n="view.npvirr.label.flows">Cash flows (comma-separated, period 0 first)</span>
                    <input type="text" name="cash_flows" value="-1000, 500, 500, 500" required></label>
                <label><span data-i18n="view.npvirr.label.rate">Discount rate (%)</span>
                    <input type="number" step="0.01" name="discount_rate_pct" value="10" required></label>
            </form>
        </div>
        <div id="npvirr-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#npvirr-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            cash_flows: parseFlows(fd.get('cash_flows')),
            discount_rate_pct: Number(fd.get('discount_rate_pct')) || 0,
        };
        try {
            const r = await api.calcNpvIrr(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.npvirr.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#npvirr-result');
    const npvClass = r.npv_usd >= 0 ? 'pos' : 'neg';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.npvirr.h2.result">The analysis</h2>
            <div class="cards">
                <div class="card ${npvClass}"><div class="label" data-i18n="view.npvirr.card.npv">NPV</div>
                    <div class="value ${npvClass}">${money(r.npv_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.npvirr.card.irr">IRR</div>
                    <div class="value">${pct(r.irr_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.npvirr.card.pi">Profitability index</div>
                    <div class="value">${idx(r.profitability_index)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.npvirr.row.irr">IRR</td><td>${pct(r.irr_pct)}</td></tr>
                    <tr><td data-i18n="view.npvirr.row.pi">Profitability index</td><td>${idx(r.profitability_index)}</td></tr>
                    <tr><td data-i18n="view.npvirr.row.payback">Payback (years)</td><td>${yrs(r.payback_years)}</td></tr>
                    <tr><td data-i18n="view.npvirr.row.discpayback">Discounted payback (years)</td><td>${yrs(r.discounted_payback_years)}</td></tr>
                    <tr><td data-i18n="view.npvirr.row.total">Total undiscounted</td><td>${money(r.total_undiscounted_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.npvirr.row.npv">NPV</td><td>${money(r.npv_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
