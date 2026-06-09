// Real-time UOA stream — every 60s the backend rotates through the
// top-20 most-active symbols, fetches each chain off Yahoo, runs the
// UOA scanner (vol/OI ≥ 2, volume ≥ 500, premium ≥ $100K by default),
// and broadcasts every fresh hit. TTS on hits over $500K.

import { wsUrl } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

let ws = null;
let viewTok = 0;
let voiceOn = true;
let typeFilter = 'all';   // 'all' | 'call' | 'put'
let sideFilter = 'all';   // 'all' | 'above_ask' | 'below_bid' | 'midpoint'
let minPremium = 100_000;
const rows = new Map();   // key → event

export async function renderUoaStream(mount, _state) {
    viewTok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.uoa_stream.title">// UOA STREAM · LIVE</span>
            <span class="status-dot" id="uoa-status" data-i18n-title="common.status.connecting" title="connecting">●</span>
            <label class="uoa-voice-toggle">
                <input type="checkbox" id="uoa-voice" ${voiceOn ? 'checked' : ''}>
                <span data-i18n="common.label.voice_alerts">voice alerts</span>
            </label>
        </h1>
        <p class="muted small" data-i18n-html="view.uoa_stream.intro">
            Rotating poller scans the top-20 most-active live-tick symbols every 60s,
            fetches each option chain off Yahoo, and runs the UOA scanner with default
            thresholds (vol/OI ≥ 2, volume ≥ 500, premium ≥ $100K). Above-ask /
            below-bid fills are flagged as aggressor side. TTS fires on hits above $500K.
        </p>
        <div class="chart-panel">
            <div class="uoa-controls" style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <div class="uoa-toggle" role="tablist" aria-label="type">
                    <button class="btn btn-sm" data-type="all"  data-i18n="view.uoa_stream.btn.all">All</button>
                    <button class="btn btn-sm" data-type="call" data-i18n="view.uoa_stream.btn.calls">Calls</button>
                    <button class="btn btn-sm" data-type="put"  data-i18n="view.uoa_stream.btn.puts">Puts</button>
                </div>
                <div class="uoa-toggle" role="tablist" aria-label="side">
                    <button class="btn btn-sm" data-side="all"       data-i18n="view.uoa_stream.btn.any_side">Any Fill</button>
                    <button class="btn btn-sm" data-side="above_ask" data-i18n="view.uoa_stream.btn.above_ask">Above Ask</button>
                    <button class="btn btn-sm" data-side="below_bid" data-i18n="view.uoa_stream.btn.below_bid">Below Bid</button>
                    <button class="btn btn-sm" data-side="midpoint"  data-i18n="view.uoa_stream.btn.midpoint">Midpoint</button>
                </div>
                <label class="uoa-min-premium">
                    <span data-i18n="view.uoa_stream.label.min_premium">min premium $</span>
                    <input type="number" id="uoa-min-premium" min="0" step="50000" value="${minPremium}" style="width:120px">
                </label>
                <button class="btn btn-sm" id="uoa-refresh" data-shortcut="r" data-i18n="common.btn.refresh">⟳ Refresh</button>
            </div>
            <table class="trades" id="uoa-table">
                <thead><tr>
                    <th data-i18n="view.uoa_stream.th.observed">Observed</th>
                    <th data-i18n="view.uoa_stream.th.symbol">Symbol</th>
                    <th data-i18n="view.uoa_stream.th.type">Type</th>
                    <th data-i18n="view.uoa_stream.th.strike">Strike</th>
                    <th data-i18n="view.uoa_stream.th.expiry">Expiry</th>
                    <th data-i18n="view.uoa_stream.th.volume">Volume</th>
                    <th data-i18n="view.uoa_stream.th.oi">OI</th>
                    <th data-i18n="view.uoa_stream.th.vol_oi">Vol/OI</th>
                    <th data-i18n="view.uoa_stream.th.premium">Premium $</th>
                    <th data-i18n="view.uoa_stream.th.side">Fill</th>
                </tr></thead>
                <tbody><tr><td colspan="10" class="muted" data-i18n="common.connecting">connecting…</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelector('#uoa-voice').addEventListener('change', (e) => { voiceOn = e.target.checked; });
    mount.querySelector('#uoa-min-premium').addEventListener('change', (e) => {
        const v = parseFloat(e.target.value);
        if (Number.isFinite(v) && v >= 0) { minPremium = v; render(); }
    });
    mount.querySelectorAll('[data-type]').forEach(b => {
        b.addEventListener('click', () => { typeFilter = b.dataset.type; applyToggleState(mount); render(); });
    });
    mount.querySelectorAll('[data-side]').forEach(b => {
        b.addEventListener('click', () => { sideFilter = b.dataset.side; applyToggleState(mount); render(); });
    });
    mount.querySelector('#uoa-refresh').addEventListener('click', () => {
        rows.clear();
        connectWs(mount, viewTok);
    });
    applyToggleState(mount);
    connectWs(mount, viewTok);
}

