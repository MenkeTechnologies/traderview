// Confluence → paper-trade autopilot. Config form + run-now button +
// recent fire log. Paper account is the safety net; once stats prove
// out, the same wiring can promote to live brokers.

import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { t } from '../i18n.js';

export async function renderConfluenceAutotrade(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.confluence_autotrade.title">// CONFLUENCE AUTOTRADE · PAPER</span></h1>
        <p class="muted small" data-i18n-html="view.confluence_autotrade.intro">
            Wires the confluence dashboard's ranked output directly into your default
            paper account. When a symbol crosses <strong>min_score</strong> AND has at
            least <strong>min_distinct_sources</strong> independent scanners hitting,
            this submits a paper-market buy for <code>notional_usd / quote</code>
            shares. Cooldown prevents re-buying the same hot symbol every tick;
            max-open-positions caps simultaneous exposure. <strong>Run-once</strong>
            below is user-triggered — a cron tick lands in a follow-up commit so you
            can verify wiring before it fires autonomously.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.confluence_autotrade.h2.config">Config</h2>
            <form id="ca-form" class="ca-form" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:12px;margin-bottom:12px">
                <label class="ca-row" style="display:flex;align-items:center;gap:8px">
                    <input type="checkbox" id="ca-enabled">
                    <span data-i18n="view.confluence_autotrade.field.enabled">Enabled</span>
                </label>
                <label>
                    <span class="muted small" data-i18n="view.confluence_autotrade.field.min_score">Min score</span>
                    <input type="number" id="ca-min-score" step="0.5" min="0" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.confluence_autotrade.field.min_distinct">Min distinct sources</span>
                    <input type="number" id="ca-min-distinct" step="1" min="1" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.confluence_autotrade.field.notional">Notional (USD)</span>
                    <input type="number" id="ca-notional" step="100" min="10" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.confluence_autotrade.field.cooldown">Cooldown (min)</span>
                    <input type="number" id="ca-cooldown" step="15" min="0" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.confluence_autotrade.field.max_open">Max open positions</span>
                    <input type="number" id="ca-max-open" step="1" min="1" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.confluence_autotrade.field.sizing_mode">Sizing mode</span>
                    <select id="ca-sizing-mode" style="width:100%">
                        <option value="fixed_notional" data-i18n="view.confluence_autotrade.opt.fixed">Fixed notional</option>
                        <option value="half_kelly" data-i18n="view.confluence_autotrade.opt.half_kelly">Half Kelly</option>
                        <option value="quarter_kelly" data-i18n="view.confluence_autotrade.opt.quarter_kelly">Quarter Kelly</option>
                    </select>
                </label>
                <label>
                    <span class="muted small" data-i18n="view.confluence_autotrade.field.kelly_horizon">Kelly horizon (days)</span>
                    <input type="number" id="ca-kelly-horizon" step="1" min="1" max="252" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.confluence_autotrade.field.kelly_max">Kelly max fraction (cap)</span>
                    <input type="number" id="ca-kelly-max" step="0.005" min="0.001" max="1" style="width:100%">
                </label>
                <label class="ca-row" style="display:flex;align-items:center;gap:8px">
                    <input type="checkbox" id="ca-corr-gate-enabled">
                    <span data-i18n="view.confluence_autotrade.field.corr_gate_enabled">Correlation gate enabled</span>
                </label>
                <label>
                    <span class="muted small" data-i18n="view.confluence_autotrade.field.max_corr">Max pairwise |r|</span>
                    <input type="number" id="ca-max-corr" step="0.05" min="0.05" max="1" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.confluence_autotrade.field.corr_window">Correlation window (days)</span>
                    <input type="number" id="ca-corr-window" step="5" min="10" max="252" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.confluence_autotrade.field.max_holding">Max holding (days)</span>
                    <input type="number" id="ca-max-holding" step="1" min="0" max="365" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.confluence_autotrade.field.degradation_checks">Degradation checks</span>
                    <input type="number" id="ca-degradation" step="1" min="1" max="20" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.confluence_autotrade.field.stop_loss">Stop-loss %</span>
                    <input type="number" id="ca-sl" step="0.5" min="0" max="50" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.confluence_autotrade.field.take_profit">Take-profit %</span>
                    <input type="number" id="ca-tp" step="0.5" min="0" max="200" style="width:100%">
                </label>
                <label class="ca-row" style="display:flex;align-items:center;gap:8px">
                    <input type="checkbox" id="ca-trail-enabled">
                    <span data-i18n="view.confluence_autotrade.field.trailing_enabled">Trailing stop enabled</span>
                </label>
                <label>
                    <span class="muted small" data-i18n="view.confluence_autotrade.field.trailing_pct">Trailing %</span>
                    <input type="number" id="ca-trail" step="0.5" min="0" max="50" style="width:100%">
                </label>
            </form>
            <div style="display:flex;gap:12px;flex-wrap:wrap;margin-bottom:8px">
                <button class="btn btn-sm primary" id="ca-save" data-i18n="view.confluence_autotrade.btn.save">💾 Save Config</button>
                <button class="btn btn-sm primary" id="ca-run" data-shortcut="r" data-i18n="view.confluence_autotrade.btn.run">⚡ Run Once</button>
                <button class="btn btn-sm" id="ca-sweep" data-i18n="view.confluence_autotrade.btn.sweep">↻ Sweep Exits</button>
                <span class="muted small" id="ca-meta"></span>
            </div>
            <div id="ca-result"></div>
            <h2 style="margin-top:1rem" data-i18n="view.confluence_autotrade.h2.log">Recent fires</h2>
            <table class="trades" id="ca-log">
                <thead><tr>
                    <th data-i18n="view.confluence_autotrade.th.when">When</th>
                    <th data-i18n="view.confluence_autotrade.th.symbol">Symbol</th>
                    <th data-i18n="view.confluence_autotrade.th.score">Score</th>
                    <th data-i18n="view.confluence_autotrade.th.sources">Sources</th>
                    <th data-i18n="view.confluence_autotrade.th.notional">Notional</th>
                    <th data-i18n="view.confluence_autotrade.th.sizing">Sizing</th>
                    <th data-i18n="view.confluence_autotrade.th.action">Action</th>
                    <th data-i18n="view.confluence_autotrade.th.reason">Reason</th>
                </tr></thead>
                <tbody><tr><td colspan="8" class="muted" data-i18n="common.loading">loading…</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelector('#ca-save').addEventListener('click', () => saveConfig(mount));
    mount.querySelector('#ca-run').addEventListener('click', () => runOnce(mount));
    mount.querySelector('#ca-sweep').addEventListener('click', () => sweepExits(mount));
    await loadConfig(mount);
    await loadLog(mount);
}

