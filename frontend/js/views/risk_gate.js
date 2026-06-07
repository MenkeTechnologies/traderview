// Risk Gate — manage pre-trade rules + dry-run the gate against a proposed trade.
//
// Backend: traderview_core::risk_gate (pure engine, unit-tested) + DB
// table risk_rules (migration 0030) + 4 routes under /api/risk-gate/.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { tConfirm } from '../dialog.js';

const RULE_TYPES = [
    { id: 'max_loss_per_trade_pct',       get label() { return t('view.risk_gate.rule.max_loss_per_trade_pct'); },     fields: [['pct', 'number', '1.0']] },
    { id: 'max_loss_per_day_pct',         get label() { return t('view.risk_gate.rule.max_loss_per_day_pct'); },       fields: [['pct', 'number', '2.0']] },
    { id: 'max_consecutive_losses_today', get label() { return t('view.risk_gate.rule.max_consecutive_losses_today'); }, fields: [['n', 'integer', '3']] },
    { id: 'cool_down_after_loss_minutes', get label() { return t('view.risk_gate.rule.cool_down_after_loss_minutes'); }, fields: [['minutes', 'integer', '15']] },
    { id: 'max_open_positions',           get label() { return t('view.risk_gate.rule.max_open_positions'); },         fields: [['n', 'integer', '5']] },
    { id: 'max_position_size_pct',        get label() { return t('view.risk_gate.rule.max_position_size_pct'); },      fields: [['pct', 'number', '20']] },
    { id: 'blocked_symbols',              get label() { return t('view.risk_gate.rule.blocked_symbols'); },            fields: [['symbols', 'text', 'GME,AMC']] },
    { id: 'require_plan_before_trade',    get label() { return t('view.risk_gate.rule.require_plan_before_trade'); },  fields: [] },
    { id: 'require_stop_loss',            get label() { return t('view.risk_gate.rule.require_stop_loss'); },          fields: [] },
    { id: 'regular_trading_hours_only',   get label() { return t('view.risk_gate.rule.regular_trading_hours_only'); }, fields: [] },
    { id: 'min_position_size_dollars',    get label() { return t('view.risk_gate.rule.min_position_size_dollars'); },  fields: [['min_dollars', 'number', '100']] },
    { id: 'kill_switch',                  get label() { return t('view.risk_gate.rule.kill_switch'); },                fields: [] },
];

