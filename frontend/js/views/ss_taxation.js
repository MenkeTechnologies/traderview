// Taxation of Social Security benefits — provisional-income tiers (0/50/85%),
// via /calc/ss-taxation. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 0 });
const pct = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 1 }) + '%';

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
        </div>
        <div id="sstax-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#sstax-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            social_security_usd: Number(fd.get('social_security_usd')) || 0,
            other_income_usd: Number(fd.get('other_income_usd')) || 0,
            tax_exempt_interest_usd: Number(fd.get('tax_exempt_interest_usd')) || 0,
            filing_status: fd.get('filing_status'),
        };
        try {
            const r = await api.calcSsTaxation(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.sstax.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#sstax-result');
    const tier = TIER[r.tier] || TIER.up_to_85;
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
