// SaaS quick ratio (growth efficiency), via /calc/saas-quick-ratio.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const num = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 4 }));
const VIEW = 'saas-quick-ratio';
let lastReport = null;
let lastBody = null;
export async function renderSaasQuickRatio(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.sqr.h1.title">// SAAS QUICK RATIO</span></h1>
        <p class="muted small" data-i18n="view.sqr.hint.intro">Growth efficiency: how fast new and expansion MRR are added relative to the contraction and churn lost in the same period. Quick ratio = (new + expansion) ÷ (churned + contraction). 4+ is excellent, 2–4 healthy, 1–2 stalling, below 1 the base is shrinking. Distinct from the accounting acid-test ratio. Not advice.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.sqr.h2.inputs">MRR movement</h2>
        <form id="sqr-form" class="inline-form">
            <label><span data-i18n="view.sqr.label.new">New MRR ($)</span><input type="number" step="1000" min="0" name="new_mrr_usd" value="40000"></label>
            <label><span data-i18n="view.sqr.label.exp">Expansion MRR ($)</span><input type="number" step="1000" min="0" name="expansion_mrr_usd" value="20000"></label>
            <label><span data-i18n="view.sqr.label.churn">Churned MRR ($)</span><input type="number" step="1000" min="0" name="churned_mrr_usd" value="15000"></label>
            <label><span data-i18n="view.sqr.label.contr">Contraction MRR ($)</span><input type="number" step="1000" min="0" name="contraction_mrr_usd" value="5000"></label>
        </form>
        <div id="sqr-tools" class="ce-toolbar"></div>
        <button type="button" id="sqr-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
        <div id="sqr-sens" class="ce-sens"></div>
        </div><div id="sqr-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#sqr-form'); const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => ({ new_mrr_usd: n('new_mrr_usd'), expansion_mrr_usd: n('expansion_mrr_usd'), churned_mrr_usd: n('churned_mrr_usd'), contraction_mrr_usd: n('contraction_mrr_usd') });
    const gen = async () => {
        const body = readBody();
        try { const d = await api.calcSaasQuickRatio(body); if (!viewIsCurrent(tok)) return; lastReport = d; lastBody = body; res(mount, d, body, tok); }
        catch (e) { showToast(e.message || t('view.sqr.toast.error'), { level: 'error' }); }
    };
    enh.mountToolbar(mount.querySelector('#sqr-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'saas-quick-ratio.csv' });
    mount.querySelector('#sqr-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function reportRows(d) {
    if (!d || !d.valid) return [];
    return [
        ['metric', 'value'],
        ['quick_ratio', d.quick_ratio],
        ['health', d.health],
        ['gained_mrr_usd', d.gained_mrr_usd],
        ['lost_mrr_usd', d.lost_mrr_usd],
        ['net_new_mrr_usd', d.net_new_mrr_usd],
    ];
}
async function res(mount, d, body, tok) {
    const el = mount.querySelector('#sqr-result');
    if (!d.valid) { el.innerHTML = `<p class="muted" data-i18n="view.sqr.invalid">Churned + contraction MRR must be positive (ratio is otherwise undefined).</p>`; applyUiI18n(el); return; }
    const cls = (d.health === 'excellent' || d.health === 'healthy') ? 'pos' : (d.health === 'shrinking' ? 'neg' : '');
    const healthLabel = t('view.sqr.health.' + d.health) || d.health;
    // Line chart: quick ratio as new MRR sweeps from 0 to 2× current new MRR.
    let chart = '';
    const hi = (body.new_mrr_usd || 40000) * 2;
    if (hi > 0) {
        const xs = enh.linspace(0, hi, 13);
        const pts = await Promise.all(xs.map(async (x) => {
            const r = await api.calcSaasQuickRatio({ ...body, new_mrr_usd: x });
            return { x: x / 1000, y: r && r.valid ? r.quick_ratio : NaN };
        }));
        if (!viewIsCurrent(tok)) return;
        chart = enh.svgLineChart(pts, { xlabel: 'new MRR $k', ylabel: 'ratio' });
    }
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card ${cls}"><div class="label" data-i18n="view.sqr.card.ratio">Quick ratio</div><div class="value">${num(d.quick_ratio)}</div></div>
        <div class="card ${cls}"><div class="label" data-i18n="view.sqr.card.health">Health</div><div class="value">${healthLabel}</div></div>
        <div class="card"><div class="label" data-i18n="view.sqr.card.gained">Gained MRR</div><div class="value">${money(d.gained_mrr_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.sqr.card.lost">Lost MRR</div><div class="value">${money(d.lost_mrr_usd)}</div></div>
        <div class="card ${cls}"><div class="label" data-i18n="view.sqr.card.net">Net new MRR</div><div class="value">${money(d.net_new_mrr_usd)}</div></div>
    </div>${chart}</div>`;
    applyUiI18n(el);
}
async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#sqr-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: churned MRR 0 → 2× current; y: new MRR 0 → 2× current. Output: quick ratio.
    const xVals = enh.linspace(0, (base.churned_mrr_usd || 15000) * 2, 5);
    const yVals = enh.linspace(0, (base.new_mrr_usd || 40000) * 2, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'churned_mrr_usd', yKey: 'new_mrr_usd', xVals, yVals, compute: (b) => api.calcSaasQuickRatio(b), pick: (r) => (r && r.valid ? r.quick_ratio : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells, fmt: (v) => (v == null ? '—' : v.toFixed(2)), xfmt: (v) => '$' + Math.round(v / 1000) + 'k', yfmt: (v) => '$' + Math.round(v / 1000) + 'k', xName: t('view.sqr.label.churn') || 'Churn', yName: t('view.sqr.label.new') || 'New' });
}
