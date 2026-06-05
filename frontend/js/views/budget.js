// Personal finance / budgeting view (#budget).
//
// Per-category monthly limits + a single savings goal. Reads live
// month snapshots from /api/budget/snapshot — actual spend per
// category, income, expense, net, savings rate, target met flag.
//
// Layout:
//   Header           — month picker, savings-goal input.
//   Summary strip    — Income / Expense / Net / Savings rate / Target met?
//   Category table   — spent / limit / progress bar / over flag / edit / delete.
//   Add row          — pick a category + monthly limit → PUT, refresh.

import { api } from '../api.js';
import { t } from '../i18n.js';
import { esc, fmtMoney, applyBarWidths } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const STATE = {
    snapshot: null,
    categories: [],
    year: new Date().getFullYear(),
    month: new Date().getMonth() + 1,
};

export async function renderBudget(mount) {
    const tok = currentViewToken();

    // Resolve ?year=&month=
    try {
        const hash = location.hash.replace(/^#/, '');
        const q = hash.includes('?') ? hash.slice(hash.indexOf('?') + 1) : '';
        const params = new URLSearchParams(q);
        const y = parseInt(params.get('year'), 10);
        const m = parseInt(params.get('month'), 10);
        if (!Number.isNaN(y)) STATE.year = y;
        if (!Number.isNaN(m) && m >= 1 && m <= 12) STATE.month = m;
    } catch (_) {}

    mount.innerHTML = `
        <div class="bg-shell">
            <header class="bg-head">
                <h2>${esc(t('view.budget.title'))}</h2>
                <select id="bg-year"></select>
                <select id="bg-month"></select>
                <span class="bg-target-wrap">
                    <label>${esc(t('view.budget.savings_target'))}
                        <input type="number" id="bg-target" step="50" value="0" min="0">
                    </label>
                </span>
            </header>
            <div id="bg-summary" class="bg-summary"></div>
            <div id="bg-cats" class="bg-cats"></div>
            <section class="bg-add">
                <h4>${esc(t('view.budget.add.h'))}</h4>
                <label>${esc(t('view.budget.add.category'))}
                    <select id="bg-add-cat"></select>
                </label>
                <label>${esc(t('view.budget.add.limit'))}
                    <input type="number" id="bg-add-lim" min="0" step="10" value="0">
                </label>
                <button type="button" id="bg-add-btn" class="btn btn-primary btn-compact">
                    ${esc(t('view.budget.add.set'))}
                </button>
            </section>
        </div>
    `;

    // Year + month pickers.
    const ysel = mount.querySelector('#bg-year');
    const msel = mount.querySelector('#bg-month');
    const now = new Date().getFullYear();
    let yhtml = '';
    for (let y = now; y >= now - 5; y--) {
        yhtml += `<option value="${y}"${y === STATE.year ? ' selected' : ''}>${y}</option>`;
    }
    ysel.innerHTML = yhtml;
    const MONTHS = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];
    msel.innerHTML = MONTHS.map((m, i) => {
        const v = i + 1;
        return `<option value="${v}"${v === STATE.month ? ' selected' : ''}>${m}</option>`;
    }).join('');
    ysel.addEventListener('change', e => { STATE.year = +e.target.value; loadAndRender(mount, tok); });
    msel.addEventListener('change', e => { STATE.month = +e.target.value; loadAndRender(mount, tok); });

    // Savings-goal input — fire on blur (avoid hammering the endpoint).
    const tgt = mount.querySelector('#bg-target');
    tgt.addEventListener('change', async () => {
        const v = +tgt.value || 0;
        try {
            await api.setSavingsGoal(v);
            await loadAndRender(mount, tok);
        } catch (e) {
            showToast(t('view.budget.err.savings', { err: e.message }), { level: 'error' });
        }
    });

    // Add-budget control.
    mount.querySelector('#bg-add-btn').addEventListener('click', async () => {
        const code = mount.querySelector('#bg-add-cat').value;
        const limit = +mount.querySelector('#bg-add-lim').value || 0;
        if (!code) {
            showToast(t('view.budget.err.pick_cat'), { level: 'warn' });
            return;
        }
        try {
            await api.upsertBudget(code, { monthly_limit: limit });
            showToast(t('view.budget.toast.set', { code }), { level: 'success' });
            await loadAndRender(mount, tok);
        } catch (e) {
            showToast(t('view.budget.err.set', { err: e.message }), { level: 'error' });
        }
    });

    // Categories list is fetched once; expense categories don't change
    // mid-session.
    try {
        STATE.categories = await api.expenseCategories();
    } catch (_) {
        STATE.categories = [];
    }
    populateAddDropdown(mount);

    await loadAndRender(mount, tok);
}

function populateAddDropdown(mount) {
    const sel = mount.querySelector('#bg-add-cat');
    if (!sel) return;
    sel.innerHTML = `<option value="">${esc(t('view.budget.add.pick'))}</option>` +
        STATE.categories
            .map(c => `<option value="${esc(c.code)}">${esc(c.label || c.code)}</option>`)
            .join('');
}

async function loadAndRender(mount, tok) {
    let snap;
    try {
        snap = await api.budgetSnapshot({ year: STATE.year, month: STATE.month });
    } catch (e) {
        const sum = mount.querySelector('#bg-summary');
        if (sum) sum.innerHTML = `<div class="err">${esc(t('view.budget.err.load', { err: e.message }))}</div>`;
        return;
    }
    if (!viewIsCurrent(tok)) return;
    STATE.snapshot = snap;

    // Sync the savings-goal input to whatever's persisted.
    const tgt = mount.querySelector('#bg-target');
    if (tgt) tgt.value = +(snap.monthly_savings_target || 0);

    renderSummary(mount);
    renderCategories(mount, tok);
}