export async function renderRiskGate(mount, state) {
    if (!mount) return;
    const tok = currentViewToken();
    const accountId = state.accountId;

    mount.innerHTML = `
        <h1 data-i18n="view.risk_gate.h1.risk_gate" class="view-title">// RISK GATE</h1>
        <p class="muted small" data-i18n-html="view.risk_gate.intro">
            Pre-trade rules that veto bad trades <em>before</em> they reach the broker.
            <code>discipline</code> tells you what you already broke; this stops the next one.
            Engine: <code>traderview_core::risk_gate</code> (pure-compute, 18 unit tests).
        </p>

        <div class="chart-panel" style="border-left:3px solid #ff2a6d">
            <h2 data-i18n="view.risk_gate.h2.kill_switch">🛑 Kill switch</h2>
            <p class="muted small" data-i18n="view.risk_gate.hint.kill_switch">Halt every trade entry across all rules with one click. Toggles a kill_switch rule that always blocks. Disable here when ready to resume.</p>
            <button data-i18n="view.risk_gate.btn.toggle_kill_switch" id="rg-kill" class="primary" type="button" style="background:#ff2a6d">Toggle kill switch</button>
            <span id="rg-kill-state" class="muted small" style="margin-left:10px" data-i18n="common.checking">checking…</span>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.risk_gate.h2.today_s_compliance_snapshot">Today's compliance snapshot</h2>
            <p data-i18n="view.risk_gate.hint.live_ping_of_the_gate_with_a_near_zero_risk_synthe" class="muted small">Live ping of the gate with a near-zero-risk synthetic trade — shows which rules would fire on a probe entry RIGHT NOW. Click refresh after every trade close.</p>
            <button data-i18n="view.risk_gate.btn.refresh_snapshot" id="rg-snap-refresh" class="primary" type="button">Refresh snapshot</button>
            <pre id="rg-snap-out" class="boot" data-i18n="view.risk_gate.snapshot.click_to_eval">click refresh to evaluate</pre>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.risk_gate.h2.install_a_preset">Install a preset</h2>
            <p data-i18n="view.risk_gate.hint.one_click_curated_rule_pack_existing_rules_are_kep" class="muted small">One-click curated rule pack. Existing rules are kept — review + delete after install if you want a clean slate.</p>
            <div class="inline-form" id="rg-presets">
                <button data-i18n="view.risk_gate.btn.beginner_strict_1_trade_3_day_requires_plan_stop" class="primary" data-preset="beginner">Beginner (strict — 1% trade, 3% day, requires plan + stop)</button>
                <button data-i18n="view.risk_gate.btn.intermediate_1_trade_5_day_requires_stop" class="primary" data-preset="intermediate">Intermediate (1% trade, 5% day, requires stop)</button>
                <button data-i18n="view.risk_gate.btn.aggressive_daily_cap_cool_down_only" class="primary" data-preset="aggressive">Aggressive (daily cap + cool-down only)</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.risk_gate.h2.active_rules">Active rules</h2>
            <table class="trades" id="rg-rules">
                <thead><tr>
                    <th data-i18n="view.risk_gate.th.type">Type</th><th data-i18n="view.risk_gate.th.config">Config</th><th data-i18n="view.risk_gate.th.account">Account</th><th data-i18n="view.risk_gate.th.enabled">Enabled</th><th></th>
                </tr></thead>
                <tbody><tr><td colspan="5" class="muted"><span data-i18n="common.loading">loading…</span></td></tr></tbody>
            </table>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.risk_gate.h2.add_rule">Add rule</h2>
            <form id="rg-add" class="inline-form">
                <label><span data-i18n="view.risk_gate.label.rule_type">Rule type</span>
                    <select name="type" id="rg-type">
                        ${RULE_TYPES.map(t =>
                            `<option value="${esc(t.id)}">${esc(t.label)}</option>`
                        ).join('')}
                    </select>
                </label>
                <div id="rg-fields" style="display:flex;gap:8px;flex-wrap:wrap"></div>
                <label><span data-i18n="view.risk_gate.label.scope">Scope</span>
                    <select name="account_id">
                        <option data-i18n="view.risk_gate.opt.all_accounts" value="">All accounts</option>
                        ${(state.accounts || []).map(a =>
                            `<option value="${esc(a.id)}">${esc(a.broker)} · ${esc(a.name)}</option>`
                        ).join('')}
                    </select>
                </label>
                <button data-i18n="view.risk_gate.btn.add" class="primary" type="submit">Add</button>
            </form>
        </div>

        <div class="chart-panel">
            <h2><span data-i18n="view.risk_gate.h2.fires_by_rule">Fires by rule</span> <span class="muted small" data-i18n="view.risk_gate.h2.fires_by_rule_sub">— last 30 days</span></h2>
            <p data-i18n="view.risk_gate.hint.which_rules_trigger_most_high_fire_rules_are_worki" class="muted small">Which rules trigger most. High-fire rules are working hard; zero-fire rules might be too lenient.</p>
            <table class="trades" id="rg-by-rule">
                <thead><tr><th data-i18n="view.risk_gate.th.rule">Rule</th><th data-i18n="view.risk_gate.th.total_fires">Total fires</th><th data-i18n="view.risk_gate.th.blocks">Blocks</th><th data-i18n="view.risk_gate.th.warnings">Warnings</th></tr></thead>
                <tbody><tr><td colspan="4" class="muted"><span data-i18n="common.loading">loading…</span></td></tr></tbody>
            </table>
            <h3 data-i18n="view.risk_gate.h3.fires_chart">Blocks vs warnings per rule</h3>
            <div id="rg-chart" style="width:100%;height:240px"></div>
            <h3 data-i18n="view.risk_gate.h3.block_ratio_chart">Block ratio per rule (% of fires that are blocks)</h3>
            <div id="rg-ratio-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.risk_gate.hint.block_ratio" class="muted small">Per-rule share of fires that are blocks. Yellow dashed = 50%. High = harsh rule (most fires kill the trade). Low = soft rule (mostly warnings).</p>
        </div>

        <div class="chart-panel">
            <h2><span data-i18n="view.risk_gate.h2.recent_fires">Recent fires</span> <span class="muted small" data-i18n="view.risk_gate.h2.recent_fires_sub">— rules that saved you</span></h2>
            <table class="trades" id="rg-fires">
                <thead><tr>
                    <th data-i18n="view.risk_gate.th.time">Time</th><th data-i18n="view.risk_gate.th.symbol">Symbol</th><th data-i18n="view.risk_gate.th.outcome">Outcome</th><th data-i18n="view.risk_gate.th.rules_that_fired">Rules that fired</th>
                </tr></thead>
                <tbody><tr><td colspan="4" class="muted"><span data-i18n="common.loading">loading…</span></td></tr></tbody>
            </table>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.risk_gate.h2.dry_run_a_proposed_trade">Dry-run a proposed trade</h2>
            <p class="muted small" data-i18n-html="view.risk_gate.dry_run.hint">Same call <code>POST /api/risk-gate/evaluate</code> the New Trade form will make before submitting. Use this to verify your rules.</p>
            <form id="rg-eval" class="inline-form">
                <label><span data-i18n="view.risk_gate.label.symbol">Symbol</span> <input name="symbol" data-shortcut="focus_search" value="AAPL" required></label>
                <label><span data-i18n="view.risk_gate.label.side">Side</span>
                    <select name="side">
                        <option data-i18n="view.risk_gate.opt.long" value="long">Long</option>
                        <option data-i18n="view.risk_gate.opt.short" value="short">Short</option>
                    </select></label>
                <label><span data-i18n="view.risk_gate.label.qty">Qty</span> <input name="qty" type="number" step="0.01" value="100" required></label>
                <label><span data-i18n="view.risk_gate.label.entry">Entry</span> <input name="entry_price" type="number" step="0.01" value="150" required></label>
                <label><span data-i18n="view.risk_gate.label.stop_loss">Stop loss</span> <input name="stop_loss" type="number" step="0.01" value="149"></label>
                <label><span data-i18n="view.risk_gate.label.multiplier">Multiplier</span> <input name="multiplier" type="number" step="0.01" value="1"></label>
                <label><input type="checkbox" name="has_attached_plan" checked> <span data-i18n="view.risk_gate.label.plan_attached">Plan attached</span></label>
                <button data-i18n="view.risk_gate.btn.evaluate" class="primary" type="submit">Evaluate</button>
            </form>
            <pre id="rg-eval-out" class="boot">—</pre>
        </div>
    `;

    // Live form fields for the picked rule type.
    const fieldsEl = mount.querySelector('#rg-fields');
    const typeSel  = mount.querySelector('#rg-type');
    const renderFields = () => {
        const rt = RULE_TYPES.find(x => x.id === typeSel.value);
        if (!rt || !rt.fields.length) { fieldsEl.innerHTML = `<span class="muted small">${esc(t('view.risk_gate.empty.no_config'))}</span>`; return; }
        fieldsEl.innerHTML = rt.fields.map(([name, kind, ph]) =>
            `<label>${esc(name)}
                <input name="${esc(name)}" type="${kind === 'text' ? 'text' : 'number'}"
                       ${kind === 'integer' ? 'step="1"' : kind === 'number' ? 'step="0.01"' : ''}
                       value="${esc(ph)}" required>
            </label>`
        ).join('');
    };
    typeSel.addEventListener('change', renderFields);
    renderFields();

    // Today's compliance snapshot — fire a near-zero-cost probe trade.
    mount.querySelector('#rg-snap-refresh').addEventListener('click', async () => {
        const out = mount.querySelector('#rg-snap-out');
        if (!accountId) { out.textContent = t('view.risk_gate.hint.pick_account'); return; }
        try {
            const probe = {
                symbol: '_PROBE_',
                side: 'long',
                qty: 1,
                entry_price: 1,
                stop_loss: 0.99,
                asset_class: 'stock',
                multiplier: 1,
                tick_size: null,
                tick_value: null,
                has_attached_plan: true,
            };
            const d = await api.evaluateProposedTrade(accountId, probe);
            if (!viewIsCurrent(tok)) return;
            const blocks = (d.violations || []).filter(v => v.severity === 'block');
            const warns  = (d.violations || []).filter(v => v.severity === 'warning');
            const lines = [];
            lines.push(blocks.length === 0
                ? `✓ Trading allowed. ${warns.length} warning${warns.length === 1 ? '' : 's'}.`
                : `✗ Trading BLOCKED by ${blocks.length} rule${blocks.length === 1 ? '' : 's'}.`);
            for (const v of d.violations || []) {
                lines.push(`  [${v.severity.toUpperCase()}] ${v.rule}: ${v.message}`);
            }
            out.textContent = lines.join('\n');
        } catch (e) { out.textContent = t('common.error', { err: e.message }); }
    });

    // Preset install buttons.
    mount.querySelectorAll('#rg-presets [data-preset]').forEach(btn => {
        btn.addEventListener('click', async () => {
            const preset = btn.dataset.preset;
            const ok = await tConfirm('view.risk_gate.confirm.install_preset', { preset }, { level: 'danger' });
            if (!ok) return;
            try {
                const r = await api.installRiskPreset(preset);
                if (!viewIsCurrent(tok)) return;
                showToast(t('view.risk_gate.alert.installed', { n: r.inserted }), { level: 'success' });
                await reloadRules(mount, tok);
            } catch (e) { showToast(t('view.risk_gate.alert.install_failed', { err: e.message }), { level: 'error' }); }
        });
    });

    // Kill switch toggle — toggles the existing kill_switch rule if present,
    // installs one otherwise.
    const killBtn = mount.querySelector('#rg-kill');
    const killState = mount.querySelector('#rg-kill-state');
    const refreshKillState = async () => {
        try {
            const rules = await api.riskRules();
            if (!viewIsCurrent(tok)) return;
            const ks = rules.find(r => r.rule.type === 'kill_switch');
            if (!ks) {
                killState.textContent = t('view.risk_gate.kill_state.not_installed');
                killBtn.dataset.id = '';
                killBtn.dataset.enabled = '';
            } else {
                killState.textContent = ks.enabled ? '🛑 ACTIVE — all trades blocked' : 'installed, disabled';
                killBtn.dataset.id = ks.id;
                killBtn.dataset.enabled = ks.enabled ? '1' : '0';
            }
        } catch (e) { killState.textContent = t('common.error', { err: e.message }); }
    };
    killBtn.addEventListener('click', async () => {
        try {
            const wasEnabled = killBtn.dataset.enabled === '1';
            // Confirm in both directions so neither accidental hit
            // halts nor accidental re-enable goes unnoticed.
            const verb = !killBtn.dataset.id ? t('view.risk_gate.kill.install')
                : wasEnabled                 ? t('view.risk_gate.kill.disable')
                                             : t('view.risk_gate.kill.enable');
            if (!await tConfirm('view.risk_gate.confirm.kill_switch', { verb }, { level: 'danger' })) return;
            if (!killBtn.dataset.id) {
                await api.createRiskRule({ rule: { type: 'kill_switch' }, account_id: null });
            } else {
                await api.toggleRiskRule(killBtn.dataset.id, !wasEnabled);
            }
            if (!viewIsCurrent(tok)) return;
            await refreshKillState();
            await reloadRules(mount, tok);
        } catch (e) { showToast(t('view.risk_gate.alert.kill_switch_failed', { err: e.message }), { level: 'error' }); }
    });
    await refreshKillState();

    // Load analytics.
    await reloadFiresByRule(mount, tok);

    // Load fire log.
    await reloadFires(mount, tok);

    // Load existing rules.
    await reloadRules(mount, tok);

    // Add-rule submit.
    mount.querySelector('#rg-add').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const ruleType = RULE_TYPES.find(x => x.id === fd.get('type'));
        const rule = { type: ruleType.id };
        for (const [name, kind] of ruleType.fields) {
            if (name === 'symbols') {
                rule.symbols = String(fd.get(name) || '')
                    .split(',').map(s => s.trim().toUpperCase()).filter(Boolean);
            } else if (kind === 'integer') {
                rule[name] = parseInt(fd.get(name), 10);
            } else {
                rule[name] = String(fd.get(name));
            }
        }
        const body = { rule, account_id: fd.get('account_id') || null };
        try {
            await api.createRiskRule(body);
            if (!viewIsCurrent(tok)) return;
            await reloadRules(mount, tok);
        } catch (err) { showToast(t('view.risk_gate.alert.create_failed', { err: err.message }), { level: 'error' }); }
    });

    // Dry-run evaluate.
    mount.querySelector('#rg-eval').addEventListener('submit', async (e) => {
        e.preventDefault();
        if (!accountId) {
            mount.querySelector('#rg-eval-out').textContent =
                t('view.risk_gate.error.no_account');
            return;
        }
        const fd = new FormData(e.target);
        const stop = fd.get('stop_loss');
        const proposed = {
            symbol: fd.get('symbol'),
            side: fd.get('side'),
            qty: fd.get('qty'),
            entry_price: fd.get('entry_price'),
            stop_loss: stop ? stop : null,
            asset_class: 'stock',
            multiplier: fd.get('multiplier'),
            tick_size: null,
            tick_value: null,
            has_attached_plan: !!fd.get('has_attached_plan'),
        };
        try {
            const decision = await api.evaluateProposedTrade(accountId, proposed);
            if (!viewIsCurrent(tok)) return;
            renderDecision(mount, decision);
        } catch (err) {
            mount.querySelector('#rg-eval-out').textContent = t('common.error', { err: err.message });
        }
    });
}

