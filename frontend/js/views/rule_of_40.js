// Rule of 40 — revenue growth + profit margin vs the 40% bar, with the
// growth/margin you'd need to clear it, via /calc/rule-of-40. Live.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 1 }) + '%');
const VIEW = 'rule-of-40';
let lastReport = null;
let lastBody = null;

export async function renderRuleOf40(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ruleof40.h1.title">// RULE OF 40</span></h1>
        <p class="muted small" data-i18n="view.ruleof40.hint.intro">
            The SaaS/growth-company health check: revenue growth rate plus profit margin should sum
            to at least 40%. It trades growth against profitability — a company can pass by growing
            fast at a loss, or growing slowly with fat margins. The margin can be FCF, EBITDA, or
            net — whatever you pair with growth. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ruleof40.h2.inputs">The numbers</h2>
            <form id="ruleof40-form" class="inline-form">
                <label><span data-i18n="view.ruleof40.label.growth">Revenue growth (%)</span>
                    <input type="number" step="0.1" name="revenue_growth_pct" value="30" required></label>
                <label><span data-i18n="view.ruleof40.label.margin">Profit margin (%)</span>
                    <input type="number" step="0.1" name="profit_margin_pct" value="15" required></label>
                <label><span data-i18n="view.ruleof40.label.target">Target (%)</span>
                    <input type="number" step="1" name="target_pct" value="40"></label>
            </form>
            <div id="ruleof40-tools" class="ce-toolbar"></div>
            <button type="button" id="ruleof40-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="ruleof40-sens" class="ce-sens"></div>
        </div>
        <div id="ruleof40-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ruleof40-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        return {
            revenue_growth_pct: Number(fd.get('revenue_growth_pct')) || 0,
            profit_margin_pct: Number(fd.get('profit_margin_pct')) || 0,
            target_pct: Number(fd.get('target_pct')) || 40,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcRuleOf40(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.ruleof40.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#ruleof40-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'rule-of-40.csv' });
    mount.querySelector('#ruleof40-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['score_pct', r.score_pct],
        ['passes', r.passes],
        ['surplus_pct', r.surplus_pct],
        ['margin_needed_pct', r.margin_needed_pct],
        ['growth_needed_pct', r.growth_needed_pct],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#ruleof40-result');
    const cls = r.passes ? 'pos' : 'neg';
    const verdictKey = r.passes ? 'view.ruleof40.verdict.pass' : 'view.ruleof40.verdict.fail';
    // Line chart: score as revenue growth sweeps 0 → 60%.
    const xs = enh.linspace(0, 60, 13);
    const pts = await Promise.all(xs.map(async (g) => {
        const rr = await api.calcRuleOf40({ ...body, revenue_growth_pct: g });
        return { x: g, y: rr ? rr.score_pct : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'growth %', ylabel: 'score %' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.ruleof40.h2.result">The verdict</h2>
            <div class="cards">
                <div class="card ${cls}"><div class="label" data-i18n="view.ruleof40.card.score">Score</div>
                    <div class="value ${cls}">${pct(r.score_pct)}</div></div>
                <div class="card ${cls}"><div class="label" data-i18n="view.ruleof40.card.verdict">Verdict</div>
                    <div class="value ${cls}" data-i18n="${verdictKey}">—</div></div>
                <div class="card"><div class="label" data-i18n="view.ruleof40.card.surplus">Surplus / gap</div>
                    <div class="value">${pct(r.surplus_pct)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.ruleof40.row.marginneeded">Margin needed at this growth</td><td>${pct(r.margin_needed_pct)}</td></tr>
                    <tr><td data-i18n="view.ruleof40.row.growthneeded">Growth needed at this margin</td><td>${pct(r.growth_needed_pct)}</td></tr>
                    <tr><td data-i18n="view.ruleof40.row.growthshare">Growth share of score</td><td>${pct(r.growth_share_pct)}</td></tr>
                    <tr><td data-i18n="view.ruleof40.row.marginshare">Margin share of score</td><td>${pct(r.margin_share_pct)}</td></tr>
                    <tr class="emph"><td data-i18n="view.ruleof40.row.score">Score</td><td>${pct(r.score_pct)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#ruleof40-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    const xVals = enh.linspace(0, 60, 5);   // revenue growth %
    const yVals = enh.linspace(-20, 40, 5); // profit margin %
    const { cells } = await enh.runSensitivity({ base, xKey: 'revenue_growth_pct', yKey: 'profit_margin_pct', xVals, yVals, compute: (b) => api.calcRuleOf40(b), pick: (r) => (r ? r.score_pct : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells, fmt: (v) => (v == null ? '—' : v.toFixed(0) + '%'), xfmt: (v) => v.toFixed(0) + '%', yfmt: (v) => v.toFixed(0) + '%', xName: t('view.ruleof40.label.growth') || 'Growth', yName: t('view.ruleof40.label.margin') || 'Margin' });
}
