// Sinking-fund planner. Multi-goal budget with target / current /
// months / monthly-contribution per goal. Reports per-goal required
// monthly + months-to-target + shortfall + on-track flag, plus
// aggregate totals + status.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

const STATE = {
    goals: [
        { name: 'Christmas',         target_usd: 1200, current_balance_usd: 200,  target_date_months: 12, monthly_contribution_usd: 100 },
        { name: 'Car insurance',     target_usd: 1800, current_balance_usd: 600,  target_date_months: 6,  monthly_contribution_usd: 200 },
        { name: 'Vacation',          target_usd: 4000, current_balance_usd: 500,  target_date_months: 18, monthly_contribution_usd: 200 },
        { name: 'New laptop',        target_usd: 2500, current_balance_usd: 0,    target_date_months: 12, monthly_contribution_usd: 100 },
        { name: 'Property tax',      target_usd: 6000, current_balance_usd: 1500, target_date_months: 9,  monthly_contribution_usd: 500 },
    ],
};

export async function renderSinkingFund(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.sinking_fund.title">// SINKING FUND PLANNER</span></h1>
        <p class="muted small" data-i18n-html="view.sinking_fund.intro">
            Set aside money each month into named buckets so the cash is there when the lumpy
            expense arrives — Christmas, car insurance, vacation, new laptop, property tax. Per
            goal: <strong>required monthly</strong> = remaining ÷ months until needed,
            <strong>months-to-target</strong> = remaining ÷ your contribution rate,
            <strong>on-track</strong> when contribution ≥ required AND months-to-target ≤ target date.
        </p>
        <div class="chart-panel">
            <h2>${esc(t('view.sinking_fund.h2.goals'))}</h2>
            <table class="trades" id="sf-table">
                <thead><tr>
                    <th data-i18n="view.sinking_fund.th.name">Goal</th>
                    <th data-i18n="view.sinking_fund.th.target">Target $</th>
                    <th data-i18n="view.sinking_fund.th.current">Balance $</th>
                    <th data-i18n="view.sinking_fund.th.months">Months</th>
                    <th data-i18n="view.sinking_fund.th.contribution">Monthly $</th>
                    <th></th>
                </tr></thead>
                <tbody id="sf-body"></tbody>
            </table>
            <button class="btn btn-sm" id="sf-add" data-i18n="view.sinking_fund.btn.add">＋ Add goal</button>
            <div style="margin-top:1rem">
                <button class="btn btn-sm primary" id="sf-run" data-shortcut="r" data-i18n="view.sinking_fund.btn.run">⚡ Compute Plan</button>
            </div>
            <div id="sf-result"></div>
        </div>
    `;
    drawGoals(mount);
    mount.querySelector('#sf-add').addEventListener('click', () => {
        STATE.goals.push({ name: 'New goal', target_usd: 1000, current_balance_usd: 0, target_date_months: 12, monthly_contribution_usd: 50 });
        drawGoals(mount);
    });
    mount.querySelector('#sf-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

function drawGoals(mount) {
    const body = mount.querySelector('#sf-body');
    body.innerHTML = STATE.goals.map((g, i) => `
        <tr>
            <td><input type="text" data-k="name" data-i="${i}" value="${esc(g.name)}" style="width:100%"></td>
            <td><input type="number" step="50" min="0" data-k="target_usd" data-i="${i}" value="${g.target_usd}" style="width:100%"></td>
            <td><input type="number" step="25" min="0" data-k="current_balance_usd" data-i="${i}" value="${g.current_balance_usd}" style="width:100%"></td>
            <td><input type="number" step="1" min="0" data-k="target_date_months" data-i="${i}" value="${g.target_date_months}" style="width:100%"></td>
            <td><input type="number" step="10" min="0" data-k="monthly_contribution_usd" data-i="${i}" value="${g.monthly_contribution_usd}" style="width:100%"></td>
            <td><button class="btn btn-xs" data-del="${i}">✕</button></td>
        </tr>
    `).join('');
    body.querySelectorAll('input').forEach(inp => {
        inp.addEventListener('input', () => {
            const i = parseInt(inp.dataset.i, 10);
            const k = inp.dataset.k;
            STATE.goals[i][k] = k === 'name' ? inp.value : (parseFloat(inp.value) || 0);
        });
    });
    body.querySelectorAll('button[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            STATE.goals.splice(parseInt(btn.dataset.del, 10), 1);
            drawGoals(mount);
        });
    });
}

async function runCompute(mount) {
    const result = mount.querySelector('#sf-result');
    result.innerHTML = `<p class="muted">${esc(t('view.sinking_fund.status.computing'))}</p>`;
    try {
        const r = await api('/sinking-fund/compute', { method: 'POST', body: JSON.stringify(STATE) });
        const statusCls = r.status === 'on-track' ? 'pos' : 'neg';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.sinking_fund.field.total_target'))}</div>
                    <strong style="font-size:1.3em">$${(r.total_target_usd / 1000).toFixed(2)}K</strong></div>
                <div><div class="muted small">${esc(t('view.sinking_fund.field.total_balance'))}</div>
                    <strong>$${(r.total_balance_usd / 1000).toFixed(2)}K</strong></div>
                <div><div class="muted small">${esc(t('view.sinking_fund.field.total_remaining'))}</div>
                    <strong class="${r.total_remaining_usd > 0 ? 'neg' : 'pos'}">$${(r.total_remaining_usd / 1000).toFixed(2)}K</strong></div>
                <div><div class="muted small">${esc(t('view.sinking_fund.field.required'))}</div>
                    <strong>$${r.total_required_monthly_usd.toFixed(0)}/mo</strong></div>
                <div><div class="muted small">${esc(t('view.sinking_fund.field.contributing'))}</div>
                    <strong>$${r.total_monthly_contribution_usd.toFixed(0)}/mo</strong></div>
                <div><div class="muted small">${esc(t('view.sinking_fund.field.shortfall'))}</div>
                    <strong class="${r.aggregate_shortfall_per_month_usd > 0 ? 'neg' : 'pos'}">$${r.aggregate_shortfall_per_month_usd.toFixed(0)}/mo</strong></div>
                <div><div class="muted small">${esc(t('view.sinking_fund.field.status'))}</div>
                    <strong class="${statusCls}" style="text-transform:uppercase">${esc(t('view.sinking_fund.status.' + r.status.replace('-', '_')) || r.status)}</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.sinking_fund.h2.per_goal'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.sinking_fund.th.name">Goal</th>
                    <th data-i18n="view.sinking_fund.th.remaining">Remaining</th>
                    <th data-i18n="view.sinking_fund.th.required_mo">Required/mo</th>
                    <th data-i18n="view.sinking_fund.th.contribution">Your $/mo</th>
                    <th data-i18n="view.sinking_fund.th.mtt">Months to target</th>
                    <th data-i18n="view.sinking_fund.th.shortfall_mo">Shortfall/mo</th>
                    <th data-i18n="view.sinking_fund.th.on_track">On track?</th>
                </tr></thead>
                <tbody>${(r.goals || []).map(g => `
                    <tr>
                        <td><strong>${esc(g.name)}</strong></td>
                        <td>$${g.remaining_usd.toFixed(0)}</td>
                        <td>${g.required_monthly_usd == null ? '—' : '$' + g.required_monthly_usd.toFixed(0)}</td>
                        <td>$${g.monthly_contribution_usd.toFixed(0)}</td>
                        <td>${g.months_to_target_at_rate == null ? '∞' : g.months_to_target_at_rate.toFixed(1)}</td>
                        <td class="${g.shortfall_per_month_usd > 0 ? 'neg' : 'pos'}">$${g.shortfall_per_month_usd.toFixed(0)}</td>
                        <td class="${g.on_track ? 'pos' : 'neg'}">${g.on_track ? '✓' : '✗'}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
