// Gamma-squeeze candidate detector — live WS stream of equities whose
// option chain shows negative dealer GEX ≥ $250M AND spot within 2% of
// the largest-negative-strike pin. Background poller runs the rule on
// the top-10 most-active symbols every 90s. TTS on every fresh hit.

import { wsUrl } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

let ws = null;
let viewTok = 0;
let voiceOn = true;
const rows = new Map();

export async function renderGammaSqueeze(mount, _state) {
    viewTok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.gamma_squeeze.title">// GAMMA SQUEEZE CANDIDATES · LIVE</span>
            <span class="status-dot" id="gs-status" data-i18n-title="common.status.connecting" title="connecting">●</span>
            <label class="gs-voice-toggle">
                <input type="checkbox" id="gs-voice" ${voiceOn ? 'checked' : ''}>
                <span data-i18n="common.label.voice_alerts">voice alerts</span>
            </label>
        </h1>
        <p class="muted small" data-i18n-html="view.gamma_squeeze.intro">
            Polls the top-10 most-active symbols' Yahoo option chains every 90s, computes
            per-strike dealer-net gamma exposure with inline Black-Scholes, and flags any
            chain whose total GEX is negative ≥ $250M AND spot is within 2% of the
            largest-negative-strike "pin". Negative GEX = dealers short gamma = momentum-
            amplifying re-hedging. TTS on every fresh hit.
        </p>
        <div class="chart-panel">
            <div class="gs-controls" style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <button class="btn btn-sm" id="gs-refresh" data-shortcut="r" data-i18n="common.btn.refresh">⟳ Refresh</button>
            </div>
            <table class="trades" id="gs-table">
                <thead><tr>
                    <th data-i18n="view.gamma_squeeze.th.observed">Observed</th>
                    <th data-i18n="view.gamma_squeeze.th.symbol">Symbol</th>
                    <th data-i18n="view.gamma_squeeze.th.spot">Spot</th>
                    <th data-i18n="view.gamma_squeeze.th.neg_strike">Pin (Neg GEX)</th>
                    <th data-i18n="view.gamma_squeeze.th.distance">Distance %</th>
                    <th data-i18n="view.gamma_squeeze.th.total_gex">Total GEX</th>
                    <th data-i18n="view.gamma_squeeze.th.zero_gamma">Zero-Γ</th>
                    <th data-i18n="view.gamma_squeeze.th.pos_strike">Largest Pos</th>
                    <th data-i18n="view.gamma_squeeze.th.expiry">Expiry</th>
                </tr></thead>
                <tbody><tr><td colspan="9" class="muted" data-i18n="common.connecting">connecting…</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelector('#gs-voice').addEventListener('change', (e) => { voiceOn = e.target.checked; });
    mount.querySelector('#gs-refresh').addEventListener('click', () => {
        rows.clear();
        connectWs(mount, viewTok);
    });
    connectWs(mount, viewTok);
}

function connectWs(mount, tok) {
    try { if (ws) { try { ws.close(); } catch (_) {} ws = null; } } catch (_) {}
    if (!viewIsCurrent(tok)) return;
    const dot = mount.querySelector('#gs-status');
    if (!dot) return;
    ws = new WebSocket(wsUrl('/api/ws/gamma-squeeze'));
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
            } else if (m.type === 'candidate' && m.row) {
                addRow(m.row);
            }
            render();
        } catch (_) {}
    });
}

function rowKey(r) {
    return `${r.symbol}|${r.expiry}|${r.largest_negative_strike}`;
}

function addRow(r) {
    const k = rowKey(r);
    const fresh = !rows.has(k);
    rows.set(k, r);
    if (fresh && voiceOn) speak(r);
}

function speak(r) {
    try {
        const gexB = Math.abs(r.total_gex) / 1_000_000_000;
        const u = new SpeechSynthesisUtterance(
            `${spell(r.symbol)} gamma squeeze candidate. ${gexB.toFixed(1)} billion negative GEX. Spot near pin.`
        );
        u.rate = 1.1; u.pitch = 1.0; u.volume = 1.0;
        window.speechSynthesis.speak(u);
    } catch (_) {}
}

function spell(s) { return s.split('').join(' '); }

function render() {
    const tbody = document.querySelector('#gs-table tbody');
    if (!tbody) return;
    const sorted = Array.from(rows.values())
        .sort((a, b) => new Date(b.observed_at) - new Date(a.observed_at));
    if (!sorted.length) {
        tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(t('view.gamma_squeeze.empty.no_rows'))}</td></tr>`;
        return;
    }
    tbody.innerHTML = sorted.map(r => {
        const distCls = r.pin_distance_pct != null && r.pin_distance_pct < 0 ? 'neg' : 'pos';
        return `<tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
            <td>${esc(fmtDateTime(r.observed_at))}</td>
            <td><strong style="color:var(--accent)">${esc(r.symbol)}</strong></td>
            <td>${r.spot.toFixed(2)}</td>
            <td>${r.largest_negative_strike == null ? '—' : r.largest_negative_strike.toFixed(2)}</td>
            <td class="${distCls}">${fmtPct(r.pin_distance_pct)}</td>
            <td class="neg">${fmtGex(r.total_gex)}</td>
            <td>${r.zero_gamma_strike == null ? '—' : r.zero_gamma_strike.toFixed(2)}</td>
            <td>${r.largest_positive_strike == null ? '—' : r.largest_positive_strike.toFixed(2)}</td>
            <td>${esc(r.expiry)}</td>
        </tr>`;
    }).join('');
}

function fmtPct(n) {
    if (n == null) return '—';
    return (n >= 0 ? '+' : '') + n.toFixed(2) + '%';
}

function fmtGex(n) {
    if (n == null) return '—';
    const abs = Math.abs(n);
    const sign = n < 0 ? '-' : '';
    if (abs >= 1_000_000_000) return `${sign}$${(abs / 1_000_000_000).toFixed(2)}B`;
    if (abs >= 1_000_000) return `${sign}$${(abs / 1_000_000).toFixed(0)}M`;
    return `${sign}$${abs.toFixed(0)}`;
}
