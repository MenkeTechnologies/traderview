// Compound strategy alerts (AND/OR/NOT over price + indicators).
// Rules are persisted as a tree AST; this UI exposes a flat rules list
// where each rule's AST is editable as raw JSON (with a templates dropdown
// for the common shapes). Server-side evaluation runs every 60s.

import { api } from '../api.js';
import { esc } from '../util.js';
import { on as onWsEvent } from '../ws.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

let wsUnsub = null;

const TEMPLATES = [
    { id: 'rsi_oversold_200d', ast: {
        kind: 'and',
        left:  { kind: 'leaf', symbol: 'AAPL', metric: { kind: 'rsi', period: 14 }, op: 'lt', value: 30 },
        right: { kind: 'leaf', symbol: 'AAPL', metric: { kind: 'price' }, op: 'gt', value: 0 },
    }},
    { id: 'breakout_50d', ast: {
        kind: 'leaf', symbol: 'SPY',
        metric: { kind: 'pct_of_high', period: 50 }, op: 'ge', value: 1.0,
    }},
    { id: 'vix_spike', ast: {
        kind: 'and',
        left:  { kind: 'leaf', symbol: '^VIX', metric: { kind: 'quote' }, op: 'gt', value: 25 },
        right: { kind: 'leaf', symbol: 'SPY', metric: { kind: 'change_pct', days: 1 }, op: 'lt', value: -2 },
    }},
    { id: 'drop_5pct_5d', ast: {
        kind: 'leaf', symbol: 'AAPL',
        metric: { kind: 'change_pct', days: 5 }, op: 'le', value: -5,
    }},
];

