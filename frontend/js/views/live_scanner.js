// Multi-panel LIVE scanner — DayTradeDash replacement.
//
// Pulls a real-time Finnhub WebSocket tick stream into a DashMap on the
// server. Browser receives every state update via /api/ws/ticks and
// re-ranks 6 panels in JS every 300 ms (Top Gappers, Top Gainers, Top
// Losers, HOD Momentum, Volume Surge, Ross 5-Pillar).

import { api, wsUrl } from '../api.js';
import { esc, fmt } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const states = new Map(); // symbol → SymbolState
let ws = null;
let rerenderHandle = null;
let voiceOn = true;
let viewTok = 0;
const announced = new Set();   // dedupe TTS alerts per session

export async function renderLiveScanner(mount, _state) {
    viewTok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.live_scanner.title">// LIVE SCANNER · DayTradeDash replacement</span>
            <span class="status-dot" id="ls-status" data-i18n-title="common.status.connecting" title="connecting">●</span>
            <label class="halt-voice-toggle" data-tip="view.live_scanner.tip.voice">
                <input type="checkbox" id="ls-voice" data-shortcut="live_scanner_toggle_voice" ${voiceOn ? 'checked' : ''}>
                <span data-i18n="common.label.voice_alerts">voice alerts</span>
            </label>
        </h1>

        <div class="chart-panel">
            <h2 data-i18n="view.live_scanner.h2.configure">Configure</h2>
            <form id="ls-config" class="inline-form">
                <label><span data-i18n="view.live_scanner.label.api_key">Finnhub API key</span>
                    <input name="api_key" type="password" placeholder="finnhub.io free tier (25 syms/conn)"
                           data-i18n-placeholder="view.live_scanner.placeholder.api_key"
                           data-tip="view.live_scanner.tip.api_key" style="min-width:280px">
                </label>
                <label><span data-i18n="view.live_scanner.label.symbols">Universe (comma-sep symbols)</span>
                    <input name="symbols" type="text" data-shortcut="focus_search" placeholder="AAPL,TSLA,NVDA,SPCE,GME,..."
                           data-i18n-placeholder="view.live_scanner.placeholder.symbols"
                           data-tip="view.live_scanner.tip.symbols" style="min-width:320px">
                </label>
                <button data-i18n="view.live_scanner.btn.connect" data-tip="view.live_scanner.tip.connect" data-shortcut="live_scanner_connect" class="primary" type="submit">Connect</button>
            </form>
            <p data-i18n="view.live_scanner.hint.finnhub_s_free_websocket_gives_25_subscriptions_pe" class="muted small">
                Finnhub's free WebSocket gives 25 subscriptions per connection.
                Larger universes chunk across parallel connections automatically.
                Key is held in process memory only — never written to disk.
            </p>
        </div>

        <div class="scanner-grid">
            <div class="scanner-panel"><h3 data-i18n="view.live_scanner.h3.top_gappers">Top Gappers</h3><div id="p-gap"></div></div>
            <div class="scanner-panel"><h3 data-i18n="view.live_scanner.h3.top_gainers_intraday">Top Gainers (intraday)</h3><div id="p-gain"></div></div>
            <div class="scanner-panel"><h3 data-i18n="view.live_scanner.h3.top_losers_intraday">Top Losers (intraday)</h3><div id="p-loss"></div></div>
            <div class="scanner-panel"><h3 data-i18n="view.live_scanner.h3.high_of_day">High of Day</h3><div id="p-hod"></div></div>
            <div class="scanner-panel"><h3 data-i18n="view.live_scanner.h3.volume_leaders">Volume Leaders</h3><div id="p-vol"></div></div>
            <div class="scanner-panel"><h3 data-i18n="view.live_scanner.h3.ross_5_pillar_gap_10_price_20">Ross 5-Pillar (Gap≥10%, Price≤$20)</h3><div id="p-ross"></div></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.live_scanner.h2.universe_chart">Universe change % snapshot</h2>
            <div id="ls-chart" style="width:100%;height:240px"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.live_scanner.h2.volume_chart">Day volume per symbol (live)</h2>
            <div id="ls-vol-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.live_scanner.hint.volume_chart" class="muted small">Sorted day volume across the live universe. Compare against change% above — a name moving big without volume is weaker than the chart suggests.</p>
        </div>
    `;

    mount.querySelector('#ls-voice').addEventListener('change', (e) => {
        voiceOn = e.target.checked;
    });

    mount.querySelector('#ls-config').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const symbols = fd.get('symbols').split(',').map(s => s.trim().toUpperCase()).filter(Boolean);
        const api_key = fd.get('api_key').trim() || undefined;
        try {
            const r = await api.configureLiveTicks({ api_key, symbols });
            if (!viewIsCurrent(viewTok)) return;
            connectWs(mount, viewTok);
            showToast(t('view.live_scanner.alert.subscribed', { n: r.subscribed, hasKey: r.has_key }), { level: 'success' });
        } catch (err) {
            showToast(t('view.live_scanner.alert.configure_failed', { err: err.message }), { level: 'error' });
        }
    });

    connectWs(mount, viewTok);
    if (rerenderHandle) clearInterval(rerenderHandle);
    rerenderHandle = setInterval(() => {
        if (!viewIsCurrent(viewTok)) {
            clearInterval(rerenderHandle);
            rerenderHandle = null;
            return;
        }
        rerender(mount);
    }, 300);
}

function connectWs(mount, tok) {
    try { if (ws) ws.close(); } catch (_) {}
    if (!viewIsCurrent(tok)) return;
    const dot = mount.querySelector('#ls-status');
    if (!dot) return;
    states.clear();
    announced.clear();
    ws = new WebSocket(wsUrl('/api/ws/ticks'));
    ws.addEventListener('open',  () => { if (viewIsCurrent(tok)) { dot.style.color = 'var(--green)'; dot.title = t('common.status.connected'); } });
    ws.addEventListener('error', () => { if (viewIsCurrent(tok)) { dot.style.color = 'var(--red)';   dot.title = t('common.status.error'); } });
    ws.addEventListener('close', () => {
        if (!viewIsCurrent(tok)) return;
        dot.style.color = 'var(--text-muted)'; dot.title = t('common.status.disconnected');
        setTimeout(() => { if (viewIsCurrent(tok)) connectWs(mount, tok); }, 4000);
    });
    ws.addEventListener('message', (e) => {
        try {
            const m = JSON.parse(e.data);
            if (m.type === 'snapshot') {
                for (const s of (m.states || [])) states.set(s.symbol, s);
            } else if (m.type === 'tick') {
                const s = m.state;
                const prev = states.get(s.symbol);
                states.set(s.symbol, s);
                checkAnnouncements(prev, s);
            }
        } catch (_) {}
    });
}

function checkAnnouncements(prev, s) {
    if (!voiceOn || !prev) return;
    // Announce when a symbol crosses the +5% gainer threshold for the first time today.
    if (prev.change_pct < 5 && s.change_pct >= 5 && !announced.has(`gain-${s.symbol}`)) {
        announced.add(`gain-${s.symbol}`);
        speak(t('view.live_scanner.tts.gainer', { symbol: spell(s.symbol), pct: s.change_pct.toFixed(1) }));
    }
    if (prev.change_pct > -5 && s.change_pct <= -5 && !announced.has(`loss-${s.symbol}`)) {
        announced.add(`loss-${s.symbol}`);
        speak(t('view.live_scanner.tts.loser', { symbol: spell(s.symbol), pct: Math.abs(s.change_pct).toFixed(1) }));
    }
    if (prev.day_high < s.day_high && s.day_pct > 0 && !announced.has(`hod-${s.symbol}-${Math.floor(s.day_high)}`)) {
        announced.add(`hod-${s.symbol}-${Math.floor(s.day_high)}`);
        speak(t('view.live_scanner.tts.new_high_of_day', { symbol: spell(s.symbol) }));
    }
}

function speak(text) {
    try {
        const u = new SpeechSynthesisUtterance(text);
        u.rate = 1.15; u.volume = 1.0;
        window.speechSynthesis.speak(u);
    } catch (_) {}
}
function spell(s) { return s.split('').join(' '); }

function rerender(mount) {
    const all = Array.from(states.values());
    panel(mount, 'p-gap',  all.filter(s => s.gap_pct      !== 0).sort((a, b) => b.gap_pct      - a.gap_pct).slice(0, 12), 'gap_pct');
    panel(mount, 'p-gain', all.filter(s => s.change_pct    > 0).sort((a, b) => b.change_pct   - a.change_pct).slice(0, 12), 'change_pct');
    panel(mount, 'p-loss', all.filter(s => s.change_pct    < 0).sort((a, b) => a.change_pct   - b.change_pct).slice(0, 12), 'change_pct');
    panel(mount, 'p-hod',  all.filter(s => Math.abs(s.hod_dist_pct) < 0.5).sort((a, b) => b.day_volume - a.day_volume).slice(0, 12), 'day_pct');
    panel(mount, 'p-vol',  all.slice().sort((a, b) => b.day_volume - a.day_volume).slice(0, 12), 'change_pct');
    panel(mount, 'p-ross', all.filter(s => s.gap_pct >= 10 && s.last > 0 && s.last <= 20).sort((a, b) => b.gap_pct - a.gap_pct).slice(0, 12), 'gap_pct');
    renderUniverseChart(all);
    renderVolumeChart(all);
}

function renderVolumeChart(all) {
    const el = document.getElementById('ls-vol-chart');
    if (!el || !window.uPlot) return;
    const valid = (all || []).filter(s => Number.isFinite(Number(s.day_volume)) && Number(s.day_volume) > 0);
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.live_scanner.empty_volume_chart">${esc(t('view.live_scanner.empty_volume_chart'))}</div>`;
        return;
    }
    if (el._uplot && el._uplot._count === valid.length) {
        const sorted = valid.slice().sort((a, b) => Number(b.day_volume) - Number(a.day_volume));
        const xs = sorted.map((_, i) => i + 1);
        const ys = sorted.map(s => Number(s.day_volume));
        el._uplot.setData([xs, ys]);
        return;
    }
    el.innerHTML = '';
    const sorted = valid.slice().sort((a, b) => Number(b.day_volume) - Number(a.day_volume));
    const labels = sorted.map(s => s.symbol);
    const ys = sorted.map(s => Number(s.day_volume));
    const xs = labels.map((_, i) => i + 1);
    const plot = new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.live_scanner.chart.symbol_idx') },
            { label: t('view.live_scanner.chart.day_volume'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 8, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
    plot._count = valid.length;
    el._uplot = plot;
}

function renderUniverseChart(all) {
    const el = document.getElementById('ls-chart');
    if (!el || !window.uPlot) return;
    const valid = (all || []).filter(s => Number.isFinite(Number(s.change_pct)));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.live_scanner.empty_chart">${esc(t('view.live_scanner.empty_chart'))}</div>`;
        return;
    }
    if (el._uplot && el._uplot._count === valid.length) {
        // Update in place to avoid re-creating chart every 300ms.
        const xs = valid.map((_, i) => i + 1);
        const ys = valid.slice().sort((a, b) => b.change_pct - a.change_pct).map(s => Number(s.change_pct));
        const zero = xs.map(() => 0);
        el._uplot.setData([xs, ys, zero]);
        return;
    }
    el.innerHTML = '';
    const sorted = valid.slice().sort((a, b) => b.change_pct - a.change_pct);
    const labels = sorted.map(s => s.symbol);
    const ys = sorted.map(s => Number(s.change_pct));
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    const plot = new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.live_scanner.chart.symbol_idx') },
            { label: t('view.live_scanner.chart.change_pct'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 8, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.live_scanner.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ys, zero], el);
    plot._count = valid.length;
    el._uplot = plot;
}

