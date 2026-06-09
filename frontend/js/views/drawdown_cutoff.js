// Drawdown auto-cutoff: when current equity (cash + MTM across alpaca
// + tradier) falls a configurable % below the rolling high-water mark,
// automatically fire the multi-broker kill-switch ONCE. After a fire,
// dormant until the user clicks Reset.

import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { t } from '../i18n.js';

export async function renderDrawdownCutoff(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.drawdown_cutoff.title">// DRAWDOWN AUTO-CUTOFF</span></h1>
        <p class="muted small" data-i18n-html="view.drawdown_cutoff.intro">
            When live broker equity (Alpaca + Tradier) falls a configurable percentage
            below the rolling high-water mark, this automatically fires the multi-broker
            kill-switch — cancelling all working orders and flattening every position
            via the same primitives the manual kill-switch uses.
            <strong>Defaults OFF</strong> — destructive automation is opt-in only.
            After a fire, the rule sleeps until you explicitly <strong>Reset</strong>
            (which clears <code>auto_killed_at</code> and re-seeds the HWM from current
            equity). That prevents a re-fire loop if the kill doesn't fully flatten.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.drawdown_cutoff.h2.config">Config</h2>
            <form id="dc-form" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:12px;margin-bottom:12px">
                <label style="display:flex;align-items:center;gap:8px">
                    <input type="checkbox" id="dc-enabled">
                    <span data-i18n="view.drawdown_cutoff.field.enabled">Enabled</span>
                </label>
                <label>
                    <span class="muted small" data-i18n="view.drawdown_cutoff.field.max_dd">Max drawdown (%)</span>
                    <input type="number" id="dc-max-dd" step="0.5" min="0.5" max="100" style="width:100%">
                </label>
            </form>
            <div style="display:flex;gap:12px;flex-wrap:wrap;margin-bottom:8px">
                <button class="btn btn-sm primary" id="dc-save" data-i18n="view.drawdown_cutoff.btn.save">💾 Save Config</button>
                <button class="btn btn-sm primary" id="dc-eval" data-shortcut="r" data-i18n="view.drawdown_cutoff.btn.evaluate">⚡ Evaluate Now</button>
                <button class="btn btn-sm" id="dc-reset" data-i18n="view.drawdown_cutoff.btn.reset">↺ Reset (after fire)</button>
                <span class="muted small" id="dc-meta"></span>
            </div>
            <div id="dc-state"></div>
            <div id="dc-result"></div>
            <h2 style="margin-top:1rem" data-i18n="view.drawdown_cutoff.h2.log">Evaluation log</h2>
            <table class="trades" id="dc-log">
                <thead><tr>
                    <th data-i18n="view.drawdown_cutoff.th.when">When</th>
                    <th data-i18n="view.drawdown_cutoff.th.equity">Equity</th>
                    <th data-i18n="view.drawdown_cutoff.th.hwm">HWM</th>
                    <th data-i18n="view.drawdown_cutoff.th.dd">Drawdown %</th>
                    <th data-i18n="view.drawdown_cutoff.th.threshold">Threshold %</th>
                    <th data-i18n="view.drawdown_cutoff.th.action">Action</th>
                </tr></thead>
                <tbody><tr><td colspan="6" class="muted" data-i18n="common.loading">loading…</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelector('#dc-save').addEventListener('click', () => saveConfig(mount));
    mount.querySelector('#dc-eval').addEventListener('click', () => evaluateNow(mount));
    mount.querySelector('#dc-reset').addEventListener('click', () => resetRule(mount));
    await loadConfig(mount);
    await loadLog(mount);
}

