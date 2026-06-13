// Yield on cost — a dividend against your cost basis vs the current price,
// with a projected YOC at the dividend growth rate, via /calc/yield-on-cost.
// Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['cost_basis_per_share_usd', 'Cost basis / share ($)', 40],
    ['current_annual_dividend_usd', 'Current annual dividend / share ($)', 2],
    ['current_price_per_share_usd', 'Current price / share ($)', 80],
    ['dividend_growth_pct', 'Dividend growth (%/yr)', 7],
    ['years', 'Projection (years)', 10],
];

const pct = (n) => Number(n).toFixed(2) + '%';
const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });

export async function renderYieldOnCost(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.yoc.h1.title">// YIELD ON COST</span></h1>
        <p class="muted small" data-i18n="view.yoc.hint.intro">
            Current yield measures the dividend against today's price; yield on cost measures it
            against what you paid. For a dividend grower, YOC climbs every year the payout is
            raised — a position bought cheaply can yield double digits on cost while its current
            yield looks modest. Shows YOC vs current yield and a projected YOC at your growth
            rate. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.yoc.h2.inputs">The position</h2>
            <form id="yoc-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.yoc.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="yoc-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#yoc-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcYieldOnCost(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.yoc.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#yoc-result');
    const cy = r.current_yield_pct == null ? '—' : pct(r.current_yield_pct);
    const dbl = r.years_to_double_dividend == null ? '—' : Number(r.years_to_double_dividend).toFixed(1) + ' ' + t('view.yoc.years');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.yoc.h2.result">The yields</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.yoc.card.yoc">Yield on cost</div>
                    <div class="value pos">${pct(r.yield_on_cost_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.yoc.card.current">Current yield</div>
                    <div class="value">${cy}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.yoc.card.projected">Projected YOC</div>
                    <div class="value pos">${pct(r.projected_yield_on_cost_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.yoc.card.double">Dividend doubles in</div>
                    <div class="value">${dbl}</div></div>
            </div>
            <table class="data-table">
                <thead><tr><th data-i18n="view.yoc.col.line">Line</th><th data-i18n="view.yoc.col.value">Value</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.yoc.row.proj_div">Projected dividend / share</td><td>${money(r.projected_dividend_usd)}</td></tr>
                    <tr><td data-i18n="view.yoc.row.yoc">Yield on cost</td><td class="pos">${pct(r.yield_on_cost_pct)}</td></tr>
                    <tr class="emph"><td data-i18n="view.yoc.row.proj_yoc">Projected yield on cost</td><td class="pos">${pct(r.projected_yield_on_cost_pct)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
