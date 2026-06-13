// Marriage penalty / bonus — joint tax vs two single filers, via
// /calc/marriage-penalty. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 }));
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
        </div>
        <div id="marriage-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#marriage-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            spouse_a_income_usd: Number(fd.get('spouse_a_income_usd')) || 0,
            spouse_b_income_usd: Number(fd.get('spouse_b_income_usd')) || 0,
            std_deduction_single_usd: Number(fd.get('std_deduction_single_usd')) || 0,
            std_deduction_mfj_usd: Number(fd.get('std_deduction_mfj_usd')) || 0,
        };
        try {
            const r = await api.calcMarriagePenalty(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.marriage.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#marriage-result');
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