async function reloadRules(mount, tok) {
    const tb = mount.querySelector('#rg-rules tbody');
    if (!tb) return;
    try {
        const rules = await api.riskRules();
        if (!viewIsCurrent(tok)) return;
        if (!rules.length) {
            tb.innerHTML = `<tr><td colspan="5" class="muted">${esc(t('view.risk_gate.empty.rules'))}</td></tr>`;
            return;
        }
        tb.innerHTML = rules.map(r => `
            <tr>
                <td><strong>${esc(r.rule.type)}</strong></td>
                <td><code>${esc(JSON.stringify(stripType(r.rule)))}</code></td>
                <td>${esc(r.account_id || 'all')}</td>
                <td>
                    <input type="checkbox" data-toggle="${esc(r.id)}" ${r.enabled ? 'checked' : ''}>
                </td>
                <td><button data-i18n="view.risk_gate.btn.delete" class="link" data-del="${esc(r.id)}">delete</button></td>
            </tr>
        `).join('');
        tb.querySelectorAll('[data-toggle]').forEach(cb => {
            cb.addEventListener('change', async () => {
                try { await api.toggleRiskRule(cb.dataset.toggle, cb.checked); }
                catch (e) { showToast(t('view.risk_gate.alert.toggle_failed', { err: e.message }), { level: 'error' }); cb.checked = !cb.checked; }
            });
        });
        tb.querySelectorAll('[data-del]').forEach(b => {
            b.addEventListener('click', async () => {
                try { await api.deleteRiskRule(b.dataset.del); }
                catch (e) { showToast(t('view.risk_gate.alert.delete_failed', { err: e.message }), { level: 'error' }); return; }
                if (viewIsCurrent(tok)) await reloadRules(mount, tok);
            });
        });
    } catch (err) {
        tb.innerHTML = `<tr><td colspan="5" class="muted">${esc(t('view.risk_gate.error', { msg: err.message }))}</td></tr>`;
    }
}