async function loadConfig(mount) {
    try {
        const c = await api('/drawdown-cutoff/config');
        mount.querySelector('#dc-enabled').checked = !!c.enabled;
        mount.querySelector('#dc-max-dd').value = c.max_drawdown_pct;
        renderState(mount, c);
    } catch (e) {
        mount.querySelector('#dc-result').innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}

function renderState(mount, c) {
    const state = mount.querySelector('#dc-state');
    if (!state) return;
    if (!c.high_water_mark) {
        state.innerHTML = `<p class="muted small">${esc(t('view.drawdown_cutoff.state.no_data'))}</p>`;
        return;
    }
    const fired = !!c.auto_killed_at;
    state.innerHTML = `
        <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px;margin-bottom:8px">
            <div><div class="muted small">${esc(t('view.drawdown_cutoff.field.hwm'))}</div><strong>$${(c.high_water_mark || 0).toFixed(2)}</strong></div>
            <div><div class="muted small">${esc(t('view.drawdown_cutoff.field.last_equity'))}</div><strong>$${(c.last_equity || 0).toFixed(2)}</strong></div>
            <div><div class="muted small">${esc(t('view.drawdown_cutoff.field.last_evaluated'))}</div><strong class="muted small">${esc(c.last_evaluated_at ? fmtDateTime(c.last_evaluated_at) : '—')}</strong></div>
            <div><div class="muted small">${esc(t('view.drawdown_cutoff.field.auto_killed'))}</div><strong class="${fired ? 'neg' : 'muted'}">${esc(c.auto_killed_at ? fmtDateTime(c.auto_killed_at) : '—')}</strong></div>
        </div>
    `;
}

async function saveConfig(mount) {
    const meta = mount.querySelector('#dc-meta');
    const body = {
        enabled: mount.querySelector('#dc-enabled').checked,
        max_drawdown_pct: parseFloat(mount.querySelector('#dc-max-dd').value),
    };
    try {
        const c = await api('/drawdown-cutoff/config', { method: 'PUT', body: JSON.stringify(body) });
        if (meta) meta.textContent = t('view.drawdown_cutoff.status.saved').replace('{t}', fmtDateTime(c.updated_at));
        renderState(mount, c);
    } catch (e) {
        if (meta) meta.textContent = `${t('common.error')}: ${String(e)}`;
    }
}

async function evaluateNow(mount) {
    const meta = mount.querySelector('#dc-meta');
    const result = mount.querySelector('#dc-result');
    if (meta) meta.textContent = t('view.drawdown_cutoff.status.evaluating');
    try {
        const r = await api('/drawdown-cutoff/evaluate', { method: 'POST' });
        renderState(mount, r.config);
        let html = `<p>${esc(t('view.drawdown_cutoff.field.equity'))}: <strong>$${r.current_equity.toFixed(2)}</strong>
            · ${esc(t('view.drawdown_cutoff.field.hwm'))}: <strong>$${r.high_water_mark.toFixed(2)}</strong>
            · ${esc(t('view.drawdown_cutoff.field.dd'))}: <strong>${r.drawdown_pct.toFixed(2)}%</strong>
            · ${esc(t('view.drawdown_cutoff.field.action'))}: <strong class="${actionCls(r.action)}">${esc(r.action)}</strong></p>`;
        if (r.kill_result) {
            html += `<div class="neg">
                <strong>${esc(t('view.drawdown_cutoff.kill.summary'))}</strong>
                <ul>
                    <li>${esc(t('view.drawdown_cutoff.kill.cancelled'))}: ${r.kill_result.cancelled_orders}</li>
                    <li>${esc(t('view.drawdown_cutoff.kill.closed'))}: ${r.kill_result.closed_positions}</li>
                    <li>${esc(t('view.drawdown_cutoff.kill.brokers'))}: ${(r.kill_result.brokers_attempted || []).map(esc).join(', ') || '—'}</li>
                </ul>
            </div>`;
        }
        result.innerHTML = html;
        if (meta) meta.textContent = '';
        await loadLog(mount);
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}

async function resetRule(mount) {
    const meta = mount.querySelector('#dc-meta');
    if (meta) meta.textContent = t('view.drawdown_cutoff.status.resetting');
    try {
        const c = await api('/drawdown-cutoff/reset', { method: 'POST' });
        renderState(mount, c);
        if (meta) meta.textContent = t('view.drawdown_cutoff.status.reset_done');
    } catch (e) {
        if (meta) meta.textContent = `${t('common.error')}: ${String(e)}`;
    }
}

async function loadLog(mount) {
    const tbody = mount.querySelector('#dc-log tbody');
    try {
        const rows = await api('/drawdown-cutoff/log?limit=100');
        if (!rows || !rows.length) {
            tbody.innerHTML = `<tr><td colspan="6" class="muted">${esc(t('view.drawdown_cutoff.empty.no_log'))}</td></tr>`;
            return;
        }
        tbody.innerHTML = rows.map(r => `
            <tr>
                <td class="muted small">${esc(fmtDateTime(r.evaluated_at))}</td>
                <td>$${r.current_equity.toFixed(2)}</td>
                <td>$${r.high_water_mark.toFixed(2)}</td>
                <td class="${r.drawdown_pct >= r.threshold_pct ? 'neg' : ''}">${r.drawdown_pct.toFixed(2)}%</td>
                <td class="muted small">${r.threshold_pct.toFixed(2)}%</td>
                <td class="${actionCls(r.action)}">${esc(r.action)}</td>
            </tr>
        `).join('');
    } catch (e) {
        tbody.innerHTML = `<tr><td colspan="6" class="neg">${esc(String(e))}</td></tr>`;
    }
}

function actionCls(a) {
    if (a === 'fired') return 'neg';
    if (a === 'evaluated') return 'pos';
    if (a && a.startsWith('skipped_')) return 'muted';
    return '';
}