function renderSummary(mount) {
    const s = STATE.snapshot;
    const sum = mount.querySelector('#bg-summary');
    if (!sum || !s) return;
    const netCls = (+s.net || 0) >= 0 ? 'bg-pos' : 'bg-neg';
    const targetCls = s.target_met ? 'bg-pos' : 'bg-neg';
    sum.innerHTML = `
        <div class="bg-stat"><span>${esc(t('view.budget.stat.income'))}</span><strong>${esc(fmtMoney(+s.income || 0))}</strong></div>
        <div class="bg-stat"><span>${esc(t('view.budget.stat.expense'))}</span><strong>${esc(fmtMoney(+s.expense || 0))}</strong></div>
        <div class="bg-stat"><span>${esc(t('view.budget.stat.net'))}</span><strong class="${netCls}">${esc(fmtMoney(+s.net || 0))}</strong></div>
        <div class="bg-stat"><span>${esc(t('view.budget.stat.savings_rate'))}</span><strong class="${netCls}">${(+s.savings_rate || 0).toFixed(1)}%</strong></div>
        <div class="bg-stat"><span>${esc(t('view.budget.stat.target_met'))}</span><strong class="${targetCls}">${s.target_met ? '✓' : '✗'}</strong></div>
        <div class="bg-stat"><span>${esc(t('view.budget.stat.over_budget'))}</span><strong class="${s.over_budget_categories > 0 ? 'bg-neg' : ''}">${s.over_budget_categories}</strong></div>
    `;
}

function renderCategories(mount, tok) {
    const s = STATE.snapshot;
    const wrap = mount.querySelector('#bg-cats');
    if (!wrap || !s) return;
    if (!s.categories || s.categories.length === 0) {
        wrap.innerHTML = `<div class="muted">${esc(t('view.budget.empty'))}</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="bg-table">
            <thead><tr>
                <th>${esc(t('view.budget.col.category'))}</th>
                <th class="num">${esc(t('view.budget.col.spent'))}</th>
                <th class="num">${esc(t('view.budget.col.limit'))}</th>
                <th>${esc(t('view.budget.col.progress'))}</th>
                <th>${esc(t('view.budget.col.actions'))}</th>
            </tr></thead>
            <tbody>
                ${s.categories.map(c => {
                    const pct = Math.min(+c.pct || 0, 100);
                    const overPct = Math.max(0, (+c.pct || 0) - 100);
                    const barCls = c.over ? 'bg-bar-over' : (c.paused ? 'bg-bar-paused' : 'bg-bar-ok');
                    return `<tr data-code="${esc(c.category_code)}" class="${c.over ? 'bg-row-over' : ''}">
                        <td>${esc(c.label || c.category_code)} ${c.paused ? '<span class="bg-paused">paused</span>' : ''}</td>
                        <td class="num">${esc(fmtMoney(+c.spent || 0))}</td>
                        <td class="num">
                            <input type="number" class="bg-lim-input" data-code="${esc(c.category_code)}" value="${+c.monthly_limit || 0}" min="0" step="10">
                        </td>
                        <td>
                            <div class="bg-bar-wrap" title="${(+c.pct || 0).toFixed(1)}%">
                                <div class="bg-bar ${barCls}" data-bar-pct="${pct}"></div>
                                ${overPct > 0 ? `<div class="bg-bar-over-extra" data-bar-pct="${Math.min(overPct, 100)}"></div>` : ''}
                            </div>
                        </td>
                        <td>
                            <button type="button" class="btn btn-secondary btn-compact bg-pause" data-code="${esc(c.category_code)}">
                                ${c.paused ? esc(t('view.budget.action.resume')) : esc(t('view.budget.action.pause'))}
                            </button>
                            <button type="button" class="btn btn-secondary btn-compact bg-del" data-code="${esc(c.category_code)}">${esc(t('view.budget.action.delete'))}</button>
                        </td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
    // Bar widths via rAF — Tauri release WebKit strips inline
    // `style="width:..."` from innerHTML-inserted nodes; the JS DOM
    // assignment survives. `applyBarWidths` walks every
    // `[data-bar-pct]` descendant of `wrap`.
    applyBarWidths(wrap);

    // Wire limit edits — debounced PUT on blur.
    wrap.querySelectorAll('input.bg-lim-input').forEach(inp => {
        inp.addEventListener('change', async () => {
            const code = inp.dataset.code;
            const limit = +inp.value || 0;
            try {
                await api.upsertBudget(code, { monthly_limit: limit });
                await loadAndRender(mount, tok);
            } catch (e) {
                showToast(t('view.budget.err.set', { err: e.message }), { level: 'error' });
            }
        });
    });
    wrap.querySelectorAll('button.bg-pause').forEach(btn => {
        btn.addEventListener('click', async () => {
            const code = btn.dataset.code;
            const cur = s.categories.find(c => c.category_code === code);
            if (!cur) return;
            try {
                await api.upsertBudget(code, { monthly_limit: cur.monthly_limit, paused: !cur.paused });
                await loadAndRender(mount, tok);
            } catch (e) {
                showToast(t('view.budget.err.set', { err: e.message }), { level: 'error' });
            }
        });
    });
    wrap.querySelectorAll('button.bg-del').forEach(btn => {
        btn.addEventListener('click', async () => {
            const code = btn.dataset.code;
            try {
                await api.deleteBudget(code);
                await loadAndRender(mount, tok);
            } catch (e) {
                showToast(t('view.budget.err.del', { err: e.message }), { level: 'error' });
            }
        });
    });
}
