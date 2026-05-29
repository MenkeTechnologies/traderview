// Mood-vs-PnL analytics — does self-reported mood predict results?

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { applyUiI18n } from '../i18n.js';

const MOOD_LABELS = { '-2': '😡 awful', '-1': '🙁 down', '0': '😐 flat', '1': '🙂 good', '2': '😄 great' };

export async function renderMoodAnalytics(mount, state) {
    const tok = currentViewToken();
    const acct = state.accounts.find(a => a.id === state.accountId);
    if (!acct) { mount.innerHTML = `<p data-i18n="view.mood_analytics.hint.no_account_selected" class="boot">No account selected.</p>`; return; }
    mount.innerHTML = `
        <h1 class="view-title">// MOOD ANALYTICS — ${esc(acct.broker)} · ${esc(acct.name)}</h1>
        <p class="muted small" data-i18n="view.mood_analytics.hint.intro">Two ingestion paths: per-trade (journal entry directly tied to a trade — cleanest signal) and per-day (daily mood × every trade opened that day). The Pearson correlation is across the union. Positive ρ means happier-mood days produce better outcomes; negative means good moods correlate with overconfidence and losses.</p>

        <div id="ma-cards" class="cards"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text">loading…</div></div></div>
        <div id="ma-out"></div>
    `;
    try {
        const r = await api.moodAnalytics(acct.id);
        if (!viewIsCurrent(tok)) return;
        render(r, mount);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#ma-out');
        if (el) el.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function render(r, mount) {
    const cls = (v) => v == null ? '' : v >= 0 ? 'pos' : 'neg';
    const corr = r.overall_correlation;
    const cardsEl = mount.querySelector('#ma-cards');
    if (!cardsEl) return;
    cardsEl.innerHTML = `
        <div class="card"><div class="label" data-i18n="view.mood_analytics.card.samples">Samples</div>
            <div class="value">${r.samples_total}</div></div>
        <div class="card"><div class="label" data-i18n="view.mood_analytics.card.pearson">Pearson(mood, P/L)</div>
            <div class="value ${cls(corr)}">${corr == null ? '—' : corr.toFixed(3)}</div>
            <div class="muted small">${interpretCorr(corr)}</div></div>
        <div class="card"><div class="label" data-i18n="view.mood_analytics.card.mood_buckets">Mood buckets</div>
            <div class="value">${r.stats.length}</div></div>
        <div class="card"><div class="label" data-i18n="view.mood_analytics.card.per_trade_vs_day">Per-trade vs per-day</div>
            <div class="value small">${r.pairs.filter(p=>p.source==='per_trade').length} / ${r.pairs.filter(p=>p.source==='per_day').length}</div></div>
    `;
    try { applyUiI18n(cardsEl); } catch (_) {}
    const outEl = mount.querySelector('#ma-out');
    if (!outEl) return;
    if (!r.stats.length) {
        outEl.innerHTML =
            '<div class="chart-panel"><p data-i18n="view.mood_analytics.hint.no_mood_tagged_journal_entries_yet_tag_your_journa" class="muted small">No mood-tagged journal entries yet — tag your journal entries with a mood -2..+2 to seed this view.</p></div>';
        return;
    }
    outEl.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.mood_analytics.h2.per_mood_stats">Per-mood stats</h2>
            ${moodTable(r.stats)}
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.mood_analytics.h2.avg_p_l_by_mood_bars">Avg P/L by mood (bars)</h2>
            ${avgPnlBars(r.stats)}
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.mood_analytics.h2.avg_r_by_mood_bars">Avg R by mood (bars)</h2>
            ${avgRBars(r.stats)}
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.mood_analytics.h2.mood_distribution">Mood distribution</h2>
            ${distBars(r.mood_distribution)}
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.mood_analytics.h2.sample_trades_latest_50">Sample trades (latest 50)</h2>
            ${sampleTable(r.pairs.slice(-50).reverse())}
        </div>
    `;
}

function interpretCorr(c) {
    if (c == null) return 'insufficient samples';
    const a = Math.abs(c);
    const sign = c >= 0 ? 'positive' : 'negative';
    if (a < 0.1) return 'no meaningful relationship';
    if (a < 0.3) return `weak ${sign}`;
    if (a < 0.5) return `moderate ${sign}`;
    return `strong ${sign}`;
}

function moodTable(stats) {
    return `<table class="trades">
        <thead><tr>
            <th data-i18n="view.mood_analytics.th.mood">Mood</th><th data-i18n="view.mood_analytics.th.samples">Samples</th><th data-i18n="view.mood_analytics.th.wins">Wins</th><th data-i18n="view.mood_analytics.th.losses">Losses</th>
            <th data-i18n="view.mood_analytics.th.win_rate">Win rate</th><th data-i18n="view.mood_analytics.th.avg_p_l">Avg P/L</th><th data-i18n="view.mood_analytics.th.median_p_l">Median P/L</th><th data-i18n="view.mood_analytics.th.total_p_l">Total P/L</th><th data-i18n="view.mood_analytics.th.avg_r">Avg R</th>
        </tr></thead>
        <tbody>
        ${stats.map(s => `<tr>
            <td>${MOOD_LABELS[String(s.mood)] || s.mood}</td>
            <td>${s.sample_count}</td>
            <td class="pos">${s.win_count}</td>
            <td class="neg">${s.loss_count}</td>
            <td>${(s.win_rate * 100).toFixed(1)}%</td>
            <td class="${s.avg_pnl >= 0 ? 'pos' : 'neg'}">$${fmt(s.avg_pnl)}</td>
            <td class="${s.median_pnl >= 0 ? 'pos' : 'neg'}">$${fmt(s.median_pnl)}</td>
            <td class="${s.total_pnl >= 0 ? 'pos' : 'neg'}">$${fmt(s.total_pnl)}</td>
            <td>${s.avg_r == null ? '—' : (s.avg_r >= 0 ? '+' : '') + s.avg_r.toFixed(2) + 'R'}</td>
        </tr>`).join('')}
        </tbody></table>`;
}

function avgPnlBars(stats) {
    const max = Math.max(...stats.map(s => Math.abs(s.avg_pnl)), 1);
    return `<div style="display:grid;grid-template-columns:120px 1fr 100px;gap:6px;font-size:11px;">
        ${stats.map(s => {
            const pct = Math.abs(s.avg_pnl) / max * 100;
            const color = s.avg_pnl >= 0 ? '#7af0a8' : '#ff1f7a';
            return `<div>${MOOD_LABELS[String(s.mood)] || s.mood}</div>
                    <div style="height:18px;background:#1a1d2e;position:relative;">
                        <div style="width:${pct}%;height:100%;background:${color};"></div>
                    </div>
                    <div class="${s.avg_pnl >= 0 ? 'pos' : 'neg'}">$${fmt(s.avg_pnl)}</div>`;
        }).join('')}
    </div>`;
}

function avgRBars(stats) {
    const present = stats.filter(s => s.avg_r != null);
    if (!present.length) return '<p data-i18n="view.mood_analytics.hint.no_r_multiples_available_need_risk_amount_set_on_t" class="muted small">No R-multiples available (need risk_amount set on trades).</p>';
    const max = Math.max(...present.map(s => Math.abs(s.avg_r)), 0.5);
    return `<div style="display:grid;grid-template-columns:120px 1fr 100px;gap:6px;font-size:11px;">
        ${present.map(s => {
            const pct = Math.abs(s.avg_r) / max * 100;
            const color = s.avg_r >= 0 ? '#7af0a8' : '#ff1f7a';
            return `<div>${MOOD_LABELS[String(s.mood)] || s.mood}</div>
                    <div style="height:18px;background:#1a1d2e;position:relative;">
                        <div style="width:${pct}%;height:100%;background:${color};"></div>
                    </div>
                    <div class="${s.avg_r >= 0 ? 'pos' : 'neg'}">${(s.avg_r >= 0 ? '+' : '') + s.avg_r.toFixed(2)}R</div>`;
        }).join('')}
    </div>`;
}

function distBars(dist) {
    if (!dist.length) return '<p data-i18n="view.mood_analytics.hint.no_data" class="muted small">no data</p>';
    const max = Math.max(...dist.map(d => d[1]), 1);
    return `<div style="display:grid;grid-template-columns:120px 1fr 60px;gap:6px;font-size:11px;">
        ${dist.map(([mood, n]) => {
            const pct = n / max * 100;
            return `<div>${MOOD_LABELS[String(mood)] || mood}</div>
                    <div style="height:18px;background:#1a1d2e;">
                        <div style="width:${pct}%;height:100%;background:#00e5ff;"></div>
                    </div>
                    <div>${n}</div>`;
        }).join('')}
    </div>`;
}

function sampleTable(pairs) {
    return `<table class="trades">
        <thead><tr>
            <th data-i18n="view.mood_analytics.th.date">Date</th><th data-i18n="view.mood_analytics.th.source">Source</th><th data-i18n="view.mood_analytics.th.mood_2">Mood</th><th data-i18n="view.mood_analytics.th.symbol">Symbol</th><th data-i18n="view.mood_analytics.th.net_p_l">Net P/L</th><th>R</th>
        </tr></thead>
        <tbody>
        ${pairs.map(p => `<tr>
            <td class="small">${new Date(p.opened_at).toLocaleDateString()}</td>
            <td class="small muted">${esc(p.source)}</td>
            <td>${MOOD_LABELS[String(p.mood)] || p.mood}</td>
            <td>${esc(p.symbol)}</td>
            <td class="${p.net_pnl >= 0 ? 'pos' : 'neg'}">$${fmt(p.net_pnl)}</td>
            <td>${p.r_multiple == null ? '—' : (p.r_multiple >= 0 ? '+' : '') + p.r_multiple.toFixed(2) + 'R'}</td>
        </tr>`).join('')}
        </tbody></table>`;
}
