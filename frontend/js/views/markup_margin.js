// Markup vs margin — from cost + one of {price, markup%, margin%}, shows
// price, profit, and both markup% (of cost) and margin% (of price), via
// /calc/markup-margin. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => Number(n).toFixed(2) + '%';
const VIEW = 'markup-margin';
let lastReport = null;
let lastBody = null;

export async function renderMarkupMargin(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.mm.h1.title">// MARKUP vs MARGIN</span></h1>
        <p class="muted small" data-i18n="view.mm.hint.intro">
            Markup is profit as a percent of cost; margin is the same profit as a percent of
            price. They are never equal — a 50% markup is only a 33% margin, and doubling cost
            (100% markup, "keystone") is a 50% margin. Pricing off the wrong one erodes profit.
            Pick what you know — price, markup%, or margin% — and see all four. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.mm.h2.inputs">What you know</h2>
            <form id="mm-form" class="inline-form">
                <label><span data-i18n="view.mm.label.mode">I'm entering</span>
                    <select name="mode" id="mm-mode">
                        <option value="price" data-i18n="view.mm.mode.price">Cost + price</option>
                        <option value="markup" data-i18n="view.mm.mode.markup">Cost + markup %</option>
                        <option value="margin" data-i18n="view.mm.mode.margin">Cost + margin %</option>
                    </select></label>
                <label><span data-i18n="view.mm.label.cost">Unit cost ($)</span>
                    <input type="number" step="0.01" min="0" name="cost_usd" value="60" required></label>
                <label id="mm-price-wrap"><span data-i18n="view.mm.label.price">Selling price ($)</span>
                    <input type="number" step="0.01" min="0" name="price_usd" value="100"></label>
                <label id="mm-markup-wrap"><span data-i18n="view.mm.label.markup">Markup (%)</span>
                    <input type="number" step="0.01" name="markup_pct" value="66.67"></label>
                <label id="mm-margin-wrap"><span data-i18n="view.mm.label.margin">Margin (%)</span>
                    <input type="number" step="0.01" name="margin_pct" value="40"></label>
            </form>
            <div id="mm-tools" class="ce-toolbar"></div>
            <button type="button" id="mm-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="mm-sens" class="ce-sens"></div>
        </div>
        <div id="mm-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#mm-form');
    const modeSel = mount.querySelector('#mm-mode');
    const wraps = {
        price: mount.querySelector('#mm-price-wrap'),
        markup: mount.querySelector('#mm-markup-wrap'),
        margin: mount.querySelector('#mm-margin-wrap'),
    };
    const syncMode = () => {
        for (const [k, el] of Object.entries(wraps)) el.style.display = modeSel.value === k ? '' : 'none';
    };
    enh.prefillForm(form, enh.readHashInputs());

    const readBody = () => {
        const fd = new FormData(form);
        return {
            mode: fd.get('mode'),
            cost_usd: Number(fd.get('cost_usd')) || 0,
            price_usd: Number(fd.get('price_usd')) || 0,
            markup_pct: Number(fd.get('markup_pct')) || 0,
            margin_pct: Number(fd.get('margin_pct')) || 0,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcMarkupMargin(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.mm.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);
    enh.mountToolbar(mount.querySelector('#mm-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'markup-margin.csv' });
    mount.querySelector('#mm-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    modeSel.addEventListener('change', () => { syncMode(); generate(); });
    form.addEventListener('input', live);
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    syncMode();
    generate();
}

function reportRows(r) {
    if (!r || !r.feasible) return [];
    return [
        ['metric', 'value'],
        ['price_usd', r.price_usd],
        ['profit_usd', r.profit_usd],
        ['markup_pct', r.markup_pct],
        ['margin_pct', r.margin_pct],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#mm-result');
    if (!r.feasible) {
        el.innerHTML = `<div class="chart-panel"><h2 data-i18n="view.mm.h2.result">The numbers</h2>
            <p class="muted small neg" data-i18n="view.mm.warn.infeasible">A margin of 100% or more is impossible — price would be infinite. Lower the margin.</p></div>`;
        applyUiI18n(el);
        return;
    }
    const profitCls = r.profit_usd >= 0 ? 'pos' : 'neg';
    // Line chart: the canonical markup→margin curve (margin % vs markup % from 0 → 200%),
    // independent of the chosen input mode — it is a pure relationship.
    const xs = enh.linspace(0, 200, 21);
    const pts = await Promise.all(xs.map(async (mk) => {
        const rr = await api.calcMarkupMargin({ mode: 'markup', cost_usd: body.cost_usd || 1, price_usd: 0, markup_pct: mk, margin_pct: 0 });
        return { x: mk, y: rr && rr.feasible ? rr.margin_pct : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'markup %', ylabel: 'margin %' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.mm.h2.result">The numbers</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.mm.card.price">Price</div>
                    <div class="value">${money(r.price_usd)}</div></div>
                <div class="card ${profitCls}"><div class="label" data-i18n="view.mm.card.profit">Profit / unit</div>
                    <div class="value ${profitCls}">${money(r.profit_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.mm.card.markup">Markup (of cost)</div>
                    <div class="value">${pct(r.markup_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.mm.card.margin">Margin (of price)</div>
                    <div class="value">${pct(r.margin_pct)}</div></div>
            </div>
            ${chart}
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#mm-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: unit cost 0.5× → 1.5×; y: markup 0 → 150%. Output: profit per unit (in markup mode).
    const cost = base.cost_usd || 60;
    const xVals = enh.linspace(cost * 0.5, cost * 1.5, 5);
    const yVals = enh.linspace(0, 150, 5);
    const { cells } = await enh.runSensitivity({ base: { mode: 'markup', cost_usd: cost, price_usd: 0, markup_pct: 0, margin_pct: 0 }, xKey: 'cost_usd', yKey: 'markup_pct', xVals, yVals, compute: (b) => api.calcMarkupMargin(b), pick: (r) => (r && r.feasible ? r.profit_usd : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells, fmt: (v) => (v == null ? '—' : '$' + v.toFixed(0)), xfmt: (v) => '$' + v.toFixed(0), yfmt: (v) => v.toFixed(0) + '%', xName: t('view.mm.label.cost') || 'Cost', yName: t('view.mm.label.markup') || 'Markup' });
}
