// Marriage penalty / bonus — joint tax vs two single filers, via
// /calc/marriage-penalty. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 }));
const VIEW = 'marriage-penalty';
let lastReport = null;
let lastBody = null;
const signed = (n) => (n == null ? '—' : (n >= 0 ? '+$' : '−$') + Math.abs(Number(n)).toLocaleString(undefined, { maximumFractionDigits: 0 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');

export async function renderMarriagePenalty(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.marriage.h1.title">// MARRIAGE PENALTY / BONUS</span></h1>
        <p class="muted small" data-i18n="view.marriage.hint.intro">
            Running each spouse's income through the single brackets and the combined income through the
            joint brackets shows whether marrying costs more (penalty) or less (bonus). On the 2026
            schedule the brackets are 2× the single ones through 32%, so a penalty only appears at very
            high combined income; unequal incomes usually produce a bonus. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.marriage.h2.inputs">The couple</h2>
            <form id="marriage-form" class="inline-form">
                <label><span data-i18n="view.marriage.label.a">Spouse A income ($)</span>
                    <input type="number" step="1000" min="0" name="spouse_a_income_usd" value="120000" required></label>
                <label><span data-i18n="view.marriage.label.b">Spouse B income ($)</span>
                    <input type="number" step="1000" min="0" name="spouse_b_income_usd" value="60000" required></label>
                <label><span data-i18n="view.marriage.label.std_single">Single standard deduction ($)</span>
                    <input type="number" step="100" min="0" name="std_deduction_single_usd" value="16100"></label>
                <label><span data-i18n="view.marriage.label.std_mfj">MFJ standard deduction ($)</span>
                    <input type="number" step="100" min="0" name="std_deduction_mfj_usd" value="32200"></label>
            </form>
            <div id="marriage-tools" class="ce-toolbar"></div>
            <button type="button" id="marriage-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="marriage-sens" class="ce-sens"></div>
        </div>
        <div id="marriage-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#marriage-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        return {
            spouse_a_income_usd: Number(fd.get('spouse_a_income_usd')) || 0,
            spouse_b_income_usd: Number(fd.get('spouse_b_income_usd')) || 0,
            std_deduction_single_usd: Number(fd.get('std_deduction_single_usd')) || 0,
            std_deduction_mfj_usd: Number(fd.get('std_deduction_mfj_usd')) || 0,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcMarriagePenalty(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.marriage.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#marriage-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'marriage-penalty.csv' });
    mount.querySelector('#marriage-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['marriage_penalty_usd', r.marriage_penalty_usd],
        ['joint_tax_usd', r.joint_tax_usd],
        ['single_total_tax_usd', r.single_total_tax_usd],
        ['spouse_a_tax_usd', r.spouse_a_tax_usd],
        ['spouse_b_tax_usd', r.spouse_b_tax_usd],
        ['joint_effective_rate_pct', r.joint_effective_rate_pct],
    ];
}

async function renderResult(mount, r, _body, _tok) {
    const el = mount.querySelector('#marriage-result');
    // Joint tax vs two-single total — the comparison the penalty/bonus comes from.
    const chart = enh.svgBarChart([
        { label: 'Joint', value: r.joint_tax_usd },
        { label: '2 singles', value: r.single_total_tax_usd },
    ]);
    // Penalty is bad (neg card), bonus is good (pos card).
    const verdictCls = r.is_penalty ? 'neg' : (r.is_bonus ? 'pos' : '');
    const verdictKey = r.is_penalty ? 'view.marriage.verdict.penalty'
        : (r.is_bonus ? 'view.marriage.verdict.bonus' : 'view.marriage.verdict.neutral');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.marriage.h2.result">The verdict</h2>
            <div class="cards">
                <div class="card ${verdictCls}"><div class="label" data-i18n="${verdictKey}">—</div>
                    <div class="value ${verdictCls}">${signed(r.marriage_penalty_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.marriage.card.joint">Tax filing jointly</div>
                    <div class="value">${money(r.joint_tax_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.marriage.card.single">Tax as two singles</div>
                    <div class="value">${money(r.single_total_tax_usd)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.marriage.row.combined">Combined income</td><td>${money(r.combined_income_usd)}</td></tr>
                    <tr><td data-i18n="view.marriage.row.a_tax">Spouse A tax (single)</td><td>${money(r.spouse_a_tax_usd)}</td></tr>
                    <tr><td data-i18n="view.marriage.row.b_tax">Spouse B tax (single)</td><td>${money(r.spouse_b_tax_usd)}</td></tr>
                    <tr><td data-i18n="view.marriage.row.single_total">Two-single total</td><td>${money(r.single_total_tax_usd)}</td></tr>
                    <tr><td data-i18n="view.marriage.row.joint">Joint tax</td><td>${money(r.joint_tax_usd)}</td></tr>
                    <tr><td data-i18n="view.marriage.row.joint_eff">Joint effective rate</td><td>${pct(r.joint_effective_rate_pct)}</td></tr>
                    <tr><td data-i18n="view.marriage.row.single_eff">Two-single effective rate</td><td>${pct(r.single_effective_rate_pct)}</td></tr>
                    <tr class="emph ${verdictCls}"><td data-i18n="view.marriage.row.penalty">Marriage penalty (+) / bonus (−)</td><td>${signed(r.marriage_penalty_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#marriage-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: spouse A income 0 -> 400k; y: spouse B income 0 -> 400k. Output: penalty(+)/bonus(-).
    const xVals = enh.linspace(0, 400000, 5);
    const yVals = enh.linspace(0, 400000, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'spouse_a_income_usd', yKey: 'spouse_b_income_usd', xVals, yVals, compute: (b) => api.calcMarriagePenalty(b), pick: (r) => (r ? r.marriage_penalty_usd : null) });
    if (!viewIsCurrent(tok)) return;
    // Bonus (negative penalty) is good, so negate for shading (green = bonus).
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells: cells.map((row) => row.map((v) => (v == null ? null : -v))), fmt: (v) => (v == null ? '—' : (-v >= 0 ? '+$' : '-$') + Math.abs(Math.round(-v / 1000)) + 'k'), xfmt: (v) => '$' + Math.round(v / 1000) + 'k', yfmt: (v) => '$' + Math.round(v / 1000) + 'k', xName: t('view.marriage.label.a') || 'A', yName: t('view.marriage.label.b') || 'B' });
}
