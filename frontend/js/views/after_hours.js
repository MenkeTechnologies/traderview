// After-hours mover scanner — live WebSocket stream of pre-market
// (04:00–09:30 ET) and post-market (16:00–20:00 ET) movers ranked by
// signed change vs prior RTH close. TTS the first time a symbol
// crosses ±5%.

import { wsUrl } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

let ws = null;
let viewTok = 0;
let voiceOn = true;
let direction = 'gainers'; // 'gainers' | 'losers'
let session = 'post';      // 'pre' | 'post'
let minPct = 1.0;
const rows = new Map();    // symbol → row
const announced = new Set(); // symbols already TTS'd at ±5% in this session

export async function renderAfterHours(mount, _state) {
    viewTok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.after_hours.title">// AFTER-HOURS MOVERS · LIVE</span>
            <span class="status-dot" id="ah-status" data-i18n-title="common.status.connecting" title="connecting">●</span>
            <label class="ah-voice-toggle">
                <input type="checkbox" id="ah-voice" ${voiceOn ? 'checked' : ''}>
                <span data-i18n="common.label.voice_alerts">voice alerts</span>
            </label>
        </h1>
        <p class="muted small" data-i18n-html="view.after_hours.intro">
            Classifies every Finnhub / Polygon / Alpaca trade by US-Eastern wall clock
            (PRE 04:00–09:30, POST 16:00–20:00) and ranks the largest signed moves vs
            the prior RTH close. Voice alert fires once per symbol when it crosses ±5%.
        </p>
        <div class="chart-panel">
            <div class="ah-controls" style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <div class="ah-toggle" role="tablist" aria-label="session">
                    <button class="btn btn-sm" id="ah-session-pre"  data-session="pre" data-i18n="view.after_hours.btn.pre">Pre-market</button>
                    <button class="btn btn-sm" id="ah-session-post" data-session="post" data-i18n="view.after_hours.btn.post">Post-market</button>
                </div>
                <div class="ah-toggle" role="tablist" aria-label="direction">
                    <button class="btn btn-sm" id="ah-dir-gainers" data-dir="gainers" data-i18n="view.after_hours.btn.gainers">Gainers</button>
                    <button class="btn btn-sm" id="ah-dir-losers"  data-dir="losers"  data-i18n="view.after_hours.btn.losers">Losers</button>
                </div>
                <label class="ah-min-pct">
                    <span data-i18n="view.after_hours.label.min_pct">min |%| change</span>
                    <input type="number" id="ah-min-pct" min="0" max="500" step="0.1" value="${minPct}" style="width:64px">
                </label>
                <button class="btn btn-sm" id="ah-refresh" data-shortcut="r" data-i18n="common.btn.refresh">⟳ Refresh</button>
            </div>
            <table class="trades" id="ah-table">
                <thead><tr>
                    <th data-i18n="view.after_hours.th.symbol">Symbol</th>
                    <th data-i18n="view.after_hours.th.last">Last</th>
                    <th data-i18n="view.after_hours.th.rth_close">RTH Close</th>
                    <th data-i18n="view.after_hours.th.change_pct">Change %</th>
                    <th data-i18n="view.after_hours.th.session_open">AH Open</th>
                    <th data-i18n="view.after_hours.th.range_pct">Range %</th>
                    <th data-i18n="view.after_hours.th.volume">AH Volume</th>
                    <th data-i18n="view.after_hours.th.trades">Trades</th>
                    <th data-i18n="view.after_hours.th.last_trade">Last Trade</th>
                </tr></thead>
                <tbody><tr><td colspan="9" class="muted" data-i18n="common.connecting">connecting…</td></tr></tbody>
            </table>
        </div>
    `;

    mount.querySelector('#ah-voice').addEventListener('change', (e) => { voiceOn = e.target.checked; });
    mount.querySelector('#ah-min-pct').addEventListener('change', (e) => {
        const v = parseFloat(e.target.value);
        if (Number.isFinite(v) && v >= 0) { minPct = v; render(); }
    });
    mount.querySelectorAll('.ah-toggle [data-session]').forEach(btn => {
        btn.addEventListener('click', () => { session = btn.dataset.session; announced.clear(); applyToggleState(mount); render(); });
    });
    mount.querySelectorAll('.ah-toggle [data-dir]').forEach(btn => {
        btn.addEventListener('click', () => { direction = btn.dataset.dir; applyToggleState(mount); render(); });
    });
    mount.querySelector('#ah-refresh').addEventListener('click', () => {
        rows.clear(); announced.clear(); render();
        connectWs(mount, viewTok);
    });
    applyToggleState(mount);
    connectWs(mount, viewTok);
}

function applyToggleState(mount) {
    mount.querySelectorAll('[data-session]').forEach(b => {
        b.classList.toggle('active', b.dataset.session === session);
    });
    mount.querySelectorAll('[data-dir]').forEach(b => {
        b.classList.toggle('active', b.dataset.dir === direction);
    });
}

function connectWs(mount, tok) {
    try { if (ws) { try { ws.close(); } catch (_) {} ws = null; } } catch (_) {}
    if (!viewIsCurrent(tok)) return;
    const dot = mount.querySelector('#ah-status');
    if (!dot) return;
    ws = new WebSocket(wsUrl('/api/ws/after-hours'));
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
            if (m.type === 'snapshot' && Array.isArray(m.rows)) {
                rows.clear();
                for (const r of m.rows) rows.set(r.symbol, r);
            } else if (m.type === 'update' && m.row) {
                addRow(m.row, /*announce=*/true);
            }
            render();
        } catch (_) {}
    });
}

function addRow(r, announce) {
    rows.set(r.symbol, r);
    if (!announce || !voiceOn) return;
    // Speak the first time a symbol crosses ±5% in the current session.
    if (r.session !== session) return;
    if (Math.abs(r.change_pct) >= 5.0 && !announced.has(r.symbol)) {
        announced.add(r.symbol);
        speak(r);
    }
}

function speak(r) {
    try {
        const dir = r.change_pct >= 0 ? 'up' : 'down';
        const pct = Math.abs(r.change_pct).toFixed(1);
        const u = new SpeechSynthesisUtterance(`${spell(r.symbol)} ${dir} ${pct} percent after hours.`);
        u.rate = 1.1; u.pitch = 1.0; u.volume = 1.0;
        window.speechSynthesis.speak(u);
    } catch (_) {}
}

function spell(s) { return s.split('').join(' '); }

function render() {
    const tbody = document.querySelector('#ah-table tbody');
    if (!tbody) return;
    const filtered = Array.from(rows.values())
        .filter(r => r.session === session && Math.abs(r.change_pct) >= minPct);
    filtered.sort((a, b) => direction === 'gainers'
        ? b.change_pct - a.change_pct
        : a.change_pct - b.change_pct);
    if (!filtered.length) {
        tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(t('view.after_hours.empty.no_movers'))}</td></tr>`;
        return;
    }
    tbody.innerHTML = filtered.map(r => {
        const cls = r.change_pct >= 0 ? 'pos' : 'neg';
        return `<tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
            <td><strong style="color:var(--accent)">${esc(r.symbol)}</strong></td>
            <td>${fmt(r.ah_last)}</td>
            <td>${r.rth_close == null ? '—' : fmt(r.rth_close)}</td>
            <td class="${cls}">${fmtPct(r.change_pct)}</td>
            <td>${r.session_open == null ? '—' : fmt(r.session_open)}</td>
            <td>${fmtPct(r.range_pct)}</td>
            <td>${fmtVol(r.ah_volume)}</td>
            <td>${r.trade_count.toLocaleString()}</td>
            <td>${esc(fmtDateTime(r.last_trade_at))}</td>
        </tr>`;
    }).join('');
}

function fmt(n) { return n == null ? '—' : (n < 1 ? n.toFixed(4) : n.toFixed(2)); }
function fmtPct(n) { return n == null ? '—' : (n >= 0 ? '+' : '') + n.toFixed(2) + '%'; }
function fmtVol(n) {
    if (n == null) return '—';
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(2) + 'M';
    if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
    return n.toFixed(0);
}
