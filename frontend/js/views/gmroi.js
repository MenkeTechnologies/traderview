// GMROI — gross-margin return on inventory investment, with turnover, days of
// inventory, and gross margin, via /calc/gmroi. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const num = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 1 }) + '%');
const VIEW = 'gmroi';
let lastReport = null;
let lastBody = null;

export async function renderGmroi(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.gmroi.h1.title">// GMROI</span></h1>
        <p class="muted small" data-i18n="view.gmroi.hint.intro">
            Gross Margin Return On Inventory Investment — how many gross-margin dollars each dollar
            tied up in inventory returns. Above 1.0 means each inventory dollar earns more than a
            dollar of margin; general-merchandise retail often targets ~3.2. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.gmroi.h2.inputs">The inventory</h2>
            <form id="gmroi-form" class="inline-form">
                <label><span data-i18n="view.gmroi.label.revenue">Revenue ($)</span>
                    <input type="number" step="0.01" min="0" name="revenue_usd" value="1000" required></label>
                <label><span data-i18n="view.gmroi.label.cogs">COGS ($)</span>
                    <input type="number" step="0.01" min="0" name="cogs_usd" value="600" required></label>
                <label><span data-i18n="view.gmroi.label.inventory">Average inventory at cost ($)</span>
                    <input type="number" step="0.01" min="0" name="average_inventory_usd" value="200" required></label>
                <label><span data-i18n="view.gmroi.label.days">Period days</span>
                    <input type="number" step="1" min="1" name="period_days" value="365"></label>
            </form>
            <div id="gmroi-tools" class="ce-toolbar"></div>
            <button type="button" id="gmroi-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="gmroi-sens" class="ce-sens"></div>
        </div>
        <div id="gmroi-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#gmroi-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        return {
            revenue_usd: Number(fd.get('revenue_usd')) || 0,
            cogs_usd: Number(fd.get('cogs_usd')) || 0,
            average_inventory_usd: Number(fd.get('average_inventory_usd')) || 0,
            period_days: Number(fd.get('period_days')) || 365,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcGmroi(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.gmroi.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#gmroi-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'gmroi.csv' });
    mount.querySelector('#gmroi-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['gmroi', r.gmroi],
        ['inventory_turnover', r.inventory_turnover],
        ['gross_margin_pct', r.gross_margin_pct],
        ['gross_margin_usd', r.gross_margin_usd],
        ['days_inventory', r.days_inventory],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#gmroi-result');
    const gmroiClass = r.gmroi == null ? '' : (r.gmroi >= 1 ? 'pos' : 'neg');
    // Line chart: GMROI as average inventory sweeps 0.5× → 2× (return falls as capital tied up rises).
    const base = body.average_inventory_usd || 200;
    const xs = enh.linspace(base * 0.5, base * 2, 13);
    const pts = await Promise.all(xs.map(async (inv) => {
        const rr = await api.calcGmroi({ ...body, average_inventory_usd: inv });
        return { x: inv, y: rr && rr.gmroi != null ? rr.gmroi : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'avg inv $', ylabel: 'GMROI' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.gmroi.h2.result">The return</h2>
            <div class="cards">
                <div class="card ${gmroiClass}"><div class="label" data-i18n="view.gmroi.card.gmroi">GMROI</div>
                    <div class="value ${gmroiClass}">${num(r.gmroi)}</div></div>
                <div class="card"><div class="label" data-i18n="view.gmroi.card.turns">Inventory turns</div>
                    <div class="value">${num(r.inventory_turnover)}</div></div>
                <div class="card"><div class="label" data-i18n="view.gmroi.card.margin">Gross margin</div>
                    <div class="value">${pct(r.gross_margin_pct)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.gmroi.row.gm">Gross margin ($)</td><td>${money(r.gross_margin_usd)}</td></tr>
                    <tr><td data-i18n="view.gmroi.row.turns">Inventory turns</td><td>${num(r.inventory_turnover)}</td></tr>
                    <tr><td data-i18n="view.gmroi.row.days">Days of inventory</td><td>${num(r.days_inventory)}</td></tr>
                    <tr class="emph"><td data-i18n="view.gmroi.row.gmroi">GMROI</td><td>${num(r.gmroi)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#gmroi-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: average inventory 0.5× → 2×; y: COGS 0.5× → 1.5×. Output: GMROI.
    const inv = base.average_inventory_usd || 200;
    const cogs = base.cogs_usd || 600;
    const xVals = enh.linspace(inv * 0.5, inv * 2, 5);
    const yVals = enh.linspace(cogs * 0.5, cogs * 1.5, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'average_inventory_usd', yKey: 'cogs_usd', xVals, yVals, compute: (b) => api.calcGmroi(b), pick: (r) => (r ? r.gmroi : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells, fmt: (v) => (v == null ? '—' : v.toFixed(2)), xfmt: (v) => '$' + Math.round(v), yfmt: (v) => '$' + Math.round(v), xName: t('view.gmroi.label.inventory') || 'Avg inv', yName: t('view.gmroi.label.cogs') || 'COGS' });
}
