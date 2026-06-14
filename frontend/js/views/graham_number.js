// Graham number — Benjamin Graham's defensive-investor value screen: the max
// price justified by earnings and book value, margin of safety, the P/E×P/B
// test, and the net-net (NCAV) screen, via /calc/graham-number. Live as typed.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const VIEW = 'graham-number';
let lastReport = null;
let lastBody = null;
const ratio = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 3 }));
const pctv = (n) => (n == null ? '—' : (Number(n) * 100).toLocaleString(undefined, { maximumFractionDigits: 1 }) + '%');

export async function renderGrahamNumber(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.graham.h1.title">// GRAHAM NUMBER</span></h1>
        <p class="muted small" data-i18n="view.graham.hint.intro">
            Benjamin Graham's value screen for a defensive investor. The Graham number — √(22.5 ×
            EPS × book value per share) — is the most you should pay; the margin of safety is how
            far the price sits below it. The P/E × P/B product should stay within 22.5. The net-net
            fields screen for deep value at two-thirds of net current asset value. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.graham.h2.inputs">The stock</h2>
            <form id="graham-form" class="inline-form">
                <label><span data-i18n="view.graham.label.eps">Trailing EPS ($)</span>
                    <input type="number" step="0.01" name="eps" value="5" required></label>
                <label><span data-i18n="view.graham.label.bvps">Book value / share ($)</span>
                    <input type="number" step="0.01" name="bvps" value="20" required></label>
                <label><span data-i18n="view.graham.label.price">Current price ($)</span>
                    <input type="number" step="0.01" name="price" value="30" required></label>
                <fieldset class="inline-fieldset">
                    <legend data-i18n="view.graham.legend.netnet">Net-net screen (optional)</legend>
                    <label><span data-i18n="view.graham.label.ca">Current assets ($)</span>
                        <input type="number" step="0.01" min="0" name="current_assets_usd" value="0"></label>
                    <label><span data-i18n="view.graham.label.tl">Total liabilities ($)</span>
                        <input type="number" step="0.01" min="0" name="total_liabilities_usd" value="0"></label>
                    <label><span data-i18n="view.graham.label.shares">Shares outstanding</span>
                        <input type="number" step="1" min="0" name="shares_outstanding" value="0"></label>
                </fieldset>
            </form>
            <div id="graham-tools" class="ce-toolbar"></div>
            <button type="button" id="graham-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="graham-sens" class="ce-sens"></div>
        </div>
        <div id="graham-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#graham-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        return {
            eps: Number(fd.get('eps')) || 0,
            bvps: Number(fd.get('bvps')) || 0,
            price: Number(fd.get('price')) || 0,
            current_assets_usd: Number(fd.get('current_assets_usd')) || 0,
            total_liabilities_usd: Number(fd.get('total_liabilities_usd')) || 0,
            shares_outstanding: Number(fd.get('shares_outstanding')) || 0,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcGrahamNumber(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.graham.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#graham-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'graham-number.csv' });
    mount.querySelector('#graham-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['graham_number', r.graham_number],
        ['margin_of_safety_pct', r.margin_of_safety_pct],
        ['pe_ratio', r.pe_ratio],
        ['pb_ratio', r.pb_ratio],
        ['pe_times_pb', r.pe_times_pb],
        ['ncav_per_share', r.ncav_per_share == null ? '' : r.ncav_per_share],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#graham-result');
    const mosClass = r.margin_of_safety_pct == null ? '' : (r.margin_of_safety_pct >= 0 ? 'pos' : 'neg');
    // Line chart: Graham number as EPS sweeps 0 -> 2x (the sqrt(22.5*EPS*BVPS) curve).
    const base = body.eps || 5;
    const xs = enh.linspace(0, base * 2, 13);
    const pts = await Promise.all(xs.map(async (e) => {
        const rr = await api.calcGrahamNumber({ ...body, eps: e });
        return { x: e, y: rr ? rr.graham_number : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'EPS $', ylabel: 'Graham # $' });
    const peClass = r.passes_graham_pe_pb ? 'pos' : 'neg';
    const netNetRows = r.ncav_per_share == null ? '' : `
        <tr><td data-i18n="view.graham.row.ncav">NCAV / share</td><td>${money(r.ncav_per_share)}</td></tr>
        <tr><td data-i18n="view.graham.row.netnet">Net-net price (⅔ NCAV)</td><td>${money(r.net_net_price)}</td></tr>
        <tr class="${r.is_net_net ? 'pos' : ''}"><td data-i18n="view.graham.row.isnetnet">Below net-net?</td><td>${r.is_net_net ? t('view.graham.yes') : t('view.graham.no')}</td></tr>`;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.graham.h2.result">The verdict</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.graham.card.graham">Graham number</div>
                    <div class="value pos">${money(r.graham_number)}</div></div>
                <div class="card ${mosClass}"><div class="label" data-i18n="view.graham.card.mos">Margin of safety</div>
                    <div class="value ${mosClass}">${pctv(r.margin_of_safety_pct)}</div></div>
                <div class="card ${peClass}"><div class="label" data-i18n="view.graham.card.pepb">P/E × P/B</div>
                    <div class="value ${peClass}">${ratio(r.pe_times_pb)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.graham.row.pe">P/E</td><td>${ratio(r.pe_ratio)}</td></tr>
                    <tr><td data-i18n="view.graham.row.pb">P/B</td><td>${ratio(r.pb_ratio)}</td></tr>
                    <tr class="${peClass}"><td data-i18n="view.graham.row.passes">Within 22.5 cap?</td><td>${r.passes_graham_pe_pb ? t('view.graham.yes') : t('view.graham.no')}</td></tr>
                    ${netNetRows}
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#graham-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: EPS 0 -> 2x; y: BVPS 0 -> 2x. Output: Graham number.
    const xVals = enh.linspace(0, (base.eps || 5) * 2, 5);
    const yVals = enh.linspace(0, (base.bvps || 20) * 2, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'eps', yKey: 'bvps', xVals, yVals, compute: (b) => api.calcGrahamNumber(b), pick: (r) => (r ? r.graham_number : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells, fmt: (v) => (v == null ? '—' : '$' + v.toFixed(0)), xfmt: (v) => '$' + v.toFixed(1), yfmt: (v) => '$' + v.toFixed(0), xName: t('view.graham.label.eps') || 'EPS', yName: t('view.graham.label.bvps') || 'BVPS' });
}
