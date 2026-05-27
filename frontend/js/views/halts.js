// Halt scanner — live WebSocket stream of Nasdaq halts with reason codes,
// TTS voice alerts on every new halt.

import { api, wsUrl } from '../api.js';
import { esc, fmtDateTime } from '../util.js';

let ws = null;
let voiceOn = true;
const halts = new Map(); // dedupe key → halt

export async function renderHalts(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title">// HALT SCANNER · LIVE
            <span class="status-dot" id="halt-status" title="connecting">●</span>
            <label class="halt-voice-toggle">
                <input type="checkbox" id="halt-voice" ${voiceOn ? 'checked' : ''}>
                voice alerts
            </label>
        </h1>
        <p class="muted small">
            Polling <code>nasdaqtrader.com/rss.aspx?feed=tradehalts</code> every 3 seconds.
            New halts speak the symbol + reason via Web Speech API.
        </p>
        <div class="chart-panel">
            <h2>Active halts (live)</h2>
            <table class="trades" id="halts-table">
                <thead><tr>
                    <th>Time</th><th>Symbol</th><th>Issue</th>
                    <th>Reason</th><th>Resume Quote</th><th>Resume Trade</th>
                </tr></thead>
                <tbody><tr><td colspan="6" class="muted">connecting…</td></tr></tbody>
            </table>
        </div>
    `;
    document.getElementById('halt-voice').addEventListener('change', (e) => {
        voiceOn = e.target.checked;
    });
    connectWs(mount);
}

function connectWs(mount) {
    try {
        if (ws) { try { ws.close(); } catch (_) {} ws = null; }
    } catch (_) {}
    halts.clear();
    ws = new WebSocket(wsUrl('/api/ws/halts'));
    const dot = document.getElementById('halt-status');
    ws.addEventListener('open',  () => { dot.style.color = 'var(--green)'; dot.title = 'connected'; });
    ws.addEventListener('error', () => { dot.style.color = 'var(--red)';   dot.title = 'error'; });
    ws.addEventListener('close', () => {
        dot.style.color = 'var(--text-muted)'; dot.title = 'disconnected';
        setTimeout(() => { if (document.body.contains(mount)) connectWs(mount); }, 4000);
    });
    ws.addEventListener('message', (e) => {
        try {
            const m = JSON.parse(e.data);
            if (m.type === 'snapshot') {
                for (const h of m.halts) addHalt(h, /*announce=*/false);
            } else if (m.type === 'halt') {
                addHalt(m.halt, /*announce=*/true);
            }
            render();
        } catch (_) {}
    });
}

function addHalt(h, announce) {
    const key = `${h.symbol}|${h.halt_time}|${h.reason_code}`;
    const is_new = !halts.has(key);
    halts.set(key, h);
    if (is_new && announce && voiceOn) speak(h);
}

function speak(h) {
    try {
        const u = new SpeechSynthesisUtterance(
            `${spell(h.symbol)} halted. ${h.reason_label}.`
        );
        u.rate = 1.1;
        u.pitch = 1.0;
        u.volume = 1.0;
        window.speechSynthesis.speak(u);
    } catch (_) {}
}

// Spell ticker letter-by-letter so TTS pronounces SPCE, not "space".
function spell(s) {
    return s.split('').join(' ');
}

function render() {
    const tbody = document.querySelector('#halts-table tbody');
    if (!tbody) return;
    const all = Array.from(halts.values())
        .sort((a, b) => new Date(b.fetched_at) - new Date(a.fetched_at));
    if (!all.length) {
        tbody.innerHTML = '<tr><td colspan="6" class="muted">no halts in feed</td></tr>';
        return;
    }
    tbody.innerHTML = all.map(h => `
        <tr>
            <td>${esc(h.halt_time || fmtDateTime(h.fetched_at))}</td>
            <td><strong style="color:var(--accent)">${esc(h.symbol)}</strong></td>
            <td>${esc(h.issue_name)}</td>
            <td>
                <span class="halt-code halt-${reasonClass(h.reason_code)}">${esc(h.reason_code)}</span>
                ${esc(h.reason_label)}
            </td>
            <td>${esc(h.resumption_quote_time || '—')}</td>
            <td>${esc(h.resumption_trade_time || '—')}</td>
        </tr>
    `).join('');
}

function reasonClass(code) {
    if (!code) return 'unknown';
    if (code.startsWith('T1') || code === 'T2' || code === 'T3') return 'news';
    if (code.startsWith('LU') || code === 'T5') return 'volatility';
    if (code.startsWith('MWC')) return 'circuit';
    if (code.startsWith('H')) return 'regulatory';
    if (code === 'IPO' || code === 'IPOQ') return 'ipo';
    if (code.startsWith('R') || code === 'C3') return 'resume';
    return 'other';
}
