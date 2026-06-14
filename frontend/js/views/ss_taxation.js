// Taxation of Social Security benefits — provisional-income tiers (0/50/85%),
// via /calc/ss-taxation. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 0 });
const pct = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 1 }) + '%';
const VIEW = 'ss-taxation';
let lastReport = null;
let lastBody = null;

const TIER = {
    none: { key: 'view.sstax.tier.none', cls: 'pos' },
    up_to_50: { key: 'view.sstax.tier.fifty', cls: '' },
    up_to_85: { key: 'view.sstax.tier.eightyfive', cls: 'neg' },
};

export async function renderSsTaxation(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.sstax.h1.title">// SOCIAL SECURITY TAXATION</span></h1>
        <p class="muted small" data-i18n="view.sstax.hint.intro">
            How much of your Social Security is taxable. It's driven by "provisional income" — other
            income plus tax-exempt interest plus half your benefits. Below the first threshold none
            is taxed; between the two up to 50%; above the second up to 85%. The thresholds
            ($25k/$34k single, $32k/$44k joint) are fixed in law and never inflation-adjusted.
            Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.sstax.h2.inputs">Your income</h2>
            <form id="sstax-form" class="inline-form">
                <label><span data-i18n="view.sstax.label.ss">Annual Social Security ($)</span>
                    <input type="number" step="0.01" min="0" name="social_security_usd" value="20000" required></label>
                <label><span data-i18n="view.sstax.label.other">Other income ($)</span>
                    <input type="number" step="0.01" min="0" name="other_income_usd" value="30000" required></label>
                <label><span data-i18n="view.sstax.label.exempt">Tax-exempt interest ($)</span>
                    <input type="number" step="0.01" min="0" name="tax_exempt_interest_usd" value="0"></label>
                <label><span data-i18n="view.sstax.label.status">Filing status</span>
                    <select name="filing_status">
                        <option value="single" data-i18n="view.sstax.status.single">Single</option>
                        <option value="married_joint" data-i18n="view.sstax.status.mfj">Married filing jointly</option>
                    </select></label>
            </form>
            <div id="sstax-tools" class="ce-toolbar"></div>
            <button type="button" id="sstax-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="sstax-sens" class="ce-sens"></div>
        </div>
        <div id="sstax-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#sstax-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        return {
            social_security_usd: Number(fd.get('social_security_usd')) || 0,
            other_income_usd: Number(fd.get('other_income_usd')) || 0,
            tax_exempt_interest_usd: Number(fd.get('tax_exempt_interest_usd')) || 0,
            filing_status: fd.get('filing_status'),
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcSsTaxation(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.sstax.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#sstax-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'ss-taxation.csv' });
    mount.querySelector('#sstax-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['taxable_benefits_usd', r.taxable_benefits_usd],
        ['taxable_pct', r.taxable_pct],
        ['tier', r.tier],
        ['provisional_income_usd', r.provisional_income_usd],
        ['nontaxable_benefits_usd', r.nontaxable_benefits_usd],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#sstax-result');
    const tier = TIER[r.tier] || TIER.up_to_85;
    // Line chart: taxable benefits as other income sweeps 0 -> $120k (the 0/50/85% phase-in).
    const xs = enh.linspace(0, 120000, 16);
    const pts = await Promise.all(xs.map(async (oi) => {
        const rr = await api.calcSsTaxation({ ...body, other_income_usd: oi });
        return { x: oi / 1000, y: rr ? rr.taxable_benefits_usd / 1000 : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'other inc $k', ylabel: 'taxable $k' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.sstax.h2.result">What's taxable</h2>
            <div class="cards">
                <div class="card ${tier.cls}"><div class="label" data-i18n="view.sstax.card.taxable">Taxable benefits</div>
                    <div class="value ${tier.cls}">${money(r.taxable_benefits_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.sstax.card.pct">% of benefits</div>
                    <div class="value">${pct(r.taxable_pct)}</div></div>
                <div class="card ${tier.cls}"><div class="label" data-i18n="view.sstax.card.tier">Tier</div>
                    <div class="value ${tier.cls}" data-i18n="${tier.key}">—</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.sstax.row.provisional">Provisional income</td><td>${money(r.provisional_income_usd)}</td></tr>
                    <tr><td data-i18n="view.sstax.row.base1">First threshold</td><td>${money(r.base1_usd)}</td></tr>
                    <tr><td data-i18n="view.sstax.row.base2">Second threshold</td><td>${money(r.base2_usd)}</td></tr>
                    <tr><td data-i18n="view.sstax.row.nontaxable">Tax-free benefits</td><td>${money(r.nontaxable_benefits_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.sstax.row.taxable">Taxable benefits</td><td>${money(r.taxable_benefits_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#sstax-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: other income 0 -> 120k; y: Social Security 0 -> 60k. Output: taxable benefits.
    const xVals = enh.linspace(0, 120000, 5);
    const yVals = enh.linspace(0, 60000, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'other_income_usd', yKey: 'social_security_usd', xVals, yVals, compute: (b) => api.calcSsTaxation(b), pick: (r) => (r ? r.taxable_benefits_usd : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells, fmt: (v) => (v == null ? '—' : '$' + Math.round(v / 1000) + 'k'), xfmt: (v) => '$' + Math.round(v / 1000) + 'k', yfmt: (v) => '$' + Math.round(v / 1000) + 'k', xName: t('view.sstax.label.other') || 'Other', yName: t('view.sstax.label.ss') || 'SS' });
}
