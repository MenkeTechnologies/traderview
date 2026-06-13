// Roth bracket-fill — convert just enough to top off a tax bracket: the
// headroom to the ceiling (capped at the balance) and the tax it triggers,
// via /calc/roth-bracket-fill. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['current_taxable_income_usd', 'Current taxable income ($)', 60000],
    ['bracket_ceiling_usd', 'Bracket ceiling ($)', 100000],
    ['marginal_rate_pct', 'Bracket marginal rate (%)', 22],
    ['traditional_balance_usd', 'Traditional balance ($, 0 = no cap)', 0],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });

export async function renderRothBracketFill(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.rbf.h1.title">// ROTH BRACKET-FILL</span></h1>
        <p class="muted small" data-i18n="view.rbf.hint.intro">
            In a low-income year, convert traditional IRA dollars to Roth up to the top of your
            current tax bracket — but not a dollar more, which would spill into the next, higher
            bracket. This fills the cheap bracket now to avoid larger RMDs taxed higher later.
            Enter your taxable income, the bracket ceiling, and its rate (look up your year's
            brackets); the conversion is capped at your traditional balance. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.rbf.h2.inputs">This year</h2>
            <form id="rbf-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.rbf.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="rbf-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#rbf-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcRothBracketFill(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.rbf.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#rbf-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.rbf.h2.result">The conversion</h2>
            <div class="cards">
                <div class="card ${r.already_at_ceiling ? 'neg' : 'pos'}"><div class="label" data-i18n="view.rbf.card.convert">Convert</div>
                    <div class="value ${r.already_at_ceiling ? 'neg' : 'pos'}">${money(r.conversion_amount_usd)}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.rbf.card.tax">Tax this year</div>
                    <div class="value neg">${money(r.conversion_tax_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.rbf.card.headroom">Bracket headroom</div>
                    <div class="value">${money(r.headroom_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.rbf.card.newincome">New taxable income</div>
                    <div class="value">${money(r.new_taxable_income_usd)}</div></div>
            </div>
            ${r.already_at_ceiling ? `<p class="muted small neg" data-i18n="view.rbf.warn.over">Income already meets or exceeds the bracket ceiling — no room to convert at this rate.</p>` : ''}
        </div>
    `;
    applyUiI18n(el);
}
