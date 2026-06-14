// Social Security PIA — the 90/32/15 bend-point formula turning AIME into
// the full-retirement-age benefit, with the tier breakdown, via /calc/ss-pia.
// Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const FIELDS = [
    ['aime_usd', 'Average indexed monthly earnings ($)', 6000],
    ['bend_point_1_usd', 'First bend point ($, 0 = 2026 default)', 0],
    ['bend_point_2_usd', 'Second bend point ($, 0 = 2026 default)', 0],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 });
const pct = (n) => Number(n).toFixed(1) + '%';
const VIEW = 'ss-pia';
let lastReport = null;
let lastBody = null;

export async function renderSsPia(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.pia.h1.title">// SOCIAL SECURITY PIA</span></h1>
        <p class="muted small" data-i18n="view.pia.hint.intro">
            Your full-retirement-age benefit comes from Average Indexed Monthly Earnings (AIME)
            through a progressive three-tier formula: 90% of AIME up to the first bend point,
            32% between the two bend points, and 15% above the second. The 90/32/15 rates are
            fixed by law; the bend points (2026: $1,286 and $7,749) are indexed yearly and lock
            in at age 62. Lower earners get a higher replacement rate. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.pia.h2.inputs">Your earnings</h2>
            <form id="pia-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.pia.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
            <div id="pia-tools" class="ce-toolbar"></div>
        </div>
        <div id="pia-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#pia-form');
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
            const r = await api.calcSsPia(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.pia.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#pia-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'ss-pia.csv' });
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['pia_monthly_usd', r.pia_monthly_usd],
        ['pia_annual_usd', r.pia_annual_usd],
        ['replacement_rate_pct', r.replacement_rate_pct],
        ['tier1_usd', r.tier1_usd],
        ['tier2_usd', r.tier2_usd],
        ['tier3_usd', r.tier3_usd],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#pia-result');
    // Line chart: PIA as AIME sweeps 0 -> $12k — the 90/32/15 kinked curve (bend points show as slope breaks).
    const xs = enh.linspace(0, 12000, 16);
    const pts = await Promise.all(xs.map(async (a) => {
        const rr = await api.calcSsPia({ ...body, aime_usd: a });
        return { x: a, y: rr ? rr.pia_monthly_usd : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'AIME $', ylabel: 'PIA $/mo' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.pia.h2.result">Your benefit</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.pia.card.monthly">PIA (monthly, at FRA)</div>
                    <div class="value pos">${money(r.pia_monthly_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.pia.card.annual">Annual</div>
                    <div class="value">${money(r.pia_annual_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.pia.card.replacement">Replacement rate</div>
                    <div class="value">${pct(r.replacement_rate_pct)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <thead><tr>
                    <th data-i18n="view.pia.col.tier">Tier</th>
                    <th data-i18n="view.pia.col.contribution">Contribution to PIA</th>
                </tr></thead>
                <tbody>
                    <tr><td>${t('view.pia.row.t1', { bp: money(r.bend_point_1_usd) })}</td><td>${money(r.tier1_usd)}</td></tr>
                    <tr><td>${t('view.pia.row.t2', { bp: money(r.bend_point_2_usd) })}</td><td>${money(r.tier2_usd)}</td></tr>
                    <tr><td>${t('view.pia.row.t3', { bp: money(r.bend_point_2_usd) })}</td><td>${money(r.tier3_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.pia.row.total">Total PIA</td><td class="pos">${money(r.pia_monthly_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