function applyToggleState(mount) {
    mount.querySelectorAll('[data-type]').forEach(b => {
        b.classList.toggle('active', b.dataset.type === typeFilter);
    });
    mount.querySelectorAll('[data-side]').forEach(b => {
        b.classList.toggle('active', b.dataset.side === sideFilter);
    });
}

function connectWs(mount, tok) {
    try { if (ws) { try { ws.close(); } catch (_) {} ws = null; } } catch (_) {}
    if (!viewIsCurrent(tok)) return;
    const dot = mount.querySelector('#uoa-status');
    if (!dot) return;
    ws = new WebSocket(wsUrl('/api/ws/uoa-stream'));
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
                for (const r of m.rows) rows.set(rowKey(r), r);
            } else if (m.type === 'hit' && m.row) {
                addRow(m.row);
            }
            render();
        } catch (_) {}
    });
}

function rowKey(r) { return `${r.symbol}|${r.expiry}|${r.strike}|${r.option_type}`; }

function addRow(r) {
    const k = rowKey(r);
    const fresh = !rows.has(k);
    rows.set(k, r);
    if (fresh && voiceOn && r.premium_paid >= 500_000) speak(r);
}

function speak(r) {
    try {
        const k = r.option_type === 'call' ? 'call' : 'put';
        const mm = Math.round(r.premium_paid / 1_000_000 * 10) / 10;
        const txt = mm >= 1
            ? `${spell(r.symbol)} ${mm} million ${k} premium.`
            : `${spell(r.symbol)} ${Math.round(r.premium_paid / 1000)} K ${k} premium.`;
        const u = new SpeechSynthesisUtterance(txt);
        u.rate = 1.1; u.pitch = 1.0; u.volume = 1.0;
        window.speechSynthesis.speak(u);
    } catch (_) {}
}

function spell(s) { return s.split('').join(' '); }

function render() {
    const tbody = document.querySelector('#uoa-table tbody');
    if (!tbody) return;
    let filtered = Array.from(rows.values()).filter(r => r.premium_paid >= minPremium);
    if (typeFilter !== 'all') {
        filtered = filtered.filter(r => r.option_type === typeFilter);
    }
    if (sideFilter !== 'all') {
        filtered = filtered.filter(r => r.fill_side === sideFilter);
    }
    filtered.sort((a, b) => new Date(b.observed_at) - new Date(a.observed_at));
    if (!filtered.length) {
        tbody.innerHTML = `<tr><td colspan="10" class="muted">${esc(t('view.uoa_stream.empty.no_rows'))}</td></tr>`;
        return;
    }
    tbody.innerHTML = filtered.map(r => {
        const cls = r.option_type === 'call' ? 'pos' : 'neg';
        const sideCls = r.fill_side === 'above_ask' ? 'pos'
                      : r.fill_side === 'below_bid' ? 'neg' : '';
        return `<tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
            <td>${esc(fmtDateTime(r.observed_at))}</td>
            <td><strong style="color:var(--accent)">${esc(r.symbol)}</strong></td>
            <td class="${cls}">${esc(r.option_type)}</td>
            <td>${r.strike.toFixed(2)}</td>
            <td>${esc(r.expiry)}</td>
            <td>${fmtN(r.volume)}</td>
            <td>${fmtN(r.open_interest)}</td>
            <td>${r.vol_oi_ratio.toFixed(2)}</td>
            <td>${fmtDollar(r.premium_paid)}</td>
            <td class="${sideCls}">${esc(fillLabel(r.fill_side))}</td>
        </tr>`;
    }).join('');
}

function fmtN(n) {
    if (n == null) return '—';
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(2) + 'M';
    if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
    return n.toFixed(0);
}
function fmtDollar(n) {
    if (n == null) return '—';
    if (n >= 1_000_000) return '$' + (n / 1_000_000).toFixed(2) + 'M';
    if (n >= 1_000) return '$' + (n / 1_000).toFixed(0) + 'K';
    return '$' + n.toFixed(0);
}
function fillLabel(s) {
    return s === 'above_ask' ? 'Above Ask'
         : s === 'below_bid' ? 'Below Bid'
         : 'Midpoint';
}
