// Envelope budgeting (digital simulation of the cash-envelope method).
// Per envelope: period allotment, starting balance, spent this period,
// rollover flag. Reports remaining, usage %, status (ok / warning /
// empty), next-period balance, plus aggregate totals + overall status.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

const STATE = {
    envelopes: [
        { name: 'Groceries',     period_allotment_usd: 500, starting_balance_usd: 500, spent_this_period_usd: 380, rollover: false },
        { name: 'Dining out',    period_allotment_usd: 200, starting_balance_usd: 200, spent_this_period_usd: 140, rollover: false },
        { name: 'Gas',           period_allotment_usd: 250, starting_balance_usd: 250, spent_this_period_usd: 175, rollover: false },
        { name: 'Clothing',      period_allotment_usd: 100, starting_balance_usd: 320, spent_this_period_usd: 60,  rollover: true },
        { name: 'Christmas',     period_allotment_usd: 100, starting_balance_usd: 500, spent_this_period_usd: 0,   rollover: true },
        { name: 'Entertainment', period_allotment_usd: 150, starting_balance_usd: 150, spent_this_period_usd: 165, rollover: false },
    ],
};

export async function renderEnvelopeBudget(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.envelope_budget.title">// ENVELOPE BUDGET</span></h1>
        <p class="muted small" data-i18n-html="view.envelope_budget.intro">
            Digital simulation of the cash-envelope method (Larry Burkett / Crown Financial,
            1970s). Each envelope holds a period allotment + any rollover from prior periods.
            Status: <span class="pos">ok</span> (usage &lt; 75%) /
            <span>warning</span> (≥ 75 &lt; 100%) / <span class="neg">empty</span> (≥ 100%).
            <strong>Rollover</strong> envelopes carry the leftover into next period;
            <strong>non-rollover</strong> envelopes reset (use-it-or-lose-it).
        </p>
        <div class="chart-panel">
            <h2>${esc(t('view.envelope_budget.h2.envelopes'))}</h2>
            <table class="trades" id="eb-table">
                <thead><tr>
                    <th data-i18n="view.envelope_budget.th.name">Envelope</th>
                    <th data-i18n="view.envelope_budget.th.allotment">Allotment $</th>
                    <th data-i18n="view.envelope_budget.th.start">Starting $</th>
                    <th data-i18n="view.envelope_budget.th.spent">Spent $</th>
                    <th data-i18n="view.envelope_budget.th.rollover">Rollover?</th>
                    <th></th>
                </tr></thead>
                <tbody id="eb-body"></tbody>
            </table>
            <button class="btn btn-sm" id="eb-add" data-i18n="view.envelope_budget.btn.add">＋ Add envelope</button>
            <div style="margin-top:1rem">
                <button class="btn btn-sm primary" id="eb-run" data-shortcut="r" data-i18n="view.envelope_budget.btn.run">⚡ Compute Envelopes</button>
            </div>
            <div id="eb-result"></div>
        </div>
    `;
    drawRows(mount);
    mount.querySelector('#eb-add').addEventListener('click', () => {
        STATE.envelopes.push({ name: 'New envelope', period_allotment_usd: 100, starting_balance_usd: 100, spent_this_period_usd: 0, rollover: false });
        drawRows(mount);
    });
    mount.querySelector('#eb-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

function drawRows(mount) {
    const body = mount.querySelector('#eb-body');
    body.innerHTML = STATE.envelopes.map((e, i) => `
        <tr>
            <td><input type="text" data-k="name" data-i="${i}" value="${esc(e.name)}" style="width:100%"></td>
            <td><input type="number" step="25" min="0" data-k="period_allotment_usd" data-i="${i}" value="${e.period_allotment_usd}" style="width:100%"></td>
            <td><input type="number" step="25" data-k="starting_balance_usd" data-i="${i}" value="${e.starting_balance_usd}" style="width:100%"></td>
            <td><input type="number" step="10" min="0" data-k="spent_this_period_usd" data-i="${i}" value="${e.spent_this_period_usd}" style="width:100%"></td>
            <td style="text-align:center"><input type="checkbox" data-k="rollover" data-i="${i}" ${e.rollover ? 'checked' : ''}></td>
            <td><button class="btn btn-xs" data-del="${i}">✕</button></td>
        </tr>
    `).join('');
    body.querySelectorAll('input').forEach(inp => {
        inp.addEventListener('input', () => {
            const i = parseInt(inp.dataset.i, 10);
            const k = inp.dataset.k;
            if (k === 'rollover') STATE.envelopes[i][k] = inp.checked;
            else if (k === 'name') STATE.envelopes[i][k] = inp.value;
            else STATE.envelopes[i][k] = parseFloat(inp.value) || 0;
        });
    });
    body.querySelectorAll('button[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            STATE.envelopes.splice(parseInt(btn.dataset.del, 10), 1);
            drawRows(mount);
        });
    });
}

async function runCompute(mount) {
    const result = mount.querySelector('#eb-result');
    result.innerHTML = `<p class="muted">${esc(t('view.envelope_budget.status.computing'))}</p>`;
    try {
        const r = await api('/envelope-budget/compute', { method: 'POST', body: JSON.stringify(STATE) });
        const overallCls = r.overall_status === 'healthy' ? 'pos'
                         : r.overall_status === 'envelope_empty' ? 'neg' : '';
        const stCls = s => s === 'ok' ? 'pos' : s === 'empty' ? 'neg' : '';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.envelope_budget.field.total_allotment'))}</div>
                    <strong>$${r.total_allotment_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.envelope_budget.field.total_starting'))}</div>
                    <strong>$${r.total_starting_balance_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.envelope_budget.field.total_spent'))}</div>
                    <strong>$${r.total_spent_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.envelope_budget.field.total_remaining'))}</div>
                    <strong class="${r.total_remaining_usd >= 0 ? 'pos' : 'neg'}">$${r.total_remaining_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.envelope_budget.field.empty'))}</div>
                    <strong class="${r.envelopes_empty_count > 0 ? 'neg' : 'pos'}">${r.envelopes_empty_count}</strong></div>
                <div><div class="muted small">${esc(t('view.envelope_budget.field.warning'))}</div>
                    <strong>${r.envelopes_warning_count}</strong></div>
                <div><div class="muted small">${esc(t('view.envelope_budget.field.overall'))}</div>
                    <strong class="${overallCls}" style="text-transform:uppercase">${esc(t('view.envelope_budget.status.' + r.overall_status) || r.overall_status)}</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.envelope_budget.h2.per_envelope'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.envelope_budget.th.name">Envelope</th>
                    <th data-i18n="view.envelope_budget.th.remaining">Remaining</th>
                    <th data-i18n="view.envelope_budget.th.usage">Usage</th>
                    <th data-i18n="view.envelope_budget.th.status">Status</th>
                    <th data-i18n="view.envelope_budget.th.rollover_h">Rollover?</th>
                    <th data-i18n="view.envelope_budget.th.next">Next period $</th>
                </tr></thead>
                <tbody>${(r.envelopes || []).map(env => `
                    <tr>
                        <td><strong>${esc(env.name)}</strong></td>
                        <td class="${env.remaining_usd < 0 ? 'neg' : ''}">$${env.remaining_usd.toFixed(0)}</td>
                        <td>${env.usage_pct.toFixed(0)}%</td>
                        <td class="${stCls(env.status)}" style="text-transform:uppercase">${esc(t('view.envelope_budget.status.' + env.status) || env.status)}</td>
                        <td>${env.rollover ? '✓' : '—'}</td>
                        <td>$${env.next_period_balance_usd.toFixed(0)}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
