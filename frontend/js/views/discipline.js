// Streaks + discipline scorecard. Walks closed-trade history for streaks
// and trade_plans→trades joins for rule-violation tracking.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderDiscipline(mount, state) {
    const tok = currentViewToken();
    const acct = state.accounts.find(a => a.id === state.accountId);
    if (!acct) { mount.innerHTML = `<p data-i18n="view.discipline.hint.no_account_selected" class="boot">No account selected.</p>`; return; }
    mount.innerHTML = `
        <h1 class="view-title">// DISCIPLINE — ${esc(acct.broker)} · ${esc(acct.name)}</h1>
        <p class="muted small" data-i18n="view.discipline.hint.intro">Streaks computed from chronological closed-trade P/L sign. Rule-violation tracker joins your trade_plans rows to filled trades via linked_trade_id and grades on four checks: stop_set, stop_honored, qty_within (≤ 1.10× planned), direction_match. Discipline % = passing / linked across week / month / all-time.</p>

        <div id="d-out"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text">loading…</div></div></div>
    `;
    try {
        const r = await api.discipline(acct.id);
        if (!viewIsCurrent(tok)) return;
        render(r, mount);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#d-out');
        if (el) el.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function render(r, mount) {
    const s = r.streaks;
    const streakColor = s.current_streak_kind === 'win' ? 'pos' :
                        s.current_streak_kind === 'loss' ? 'neg' : 'muted';
    const el = mount.querySelector('#d-out');
    if (!el) return;
    el.innerHTML = `
        <div class="cards">
            <div class="card"><div class="label">Total closed trades</div>
                <div class="value">${s.total_closed}</div></div>
            <div class="card"><div class="label">Longest win streak</div>
                <div class="value pos">${s.longest_win_streak}</div></div>
            <div class="card"><div class="label">Longest loss streak</div>
                <div class="value neg">${s.longest_loss_streak}</div></div>
            <div class="card"><div class="label">Current streak</div>
                <div class="value ${streakColor}">${s.current_streak_length} ${s.current_streak_kind}</div></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.discipline.h2.last_60_trade_outcome_sparkline">Last-60 trade outcome sparkline</h2>
            ${streakSpark(s.sparkline)}
        </div>

        <div class="cards">
            ${scoreCard('This week (7d)',  r.weekly)}
            ${scoreCard('This month (30d)', r.monthly)}
            ${scoreCard('All time',         r.all_time)}
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.discipline.h2.rule_pass_rates_all_linked_trades">Rule pass rates (all linked trades)</h2>
            ${ruleBars(r.rule_breakdown)}
        </div>

        <div class="chart-panel">
            <h2>${esc(t('view.discipline.h2.latest_rules', { count: r.rule_evals.length }))}</h2>
            ${ruleTable(r.rule_evals.slice(-50).reverse())}
        </div>
    `;
}

function streakSpark(bits) {
    if (!bits.length) return '<p data-i18n="view.discipline.hint.no_closed_trades_yet" class="muted small">No closed trades yet.</p>';
    const W = 720, H = 36, gap = 1;
    const cellW = Math.max(2, (W - bits.length * gap) / bits.length);
    return `<svg viewBox="0 0 ${W} ${H}" width="100%" height="${H}" style="display:block;">
        ${bits.map((b, i) => {
            const x = i * (cellW + gap);
            const color = b > 0 ? '#7af0a8' : b < 0 ? '#ff1f7a' : '#444';
            return `<rect x="${x}" y="0" width="${cellW}" height="${H}" fill="${color}" opacity="${b === 0 ? 0.4 : 0.9}"/>`;
        }).join('')}
    </svg>`;
}

function scoreCard(label, w) {
    const cls = w.discipline_pct >= 80 ? 'pos' :
                w.discipline_pct >= 60 ? '' : 'neg';
    return `<div class="card"><div class="label">${esc(label)}</div>
        <div class="value ${cls}">${w.discipline_pct.toFixed(1)}%</div>
        <div class="muted small">${w.passing} / ${w.linked_trades} passing</div></div>`;
}

function ruleBars(rb) {
    const row = (label, pct) => {
        const cls = pct >= 80 ? 'pos' : pct >= 60 ? '' : 'neg';
        const color = pct >= 80 ? '#7af0a8' : pct >= 60 ? '#9aa0c8' : '#ff1f7a';
        return `<div>${esc(label)}</div>
            <div style="height:18px;background:#1a1d2e;">
                <div style="width:${pct}%;height:100%;background:${color};"></div>
            </div>
            <div class="${cls}">${pct.toFixed(1)}%</div>`;
    };
    return `<div style="display:grid;grid-template-columns:140px 1fr 60px;gap:6px;font-size:11px;">
        ${row('stop_set', rb.stop_set_rate)}
        ${row('stop_honored', rb.stop_honored_rate)}
        ${row('qty_within', rb.qty_within_rate)}
        ${row('direction_match', rb.direction_match_rate)}
    </div>`;
}

function ruleTable(rows) {
    if (!rows.length) return '<p data-i18n="view.discipline.hint.no_linked_plans_yet_create_a_trade_plan_in_the_pla" class="muted small">No linked plans yet — create a trade plan in the Plans tab and link it to a trade.</p>';
    const tick = (b) => b ? '<span class="pos">✓</span>' : '<span class="neg">✗</span>';
    return `<table class="trades">
        <thead><tr>
            <th data-i18n="view.discipline.th.date">Date</th><th data-i18n="view.discipline.th.symbol">Symbol</th>
            <th data-i18n="view.discipline.th.stop_set">stop_set</th><th data-i18n="view.discipline.th.stop_honored">stop_honored</th><th data-i18n="view.discipline.th.qty_within">qty_within</th><th data-i18n="view.discipline.th.direction_match">direction_match</th>
            <th data-i18n="view.discipline.th.pass">Pass</th>
        </tr></thead>
        <tbody>
        ${rows.map(r => `<tr>
            <td class="small">${r.date}</td>
            <td><a href="#trade/${r.trade_id}">${esc(r.symbol)}</a></td>
            <td>${tick(r.stop_set)}</td>
            <td>${tick(r.stop_honored)}</td>
            <td>${tick(r.qty_within)}</td>
            <td>${tick(r.direction_match)}</td>
            <td class="${r.overall_pass ? 'pos' : 'neg'}">${r.overall_pass ? 'PASS' : 'FAIL'}</td>
        </tr>`).join('')}
        </tbody></table>`;
}
