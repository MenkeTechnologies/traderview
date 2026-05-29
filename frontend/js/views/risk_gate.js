// Risk Gate — manage pre-trade rules + dry-run the gate against a proposed trade.
//
// Backend: traderview_core::risk_gate (pure engine, unit-tested) + DB
// table risk_rules (migration 0030) + 4 routes under /api/risk-gate/.

import { api } from '../api.js';
import { esc, fmtMoney, fmtPct } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const RULE_TYPES = [
    { id: 'max_loss_per_trade_pct',     label: 'Max loss per trade (% of equity)',     fields: [['pct', 'number', '1.0']] },
    { id: 'max_loss_per_day_pct',       label: 'Max loss per day (% of equity)',       fields: [['pct', 'number', '2.0']] },
    { id: 'max_consecutive_losses_today', label: 'Max consecutive losses today',       fields: [['n', 'integer', '3']] },
    { id: 'cool_down_after_loss_minutes', label: 'Cool-down after loss (minutes)',     fields: [['minutes', 'integer', '15']] },
    { id: 'max_open_positions',         label: 'Max open positions',                   fields: [['n', 'integer', '5']] },
    { id: 'max_position_size_pct',      label: 'Max position size (% of equity)',      fields: [['pct', 'number', '20']] },
    { id: 'blocked_symbols',            label: 'Blocked symbols (comma-sep)',          fields: [['symbols', 'text', 'GME,AMC']] },
    { id: 'require_plan_before_trade',  label: 'Require pre-trade plan',                fields: [] },
    { id: 'require_stop_loss',          label: 'Require stop loss (warning only)',     fields: [] },
    { id: 'regular_trading_hours_only', label: 'Block outside RTH (09:30-16:00 ET, Mon-Fri)', fields: [] },
    { id: 'min_position_size_dollars',  label: 'Min notional $ (fat-finger guard)',    fields: [['min_dollars', 'number', '100']] },
    { id: 'kill_switch',                label: 'Kill switch (always blocks)',          fields: [] },
];

