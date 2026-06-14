// Gross rent multiplier — price ÷ gross annual rent, with vacancy/credit
// loss and other income rolled into effective gross income and an effective
// GRM, via /calc/grm. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const FIELDS = [
    ['property_price_usd', 'Property price ($)', 600000],
    ['gross_monthly_rent_usd', 'Scheduled monthly rent ($)', 5000],
    ['vacancy_rate_pct', 'Vacancy + credit loss (%)', 5],
    ['other_income_monthly_usd', 'Other monthly income ($)', 0],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });
const VIEW = 'grm';
let lastReport = null;
let lastBody = null;

export async function renderGrm(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.grm.h1.title">// GROSS RENT MULTIPLIER</span></h1>
        <p class="muted small" data-i18n="view.grm.hint.intro">
            A fast way to compare rental prices: property price ÷ gross annual rent. A lower GRM
            is cheaper relative to the rent it produces. Scheduled rent overstates reality, so
            vacancy and credit loss are netted out (and other income added) into effective gross
            income — and an effective GRM on what's actually collected. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.grm.h2.inputs">The property</h2>
            <form id="grm-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.grm.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
            <div id="grm-tools" class="ce-toolbar"></div>
            <button type="button" id="grm-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="grm-sens" class="ce-sens"></div>
        </div>
        <div id="grm-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#grm-form');
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
            const r = await api.calcGrm(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.grm.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#grm-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'grm.csv' });
    mount.querySelector('#grm-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['gross_rent_multiplier', r.gross_rent_multiplier],
        ['effective_grm', r.effective_grm],
        ['effective_gross_income_usd', r.effective_gross_income_usd],
        ['gross_scheduled_income_usd', r.gross_scheduled_income_usd],
        ['vacancy_loss_usd', r.vacancy_loss_usd],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#grm-result');
    // Line chart: GRM as property price sweeps 0.5× → 1.5× current (linear in price).
    const base = body.property_price_usd || 600000;
    const xs = enh.linspace(base * 0.5, base * 1.5, 13);
    const pts = await Promise.all(xs.map(async (p) => {
        const rr = await api.calcGrm({ ...body, property_price_usd: p });
        return { x: p / 1000, y: rr ? rr.gross_rent_multiplier : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'price $k', ylabel: 'GRM' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.grm.h2.result">The multiplier</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.grm.card.grm">Gross rent multiplier</div>
                    <div class="value">${Number(r.gross_rent_multiplier).toFixed(1)}</div></div>
                <div class="card"><div class="label" data-i18n="view.grm.card.effgrm">Effective GRM</div>
                    <div class="value">${Number(r.effective_grm).toFixed(1)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.grm.card.egi">Effective gross income</div>
                    <div class="value pos">${money(r.effective_gross_income_usd)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <thead><tr><th data-i18n="view.grm.col.line">Line</th><th data-i18n="view.grm.col.amount">Annual</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.grm.row.gsi">Gross scheduled income</td><td>${money(r.gross_scheduled_income_usd)}</td></tr>
                    <tr><td data-i18n="view.grm.row.vacancy">Vacancy + credit loss</td><td class="neg">-${money(r.vacancy_loss_usd)}</td></tr>
                    <tr><td data-i18n="view.grm.row.other">Other income</td><td class="pos">+${money(r.other_income_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.grm.row.egi">Effective gross income</td><td>${money(r.effective_gross_income_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#grm-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: monthly rent 0.5× → 1.5×; y: property price 0.5× → 1.5×. Output: effective GRM (lower is better).
    const rent = base.gross_monthly_rent_usd || 5000;
    const price = base.property_price_usd || 600000;
    const xVals = enh.linspace(rent * 0.5, rent * 1.5, 5);
    const yVals = enh.linspace(price * 0.5, price * 1.5, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'gross_monthly_rent_usd', yKey: 'property_price_usd', xVals, yVals, compute: (b) => api.calcGrm(b), pick: (r) => (r ? r.effective_grm : null) });
    if (!viewIsCurrent(tok)) return;
    // Lower GRM is a better deal, so negate for shading (green = lower) while showing the real value.
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells: cells.map((row) => row.map((v) => (v == null ? null : -v))), fmt: (v) => (v == null ? '—' : (-v).toFixed(1)), xfmt: (v) => '$' + Math.round(v), yfmt: (v) => '$' + Math.round(v / 1000) + 'k', xName: t('view.grm.label.gross_monthly_rent_usd') || 'Rent', yName: t('view.grm.label.property_price_usd') || 'Price' });
}
