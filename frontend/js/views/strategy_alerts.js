// Compound strategy alerts (AND/OR/NOT over price + indicators).
// Rules are persisted as a tree AST; this UI exposes a flat rules list
// where each rule's AST is editable as raw JSON (with a templates dropdown
// for the common shapes). Server-side evaluation runs every 60s.

import { api } from '../api.js';
import { esc } from '../util.js';
import { on as onWsEvent } from '../ws.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let wsUnsub = null;

const TEMPLATES = {
    'RSI oversold + above 200d SMA': {
        kind: 'and',
        left:  { kind: 'leaf', symbol: 'AAPL', metric: { kind: 'rsi', period: 14 }, op: 'lt', value: 30 },
        right: { kind: 'leaf', symbol: 'AAPL', metric: { kind: 'price' }, op: 'gt', value: 0 },
    },
    'Breakout above 50d high': {
        kind: 'leaf', symbol: 'SPY',
        metric: { kind: 'pct_of_high', period: 50 }, op: 'ge', value: 1.0,
    },
    'VIX spike + sell-off': {
        kind: 'and',
        left:  { kind: 'leaf', symbol: '^VIX', metric: { kind: 'quote' }, op: 'gt', value: 25 },
        right: { kind: 'leaf', symbol: 'SPY', metric: { kind: 'change_pct', days: 1 }, op: 'lt', value: -2 },
    },
    '5% drop in last 5 days': {
        kind: 'leaf', symbol: 'AAPL',
        metric: { kind: 'change_pct', days: 5 }, op: 'le', value: -5,
    },
};

export async function renderStrategyAlerts(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// STRATEGY ALERTS</h1>
        <p class="muted small">Compound AND/OR/NOT rules over price + RSI/SMA/EMA + change-pct +
            pct-of-high + ^VIX / ^CPC / ^ADD quote leaves. Server evaluates every 60s and fires
            on the false→true edge so a persistent condition won't spam. Connected webhooks
            fan-out on every fire. AST is JSON; pick a template to start.</p>

        <div class="chart-panel">
            <form id="sa-form" class="inline-form">
                <input name="name" placeholder="rule name" required style="min-width:240px;">
                <label>Template
                    <select name="template">
                        <option value="">(custom)</option>
                        ${Object.keys(TEMPLATES).map(k => `<option>${esc(k)}</option>`).join('')}
                    </select>
                </label>
                <button class="primary" type="submit">Create</button>
            </form>
            <textarea id="sa-ast" rows="10"
                style="width:100%;font-family:'Share Tech Mono',monospace;font-size:11px;background:#070714;color:#cfd2e8;border:1px solid var(--border);padding:8px;margin-top:8px;"
                placeholder='AST JSON, e.g. {"kind":"leaf","symbol":"AAPL","metric":{"kind":"price"},"op":"gt","value":200}'></textarea>
        </div>

        <div class="chart-panel">
            <h2>Active rules</h2>
            <div id="sa-list"><div class="boot">loading…</div></div>
            <button id="sa-eval-now" class="btn">Evaluate now</button>
            <span id="sa-status" class="muted small" style="margin-left:8px;"></span>
        </div>

        <div class="chart-panel">
            <h2>Recent fires</h2>
            <div id="sa-fires"></div>
        </div>
    `;
    mount.querySelector('#sa-form [name=template]').addEventListener('change', (e) => {
        const t = e.target.value;
        if (t && TEMPLATES[t]) {
            const ast = mount.querySelector('#sa-ast');
            if (ast) ast.value = JSON.stringify(TEMPLATES[t], null, 2);
        }
    });
    mount.querySelector('#sa-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        let ast;
        try { ast = JSON.parse(mount.querySelector('#sa-ast').value); }
        catch (err) { alert('AST JSON invalid: ' + err.message); return; }
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
        } catch (err) { alert(err.message); }
    });
    mount.querySelector('#sa-eval-now').addEventListener('click', async () => {
        const status = mount.querySelector('#sa-status');
        if (status) status.textContent = 'evaluating…';
        try {
            const r = await api.strategyAlertsEvaluateNow();
            if (!viewIsCurrent(tok)) return;
            const status2 = mount.querySelector('#sa-status');
            if (status2) status2.textContent = `${r.evaluated} evaluated · ${r.fired} fired · ${r.errors} errors`;
            await refresh(mount, tok);
        } catch (e) {
            if (!viewIsCurrent(tok)) return;
            const status2 = mount.querySelector('#sa-status');
            if (status2) status2.textContent = 'error: ' + e.message;
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
    if (!rules.length) { el.innerHTML = '<p class="muted small">No rules yet.</p>'; return; }
    el.innerHTML = `<table class="trades">
        <thead><tr>
            <th>Name</th><th>Enabled</th><th>Last truth</th><th>Fires</th>
            <th>Last eval</th><th>Last fired</th><th>Error</th><th></th>
        </tr></thead>
        <tbody>
        ${rules.map(r => `<tr>
            <td>${esc(r.name)}</td>
            <td class="${r.enabled ? 'pos' : 'muted'}">${r.enabled ? 'on' : 'off'}</td>
            <td>${r.last_truth == null ? '—' : (r.last_truth ? '✓' : '✗')}</td>
            <td>${r.fire_count}</td>
            <td class="small">${r.last_evaluated_at ? new Date(r.last_evaluated_at).toLocaleString() : '—'}</td>
            <td class="small">${r.last_fired_at ? new Date(r.last_fired_at).toLocaleString() : '—'}</td>
            <td class="small neg">${esc(r.last_eval_error || '')}</td>
            <td>
                <button class="btn sa-toggle" data-id="${r.id}">${r.enabled ? 'Disable' : 'Enable'}</button>
                <button class="btn sa-del" data-id="${r.id}">Delete</button>
            </td>
        </tr>
        <tr><td colspan="8"><pre class="muted small" style="margin:0;font-size:10px;background:#070714;padding:6px;overflow:auto;">${esc(JSON.stringify(r.ast))}</pre></td></tr>
        `).join('')}
        </tbody></table>`;
    el.querySelectorAll('.sa-del').forEach(b =>
        b.addEventListener('click', async () => {
            if (!confirm('Delete this rule?')) return;
            try {
                await api.deleteStrategyAlert(b.dataset.id);
                if (!viewIsCurrent(tok)) return;
                await refresh(mount, tok);
            }
            catch (e) { alert(e.message); }
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
            } catch (e) { alert(e.message); }
        }));
}

function renderFires(fires, rules, mount) {
    const el = mount.querySelector('#sa-fires');
    if (!el) return;
    if (!fires.length) { el.innerHTML = '<p class="muted small">No fires yet.</p>'; return; }
    const nameOf = (id) => rules.find(r => r.id === id)?.name || id;
    el.innerHTML = `<table class="trades">
        <thead><tr><th>When</th><th>Rule</th><th>Snapshot</th></tr></thead>
        <tbody>
        ${fires.map(f => `<tr>
            <td class="small">${new Date(f.fired_at).toLocaleString()}</td>
            <td>${esc(nameOf(f.alert_id))}</td>
            <td class="small muted">${esc(JSON.stringify(f.snapshot?.leaves || []).slice(0, 240))}…</td>
        </tr>`).join('')}
        </tbody></table>`;
}
