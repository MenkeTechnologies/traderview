// Zero-based budget (Dave Ramsey / YNAB style). Every dollar of
// monthly income is assigned a job; the leftover after planning must
// be 0. Reports total planned, leftover, status, plus per-category
// variance (actual − planned) for the second half of the month.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

const STATE = {
    monthly_income_usd: 6000,
    categories: [
        { name: 'Rent / mortgage',  planned_usd: 1800, actual_usd: 1800 },
        { name: 'Groceries',        planned_usd: 600,  actual_usd: 580 },
        { name: 'Utilities',        planned_usd: 200,  actual_usd: 210 },
        { name: 'Transportation',   planned_usd: 300,  actual_usd: 280 },
        { name: 'Insurance',        planned_usd: 250,  actual_usd: 250 },
        { name: '401k / IRA',       planned_usd: 1000, actual_usd: 1000 },
        { name: 'Brokerage',        planned_usd: 500,  actual_usd: 500 },
        { name: 'Sinking funds',    planned_usd: 400,  actual_usd: 400 },
        { name: 'Entertainment',    planned_usd: 200,  actual_usd: 240 },
        { name: 'Misc / buffer',    planned_usd: 750,  actual_usd: 700 },
    ],
};

export async function renderZeroBasedBudget(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.zero_based_budget.title">// ZERO-BASED BUDGET</span></h1>
        <p class="muted small" data-i18n-html="view.zero_based_budget.intro">
            Dave Ramsey / YNAB rule: <strong>every dollar of monthly income is assigned
            a job before the month begins</strong>, so income − Σ planned = 0. Reports
            <strong>total planned</strong>, <strong>leftover</strong> (positive = unassigned,
            negative = over-allocated), and per-category <strong>variance</strong>
            (actual − planned) once you fill in actuals after the month.
        </p>
        <div class="chart-panel">
            <div style="display:flex;gap:8px;align-items:end;margin-bottom:12px">
                <label style="flex:0 0 240px">
                    <span class="muted small" data-i18n="view.zero_based_budget.field.income">Monthly income $</span>
                    <input type="number" id="zbb-income" step="100" min="0" value="${STATE.monthly_income_usd}" style="width:100%">
                </label>
                <button class="btn btn-sm primary" id="zbb-run" data-shortcut="r" data-i18n="view.zero_based_budget.btn.run">⚡ Compute Budget</button>
            </div>
            <h2>${esc(t('view.zero_based_budget.h2.categories'))}</h2>
            <table class="trades" id="zbb-table">
                <thead><tr>
                    <th data-i18n="view.zero_based_budget.th.name">Category</th>
                    <th data-i18n="view.zero_based_budget.th.planned">Planned $</th>
                    <th data-i18n="view.zero_based_budget.th.actual">Actual $</th>
                    <th></th>
                </tr></thead>
                <tbody id="zbb-body"></tbody>
            </table>
            <button class="btn btn-sm" id="zbb-add" data-i18n="view.zero_based_budget.btn.add">＋ Add category</button>
            <div id="zbb-result"></div>
        </div>
    `;
    drawRows(mount);
    mount.querySelector('#zbb-income').addEventListener('input', e => {
        STATE.monthly_income_usd = parseFloat(e.target.value) || 0;
    });
    mount.querySelector('#zbb-add').addEventListener('click', () => {
        STATE.categories.push({ name: 'New category', planned_usd: 0, actual_usd: 0 });
        drawRows(mount);
    });
    mount.querySelector('#zbb-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

function drawRows(mount) {
    const body = mount.querySelector('#zbb-body');
    body.innerHTML = STATE.categories.map((c, i) => `
        <tr>
            <td><input type="text" data-k="name" data-i="${i}" value="${esc(c.name)}" style="width:100%"></td>
            <td><input type="number" step="25" min="0" data-k="planned_usd" data-i="${i}" value="${c.planned_usd}" style="width:100%"></td>
            <td><input type="number" step="25" min="0" data-k="actual_usd" data-i="${i}" value="${c.actual_usd}" style="width:100%"></td>
            <td><button class="btn btn-xs" data-del="${i}" data-tip="common.tip.remove_row" data-i18n-aria-label="common.aria.remove" aria-label="Remove">✕</button></td>
        </tr>
    `).join('');
    body.querySelectorAll('input').forEach(inp => {
        inp.addEventListener('input', () => {
            const i = parseInt(inp.dataset.i, 10);
            const k = inp.dataset.k;
            STATE.categories[i][k] = k === 'name' ? inp.value : (parseFloat(inp.value) || 0);
        });
    });
    body.querySelectorAll('button[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            STATE.categories.splice(parseInt(btn.dataset.del, 10), 1);
            drawRows(mount);
        });
    });
}

async function runCompute(mount) {
    const result = mount.querySelector('#zbb-result');
    result.innerHTML = `<p class="muted">${esc(t('view.zero_based_budget.status.computing'))}</p>`;
    try {
        const r = await api.request('/zero-based-budget/compute', { method: 'POST', body: JSON.stringify(STATE) });
        const leftoverCls = r.is_zero_based ? 'pos' : r.leftover_usd > 0 ? '' : 'neg';
        const statusKey = r.status.replace(/-/g, '_');
        const statusCls = r.status === 'zero-based' ? 'pos' : r.status === 'over-allocated' ? 'neg' : '';
        const varCls = r.total_variance_usd > 0 ? 'neg' : r.total_variance_usd < 0 ? 'pos' : '';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.zero_based_budget.field.total_planned'))}</div>
                    <strong>$${r.total_planned_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.zero_based_budget.field.total_actual'))}</div>
                    <strong>$${r.total_actual_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.zero_based_budget.field.leftover'))}</div>
                    <strong class="${leftoverCls}" style="font-size:1.4em">$${r.leftover_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.zero_based_budget.field.variance'))}</div>
                    <strong class="${varCls}">$${r.total_variance_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.zero_based_budget.field.status'))}</div>
                    <strong class="${statusCls}" style="text-transform:uppercase">${esc(t('view.zero_based_budget.status.' + statusKey) || r.status)}</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.zero_based_budget.h2.variance'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.zero_based_budget.th.name">Category</th>
                    <th data-i18n="view.zero_based_budget.th.planned">Planned $</th>
                    <th data-i18n="view.zero_based_budget.th.actual">Actual $</th>
                    <th data-i18n="view.zero_based_budget.th.variance">Variance $</th>
                    <th data-i18n="view.zero_based_budget.th.variance_pct">Variance %</th>
                </tr></thead>
                <tbody>${(r.categories || []).map(c => `
                    <tr>
                        <td><strong>${esc(c.name)}</strong></td>
                        <td>$${c.planned_usd.toFixed(0)}</td>
                        <td>$${c.actual_usd.toFixed(0)}</td>
                        <td class="${c.variance_usd > 0 ? 'neg' : c.variance_usd < 0 ? 'pos' : ''}">$${c.variance_usd.toFixed(0)}</td>
                        <td>${c.variance_pct == null ? '—' : c.variance_pct.toFixed(1) + '%'}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
