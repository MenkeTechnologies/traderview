// Paper-account rebalancer. Stores named target weight sets per user,
// computes drift between current paper positions and target weights,
// surfaces "above tolerance" warnings, and suggests trades to bring the
// portfolio back to target.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderPaperRebalance(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.paper_rebalance.title">// PAPER REBALANCER</span></h1>
        <p class="muted small" data-i18n-html="view.paper_rebalance.intro">
            Define per-user named target weight sets (e.g. "60/40", "Boglehead 3-fund",
            "magic-formula top 20"); the rebalancer computes current vs target drift
            for each, surfaces "above tolerance" warnings, and suggests trades to
            return to target. <strong>drift_threshold_pct</strong> controls when the
            UI flags "rebalance recommended" (default 5%).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.paper_rebalance.h2.add">Add target set</h2>
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <input type="text" id="pr-name" placeholder="${esc(t('view.paper_rebalance.field.name_ph'))}">
                <input type="number" id="pr-cash" step="0.5" min="0" max="100" placeholder="${esc(t('view.paper_rebalance.field.cash_ph'))}" value="0">
                <input type="number" id="pr-drift" step="0.5" min="0.5" max="50" placeholder="${esc(t('view.paper_rebalance.field.drift_ph'))}" value="5">
                <input type="number" id="pr-maxtrades" step="1" min="1" max="100" placeholder="${esc(t('view.paper_rebalance.field.maxtrades_ph'))}" value="20">
            </div>
            <textarea id="pr-targets" rows="4" style="width:100%;font-family:monospace;font-size:12px" placeholder='${esc(t('view.paper_rebalance.field.targets_ph'))}'></textarea>
            <div style="margin-top:8px">
                <button class="btn btn-sm primary" id="pr-save" data-i18n="view.paper_rebalance.btn.save">💾 Save Target Set</button>
                <span class="muted small" id="pr-meta"></span>
            </div>
            <h2 style="margin-top:1rem" data-i18n="view.paper_rebalance.h2.list">Saved target sets</h2>
            <div id="pr-list"></div>
            <h2 style="margin-top:1rem" data-i18n="view.paper_rebalance.h2.plan">Rebalance plan</h2>
            <div id="pr-plan"></div>
        </div>
    `;
    mount.querySelector('#pr-save').addEventListener('click', () => saveTarget(mount));
    await loadList(mount);
}

async function saveTarget(mount) {
    const meta = mount.querySelector('#pr-meta');
    const name = mount.querySelector('#pr-name').value.trim();
    const cash = parseFloat(mount.querySelector('#pr-cash').value) || 0;
    const drift = parseFloat(mount.querySelector('#pr-drift').value) || 5;
    const maxTrades = parseInt(mount.querySelector('#pr-maxtrades').value, 10) || 20;
    const targetsText = mount.querySelector('#pr-targets').value.trim();
    if (!name || !targetsText) {
        if (meta) meta.textContent = t('view.paper_rebalance.status.name_and_targets_required');
        return;
    }
    let targets;
    try {
        targets = JSON.parse(targetsText);
    } catch (e) {
        if (meta) meta.textContent = `${t('view.paper_rebalance.status.invalid_json')}: ${e.message}`;
        return;
    }
    try {
        await api.request('/paper-rebalance/targets', {
            method: 'POST',
            body: JSON.stringify({
                name,
                targets,
                cash_target_pct: cash,
                drift_threshold_pct: drift,
                max_trades: maxTrades,
            }),
        });
        if (meta) meta.textContent = t('view.paper_rebalance.status.saved');
        await loadList(mount);
    } catch (e) {
        if (meta) meta.textContent = `${t('common.error')}: ${String(e)}`;
    }
}

async function loadList(mount) {
    const list = mount.querySelector('#pr-list');
    try {
        const targets = await api.request('/paper-rebalance/targets');
        if (!targets || !targets.length) {
            list.innerHTML = `<p class="muted">${esc(t('view.paper_rebalance.empty.no_targets'))}</p>`;
            return;
        }
        list.innerHTML = `
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.paper_rebalance.th.name">Name</th>
                    <th data-i18n="view.paper_rebalance.th.symbols">Symbols</th>
                    <th data-i18n="view.paper_rebalance.th.drift_threshold">Drift threshold</th>
                    <th data-i18n="view.paper_rebalance.th.actions">Actions</th>
                </tr></thead>
                <tbody>${targets.map(t => `
                    <tr>
                        <td><strong>${esc(t.name)}</strong></td>
                        <td class="muted small">${Object.keys(t.targets || {}).join(', ')}</td>
                        <td>${t.drift_threshold_pct.toFixed(1)}%</td>
                        <td>
                            <button class="btn btn-sm primary" data-plan="${t.id}">Plan</button>
                            <button class="btn btn-sm" data-delete="${t.id}">Delete</button>
                        </td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
        list.querySelectorAll('[data-plan]').forEach(b => {
            b.addEventListener('click', () => loadPlan(mount, b.getAttribute('data-plan')));
        });
        list.querySelectorAll('[data-delete]').forEach(b => {
            b.addEventListener('click', () => deleteTarget(mount, b.getAttribute('data-delete')));
        });
    } catch (e) {
        list.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}

async function deleteTarget(mount, id) {
    try {
        await api.request(`/paper-rebalance/targets/${id}`, { method: 'DELETE' });
        await loadList(mount);
    } catch (e) {
        const meta = mount.querySelector('#pr-meta');
        if (meta) meta.textContent = `${t('common.error')}: ${String(e)}`;
    }
}

async function loadPlan(mount, id) {
    const planEl = mount.querySelector('#pr-plan');
    planEl.innerHTML = `<p class="muted">${esc(t('view.paper_rebalance.status.computing'))}</p>`;
    try {
        const r = await api.request(`/paper-rebalance/plan/${id}`, { method: 'POST' });
        const driftCls = r.above_threshold ? 'neg' : 'pos';
        const trades = r.plan.trades || [];
        planEl.innerHTML = `
            <h3>${esc(r.target.name)} · ${esc(t('view.paper_rebalance.field.max_drift'))}:
                <strong class="${driftCls}">${r.max_drift_pct.toFixed(2)}%</strong>
                <span class="muted small">(${esc(t('view.paper_rebalance.field.threshold'))} ${r.target.drift_threshold_pct.toFixed(1)}%)</span>
            </h3>
            <p class="${driftCls} small">
                ${r.above_threshold
                    ? esc(t('view.paper_rebalance.status.above_threshold'))
                    : esc(t('view.paper_rebalance.status.within_tolerance'))}
            </p>
            <table class="trades">
                <thead><tr>
                    <th>Symbol</th>
                    <th>Current %</th>
                    <th>Target %</th>
                    <th>Drift %</th>
                    <th>Trade Qty</th>
                    <th>Trade $</th>
                    <th>Side</th>
                </tr></thead>
                <tbody>${r.plan.rows.map(row => {
                    const sideCls = row.side === 'buy' ? 'pos' : row.side === 'sell' ? 'neg' : 'muted';
                    const driftAbs = Math.abs(row.drift_pct);
                    const driftCls = driftAbs >= r.target.drift_threshold_pct ? 'neg' : '';
                    return `<tr>
                        <td><strong>${esc(row.symbol)}</strong></td>
                        <td>${row.current_pct.toFixed(2)}</td>
                        <td>${row.target_pct.toFixed(2)}</td>
                        <td class="${driftCls}">${row.drift_pct >= 0 ? '+' : ''}${row.drift_pct.toFixed(2)}</td>
                        <td>${row.trade_qty}</td>
                        <td>$${row.trade_value.toFixed(0)}</td>
                        <td class="${sideCls}">${esc(row.side)}</td>
                    </tr>`;
                }).join('')}</tbody>
            </table>
            ${r.plan.warnings.length ? `<ul>${r.plan.warnings.map(w => `<li class="neg small">${esc(w)}</li>`).join('')}</ul>` : ''}
            <p class="muted small">
                ${esc(t('view.paper_rebalance.field.trade_summary')
                    .replace('{n}', trades.length)
                    .replace('{v}', '$' + r.plan.total_trade_value.toFixed(0)))}
            </p>
        `;
    } catch (e) {
        planEl.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
