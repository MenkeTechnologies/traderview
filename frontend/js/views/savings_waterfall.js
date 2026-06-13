// Savings Waterfall — the financial order of operations. Allocate a
// month's available savings down the priority ladder (employer match,
// high-interest debt, emergency fund, tax-advantaged, then taxable) via
// /calc/savings-waterfall.

import { api } from '../api.js';
import { esc } from '../util.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const STEP_LABEL = {
    starter_emergency: 'Starter emergency buffer',
    employer_match: 'Capture employer match',
    high_interest_debt: 'High-interest debt payoff',
    full_emergency: 'Full emergency fund (3–6 mo)',
    tax_advantaged: 'Tax-advantaged (HSA / Roth)',
    max_retirement: 'Max retirement (401k / IRA)',
    taxable_brokerage: 'Taxable brokerage',
};

const FIELDS = [
    ['monthly_available', 'Monthly savings available ($)', 2000],
    ['starter_emergency_gap', 'Starter buffer remaining ($)', 1000],
    ['employer_match_monthly', 'Monthly to full employer match ($)', 500],
    ['high_interest_debt', 'High-interest debt balance ($)', 3000],
    ['full_emergency_gap', 'Full emergency fund remaining ($)', 12000],
    ['tax_advantaged_room_monthly', 'HSA + Roth room this month ($)', 1000],
    ['retirement_room_monthly', '401k/IRA room this month ($)', 1500],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });

export async function renderSavingsWaterfall(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.waterfall.h1.title">// SAVINGS WATERFALL</span></h1>
        <p class="muted small" data-i18n="view.waterfall.hint.intro">
            The financial order of operations: where the next dollar goes. Each month's
            savings flows down the ladder — secure the employer match (free money) and a
            starter buffer before chasing returns, kill high-interest debt before a taxable
            brokerage, fill tax-advantaged space (HSA, Roth) before ordinary investing —
            with whatever is left landing in a taxable account.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.waterfall.h2.inputs">Your numbers</h2>
            <form id="wf-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.waterfall.label.${key}">${label}</span>
                        <input type="number" step="1" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
                <button class="primary" type="submit" data-i18n="view.waterfall.btn.plan">Build waterfall</button>
            </form>
        </div>
        <div id="wf-result"></div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#wf-form');
    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const plan = await api.calcSavingsWaterfall(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, plan);
        } catch (err) {
            showToast(err.message || t('view.waterfall.toast.error'), { level: 'error' });
        }
    });
    // Auto-run once with the defaults so the ladder is visible on open.
    form.dispatchEvent(new Event('submit'));
}

function renderResult(mount, plan) {
    const el = mount.querySelector('#wf-result');
    const total = Number(plan.total_allocated);
    const rows = plan.allocations.map((a) => {
        const amt = Number(a.amount);
        const pct = total > 0 ? (amt / total) * 100 : 0;
        return `<tr>
            <td>${esc(STEP_LABEL[a.step] || a.step)}</td>
            <td>${money(amt)}</td>
            <td class="muted">${pct.toFixed(1)}%</td>
        </tr>`;
    }).join('');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.waterfall.h2.plan">This month's allocation</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.waterfall.card.allocated">Allocated</div>
                    <div class="value">${money(plan.total_allocated)}</div></div>
            </div>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.waterfall.th.step">Priority</th>
                    <th data-i18n="view.waterfall.th.amount">Amount</th>
                    <th data-i18n="view.waterfall.th.share">Share</th>
                </tr></thead>
                <tbody>${rows || `<tr><td colspan="3" class="muted" data-i18n="view.waterfall.empty">Nothing to allocate.</td></tr>`}</tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