function stripType(rule) {
    const { type: _, ...rest } = rule;
    return rest;
}

async function reloadFiresByRule(mount, tok) {
    const tb = mount.querySelector('#rg-by-rule tbody');
    if (!tb) return;
    try {
        const stats = await api.riskFiresByRule(30);
        if (!viewIsCurrent(tok)) return;
        if (!stats.length) {
            tb.innerHTML = `<tr><td colspan="4" class="muted">${esc(t('view.risk_gate.empty.no_fires_30d'))}</td></tr>`;
            return;
        }
        tb.innerHTML = stats.map(s => `
            <tr>
                <td><code>${esc(s.rule)}</code></td>
                <td>${s.fires}</td>
                <td>${s.blocks > 0 ? `<strong style="color:#ff2a6d">${s.blocks}</strong>` : 0}</td>
                <td>${s.fires - s.blocks}</td>
            </tr>
        `).join('');
        renderFiresChart(stats);
        renderBlockRatioChart(stats);
    } catch (err) {
        tb.innerHTML = `<tr><td colspan="4" class="muted">${esc(t('view.risk_gate.error', { msg: err.message }))}</td></tr>`;
    }
}

function renderFiresChart(stats) {
    const el = document.getElementById('rg-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const top = (stats || [])
        .filter(s => Number.isFinite(Number(s.fires)))
        .sort((a, b) => Number(b.fires) - Number(a.fires))
        .slice(0, 20);
    if (top.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.risk_gate.empty_chart">${esc(t('view.risk_gate.empty_chart'))}</div>`;
        return;
    }
    const labels = top.map(s => s.rule);
    const blocks = top.map(s => Number(s.blocks) || 0);
    const warnings = top.map(s => (Number(s.fires) || 0) - (Number(s.blocks) || 0));
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.risk_gate.chart.rule_idx') },
            { label: t('view.risk_gate.chart.blocks'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 10, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.risk_gate.chart.warnings'),
              stroke: '#ffd84a', width: 0,
              points: { show: true, size: 8, fill: '#ffd84a', stroke: '#ffd84a' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, blocks, warnings], el);
}

function renderBlockRatioChart(stats) {
    const el = document.getElementById('rg-ratio-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const top = (stats || [])
        .filter(s => Number.isFinite(Number(s.fires)) && Number(s.fires) > 0)
        .sort((a, b) => Number(b.fires) - Number(a.fires))
        .slice(0, 20);
    if (top.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.risk_gate.empty_ratio_chart">${esc(t('view.risk_gate.empty_ratio_chart'))}</div>`;
        return;
    }
    const labels = top.map(s => s.rule);
    const ratios = top.map(s => Number(s.blocks) / Number(s.fires) * 100);
    const xs = labels.map((_, i) => i + 1);
    const half = xs.map(() => 50);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { range: [0, 100] } },
        series: [
            { label: t('view.risk_gate.chart.rule_idx') },
            { label: t('view.risk_gate.chart.block_ratio'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 14, fill: '#b86bff', stroke: '#b86bff' } },
            { label: t('view.risk_gate.chart.fifty'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40,
              values: (_u, splits) => splits.map(v => v.toFixed(0) + '%') },
        ],
        legend: { show: true },
    }, [xs, ratios, half], el);
}

async function reloadFires(mount, tok) {
    const tb = mount.querySelector('#rg-fires tbody');
    if (!tb) return;
    try {
        const fires = await api.riskFires(50);
        if (!viewIsCurrent(tok)) return;
        if (!fires.length) {
            tb.innerHTML = `<tr><td colspan="4" class="muted">${esc(t('view.risk_gate.empty.no_fires_ever'))}</td></tr>`;
            return;
        }
        tb.innerHTML = fires.map(f => {
            const rules = (f.decision.violations || [])
                .map(v => `<span class="halt-code halt-${v.severity === 'block' ? 'volatility' : 'news'}">${esc(v.severity)}</span> ${esc(v.rule)}`)
                .join('<br>');
            return `
                <tr>
                    <td>${new Date(f.fired_at).toLocaleString(undefined, { hour12: false })}</td>
                    <td><strong>${esc(f.symbol)}</strong></td>
                    <td>${f.blocked ? `<strong style="color:#ff2a6d">${t('view.risk_gate.status.blocked')}</strong>` : `<span class="muted">${t('view.risk_gate.status.warned_allowed')}</span>`}</td>
                    <td>${rules}</td>
                </tr>`;
        }).join('');
    } catch (err) {
        tb.innerHTML = `<tr><td colspan="4" class="muted">${esc(t('view.risk_gate.error', { msg: err.message }))}</td></tr>`;
    }
}

function renderDecision(mount, d) {
    const el = mount.querySelector('#rg-eval-out');
    if (!el) return;
    const lines = [];
    lines.push(d.allow ? '✓ ALLOW — no Block-level rules fired' : '✗ BLOCK — at least one Block-severity rule fired');
    if (d.violations.length) {
        lines.push('');
        lines.push('Violations:');
        for (const v of d.violations) {
            lines.push(`  [${v.severity.toUpperCase()}] ${v.rule}: ${v.message}`);
        }
    }
    el.textContent = lines.join('\n');
}
