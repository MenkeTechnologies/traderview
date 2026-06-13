// Interest coverage — Times Interest Earned, EBITDA coverage, and fixed-charge
// coverage from the income statement, via /calc/interest-coverage. Live.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 0 });
const cov = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '×');

export async function renderInterestCoverage(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.intcov.h1.title">// INTEREST COVERAGE</span></h1>
        <p class="muted small" data-i18n="view.intcov.hint.intro">
            Corporate solvency from the income statement: how many times earnings cover the firm's
            interest and fixed charges. Times-interest-earned (EBIT ÷ interest) above ~2.5 is
            generally comfortable; below ~1.5 the firm is straining. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.intcov.h2.inputs">The income statement</h2>
            <form id="intcov-form" class="inline-form">
                <label><span data-i18n="view.intcov.label.ebit">EBIT ($)</span>
                    <input type="number" step="0.01" name="ebit_usd" value="500" required></label>
                <label><span data-i18n="view.intcov.label.da">Depreciation & amortization ($)</span>
                    <input type="number" step="0.01" min="0" name="depreciation_amortization_usd" value="100"></label>
                <label><span data-i18n="view.intcov.label.interest">Interest expense ($)</span>
                    <input type="number" step="0.01" min="0" name="interest_expense_usd" value="200" required></label>
                <label><span data-i18n="view.intcov.label.lease">Lease payments ($)</span>
                    <input type="number" step="0.01" min="0" name="lease_payments_usd" value="50"></label>
            </form>
        </div>
        <div id="intcov-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#intcov-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            ebit_usd: Number(fd.get('ebit_usd')) || 0,
            depreciation_amortization_usd: Number(fd.get('depreciation_amortization_usd')) || 0,
            interest_expense_usd: Number(fd.get('interest_expense_usd')) || 0,
            lease_payments_usd: Number(fd.get('lease_payments_usd')) || 0,
        };
        try {
            const r = await api.calcInterestCoverage(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.intcov.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#intcov-result');
    const tieClass = r.covers_interest ? 'pos' : 'neg';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.intcov.h2.result">The coverage</h2>
            <div class="cards">
                <div class="card ${tieClass}"><div class="label" data-i18n="view.intcov.card.tie">Times interest earned</div>
                    <div class="value ${tieClass}">${cov(r.times_interest_earned)}</div></div>
                <div class="card"><div class="label" data-i18n="view.intcov.card.ebitda">EBITDA coverage</div>
                    <div class="value">${cov(r.ebitda_interest_coverage)}</div></div>
                <div class="card"><div class="label" data-i18n="view.intcov.card.fccr">Fixed-charge coverage</div>
                    <div class="value">${cov(r.fixed_charge_coverage)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.intcov.row.ebitda">EBITDA</td><td>${money(r.ebitda_usd)}</td></tr>
                    <tr><td data-i18n="view.intcov.row.tie">Times interest earned</td><td>${cov(r.times_interest_earned)}</td></tr>
                    <tr><td data-i18n="view.intcov.row.ebitdacov">EBITDA interest coverage</td><td>${cov(r.ebitda_interest_coverage)}</td></tr>
                    <tr><td data-i18n="view.intcov.row.fccr">Fixed-charge coverage</td><td>${cov(r.fixed_charge_coverage)}</td></tr>
                    <tr class="${tieClass}"><td data-i18n="view.intcov.row.covers">Covers interest?</td><td>${r.covers_interest ? t('view.intcov.yes') : t('view.intcov.no')}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
