// Revenue retention (NRR & GRR), via /calc/revenue-retention.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
const VIEW = 'revenue-retention';
let lastReport = null;
let lastBody = null;
export async function renderRevenueRetention(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.nrr.h1.title">// REVENUE RETENTION</span></h1>
        <p class="muted small" data-i18n="view.nrr.hint.intro">The core subscription-revenue health metrics. From a cohort's starting recurring revenue, expansion adds revenue while contraction and churn remove it. Net revenue retention (NRR) can exceed 100% when expansion outruns losses; gross revenue retention (GRR) ignores expansion and caps at 100%. New-logo revenue is excluded. Not advice.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.nrr.h2.inputs">Cohort movement</h2>
        <form id="nrr-form" class="inline-form">
            <label><span data-i18n="view.nrr.label.period">Period</span><input type="text" name="period_label" value="Q2"></label>
            <label><span data-i18n="view.nrr.label.start">Starting MRR ($)</span><input type="number" step="1000" min="0" name="starting_mrr_usd" value="100000" required></label>
            <label><span data-i18n="view.nrr.label.exp">Expansion MRR ($)</span><input type="number" step="500" min="0" name="expansion_mrr_usd" value="15000"></label>
            <label><span data-i18n="view.nrr.label.contr">Contraction MRR ($)</span><input type="number" step="500" min="0" name="contraction_mrr_usd" value="5000"></label>
            <label><span data-i18n="view.nrr.label.churn">Churned MRR ($)</span><input type="number" step="500" min="0" name="churned_mrr_usd" value="8000"></label>
        </form>
        <div id="nrr-tools" class="ce-toolbar"></div>
        <button type="button" id="nrr-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
        <div id="nrr-sens" class="ce-sens"></div>
        </div><div id="nrr-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#nrr-form'); const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => ({ period_label: (form.querySelector('[name="period_label"]').value || '').trim(), starting_mrr_usd: n('starting_mrr_usd'), expansion_mrr_usd: n('expansion_mrr_usd'), contraction_mrr_usd: n('contraction_mrr_usd'), churned_mrr_usd: n('churned_mrr_usd') });
    const gen = async () => {
        const body = readBody();
        try { const d = await api.calcRevenueRetention(body); if (!viewIsCurrent(tok)) return; lastReport = d; lastBody = body; res(mount, d, body, tok); }
        catch (e) { showToast(e.message || t('view.nrr.toast.error'), { level: 'error' }); }
    };
    enh.mountToolbar(mount.querySelector('#nrr-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'revenue-retention.csv' });
    mount.querySelector('#nrr-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function reportRows(d) {
    if (!d || !d.valid) return [];
    return [
        ['metric', 'value'],
        ['nrr_pct', d.nrr_pct],
        ['grr_pct', d.grr_pct],
        ['net_change_usd', d.net_change_usd],
        ['ending_mrr_usd', d.ending_mrr_usd],
    ];
}
async function res(mount, d, body, tok) {
    const el = mount.querySelector('#nrr-result');
    if (!d.valid) { el.innerHTML = `<p class="muted" data-i18n="view.nrr.invalid">Starting MRR must be positive.</p>`; applyUiI18n(el); return; }
    const cls = d.net_expanding ? 'pos' : 'neg';
    // Line chart: NRR as expansion MRR sweeps 0 → 2× current.
    let chart = '';
    const hi = (body.expansion_mrr_usd || 15000) * 2;
    if (hi > 0 && body.starting_mrr_usd > 0) {
        const xs = enh.linspace(0, hi, 13);
        const pts = await Promise.all(xs.map(async (xv) => {
            const r = await api.calcRevenueRetention({ ...body, expansion_mrr_usd: xv });
            return { x: xv / 1000, y: r && r.valid ? r.nrr_pct : NaN };
        }));
        if (!viewIsCurrent(tok)) return;
        chart = enh.svgLineChart(pts, { xlabel: 'expansion $k', ylabel: 'NRR %' });
    }
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card ${cls}"><div class="label" data-i18n="view.nrr.card.nrr">Net retention (NRR)</div><div class="value">${pct(d.nrr_pct)}</div></div>
        <div class="card"><div class="label" data-i18n="view.nrr.card.grr">Gross retention (GRR)</div><div class="value">${pct(d.grr_pct)}</div></div>
        <div class="card ${cls}"><div class="label" data-i18n="view.nrr.card.net">Net change</div><div class="value">${money(d.net_change_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.nrr.card.end">Ending MRR</div><div class="value">${money(d.ending_mrr_usd)}</div></div>
    </div>${chart}</div>`;
    applyUiI18n(el);
}
async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#nrr-sens');
    if (base.starting_mrr_usd <= 0) return;
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: churned MRR 0 → 30% of start; y: expansion MRR 0 → 40% of start. Output: NRR %.
    const xVals = enh.linspace(0, base.starting_mrr_usd * 0.3, 5);
    const yVals = enh.linspace(0, base.starting_mrr_usd * 0.4, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'churned_mrr_usd', yKey: 'expansion_mrr_usd', xVals, yVals, compute: (b) => api.calcRevenueRetention(b), pick: (r) => (r && r.valid ? r.nrr_pct : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells, fmt: (v) => (v == null ? '—' : v.toFixed(1) + '%'), xfmt: (v) => '$' + Math.round(v / 1000) + 'k', yfmt: (v) => '$' + Math.round(v / 1000) + 'k', xName: t('view.nrr.label.churn') || 'Churn', yName: t('view.nrr.label.exp') || 'Expansion' });
}
