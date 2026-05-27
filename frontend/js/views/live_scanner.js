// Multi-panel LIVE scanner — DayTradeDash replacement.
//
// Pulls a real-time Finnhub WebSocket tick stream into a DashMap on the
// server. Browser receives every state update via /api/ws/ticks and
// re-ranks 6 panels in JS every 300 ms (Top Gappers, Top Gainers, Top
// Losers, HOD Momentum, Volume Surge, Ross 5-Pillar).

import { api, wsUrl } from '../api.js';
import { esc, fmt } from '../util.js';

const states = new Map(); // symbol → SymbolState
let ws = null;
let rerenderHandle = null;
let voiceOn = true;
const announced = new Set();   // dedupe TTS alerts per session

export async function renderLiveScanner(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title">// LIVE SCANNER · DayTradeDash replacement
            <span class="status-dot" id="ls-status" title="connecting">●</span>
            <label class="halt-voice-toggle">
                <input type="checkbox" id="ls-voice" ${voiceOn ? 'checked' : ''}>
                voice alerts
            </label>
        </h1>

        <div class="chart-panel">
            <h2>Configure</h2>
            <form id="ls-config" class="inline-form">
                <label>Finnhub API key
                    <input name="api_key" type="password" placeholder="finnhub.io free tier (25 syms/conn)" style="min-width:280px">
                </label>
                <label>Universe (comma-sep symbols)
                    <input name="symbols" type="text" placeholder="AAPL,TSLA,NVDA,SPCE,GME,..." style="min-width:320px">
                </label>
                <button class="primary" type="submit">Connect</button>
            </form>
            <p class="muted small">
                Finnhub's free WebSocket gives 25 subscriptions per connection.
                Larger universes chunk across parallel connections automatically.
                Key is held in process memory only — never written to disk.
            </p>
        </div>

        <div class="scanner-grid">
            <div class="scanner-panel"><h3>Top Gappers</h3><div id="p-gap"></div></div>
            <div class="scanner-panel"><h3>Top Gainers (intraday)</h3><div id="p-gain"></div></div>
            <div class="scanner-panel"><h3>Top Losers (intraday)</h3><div id="p-loss"></div></div>
            <div class="scanner-panel"><h3>High of Day</h3><div id="p-hod"></div></div>
            <div class="scanner-panel"><h3>Volume Leaders</h3><div id="p-vol"></div></div>
            <div class="scanner-panel"><h3>Ross 5-Pillar (Gap≥10%, Price≤$20)</h3><div id="p-ross"></div></div>
        </div>
    `;

    document.getElementById('ls-voice').addEventListener('change', (e) => {
        voiceOn = e.target.checked;
    });

    document.getElementById('ls-config').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const symbols = fd.get('symbols').split(',').map(s => s.trim().toUpperCase()).filter(Boolean);
        const api_key = fd.get('api_key').trim() || undefined;
        try {
            const r = await api.configureLiveTicks({ api_key, symbols });
            connectWs(mount);
            alert(`Subscribed: ${r.subscribed} symbols. has_key=${r.has_key}`);
        } catch (err) {
            alert('Configure failed: ' + err.message);
        }
    });

    connectWs(mount);
    if (rerenderHandle) clearInterval(rerenderHandle);
    rerenderHandle = setInterval(rerender, 300);
}

function connectWs(mount) {
    try { if (ws) ws.close(); } catch (_) {}
    states.clear();
    announced.clear();
    ws = new WebSocket(wsUrl('/api/ws/ticks'));
    const dot = document.getElementById('ls-status');
    if (!dot) return;
    ws.addEventListener('open',  () => { dot.style.color = 'var(--green)';      dot.title = 'connected'; });
    ws.addEventListener('error', () => { dot.style.color = 'var(--red)';        dot.title = 'error'; });
    ws.addEventListener('close', () => {
        dot.style.color = 'var(--text-muted)'; dot.title = 'disconnected';
        setTimeout(() => { if (document.body.contains(mount)) connectWs(mount); }, 4000);
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

function rerender() {
    const all = Array.from(states.values());
    panel('p-gap',  all.filter(s => s.gap_pct      !== 0).sort((a, b) => b.gap_pct      - a.gap_pct).slice(0, 12), 'gap_pct');
    panel('p-gain', all.filter(s => s.change_pct    > 0).sort((a, b) => b.change_pct   - a.change_pct).slice(0, 12), 'change_pct');
    panel('p-loss', all.filter(s => s.change_pct    < 0).sort((a, b) => a.change_pct   - b.change_pct).slice(0, 12), 'change_pct');
    panel('p-hod',  all.filter(s => Math.abs(s.hod_dist_pct) < 0.5).sort((a, b) => b.day_volume - a.day_volume).slice(0, 12), 'day_pct');
    panel('p-vol',  all.slice().sort((a, b) => b.day_volume - a.day_volume).slice(0, 12), 'change_pct');
    panel('p-ross', all.filter(s => s.gap_pct >= 10 && s.last > 0 && s.last <= 20).sort((a, b) => b.gap_pct - a.gap_pct).slice(0, 12), 'gap_pct');
}

function panel(id, rows, pctField) {
    const el = document.getElementById(id);
    if (!el) return;
    if (!rows.length) {
        el.innerHTML = '<div class="muted small">—</div>';
        return;
    }
    el.innerHTML = `<table class="trades scanner-table">
        <thead><tr><th>Sym</th><th>Last</th><th>${pctLabel(pctField)}</th><th>Vol</th></tr></thead>
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
