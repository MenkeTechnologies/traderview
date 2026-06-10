// Credit utilization tracker. Per-card balance / limit → utilization %,
// status (good ≤ 10 / ok ≤ 30 / high > 30), recommended paydown.
// Plus aggregate utilization, status, count above 30%, total paydown
// needed.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

const STATE = {
    cards: [
        { name: 'Chase Sapphire', balance_usd: 850,  limit_usd: 8000 },
        { name: 'Amex Platinum',  balance_usd: 1200, limit_usd: 25000 },
        { name: 'Citi Costco',    balance_usd: 3200, limit_usd: 5000 },
        { name: 'Capital One',    balance_usd: 0,    limit_usd: 3000 },
    ],
};

export async function renderCreditUtilization(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.credit_utilization.title">// CREDIT UTILIZATION</span></h1>
        <p class="muted small" data-i18n-html="view.credit_utilization.intro">
            FICO weights credit utilization (revolving balance / credit limit) heavily —
            "amounts owed" is ~30% of the score, and utilization is the dominant component.
            Standard thresholds: <strong>≤ 30%</strong> minimum to avoid damage,
            <strong>≤ 10%</strong> Experian-published "excellent", <strong>~1%</strong>
            observed FICO 800-club median (with at least one card reporting a balance).
            Per-card cards above 30% can ding the score even if aggregate is fine.
        </p>
        <div class="chart-panel">
            <h2>${esc(t('view.credit_utilization.h2.cards'))}</h2>
            <table class="trades" id="cu-table">
                <thead><tr>
                    <th data-i18n="view.credit_utilization.th.name">Card</th>
                    <th data-i18n="view.credit_utilization.th.balance">Balance $</th>
                    <th data-i18n="view.credit_utilization.th.limit">Limit $</th>
                    <th></th>
                </tr></thead>
                <tbody id="cu-body"></tbody>
            </table>
            <button class="btn btn-sm" id="cu-add" data-i18n="view.credit_utilization.btn.add">＋ Add card</button>
            <div style="margin-top:1rem">
                <button class="btn btn-sm primary" id="cu-run" data-shortcut="r" data-i18n="view.credit_utilization.btn.run">⚡ Compute Utilization</button>
            </div>
            <div id="cu-result"></div>
        </div>
    `;
    drawRows(mount);
    mount.querySelector('#cu-add').addEventListener('click', () => {
        STATE.cards.push({ name: 'New card', balance_usd: 0, limit_usd: 1000 });
        drawRows(mount);
    });
    mount.querySelector('#cu-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

function drawRows(mount) {
    const body = mount.querySelector('#cu-body');
    body.innerHTML = STATE.cards.map((c, i) => `
        <tr>
            <td><input type="text" data-k="name" data-i="${i}" value="${esc(c.name)}" style="width:100%"></td>
            <td><input type="number" step="25" min="0" data-k="balance_usd" data-i="${i}" value="${c.balance_usd}" style="width:100%"></td>
            <td><input type="number" step="500" min="0" data-k="limit_usd" data-i="${i}" value="${c.limit_usd}" style="width:100%"></td>
            <td><button class="btn btn-xs" data-del="${i}" data-tip="common.tip.remove_row" data-i18n-aria-label="common.aria.remove" aria-label="Remove">✕</button></td>
        </tr>
    `).join('');
    body.querySelectorAll('input').forEach(inp => {
        inp.addEventListener('input', () => {
            const i = parseInt(inp.dataset.i, 10);
            const k = inp.dataset.k;
            STATE.cards[i][k] = k === 'name' ? inp.value : (parseFloat(inp.value) || 0);
        });
    });
    body.querySelectorAll('button[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            STATE.cards.splice(parseInt(btn.dataset.del, 10), 1);
            drawRows(mount);
        });
    });
}

async function runCompute(mount) {
    const result = mount.querySelector('#cu-result');
    result.innerHTML = `<p class="muted">${esc(t('view.credit_utilization.status.computing'))}</p>`;
    try {
        const r = await api.request('/credit-utilization/compute', { method: 'POST', body: JSON.stringify(STATE) });
        const stCls = s => s === 'good' ? 'pos' : s === 'high' ? 'neg' : '';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.credit_utilization.field.aggregate'))}</div>
                    <strong class="${stCls(r.aggregate_status)}" style="font-size:1.4em">${r.aggregate_utilization_pct.toFixed(1)}%</strong>
                    <div class="muted small" style="text-transform:uppercase">${esc(t('view.credit_utilization.util.' + r.aggregate_status) || r.aggregate_status)}</div></div>
                <div><div class="muted small">${esc(t('view.credit_utilization.field.total_balance'))}</div>
                    <strong>$${r.total_balance_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.credit_utilization.field.total_limit'))}</div>
                    <strong>$${(r.total_limit_usd / 1000).toFixed(1)}K</strong></div>
                <div><div class="muted small">${esc(t('view.credit_utilization.field.above_30'))}</div>
                    <strong class="${r.cards_above_30_count > 0 ? 'neg' : 'pos'}">${r.cards_above_30_count}</strong></div>
                <div><div class="muted small">${esc(t('view.credit_utilization.field.paydown'))}</div>
                    <strong class="${r.total_paydown_recommended_usd > 0 ? 'neg' : 'pos'}">$${r.total_paydown_recommended_usd.toFixed(0)}</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.credit_utilization.h2.per_card'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.credit_utilization.th.name">Card</th>
                    <th data-i18n="view.credit_utilization.th.util">Utilization</th>
                    <th data-i18n="view.credit_utilization.th.status">Status</th>
                    <th data-i18n="view.credit_utilization.th.paydown">Paydown to 30%</th>
                </tr></thead>
                <tbody>${(r.cards || []).map(c => `
                    <tr>
                        <td><strong>${esc(c.name)}</strong></td>
                        <td>${c.utilization_pct.toFixed(1)}% <span class="muted small">($${c.balance_usd.toFixed(0)} / $${c.limit_usd.toFixed(0)})</span></td>
                        <td class="${stCls(c.status)}" style="text-transform:uppercase"><strong>${esc(t('view.credit_utilization.util.' + c.status) || c.status)}</strong></td>
                        <td>${c.recommended_paydown_to_30_usd > 0 ? '$' + c.recommended_paydown_to_30_usd.toFixed(0) : '—'}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