export async function renderStrategyAlerts(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.strategy_alerts.h1.strategy_alerts" class="view-title">// STRATEGY ALERTS</h1>
        <p data-i18n="view.strategy_alerts.hint.compound_and_or_not_rules_over_price_rsi_sma_ema_c" class="muted small">Compound AND/OR/NOT rules over price + RSI/SMA/EMA + change-pct +
            pct-of-high + ^VIX / ^CPC / ^ADD quote leaves. Server evaluates every 60s and fires
            on the false→true edge so a persistent condition won't spam. Connected webhooks
            fan-out on every fire. AST is JSON; pick a template to start.</p>

        <div class="chart-panel">
            <form id="sa-form" class="inline-form">
                <input name="name" placeholder="rule name" data-i18n-placeholder="view.strategy_alerts.placeholder.name"
                       required style="min-width:240px;">
                <label><span data-i18n="view.strategy_alerts.label.template">Template</span>
                    <select name="template">
                        <option data-i18n="view.strategy_alerts.opt.custom" value="">(custom)</option>
                        ${TEMPLATES.map(tpl => `<option value="${tpl.id}" data-i18n="view.strategy_alerts.template.${tpl.id}">${esc(t(`view.strategy_alerts.template.${tpl.id}`))}</option>`).join('')}
                    </select>
                </label>
                <button data-i18n="view.strategy_alerts.btn.create" class="primary" type="submit">Create</button>
            </form>
            <textarea id="sa-ast" rows="10"
                style="width:100%;font-family:'Share Tech Mono',monospace;font-size:11px;background:#070714;color:#cfd2e8;border:1px solid var(--border);padding:8px;margin-top:8px;"
                data-i18n-placeholder="view.strategy_alerts.placeholder.ast"
                placeholder='AST JSON, e.g. {"kind":"leaf","symbol":"AAPL","metric":{"kind":"price"},"op":"gt","value":200}'></textarea>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.strategy_alerts.h2.active_rules">Active rules</h2>
            <div id="sa-list"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.loading">loading…</div></div></div>
            <button data-i18n="view.strategy_alerts.btn.evaluate_now" id="sa-eval-now" class="btn">Evaluate now</button>
            <span id="sa-status" class="muted small" style="margin-left:8px;"></span>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.strategy_alerts.h2.recent_fires">Recent fires</h2>
            <div id="sa-fires"></div>
        </div>
    `;
    mount.querySelector('#sa-form [name=template]').addEventListener('change', (e) => {
        const tpl = TEMPLATES.find(x => x.id === e.target.value);
        if (tpl) {
            const ast = mount.querySelector('#sa-ast');
            if (ast) ast.value = JSON.stringify(tpl.ast, null, 2);
        }
    });
    mount.querySelector('#sa-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        let ast;
        try { ast = JSON.parse(mount.querySelector('#sa-ast').value); }
        catch (err) { alert(t('view.strategy_alerts.alert.ast_invalid', { msg: err.message })); return; }
        try {
            await api.createStrategyAlert({
                name: fd.get('name').trim(),
                enabled: true, ast, webhook_ids: [],
            });
            if (!viewIsCurrent(tok)) return;
            const astEl = mount.querySelector('#sa-ast');
            if (astEl) astEl.value = '';
            e.target.reset();
            await refresh(mount, tok);
        } catch (err) { alert(t('common.error', { err: err.message })); }
    });
    mount.querySelector('#sa-eval-now').addEventListener('click', async () => {
        const status = mount.querySelector('#sa-status');
        if (status) status.textContent = t('common.status.evaluating');
        try {
            const r = await api.strategyAlertsEvaluateNow();
            if (!viewIsCurrent(tok)) return;
            const status2 = mount.querySelector('#sa-status');
            if (status2) status2.textContent = t('view.strategy_alerts.status.result', { evaluated: r.evaluated, fired: r.fired, errors: r.errors });
            await refresh(mount, tok);
        } catch (e) {
            if (!viewIsCurrent(tok)) return;
            const status2 = mount.querySelector('#sa-status');
            if (status2) status2.textContent = t('common.error', { err: e.message });
        }
    });

    if (wsUnsub) wsUnsub();
    wsUnsub = onWsEvent('alert_fired', () => { if (viewIsCurrent(tok)) refresh(mount, tok); });
    window.addEventListener('hashchange', () => {
        if (!window.location.hash.startsWith('#strategy-alerts')) {
            if (wsUnsub) { wsUnsub(); wsUnsub = null; }
        }
    }, { once: true });

    await refresh(mount, tok);
}

async function refresh(mount, tok) {
    try {
        const [rules, fires] = await Promise.all([
            api.listStrategyAlerts(),
            api.strategyAlertFires(),
        ]);
        if (!viewIsCurrent(tok)) return;
        renderRules(rules, mount, tok);
        renderFires(fires, rules, mount);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#sa-list');
        if (el) el.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function renderRules(rules, mount, tok) {
    const el = mount.querySelector('#sa-list');
    if (!el) return;
    if (!rules.length) { el.innerHTML = '<p data-i18n="view.strategy_alerts.hint.no_rules_yet" class="muted small">No rules yet.</p>'; return; }
    el.innerHTML = `<table class="trades">
        <thead><tr>
            <th data-i18n="view.strategy_alerts.th.name">Name</th><th data-i18n="view.strategy_alerts.th.enabled">Enabled</th><th data-i18n="view.strategy_alerts.th.last_truth">Last truth</th><th data-i18n="view.strategy_alerts.th.fires">Fires</th>
            <th data-i18n="view.strategy_alerts.th.last_eval">Last eval</th><th data-i18n="view.strategy_alerts.th.last_fired">Last fired</th><th data-i18n="view.strategy_alerts.th.error">Error</th><th></th>
        </tr></thead>
        <tbody>
        ${rules.map(r => `<tr>
            <td>${esc(r.name)}</td>
            <td class="${r.enabled ? 'pos' : 'muted'}">${t(r.enabled ? 'common.on_lc' : 'common.off_lc')}</td>
            <td>${r.last_truth == null ? '—' : (r.last_truth ? '✓' : '✗')}</td>
            <td>${r.fire_count}</td>
            <td class="small">${r.last_evaluated_at ? new Date(r.last_evaluated_at).toLocaleString() : '—'}</td>
            <td class="small">${r.last_fired_at ? new Date(r.last_fired_at).toLocaleString() : '—'}</td>
            <td class="small neg">${esc(r.last_eval_error || '')}</td>
            <td>
                <button class="btn sa-toggle" data-id="${r.id}">${r.enabled ? t('common.btn.disable') : t('common.btn.enable')}</button>
                <button data-i18n="view.strategy_alerts.btn.delete" class="btn sa-del" data-id="${r.id}">Delete</button>
            </td>
        </tr>
        <tr><td colspan="8"><pre class="muted small" style="margin:0;font-size:10px;background:#070714;padding:6px;overflow:auto;">${esc(JSON.stringify(r.ast))}</pre></td></tr>
        `).join('')}
        </tbody></table>`;
    el.querySelectorAll('.sa-del').forEach(b =>
        b.addEventListener('click', async () => {
            if (!confirm(t('view.strategy_alerts.confirm.delete'))) return;
            try {
                await api.deleteStrategyAlert(b.dataset.id);
                if (!viewIsCurrent(tok)) return;
                await refresh(mount, tok);
            }
            catch (e) { alert(t('common.error', { err: e.message })); }
        }));
    el.querySelectorAll('.sa-toggle').forEach(b =>
        b.addEventListener('click', async () => {
            const row = (await api.listStrategyAlerts()).find(x => x.id === b.dataset.id);
            if (!viewIsCurrent(tok)) return;
            if (!row) return;
            try {
                await api.updateStrategyAlert(row.id, {
                    name: row.name, ast: row.ast,
                    enabled: !row.enabled, webhook_ids: row.webhook_ids,
                });
                if (!viewIsCurrent(tok)) return;
                await refresh(mount, tok);
            } catch (e) { alert(t('common.error', { err: e.message })); }
        }));
}

function renderFires(fires, rules, mount) {
    const el = mount.querySelector('#sa-fires');
    if (!el) return;
    if (!fires.length) { el.innerHTML = '<p data-i18n="view.strategy_alerts.hint.no_fires_yet" class="muted small">No fires yet.</p>'; return; }
    const nameOf = (id) => rules.find(r => r.id === id)?.name || id;
    el.innerHTML = `<table class="trades">
        <thead><tr><th data-i18n="view.strategy_alerts.th.when">When</th><th data-i18n="view.strategy_alerts.th.rule">Rule</th><th data-i18n="view.strategy_alerts.th.snapshot">Snapshot</th></tr></thead>
        <tbody>
        ${fires.map(f => `<tr>
            <td class="small">${new Date(f.fired_at).toLocaleString()}</td>
            <td>${esc(nameOf(f.alert_id))}</td>
            <td class="small muted">${esc(JSON.stringify(f.snapshot?.leaves || []).slice(0, 240))}…</td>
        </tr>`).join('')}
        </tbody></table>`;
}
