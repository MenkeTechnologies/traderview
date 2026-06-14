// Price-to-rent ratio — home price ÷ annual rent and gross rental yield,
// with the buy/borderline/rent verdict, via /calc/price-to-rent. Live.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const VIEW = 'price-to-rent';
let lastReport = null;
let lastBody = null;

const FIELDS = [
    ['home_price_usd', 'Home price ($)', 400000],
    ['monthly_rent_usd', 'Comparable monthly rent ($)', 2000],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });
const VERDICT = {
    'favors buying': ['Favors buying (<15)', 'pos'],
    borderline: ['Borderline (15–20)', ''],
    'favors renting': ['Favors renting (>20)', 'neg'],
    'n/a': ['—', ''],
};

export async function renderPriceToRent(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ptr.h1.title">// PRICE-TO-RENT RATIO</span></h1>
        <p class="muted small" data-i18n="view.ptr.hint.intro">
            A quick read on whether a market favors buying or renting: home price ÷ annual rent
            for a comparable home. Under 15 generally favors buying, 15–20 is borderline, over
            20 favors renting (the home is pricey relative to what it rents for). The gross
            rental yield is the inverse. A screen, not a full rent-vs-buy NPV. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ptr.h2.inputs">The market</h2>
            <form id="ptr-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.ptr.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
            <div id="ptr-tools" class="ce-toolbar"></div>
            <button type="button" id="ptr-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="ptr-sens" class="ce-sens"></div>
        </div>
        <div id="ptr-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ptr-form');
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
            const r = await api.calcPriceToRent(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.ptr.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#ptr-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'price-to-rent.csv' });
    mount.querySelector('#ptr-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['price_to_rent_ratio', r.price_to_rent_ratio],
        ['gross_rental_yield_pct', r.gross_rental_yield_pct],
        ['annual_rent_usd', r.annual_rent_usd],
        ['verdict', r.verdict],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#ptr-result');
    const [verdictLabel, verdictCls] = VERDICT[r.verdict] || [r.verdict, ''];
    // Line chart: price-to-rent ratio as home price sweeps 0.5x -> 1.5x (linear in price).
    const base = body.home_price_usd || 400000;
    const xs = enh.linspace(base * 0.5, base * 1.5, 13);
    const pts = await Promise.all(xs.map(async (p) => {
        const rr = await api.calcPriceToRent({ ...body, home_price_usd: p });
        return { x: p / 1000, y: rr ? rr.price_to_rent_ratio : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'price $k', ylabel: 'P/R ratio' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.ptr.h2.result">The verdict</h2>
            <div class="cards">
                <div class="card ${verdictCls}"><div class="label" data-i18n="view.ptr.card.ratio">Price-to-rent</div>
                    <div class="value ${verdictCls}">${Number(r.price_to_rent_ratio).toFixed(1)}</div></div>
                <div class="card ${verdictCls}"><div class="label" data-i18n="view.ptr.card.verdict">Verdict</div>
                    <div class="value ${verdictCls}">${verdictLabel}</div></div>
                <div class="card"><div class="label" data-i18n="view.ptr.card.yield">Gross rental yield</div>
                    <div class="value">${Number(r.gross_rental_yield_pct).toFixed(2)}%</div></div>
                <div class="card"><div class="label" data-i18n="view.ptr.card.annual">Annual rent</div>
                    <div class="value">${money(r.annual_rent_usd)}</div></div>
            </div>
            ${chart}
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#ptr-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: home price 0.5x -> 1.5x; y: monthly rent 0.5x -> 1.5x. Output: price-to-rent ratio (lower favors buying -> negate).
    const p = base.home_price_usd || 400000;
    const rent = base.monthly_rent_usd || 2000;
    const xVals = enh.linspace(p * 0.5, p * 1.5, 5);
    const yVals = enh.linspace(rent * 0.5, rent * 1.5, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'home_price_usd', yKey: 'monthly_rent_usd', xVals, yVals, compute: (b) => api.calcPriceToRent(b), pick: (r) => (r ? r.price_to_rent_ratio : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells: cells.map((row) => row.map((v) => (v == null ? null : -v))), fmt: (v) => (v == null ? '—' : (-v).toFixed(1)), xfmt: (v) => '$' + Math.round(v / 1000) + 'k', yfmt: (v) => '$' + Math.round(v), xName: t('view.ptr.label.home_price_usd') || 'Price', yName: t('view.ptr.label.monthly_rent_usd') || 'Rent' });
}
