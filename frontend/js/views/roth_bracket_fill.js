// Roth bracket-fill — convert just enough to top off a tax bracket: the
// headroom to the ceiling (capped at the balance) and the tax it triggers,
// via /calc/roth-bracket-fill. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const FIELDS = [
    ['current_taxable_income_usd', 'Current taxable income ($)', 60000],
    ['bracket_ceiling_usd', 'Bracket ceiling ($)', 100000],
    ['marginal_rate_pct', 'Bracket marginal rate (%)', 22],
    ['traditional_balance_usd', 'Traditional balance ($, 0 = no cap)', 0],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });
const VIEW = 'roth-bracket-fill';
let lastReport = null;
let lastBody = null;

export async function renderRothBracketFill(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.rbf.h1.title">// ROTH BRACKET-FILL</span></h1>
        <p class="muted small" data-i18n="view.rbf.hint.intro">
            In a low-income year, convert traditional IRA dollars to Roth up to the top of your
            current tax bracket — but not a dollar more, which would spill into the next, higher
            bracket. This fills the cheap bracket now to avoid larger RMDs taxed higher later.
            Enter your taxable income, the bracket ceiling, and its rate (look up your year's
            brackets); the conversion is capped at your traditional balance. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.rbf.h2.inputs">This year</h2>
            <form id="rbf-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.rbf.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
            <div id="rbf-tools" class="ce-toolbar"></div>
            <button type="button" id="rbf-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="rbf-sens" class="ce-sens"></div>
        </div>
        <div id="rbf-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#rbf-form');
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
            const r = await api.calcRothBracketFill(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.rbf.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#rbf-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'roth-bracket-fill.csv' });
    mount.querySelector('#rbf-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['conversion_amount_usd', r.conversion_amount_usd],
        ['conversion_tax_usd', r.conversion_tax_usd],
        ['headroom_usd', r.headroom_usd],
        ['new_taxable_income_usd', r.new_taxable_income_usd],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#rbf-result');
    // Line chart: conversion amount as current taxable income sweeps 0 -> bracket ceiling
    // (headroom shrinks to zero as income approaches the ceiling).
    const cap = body.bracket_ceiling_usd || 100000;
    const xs = enh.linspace(0, cap, 13);
    const pts = await Promise.all(xs.map(async (inc) => {
        const rr = await api.calcRothBracketFill({ ...body, current_taxable_income_usd: inc });
        return { x: inc / 1000, y: rr ? rr.conversion_amount_usd / 1000 : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'income $k', ylabel: 'convert $k' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.rbf.h2.result">The conversion</h2>
            <div class="cards">
                <div class="card ${r.already_at_ceiling ? 'neg' : 'pos'}"><div class="label" data-i18n="view.rbf.card.convert">Convert</div>
                    <div class="value ${r.already_at_ceiling ? 'neg' : 'pos'}">${money(r.conversion_amount_usd)}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.rbf.card.tax">Tax this year</div>
                    <div class="value neg">${money(r.conversion_tax_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.rbf.card.headroom">Bracket headroom</div>
                    <div class="value">${money(r.headroom_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.rbf.card.newincome">New taxable income</div>
                    <div class="value">${money(r.new_taxable_income_usd)}</div></div>
            </div>
            ${chart}
            ${r.already_at_ceiling ? `<p class="muted small neg" data-i18n="view.rbf.warn.over">Income already meets or exceeds the bracket ceiling — no room to convert at this rate.</p>` : ''}
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#rbf-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: current income 0 -> 150k; y: bracket ceiling 50k -> 250k. Output: conversion amount.
    const xVals = enh.linspace(0, 150000, 5);
    const yVals = enh.linspace(50000, 250000, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'current_taxable_income_usd', yKey: 'bracket_ceiling_usd', xVals, yVals, compute: (b) => api.calcRothBracketFill(b), pick: (r) => (r ? r.conversion_amount_usd : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells, fmt: (v) => (v == null ? '—' : '$' + Math.round(v / 1000) + 'k'), xfmt: (v) => '$' + Math.round(v / 1000) + 'k', yfmt: (v) => '$' + Math.round(v / 1000) + 'k', xName: t('view.rbf.label.current_taxable_income_usd') || 'Income', yName: t('view.rbf.label.bracket_ceiling_usd') || 'Ceiling' });
}
