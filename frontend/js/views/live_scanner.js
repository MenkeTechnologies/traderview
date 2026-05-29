// Multi-panel LIVE scanner — DayTradeDash replacement.
//
// Pulls a real-time Finnhub WebSocket tick stream into a DashMap on the
// server. Browser receives every state update via /api/ws/ticks and
// re-ranks 6 panels in JS every 300 ms (Top Gappers, Top Gainers, Top
// Losers, HOD Momentum, Volume Surge, Ross 5-Pillar).

import { api, wsUrl } from '../api.js';
import { esc, fmt } from '../util.js';
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
        <h1 class="view-title">// LIVE SCANNER · DayTradeDash replacement
            <span class="status-dot" id="ls-status" title="connecting">●</span>
            <label class="halt-voice-toggle">
                <input type="checkbox" id="ls-voice" ${voiceOn ? 'checked' : ''}>
                voice alerts
            </label>
        </h1>

        <div class="chart-panel">
            <h2 data-i18n="view.live_scanner.h2.configure">Configure</h2>
            <form id="ls-config" class="inline-form">
                <label><span data-i18n="view.live_scanner.label.api_key">Finnhub API key</span>
                    <input name="api_key" type="password" placeholder="finnhub.io free tier (25 syms/conn)"
                           data-i18n-placeholder="view.live_scanner.placeholder.api_key" style="min-width:280px">
                </label>
                <label><span data-i18n="view.live_scanner.label.symbols">Universe (comma-sep symbols)</span>
                    <input name="symbols" type="text" placeholder="AAPL,TSLA,NVDA,SPCE,GME,..."
                           data-i18n-placeholder="view.live_scanner.placeholder.symbols" style="min-width:320px">
                </label>
                <button data-i18n="view.live_scanner.btn.connect" class="primary" type="submit">Connect</button>
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
            alert(`Subscribed: ${r.subscribed} symbols. has_key=${r.has_key}`);
        } catch (err) {
            alert('Configure failed: ' + err.message);
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
    ws.addEventListener('open',  () => { if (viewIsCurrent(tok)) { dot.style.color = 'var(--green)'; dot.title = 'connected'; } });
    ws.addEventListener('error', () => { if (viewIsCurrent(tok)) { dot.style.color = 'var(--red)';   dot.title = 'error'; } });
    ws.addEventListener('close', () => {
        if (!viewIsCurrent(tok)) return;
        dot.style.color = 'var(--text-muted)'; dot.title = 'disconnected';
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
        speak(`${spell(s.symbol)} up ${s.change_pct.toFixed(1)} percent`);
    }
    if (prev.change_pct > -5 && s.change_pct <= -5 && !announced.has(`loss-${s.symbol}`)) {
        announced.add(`loss-${s.symbol}`);
        speak(`${spell(s.symbol)} down ${Math.abs(s.change_pct).toFixed(1)} percent`);
    }
    if (prev.day_high < s.day_high && s.day_pct > 0 && !announced.has(`hod-${s.symbol}-${Math.floor(s.day_high)}`)) {
        announced.add(`hod-${s.symbol}-${Math.floor(s.day_high)}`);
        speak(`${spell(s.symbol)} new high of day`);
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
            return `<tr>
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
