// 401(k) Per-Paycheck Maximizer — the even deferral to max by year-end,
// and the match you forfeit by front-loading, via /calc/paycheck-401k.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const FIELDS = [
    ['annual_limit_usd', 'Annual deferral limit ($)', 23000],
    ['ytd_contributed_usd', 'Contributed YTD ($)', 0],
    ['pay_periods_remaining', 'Pay periods remaining', 24],
    ['gross_per_period_usd', 'Gross pay per period ($)', 4000],
    ['employer_match_pct', 'Employer match (¢ per $)', 50],
    ['match_limit_pct_of_pay', 'Match up to (% of pay)', 5],
    ['planned_per_period_usd', 'Planned deferral per period ($, 0 = skip)', 0],
];
const INT_FIELDS = new Set(['pay_periods_remaining']);

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });

export async function renderPaycheck401k(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.paycheck401k.h1.title">// 401(k) PER-PAYCHECK MAXIMIZER</span></h1>
        <p class="muted small" data-i18n="view.paycheck401k.hint.intro">
            How much per paycheck to hit the annual 401(k) limit by December — and whether
            you're about to leave free money on the table. Most plans match per paycheck, so
            front-loading hits the limit early and stops your contributions (and the match)
            for the rest of the year, unless the plan trues up the missed match. Spreading
            evenly captures the full match every period.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.paycheck401k.h2.inputs">Your numbers</h2>
            <form id="pc-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.paycheck401k.label.${key}">${label}</span>
                        <input type="number" step="${INT_FIELDS.has(key) ? '1' : '0.01'}" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
                <label data-tip="view.paycheck401k.tip.true_up"><input type="checkbox" name="plan_has_true_up"> <span data-i18n="view.paycheck401k.label.true_up">Plan trues up match</span></label>
                <button class="primary" type="submit" data-i18n="view.paycheck401k.btn.run">Calculate</button>
            </form>
        </div>
        <div id="pc-result"></div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#pc-form');
    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = { plan_has_true_up: fd.get('plan_has_true_up') != null };
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcPaycheck401k(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.paycheck401k.toast.error'), { level: 'error' });
        }
    });
    form.dispatchEvent(new Event('submit'));
}

function renderResult(mount, r) {
    const el = mount.querySelector('#pc-result');
    const forfeited = Number(r.forfeited_match_usd);
    const ptl = r.periods_to_limit_at_planned;
    let note;
    if (forfeited > 0) {
        note = `<span class="neg">${t('view.paycheck401k.note.forfeit', { amount: money(forfeited), periods: r.empty_periods })}</span>`;
    } else if (ptl != null && r.empty_periods === 0) {
        note = `<span class="pos" data-i18n="view.paycheck401k.note.ok">Your planned deferral spreads to year-end — full match every check.</span>`;
    } else if (ptl != null) {
        note = `<span class="pos" data-i18n="view.paycheck401k.note.trueup">You hit the limit early, but the plan's true-up pays the missed match.</span>`;
    } else {
        note = `<span data-i18n="view.paycheck401k.note.even">Defer the even amount to capture the match every period.</span>`;
    }
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.paycheck401k.h2.result">The plan</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.paycheck401k.card.remaining">Remaining to limit</div>
                    <div class="value">${money(r.remaining_to_limit_usd)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.paycheck401k.card.even">Defer per period</div>
                    <div class="value">${money(r.even_per_period_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.paycheck401k.card.pct">% of each check</div>
                    <div class="value">${Number(r.even_pct_of_pay).toFixed(1)}%</div></div>
                <div class="card"><div class="label" data-i18n="view.paycheck401k.card.threshold">Threshold for full match</div>
                    <div class="value">${money(r.match_threshold_per_period_usd)}/check</div></div>
                <div class="card pos"><div class="label" data-i18n="view.paycheck401k.card.match">Match per check</div>
                    <div class="value">${money(r.full_match_per_period_usd)}</div></div>
                <div class="card ${forfeited > 0 ? 'neg' : ''}"><div class="label" data-i18n="view.paycheck401k.card.forfeit">Match forfeited (front-load)</div>
                    <div class="value ${forfeited > 0 ? 'neg' : ''}">${money(forfeited)}</div></div>
            </div>
            <p class="muted small">${note}</p>
        </div>
    `;
    applyUiI18n(el);
}
