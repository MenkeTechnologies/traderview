// SaaS magic number (sales efficiency), via /calc/saas-magic-number.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const num = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 4 }));
const VIEW = 'saas-magic-number';
let lastReport = null;
let lastBody = null;
export async function renderSaasMagicNumber(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.smn.h1.title">// SAAS MAGIC NUMBER</span></h1>
        <p class="muted small" data-i18n="view.smn.hint.intro">Sales efficiency: the quarter-over-quarter revenue increase, annualized (×4), divided by the prior quarter's sales & marketing spend. It measures how much new annual recurring revenue each dollar of S&M bought. Above ~0.75 is efficient enough to invest harder; below ~0.5 the go-to-market is not paying back. Not advice.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.smn.h2.inputs">Quarterly revenue & spend</h2>
        <form id="smn-form" class="inline-form">
            <label><span data-i18n="view.smn.label.cur">Current quarter revenue ($)</span><input type="number" step="10000" min="0" name="current_quarter_revenue_usd" value="1100000" required></label>
            <label><span data-i18n="view.smn.label.prior">Prior quarter revenue ($)</span><input type="number" step="10000" min="0" name="prior_quarter_revenue_usd" value="1000000" required></label>
            <label><span data-i18n="view.smn.label.sm">Prior quarter S&M spend ($)</span><input type="number" step="10000" min="0" name="prior_quarter_sm_spend_usd" value="400000" required></label>
        </form>
        <div id="smn-tools" class="ce-toolbar"></div>
        <button type="button" id="smn-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
        <div id="smn-sens" class="ce-sens"></div>
        </div><div id="smn-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#smn-form'); const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => ({ current_quarter_revenue_usd: n('current_quarter_revenue_usd'), prior_quarter_revenue_usd: n('prior_quarter_revenue_usd'), prior_quarter_sm_spend_usd: n('prior_quarter_sm_spend_usd') });
    const gen = async () => {
        const body = readBody();
        try { const d = await api.calcSaasMagicNumber(body); if (!viewIsCurrent(tok)) return; lastReport = d; lastBody = body; res(mount, d, body, tok); }
        catch (e) { showToast(e.message || t('view.smn.toast.error'), { level: 'error' }); }
    };
    enh.mountToolbar(mount.querySelector('#smn-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'saas-magic-number.csv' });
    mount.querySelector('#smn-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function reportRows(d) {
    if (!d || !d.valid) return [];
    return [
        ['metric', 'value'],
        ['magic_number', d.magic_number],
        ['efficiency', d.efficiency],
        ['annualized_net_new_arr_usd', d.annualized_net_new_arr_usd],
        ['sm_payback_months', d.sm_payback_months],
    ];
}
async function res(mount, d, body, tok) {
    const el = mount.querySelector('#smn-result');
    if (!d.valid) { el.innerHTML = `<p class="muted" data-i18n="view.smn.invalid">Prior quarter S&M spend must be positive.</p>`; applyUiI18n(el); return; }
    const cls = d.efficiency === 'efficient' ? 'pos' : (d.efficiency === 'poor' ? 'neg' : '');
    const effLabel = t('view.smn.eff.' + d.efficiency) || d.efficiency;
    // Line chart: magic number as prior S&M spend sweeps ±50% (the dominant lever).
    let chart = '';
    const lo = body.prior_quarter_sm_spend_usd * 0.5, hi = body.prior_quarter_sm_spend_usd * 1.5;
    if (hi > 0) {
        const xs = enh.linspace(lo, hi, 13);
        const pts = await Promise.all(xs.map(async (x) => {
            const r = await api.calcSaasMagicNumber({ ...body, prior_quarter_sm_spend_usd: x });
            return { x: x / 1000, y: r && r.valid ? r.magic_number : NaN };
        }));
        if (!viewIsCurrent(tok)) return;
        chart = enh.svgLineChart(pts, { xlabel: 'S&M $k', ylabel: 'magic#' });
    }
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card ${cls}"><div class="label" data-i18n="view.smn.card.magic">Magic number</div><div class="value">${num(d.magic_number)}</div></div>
        <div class="card ${cls}"><div class="label" data-i18n="view.smn.card.eff">Efficiency</div><div class="value">${effLabel}</div></div>
        <div class="card"><div class="label" data-i18n="view.smn.card.arr">Annualized net-new ARR</div><div class="value">${money(d.annualized_net_new_arr_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.smn.card.payback">S&M payback (months)</div><div class="value">${num(d.sm_payback_months)}</div></div>
    </div>${chart}</div>`;
    applyUiI18n(el);
}
async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#smn-sens');
    if (base.prior_quarter_sm_spend_usd <= 0) return;
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: prior S&M spend ±50%; y: current-quarter revenue from prior to +30%.
    const xVals = enh.linspace(base.prior_quarter_sm_spend_usd * 0.5, base.prior_quarter_sm_spend_usd * 1.5, 5);
    const yVals = enh.linspace(base.prior_quarter_revenue_usd, base.prior_quarter_revenue_usd * 1.3, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'prior_quarter_sm_spend_usd', yKey: 'current_quarter_revenue_usd', xVals, yVals, compute: (b) => api.calcSaasMagicNumber(b), pick: (r) => (r && r.valid ? r.magic_number : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells, fmt: (v) => (v == null ? '—' : v.toFixed(2)), xfmt: (v) => '$' + Math.round(v / 1000) + 'k', yfmt: (v) => '$' + Math.round(v / 1000) + 'k', xName: t('view.smn.label.sm') || 'S&M', yName: t('view.smn.label.cur') || 'Cur rev' });
}
