// Real return — the Fisher inflation-adjusted return (exact + shortcut),
// after-tax real return, and purchasing power over time, via
// /calc/real-return. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['nominal_return_pct', 'Nominal return (%)', 7],
    ['inflation_pct', 'Inflation (%)', 3],
    ['tax_rate_pct', 'Tax on return (%)', 15],
    ['principal_usd', 'Principal ($, optional)', 100000],
    ['years', 'Years (optional)', 20],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });
const pct = (n) => Number(n).toFixed(3) + '%';

export async function renderRealReturn(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.rr.h1.title">// REAL (INFLATION-ADJUSTED) RETURN</span></h1>
        <p class="muted small" data-i18n="view.rr.hint.intro">
            A nominal return overstates your gain — inflation erodes purchasing power and taxes
            take a cut. The Fisher equation gives the exact real return: (1 + nominal) ÷ (1 +
            inflation) − 1. The "nominal − inflation" shortcut runs a touch high. After-tax real
            applies the tax to the return first. Over a horizon, the real future value is the
            principal's purchasing power in today's dollars. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.rr.h2.inputs">The numbers</h2>
            <form id="rr-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.rr.label.${key}">${label}</span>
                        <input type="number" step="0.01" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="rr-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#rr-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcRealReturn(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.rr.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#rr-result');
    const realCls = r.real_return_pct >= 0 ? 'pos' : 'neg';
    const atrCls = r.after_tax_real_pct >= 0 ? 'pos' : 'neg';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.rr.h2.result">Real return</h2>
            <div class="cards">
                <div class="card ${realCls}"><div class="label" data-i18n="view.rr.card.real">Real return (Fisher)</div>
                    <div class="value ${realCls}">${pct(r.real_return_pct)}</div></div>
                <div class="card ${atrCls}"><div class="label" data-i18n="view.rr.card.aftertax">After-tax real</div>
                    <div class="value ${atrCls}">${pct(r.after_tax_real_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.rr.card.fv">Real future value</div>
                    <div class="value">${money(r.real_future_value_usd)}</div></div>
            </div>
            <table class="data-table">
                <thead><tr><th data-i18n="view.rr.col.line">Line</th><th data-i18n="view.rr.col.rate">Rate</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.rr.row.exact">Real (exact)</td><td>${pct(r.real_return_pct)}</td></tr>
                    <tr><td data-i18n="view.rr.row.approx">Real (shortcut)</td><td>${pct(r.approx_real_return_pct)}</td></tr>
                    <tr><td data-i18n="view.rr.row.atnom">After-tax nominal</td><td>${pct(r.after_tax_nominal_pct)}</td></tr>
                    <tr class="emph"><td data-i18n="view.rr.row.atreal">After-tax real</td><td class="${atrCls}">${pct(r.after_tax_real_pct)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
