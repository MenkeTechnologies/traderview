// Bonus gross-up — the gross payment needed to net a target after tax, via
// /calc/bonus-grossup. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%';
const VIEW = 'bonus-grossup';
let lastReport = null;
let lastBody = null;

export async function renderBonusGrossup(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.grossup.h1.title">// BONUS GROSS-UP</span></h1>
        <p class="muted small" data-i18n="view.grossup.hint.intro">
            The gross payment needed so the recipient nets a target after tax — for grossing up a
            bonus, relocation, or make-whole payment. The federal supplemental-wage flat rate is 22%
            (37% above $1M); state and the 7.65% FICA stack on top. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.grossup.h2.inputs">The payment</h2>
            <form id="grossup-form" class="inline-form">
                <label><span data-i18n="view.grossup.label.net">Desired net ($)</span>
                    <input type="number" step="0.01" min="0" name="desired_net_usd" value="1000" required></label>
                <label><span data-i18n="view.grossup.label.federal">Federal rate (%)</span>
                    <input type="number" step="0.1" min="0" name="federal_rate_pct" value="22" required></label>
                <label><span data-i18n="view.grossup.label.state">State rate (%)</span>
                    <input type="number" step="0.1" min="0" name="state_rate_pct" value="5"></label>
                <label><span data-i18n="view.grossup.label.fica">Include FICA (7.65%)</span>
                    <input type="checkbox" name="include_fica" checked></label>
            </form>
            <div id="grossup-tools" class="ce-toolbar"></div>
            <button type="button" id="grossup-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="grossup-sens" class="ce-sens"></div>
        </div>
        <div id="grossup-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#grossup-form');
    const hashIn = enh.readHashInputs();
    enh.prefillForm(form, hashIn);
    if ('include_fica' in hashIn) form.querySelector('[name=include_fica]').checked = hashIn.include_fica === 'true';
    const readBody = () => {
        const fd = new FormData(form);
        return {
            desired_net_usd: Number(fd.get('desired_net_usd')) || 0,
            federal_rate_pct: Number(fd.get('federal_rate_pct')) || 0,
            state_rate_pct: Number(fd.get('state_rate_pct')) || 0,
            include_fica: form.querySelector('[name=include_fica]').checked,
            fica_rate_pct: 7.65,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcBonusGrossup(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.grossup.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#grossup-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'bonus-grossup.csv' });
    mount.querySelector('#grossup-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['gross_usd', r.gross_usd],
        ['total_tax_usd', r.total_tax_usd],
        ['combined_rate_pct', r.combined_rate_pct],
        ['federal_withholding_usd', r.federal_withholding_usd],
        ['state_withholding_usd', r.state_withholding_usd],
        ['fica_withholding_usd', r.fica_withholding_usd],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#grossup-result');
    // Line chart: required gross as the desired net sweeps 0 -> 2x (linear in net).
    const base = body.desired_net_usd || 1000;
    const xs = enh.linspace(0, base * 2, 13);
    const pts = await Promise.all(xs.map(async (n) => {
        const rr = await api.calcBonusGrossup({ ...body, desired_net_usd: n });
        return { x: n, y: rr ? rr.gross_usd : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'net $', ylabel: 'gross $' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.grossup.h2.result">The gross-up</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.grossup.card.gross">Gross payment</div>
                    <div class="value pos">${money(r.gross_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.grossup.card.tax">Total tax</div>
                    <div class="value">${money(r.total_tax_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.grossup.card.rate">Combined rate</div>
                    <div class="value">${pct(r.combined_rate_pct)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.grossup.row.federal">Federal withholding</td><td>${money(r.federal_withholding_usd)}</td></tr>
                    <tr><td data-i18n="view.grossup.row.state">State withholding</td><td>${money(r.state_withholding_usd)}</td></tr>
                    <tr><td data-i18n="view.grossup.row.fica">FICA withholding</td><td>${money(r.fica_withholding_usd)}</td></tr>
                    <tr><td data-i18n="view.grossup.row.tax">Total tax</td><td>${money(r.total_tax_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.grossup.row.gross">Gross payment</td><td>${money(r.gross_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#grossup-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: federal rate 10% -> 40%; y: state rate 0% -> 13%. Output: required gross.
    const xVals = enh.linspace(10, 40, 5);
    const yVals = enh.linspace(0, 13, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'federal_rate_pct', yKey: 'state_rate_pct', xVals, yVals, compute: (b) => api.calcBonusGrossup(b), pick: (r) => (r ? r.gross_usd : null) });
    if (!viewIsCurrent(tok)) return;
    // Lower required gross is better -> negate for shading (green = cheaper).
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells: cells.map((row) => row.map((v) => (v == null ? null : -v))), fmt: (v) => (v == null ? '—' : '$' + Math.round(-v)), xfmt: (v) => v.toFixed(0) + '%', yfmt: (v) => v.toFixed(0) + '%', xName: t('view.grossup.label.federal') || 'Fed', yName: t('view.grossup.label.state') || 'State' });
}