function panel(mount, id, rows, pctField) {
    const el = mount.querySelector('#' + id);
    if (!el) return;
    if (!rows.length) {
        el.innerHTML = '<div class="muted small">—</div>';
        return;
    }
    el.innerHTML = `<table class="trades scanner-table">
        <thead><tr><th data-i18n="view.live_scanner.th.sym">Sym</th><th data-i18n="view.live_scanner.th.last">Last</th><th>${pctLabel(pctField)}</th><th data-i18n="view.live_scanner.th.vol">Vol</th></tr></thead>
        <tbody>${rows.map(s => {
            const v = s[pctField] ?? 0;
            const cls = v >= 0 ? 'pos' : 'neg';
            return `<tr data-context-scope="symbol-row" data-symbol="${esc(s.symbol)}">
                <td><strong style="color:var(--accent)">${esc(s.symbol)}</strong></td>
                <td>${fmt(s.last)}</td>
                <td class="${cls}">${v.toFixed(2)}%</td>
                <td>${shortenVol(s.day_volume)}</td>
            </tr>`;
        }).join('')}</tbody>
    </table>`;
}

function pctLabel(field) {
    return ({
        gap_pct: 'Gap%',
        change_pct: 'Chg%',
        day_pct: 'Day%',
        hod_dist_pct: 'HOD%',
    })[field] || '%';
}

function shortenVol(v) {
    v = Number(v);
    if (!Number.isFinite(v)) return '—';
    if (v >= 1e9) return (v / 1e9).toFixed(2) + 'B';
    if (v >= 1e6) return (v / 1e6).toFixed(2) + 'M';
    if (v >= 1e3) return (v / 1e3).toFixed(1) + 'K';
    return v.toFixed(0);
}
