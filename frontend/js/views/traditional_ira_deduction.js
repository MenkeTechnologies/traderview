// Traditional IRA deduction after the MAGI phase-out, via
// /calc/traditional-ira-deduction. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 0 });

const STATUS_CLS = { full: 'pos', partial: '', none: 'neg' };
const VIEW = 'traditional-ira-deduction';
let lastReport = null;
let lastBody = null;

export async function renderTraditionalIraDeduction(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.tradira.h1.title">// TRADITIONAL IRA DEDUCTION</span></h1>
        <p class="muted small" data-i18n="view.tradira.hint.intro">
            How much of a traditional IRA contribution you can deduct. If neither you nor your spouse
            is covered by a workplace plan, it's fully deductible at any income. If you're covered,
            it phases out by MAGI. 2026 ranges: covered single $81k–$91k, covered married-joint
            $129k–$149k, spouse-covered $242k–$252k. What isn't deductible can still be contributed
            nondeductibly. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.tradira.h2.inputs">Your situation</h2>
            <form id="tradira-form" class="inline-form">
                <label><span data-i18n="view.tradira.label.magi">Modified AGI ($)</span>
                    <input type="number" step="0.01" min="0" name="magi_usd" value="85000" required></label>
                <label><span data-i18n="view.tradira.label.status">Filing status</span>
                    <select name="filing_status">
                        <option value="single" data-i18n="view.tradira.status.single">Single / HoH</option>
                        <option value="married_joint" data-i18n="view.tradira.status.mfj">Married filing jointly</option>
                        <option value="married_separate" data-i18n="view.tradira.status.mfs">Married filing separately</option>
                    </select></label>
                <label><span data-i18n="view.tradira.label.covered">You are covered by a workplace plan</span>
                    <input type="checkbox" name="covered_by_plan" checked></label>
                <label><span data-i18n="view.tradira.label.spouse">Spouse is covered (you are not)</span>
                    <input type="checkbox" name="spouse_covered"></label>
                <label><span data-i18n="view.tradira.label.age50">Age 50 or older (catch-up)</span>
                    <input type="checkbox" name="age_50_plus"></label>
                <label><span data-i18n="view.tradira.label.base">Base limit ($)</span>
                    <input type="number" step="1" min="0" name="base_limit_usd" value="7500"></label>
                <label><span data-i18n="view.tradira.label.catchup">Catch-up ($)</span>
                    <input type="number" step="1" min="0" name="catch_up_usd" value="1100"></label>
            </form>
            <div id="tradira-tools" class="ce-toolbar"></div>
        </div>
        <div id="tradira-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#tradira-form');
    const hashIn = enh.readHashInputs();
    enh.prefillForm(form, hashIn);
    ['covered_by_plan', 'spouse_covered', 'age_50_plus'].forEach((k) => {
        if (k in hashIn) form.querySelector(`[name=${k}]`).checked = hashIn[k] === 'true';
    });
    const readBody = () => {
        const fd = new FormData(form);
        return {
            magi_usd: Number(fd.get('magi_usd')) || 0,
            filing_status: fd.get('filing_status'),
            covered_by_plan: form.querySelector('[name=covered_by_plan]').checked,
            spouse_covered: form.querySelector('[name=spouse_covered]').checked,
            age_50_plus: form.querySelector('[name=age_50_plus]').checked,
            base_limit_usd: Number(fd.get('base_limit_usd')) || 0,
            catch_up_usd: Number(fd.get('catch_up_usd')) || 0,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcTraditionalIraDeduction(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.tradira.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#tradira-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'traditional-ira-deduction.csv' });
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['deductible_usd', r.deductible_usd],
        ['max_contribution_usd', r.max_contribution_usd],
        ['nondeductible_usd', r.nondeductible_usd],
        ['phaseout_start_usd', r.phaseout_start_usd],
        ['phaseout_end_usd', r.phaseout_end_usd],
        ['status', r.status],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#tradira-result');
    const cls = STATUS_CLS[r.status] ?? '';
    // Line chart: deductible contribution across MAGI 0 -> $300k (the phase-out ramp).
    const xs = enh.linspace(0, 300000, 16);
    const pts = await Promise.all(xs.map(async (m) => {
        const rr = await api.calcTraditionalIraDeduction({ ...body, magi_usd: m });
        return { x: m / 1000, y: rr ? rr.deductible_usd : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'MAGI $k', ylabel: 'deductible $' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.tradira.h2.result">What you can deduct</h2>
            <div class="cards">
                <div class="card ${cls}"><div class="label" data-i18n="view.tradira.card.deductible">Deductible</div>
                    <div class="value ${cls}">${money(r.deductible_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.tradira.card.max">Max contribution</div>
                    <div class="value">${money(r.max_contribution_usd)}</div></div>
                <div class="card ${cls}"><div class="label" data-i18n="view.tradira.card.status">Status</div>
                    <div class="value ${cls}" data-i18n="view.tradira.status.${r.status}">—</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.tradira.row.max">Max contribution</td><td>${money(r.max_contribution_usd)}</td></tr>
                    <tr><td data-i18n="view.tradira.row.nondeductible">Nondeductible portion</td><td>${money(r.nondeductible_usd)}</td></tr>
                    <tr><td data-i18n="view.tradira.row.start">Phase-out start</td><td>${money(r.phaseout_start_usd)}</td></tr>
                    <tr><td data-i18n="view.tradira.row.end">Phase-out end</td><td>${money(r.phaseout_end_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.tradira.row.deductible">Deductible contribution</td><td>${money(r.deductible_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
