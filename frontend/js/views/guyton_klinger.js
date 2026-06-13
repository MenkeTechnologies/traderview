// Guyton-Klinger Guardrails — dynamic retirement withdrawals. One year's
// decision: inflation raise, capital-preservation cut, or prosperity
// raise, via /calc/guyton-klinger.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const FIELDS = [
    ['portfolio_value_usd', 'Portfolio value ($)', 1000000],
    ['last_withdrawal_usd', "Last year's withdrawal ($)", 50000],
    ['initial_withdrawal_rate_pct', 'Initial withdrawal rate (%)', 5],
    ['inflation_pct', 'Inflation this year (%)', 3],
    ['guardrail_pct', 'Guardrail band (% of initial)', 20],
    ['adjustment_pct', 'Cut/raise size (%)', 10],
];

const RULE = {
    inflation_adjusted: ['Inflation adjustment', ''],
    frozen_down_year: ['Raise frozen (down year)', ''],
    capital_preservation: ['Capital-preservation CUT', 'neg'],
    prosperity: ['Prosperity RAISE', 'pos'],
};

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });

export async function renderGuytonKlinger(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.guardrails.h1.title">// GUYTON-KLINGER GUARDRAILS</span></h1>
        <p class="muted small" data-i18n="view.guardrails.hint.intro">
            Dynamic retirement withdrawals. A static 4% rule ignores how the portfolio
            actually does; guardrails adjust each year's draw so a bad sequence doesn't
            drain the account and a good one isn't under-spent. Raise with inflation — but
            freeze the raise in a down year; cut 10% if the withdrawal rate climbs past the
            upper guardrail; raise 10% if it falls below the lower one.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.guardrails.h2.inputs">This year</h2>
            <form id="gk-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.guardrails.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
                <label data-tip="view.guardrails.tip.gained"><input type="checkbox" name="portfolio_gained" checked> <span data-i18n="view.guardrails.label.gained">Portfolio gained this year</span></label>
                <button class="primary" type="submit" data-i18n="view.guardrails.btn.run">Decide</button>
            </form>
        </div>
        <div id="gk-result"></div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#gk-form');
    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = { portfolio_gained: fd.get('portfolio_gained') != null };
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcGuytonKlinger(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.guardrails.toast.error'), { level: 'error' });
        }
    });
    form.dispatchEvent(new Event('submit'));
}

function renderResult(mount, r) {
    const el = mount.querySelector('#gk-result');
    const [ruleLabel, ruleCls] = RULE[r.rule] || [r.rule, ''];
    const chg = Number(r.change_vs_last_pct);
    const chgCls = chg > 0 ? 'pos' : chg < 0 ? 'neg' : '';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.guardrails.h2.result">The decision</h2>
            <div class="cards">
                <div class="card ${ruleCls}"><div class="label" data-i18n="view.guardrails.card.rule">Rule triggered</div>
                    <div class="value ${ruleCls}">${ruleLabel}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.guardrails.card.withdrawal">This year's withdrawal</div>
                    <div class="value">${money(r.final_withdrawal_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.guardrails.card.change">Change vs last year</div>
                    <div class="value ${chgCls}">${chg >= 0 ? '+' : ''}${chg.toFixed(1)}%</div></div>
                <div class="card"><div class="label" data-i18n="view.guardrails.card.rate">Withdrawal rate</div>
                    <div class="value">${Number(r.current_rate_pct).toFixed(2)}%</div></div>
                <div class="card"><div class="label" data-i18n="view.guardrails.card.band">Guardrails</div>
                    <div class="value">${Number(r.lower_guardrail_pct).toFixed(1)}% – ${Number(r.upper_guardrail_pct).toFixed(1)}%</div></div>
            </div>
        </div>
    `;
    applyUiI18n(el);
}