export async function renderRiskGate(mount, state) {
    if (!mount) return;
    const tok = currentViewToken();
    const accountId = state.accountId;

    mount.innerHTML = `
        <h1 data-i18n="view.risk_gate.h1.risk_gate" class="view-title">// RISK GATE</h1>
        <p class="muted small">
            Pre-trade rules that veto bad trades <em>before</em> they reach the broker.
            <code>discipline</code> tells you what you already broke; this stops the next one.
            Engine: <code>traderview_core::risk_gate</code> (pure-compute, 18 unit tests).
        </p>

        <div class="chart-panel" style="border-left:3px solid #ff2a6d">
            <h2 data-i18n="view.risk_gate.h2.kill_switch">🛑 Kill switch</h2>
            <p class="muted small">Halt every trade entry across all rules with one click. Toggles a <code>kill_switch</code> rule that always blocks. Disable here when ready to resume.</p>
            <button data-i18n="view.risk_gate.btn.toggle_kill_switch" id="rg-kill" class="primary" type="button" style="background:#ff2a6d">Toggle kill switch</button>
            <span id="rg-kill-state" class="muted small" style="margin-left:10px">checking…</span>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.risk_gate.h2.today_s_compliance_snapshot">Today's compliance snapshot</h2>
            <p data-i18n="view.risk_gate.hint.live_ping_of_the_gate_with_a_near_zero_risk_synthe" class="muted small">Live ping of the gate with a near-zero-risk synthetic trade — shows which rules would fire on a probe entry RIGHT NOW. Click refresh after every trade close.</p>
            <button data-i18n="view.risk_gate.btn.refresh_snapshot" id="rg-snap-refresh" class="primary" type="button">Refresh snapshot</button>
            <pre id="rg-snap-out" class="boot">click refresh to evaluate</pre>
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
                <tbody><tr><td colspan="5" class="muted">loading…</td></tr></tbody>
            </table>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.risk_gate.h2.add_rule">Add rule</h2>
            <form id="rg-add" class="inline-form">
                <label>Rule type
                    <select name="type" id="rg-type">
                        ${RULE_TYPES.map(t =>
                            `<option value="${esc(t.id)}">${esc(t.label)}</option>`
                        ).join('')}
                    </select>
                </label>
                <div id="rg-fields" style="display:flex;gap:8px;flex-wrap:wrap"></div>
                <label>Scope
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
            <h2>Fires by rule <span class="muted small">— last 30 days</span></h2>
            <p data-i18n="view.risk_gate.hint.which_rules_trigger_most_high_fire_rules_are_worki" class="muted small">Which rules trigger most. High-fire rules are working hard; zero-fire rules might be too lenient.</p>
            <table class="trades" id="rg-by-rule">
                <thead><tr><th data-i18n="view.risk_gate.th.rule">Rule</th><th data-i18n="view.risk_gate.th.total_fires">Total fires</th><th data-i18n="view.risk_gate.th.blocks">Blocks</th><th data-i18n="view.risk_gate.th.warnings">Warnings</th></tr></thead>
                <tbody><tr><td colspan="4" class="muted">loading…</td></tr></tbody>
            </table>
        </div>

        <div class="chart-panel">
            <h2>Recent fires <span class="muted small">— rules that saved you</span></h2>
            <table class="trades" id="rg-fires">
                <thead><tr>
                    <th data-i18n="view.risk_gate.th.time">Time</th><th data-i18n="view.risk_gate.th.symbol">Symbol</th><th data-i18n="view.risk_gate.th.outcome">Outcome</th><th data-i18n="view.risk_gate.th.rules_that_fired">Rules that fired</th>
                </tr></thead>
                <tbody><tr><td colspan="4" class="muted">loading…</td></tr></tbody>
            </table>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.risk_gate.h2.dry_run_a_proposed_trade">Dry-run a proposed trade</h2>
            <p class="muted small">Same call <code>POST /api/risk-gate/evaluate</code> the New Trade form will make before submitting. Use this to verify your rules.</p>
            <form id="rg-eval" class="inline-form">
                <label>Symbol <input name="symbol" value="AAPL" required></label>
                <label>Side
                    <select name="side">
                        <option data-i18n="view.risk_gate.opt.long" value="long">Long</option>
                        <option data-i18n="view.risk_gate.opt.short" value="short">Short</option>
                    </select></label>
                <label>Qty <input name="qty" type="number" step="any" value="100" required></label>
                <label>Entry <input name="entry_price" type="number" step="any" value="150" required></label>
                <label>Stop loss <input name="stop_loss" type="number" step="any" value="149"></label>
                <label>Multiplier <input name="multiplier" type="number" step="any" value="1"></label>
                <label><input type="checkbox" name="has_attached_plan" checked> Plan attached</label>
                <button data-i18n="view.risk_gate.btn.evaluate" class="primary" type="submit">Evaluate</button>
            </form>
            <pre id="rg-eval-out" class="boot">—</pre>
        </div>
    `;

    // Live form fields for the picked rule type.
    const fieldsEl = mount.querySelector('#rg-fields');
    const typeSel  = mount.querySelector('#rg-type');
    const renderFields = () => {
        const t = RULE_TYPES.find(x => x.id === typeSel.value);
        if (!t || !t.fields.length) { fieldsEl.innerHTML = '<span class="muted small">no config</span>'; return; }
        fieldsEl.innerHTML = t.fields.map(([name, kind, ph]) =>
            `<label>${esc(name)}
                <input name="${esc(name)}" type="${kind === 'text' ? 'text' : 'number'}"
                       ${kind === 'integer' ? 'step="1"' : kind === 'number' ? 'step="any"' : ''}
                       value="${esc(ph)}" required>
            </label>`
        ).join('');
    };
    typeSel.addEventListener('change', renderFields);
    renderFields();

    // Today's compliance snapshot — fire a near-zero-cost probe trade.
    mount.querySelector('#rg-snap-refresh').addEventListener('click', async () => {
        const out = mount.querySelector('#rg-snap-out');
        if (!accountId) { out.textContent = 'pick an account in the topbar'; return; }
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
        } catch (e) { out.textContent = 'Error: ' + e.message; }
    });

    // Preset install buttons.
    mount.querySelectorAll('#rg-presets [data-preset]').forEach(btn => {
        btn.addEventListener('click', async () => {
            const preset = btn.dataset.preset;
            const ok = confirm(`Install ${preset} preset? Existing rules are kept (deduplication is manual).`);
            if (!ok) return;
            try {
                const r = await api.installRiskPreset(preset);
                if (!viewIsCurrent(tok)) return;
                alert(`Installed ${r.inserted} rules.`);
                await reloadRules(mount, tok);
            } catch (e) { alert('Install failed: ' + e.message); }
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
                killState.textContent = 'not installed — click to enable';
                killBtn.dataset.id = '';
                killBtn.dataset.enabled = '';
            } else {
                killState.textContent = ks.enabled ? '🛑 ACTIVE — all trades blocked' : 'installed, disabled';
                killBtn.dataset.id = ks.id;
                killBtn.dataset.enabled = ks.enabled ? '1' : '0';
            }
        } catch (e) { killState.textContent = 'Error: ' + e.message; }
    };
    killBtn.addEventListener('click', async () => {
        try {
            const wasEnabled = killBtn.dataset.enabled === '1';
            // Confirm in both directions so neither accidental hit
            // halts nor accidental re-enable goes unnoticed.
            const verb = !killBtn.dataset.id ? 'INSTALL + ENABLE'
                : wasEnabled                 ? 'DISABLE (resume trading)'
                                             : 'ENABLE (halt all trades)';
            if (!confirm(`${verb} the kill switch?`)) return;
            if (!killBtn.dataset.id) {
                await api.createRiskRule({ rule: { type: 'kill_switch' }, account_id: null });
            } else {
                await api.toggleRiskRule(killBtn.dataset.id, !wasEnabled);
            }
            if (!viewIsCurrent(tok)) return;
            await refreshKillState();
            await reloadRules(mount, tok);
        } catch (e) { alert('Kill switch toggle failed: ' + e.message); }
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
        const t = RULE_TYPES.find(x => x.id === fd.get('type'));
        const rule = { type: t.id };
        for (const [name, kind] of t.fields) {
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
        } catch (err) { alert('Create failed: ' + err.message); }
    });

    // Dry-run evaluate.
    mount.querySelector('#rg-eval').addEventListener('submit', async (e) => {
        e.preventDefault();
        if (!accountId) {
            mount.querySelector('#rg-eval-out').textContent =
                'No account selected. Pick one in the topbar account strip.';
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
            mount.querySelector('#rg-eval-out').textContent = 'Error: ' + err.message;
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
            tb.innerHTML = '<tr><td colspan="5" class="muted">No rules yet. Add one below — the gate is a no-op until you do.</td></tr>';
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
                catch (e) { alert('Toggle failed: ' + e.message); cb.checked = !cb.checked; }
            });
        });
        tb.querySelectorAll('[data-del]').forEach(b => {
            b.addEventListener('click', async () => {
                try { await api.deleteRiskRule(b.dataset.del); }
                catch (e) { alert('Delete failed: ' + e.message); return; }
                if (viewIsCurrent(tok)) await reloadRules(mount, tok);
            });
        });
    } catch (err) {
        tb.innerHTML = `<tr><td colspan="5" class="muted">Error: ${esc(err.message)}</td></tr>`;
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
            tb.innerHTML = '<tr><td colspan="4" class="muted">no fires in the last 30 days</td></tr>';
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
    } catch (err) {
        tb.innerHTML = `<tr><td colspan="4" class="muted">Error: ${esc(err.message)}</td></tr>`;
    }
}

async function reloadFires(mount, tok) {
    const tb = mount.querySelector('#rg-fires tbody');
    if (!tb) return;
    try {
        const fires = await api.riskFires(50);
        if (!viewIsCurrent(tok)) return;
        if (!fires.length) {
            tb.innerHTML = '<tr><td colspan="4" class="muted">no fires yet — every gate check is recorded here</td></tr>';
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
                    <td>${f.blocked ? '<strong style="color:#ff2a6d">BLOCKED</strong>' : '<span class="muted">warned, allowed</span>'}</td>
                    <td>${rules}</td>
                </tr>`;
        }).join('');
    } catch (err) {
        tb.innerHTML = `<tr><td colspan="4" class="muted">Error: ${esc(err.message)}</td></tr>`;
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