async function sweepExits(mount) {
    const meta = mount.querySelector('#ca-meta');
    const result = mount.querySelector('#ca-result');
    if (meta) meta.textContent = t('view.confluence_autotrade.status.sweeping');
    try {
        const r = await api('/confluence/autotrade/sweep-exits', { method: 'POST' });
        const flat = r.flattened || [];
        const held = r.held || [];
        result.innerHTML = `<p>
            ${esc(t('view.confluence_autotrade.status.sweep_summary')
                .replace('{c}', r.considered).replace('{f}', flat.length).replace('{h}', held.length))}
        </p>${flat.length ? `<ul>${flat.map(f =>
            `<li class="neg small">${esc(f.symbol)} — ${esc(f.reason)} (held ${f.days_held}d)</li>`).join('')}</ul>` : ''}`;
        if (meta) meta.textContent = '';
        await loadLog(mount);
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}

async function loadConfig(mount) {
    try {
        const c = await api('/confluence/autotrade/config');
        mount.querySelector('#ca-enabled').checked = !!c.enabled;
        mount.querySelector('#ca-min-score').value = c.min_score;
        mount.querySelector('#ca-min-distinct').value = c.min_distinct_sources;
        mount.querySelector('#ca-notional').value = c.notional_usd;
        mount.querySelector('#ca-cooldown').value = c.cooldown_minutes;
        mount.querySelector('#ca-max-open').value = c.max_open_positions;
        mount.querySelector('#ca-sizing-mode').value = c.sizing_mode;
        mount.querySelector('#ca-kelly-horizon').value = c.kelly_horizon_days;
        mount.querySelector('#ca-kelly-max').value = c.kelly_max_fraction;
        mount.querySelector('#ca-corr-gate-enabled').checked = !!c.correlation_gate_enabled;
        mount.querySelector('#ca-max-corr').value = c.max_pairwise_correlation;
        mount.querySelector('#ca-corr-window').value = c.correlation_window_days;
        mount.querySelector('#ca-max-holding').value = c.max_holding_days;
        mount.querySelector('#ca-degradation').value = c.degradation_threshold_checks;
        mount.querySelector('#ca-sl').value = c.stop_loss_pct;
        mount.querySelector('#ca-tp').value = c.take_profit_pct;
        mount.querySelector('#ca-trail-enabled').checked = !!c.trailing_stop_enabled;
        mount.querySelector('#ca-trail').value = c.trailing_stop_pct;
    } catch (e) {
        mount.querySelector('#ca-result').innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}

async function saveConfig(mount) {
    const meta = mount.querySelector('#ca-meta');
    const body = {
        enabled: mount.querySelector('#ca-enabled').checked,
        min_score: parseFloat(mount.querySelector('#ca-min-score').value),
        min_distinct_sources: parseInt(mount.querySelector('#ca-min-distinct').value, 10),
        notional_usd: parseFloat(mount.querySelector('#ca-notional').value),
        cooldown_minutes: parseInt(mount.querySelector('#ca-cooldown').value, 10),
        max_open_positions: parseInt(mount.querySelector('#ca-max-open').value, 10),
        sizing_mode: mount.querySelector('#ca-sizing-mode').value,
        kelly_horizon_days: parseInt(mount.querySelector('#ca-kelly-horizon').value, 10),
        kelly_max_fraction: parseFloat(mount.querySelector('#ca-kelly-max').value),
        correlation_gate_enabled: mount.querySelector('#ca-corr-gate-enabled').checked,
        max_pairwise_correlation: parseFloat(mount.querySelector('#ca-max-corr').value),
        correlation_window_days: parseInt(mount.querySelector('#ca-corr-window').value, 10),
        max_holding_days: parseInt(mount.querySelector('#ca-max-holding').value, 10),
        degradation_threshold_checks: parseInt(mount.querySelector('#ca-degradation').value, 10),
        stop_loss_pct: parseFloat(mount.querySelector('#ca-sl').value),
        take_profit_pct: parseFloat(mount.querySelector('#ca-tp').value),
        trailing_stop_enabled: mount.querySelector('#ca-trail-enabled').checked,
        trailing_stop_pct: parseFloat(mount.querySelector('#ca-trail').value),
    };
    try {
        const c = await api('/confluence/autotrade/config', { method: 'PUT', body: JSON.stringify(body) });
        if (meta) meta.textContent = t('view.confluence_autotrade.status.saved').replace('{t}', fmtDateTime(c.updated_at));
    } catch (e) {
        if (meta) meta.textContent = `${t('common.error')}: ${String(e)}`;
    }
}

async function runOnce(mount) {
    const meta = mount.querySelector('#ca-meta');
    const result = mount.querySelector('#ca-result');
    if (meta) meta.textContent = t('view.confluence_autotrade.status.running');
    try {
        const r = await api('/confluence/autotrade/run-once', { method: 'POST' });
        if (!r.config.enabled) {
            result.innerHTML = `<p class="muted">${esc(t('view.confluence_autotrade.status.disabled'))}</p>`;
        } else {
            const sub = r.submitted.length;
            const skipped = r.skipped.length;
            result.innerHTML = `
                <p class="${sub > 0 ? 'pos' : 'muted'}">${esc(t('view.confluence_autotrade.status.summary')
                    .replace('{c}', r.candidates_considered).replace('{s}', sub).replace('{k}', skipped))}</p>
            `;
        }
        if (meta) meta.textContent = '';
        await loadLog(mount);
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}

async function loadLog(mount) {
    const tbody = mount.querySelector('#ca-log tbody');
    try {
        const rows = await api('/confluence/autotrade/log?limit=100');
        if (!rows || !rows.length) {
            tbody.innerHTML = `<tr><td colspan="8" class="muted">${esc(t('view.confluence_autotrade.empty.no_log'))}</td></tr>`;
            return;
        }
        tbody.innerHTML = rows.map(r => {
            const sizing = r.sizing_used
                ? (r.kelly_fraction != null
                    ? `${esc(r.sizing_used)} · ${(r.kelly_fraction * 100).toFixed(2)}%`
                    : esc(r.sizing_used))
                : '';
            return `
            <tr>
                <td class="muted small">${esc(fmtDateTime(r.fired_at))}</td>
                <td><strong>${esc(r.symbol)}</strong></td>
                <td>${r.score.toFixed(2)}</td>
                <td>${r.distinct_sources}</td>
                <td>$${r.notional_usd.toFixed(0)}</td>
                <td class="muted small">${sizing}</td>
                <td class="${actionCls(r.action)}">${esc(r.action)}</td>
                <td class="muted small">${esc(r.reason || '')}</td>
            </tr>`;
        }).join('');
    } catch (e) {
        tbody.innerHTML = `<tr><td colspan="8" class="neg">${esc(String(e))}</td></tr>`;
    }
}

function actionCls(a) {
    if (a === 'submitted') return 'pos';
    if (a === 'exit_take_profit') return 'pos';
    if (a === 'skipped_correlation') return 'neg';
    if (a === 'exit_stop_loss' || a === 'exit_trailing_stop'
        || a === 'exit_time_stop' || a === 'exit_degraded') return 'neg';
    if (a && a.startsWith('skipped_')) return 'muted';
    return '';
}
