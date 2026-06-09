// Sentiment velocity — fires when a symbol's hour-over-hour mention
// count grows ≥3× for 2 consecutive 15-min refresh cycles AND hits
// the 20-mention floor. 12-hour cooldown per symbol.

import { wsUrl } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

let ws = null;
let viewTok = 0;
let voiceOn = true;
const rows = new Map();

export async function renderSentimentVelocity(mount, _state) {
    viewTok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.sentiment_velocity.title">// SENTIMENT VELOCITY · LIVE</span>
            <span class="status-dot" id="sv-status" data-i18n-title="common.status.connecting" title="connecting">●</span>
            <label class="sv-voice-toggle">
                <input type="checkbox" id="sv-voice" ${voiceOn ? 'checked' : ''}>
                <span data-i18n="common.label.voice_alerts">voice alerts</span>
            </label>
        </h1>
        <p class="muted small" data-i18n-html="view.sentiment_velocity.intro">
            Wraps the existing WSB + Stocktwits mentions firehose. Every 15 min the
            detector queries hour-over-hour ranked counts. A symbol fires when its
            ratio is ≥3× and current count ≥20 for 2 consecutive checks. 12-hour
            cooldown per symbol blocks spam. Single hour of quiet resets the
            consecutive counter — sustained acceleration is the signal, one-off
            spikes aren't. TTS on every fresh velocity event.
        </p>
        <div class="chart-panel">
            <div class="sv-controls" style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <button class="btn btn-sm" id="sv-refresh" data-shortcut="r" data-i18n="common.btn.refresh">⟳ Refresh</button>
            </div>
            <table class="trades" id="sv-table">
                <thead><tr>
                    <th data-i18n="view.sentiment_velocity.th.fired">Fired</th>
                    <th data-i18n="view.sentiment_velocity.th.symbol">Symbol</th>
                    <th data-i18n="view.sentiment_velocity.th.ratio">Ratio ×</th>
                    <th data-i18n="view.sentiment_velocity.th.cur">Current</th>
                    <th data-i18n="view.sentiment_velocity.th.prev">Prior Hour</th>
                    <th data-i18n="view.sentiment_velocity.th.avg_sent">Avg Sentiment</th>
                    <th data-i18n="view.sentiment_velocity.th.delta">Δ Sentiment</th>
                    <th data-i18n="view.sentiment_velocity.th.consec">Consecutive</th>
                </tr></thead>
                <tbody><tr><td colspan="8" class="muted" data-i18n="common.connecting">connecting…</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelector('#sv-voice').addEventListener('change', (e) => { voiceOn = e.target.checked; });
    mount.querySelector('#sv-refresh').addEventListener('click', () => {
        rows.clear();
        connectWs(mount, viewTok);
    });
    connectWs(mount, viewTok);
}

function connectWs(mount, tok) {
    try { if (ws) { try { ws.close(); } catch (_) {} ws = null; } } catch (_) {}
    if (!viewIsCurrent(tok)) return;
    const dot = mount.querySelector('#sv-status');
    if (!dot) return;
    ws = new WebSocket(wsUrl('/api/ws/sentiment-velocity'));
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
            } else if (m.type === 'velocity' && m.row) {
                addRow(m.row);
            }
            render();
        } catch (_) {}
    });
}

function rowKey(r) { return `${r.symbol}|${r.fired_at}`; }

function addRow(r) {
    const k = rowKey(r);
    const fresh = !rows.has(k);
    rows.set(k, r);
    if (fresh && voiceOn) speak(r);
}

function speak(r) {
    try {
        const dir = r.avg_sentiment >= 0 ? 'bullish' : 'bearish';
        const u = new SpeechSynthesisUtterance(
            `${spell(r.symbol)} ${dir} sentiment velocity. ${r.ratio.toFixed(1)} times. ${r.mention_count} mentions.`
        );
        u.rate = 1.1; u.pitch = 1.0; u.volume = 1.0;
        window.speechSynthesis.speak(u);
    } catch (_) {}
}

function spell(s) { return s.split('').join(' '); }

function render() {
    const tbody = document.querySelector('#sv-table tbody');
    if (!tbody) return;
    const all = Array.from(rows.values())
        .sort((a, b) => new Date(b.fired_at) - new Date(a.fired_at));
    if (!all.length) {
        tbody.innerHTML = `<tr><td colspan="8" class="muted">${esc(t('view.sentiment_velocity.empty.no_rows'))}</td></tr>`;
        return;
    }
    tbody.innerHTML = all.map(r => {
        const sentCls = r.avg_sentiment >= 0 ? 'pos' : 'neg';
        const deltaCls = r.sentiment_delta >= 0 ? 'pos' : 'neg';
        const ratioStr = Number.isFinite(r.ratio) ? r.ratio.toFixed(2) + '×' : '∞';
        return `<tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
            <td>${esc(fmtDateTime(r.fired_at))}</td>
            <td><strong style="color:var(--accent)">${esc(r.symbol)}</strong></td>
            <td class="pos"><strong>${ratioStr}</strong></td>
            <td>${r.mention_count.toLocaleString()}</td>
            <td>${r.prev_count.toLocaleString()}</td>
            <td class="${sentCls}">${r.avg_sentiment.toFixed(3)}</td>
            <td class="${deltaCls}">${r.sentiment_delta >= 0 ? '+' : ''}${r.sentiment_delta.toFixed(3)}</td>
            <td>${r.consecutive_hours}</td>
        </tr>`;
    }).join('');
}
