// Catalyst → price-action correlations — live WS stream of catalysts
// that produced a ≥2% move within 60 seconds, scored by sentiment
// lexicon (bullish / bearish / neutral). TTS on every new correlation.

import { wsUrl } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

let ws = null;
let viewTok = 0;
let voiceOn = true;
let filterSentiment = 'all'; // 'all' | 'bullish' | 'bearish' | 'neutral'
let filterSymbol = '';
const rows = new Map(); // (catalyst_id|symbol) → correlation

export async function renderCatalystCorrelations(mount, _state) {
    viewTok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.catalyst_correlations.title">// CATALYST → PRICE-ACTION CORRELATIONS · LIVE</span>
            <span class="status-dot" id="cc-status" data-i18n-title="common.status.connecting" title="connecting">●</span>
            <label class="cc-voice-toggle">
                <input type="checkbox" id="cc-voice" ${voiceOn ? 'checked' : ''}>
                <span data-i18n="common.label.voice_alerts">voice alerts</span>
            </label>
        </h1>
        <p class="muted small" data-i18n-html="view.catalyst_correlations.intro">
            Every fresh SEC filing or PR-wire headline opens a 60-second watch window on
            its extracted tickers. The first trade that moves ≥2% from the catalyst-time
            baseline locks in a correlation row. Sentiment is scored by a conservative
            lexicon (no LLM). Latency column is the time from catalyst publish to the
            threshold-crossing trade.
        </p>
        <div class="chart-panel">
            <div class="cc-controls" style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <div class="cc-toggle" role="tablist" aria-label="sentiment">
                    <button class="btn btn-sm" data-sentiment="all"     data-i18n="view.catalyst_correlations.btn.all">All</button>
                    <button class="btn btn-sm" data-sentiment="bullish" data-i18n="view.catalyst_correlations.btn.bullish">Bullish</button>
                    <button class="btn btn-sm" data-sentiment="bearish" data-i18n="view.catalyst_correlations.btn.bearish">Bearish</button>
                    <button class="btn btn-sm" data-sentiment="neutral" data-i18n="view.catalyst_correlations.btn.neutral">Neutral</button>
                </div>
                <label class="cc-symbol-filter">
                    <span data-i18n="view.catalyst_correlations.label.symbol">symbol filter</span>
                    <input type="text" id="cc-symbol" placeholder="AAPL" style="width:80px;text-transform:uppercase">
                </label>
                <button class="btn btn-sm" id="cc-refresh" data-shortcut="r" data-i18n="common.btn.refresh">⟳ Refresh</button>
            </div>
            <table class="trades" id="cc-table">
                <thead><tr>
                    <th data-i18n="view.catalyst_correlations.th.observed">Observed</th>
                    <th data-i18n="view.catalyst_correlations.th.symbol">Symbol</th>
                    <th data-i18n="view.catalyst_correlations.th.sentiment">Sentiment</th>
                    <th data-i18n="view.catalyst_correlations.th.move">Signed Move %</th>
                    <th data-i18n="view.catalyst_correlations.th.peak">Peak %</th>
                    <th data-i18n="view.catalyst_correlations.th.trough">Trough %</th>
                    <th data-i18n="view.catalyst_correlations.th.latency">Latency</th>
                    <th data-i18n="view.catalyst_correlations.th.kind">Kind</th>
                    <th data-i18n="view.catalyst_correlations.th.source">Source</th>
                    <th data-i18n="view.catalyst_correlations.th.title">Headline</th>
                </tr></thead>
                <tbody><tr><td colspan="10" class="muted" data-i18n="common.connecting">connecting…</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelector('#cc-voice').addEventListener('change', (e) => { voiceOn = e.target.checked; });
    mount.querySelector('#cc-symbol').addEventListener('input', (e) => {
        filterSymbol = (e.target.value || '').trim().toUpperCase();
        render();
    });
    mount.querySelectorAll('[data-sentiment]').forEach(btn => {
        btn.addEventListener('click', () => {
            filterSentiment = btn.dataset.sentiment;
            applyToggleState(mount);
            render();
        });
    });
    mount.querySelector('#cc-refresh').addEventListener('click', () => {
        rows.clear();
        connectWs(mount, viewTok);
    });
    applyToggleState(mount);
    connectWs(mount, viewTok);
}

function applyToggleState(mount) {
    mount.querySelectorAll('[data-sentiment]').forEach(b => {
        b.classList.toggle('active', b.dataset.sentiment === filterSentiment);
    });
}

function connectWs(mount, tok) {
    try { if (ws) { try { ws.close(); } catch (_) {} ws = null; } } catch (_) {}
    if (!viewIsCurrent(tok)) return;
    const dot = mount.querySelector('#cc-status');
    if (!dot) return;
    ws = new WebSocket(wsUrl('/api/ws/catalyst-correlations'));
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
            } else if (m.type === 'correlation' && m.row) {
                addRow(m.row);
            }
            render();
        } catch (_) {}
    });
}

function rowKey(r) { return `${r.catalyst_id}|${r.symbol}`; }

function addRow(r) {
    const k = rowKey(r);
    const fresh = !rows.has(k);
    rows.set(k, r);
    if (fresh && voiceOn) speak(r);
}

function speak(r) {
    try {
        const dir = r.signed_move_pct >= 0 ? 'up' : 'down';
        const pct = Math.abs(r.signed_move_pct).toFixed(1);
        const u = new SpeechSynthesisUtterance(
            `${spell(r.symbol)} ${dir} ${pct} percent on ${r.sentiment} ${kindShort(r.kind)}.`
        );
        u.rate = 1.1; u.pitch = 1.0; u.volume = 1.0;
        window.speechSynthesis.speak(u);
    } catch (_) {}
}

function spell(s) { return s.split('').join(' '); }
function kindShort(k) { return k === 'sec_filing' ? 'filing' : 'news'; }

function render() {
    const tbody = document.querySelector('#cc-table tbody');
    if (!tbody) return;
    let filtered = Array.from(rows.values());
    if (filterSentiment !== 'all') {
        filtered = filtered.filter(r => r.sentiment === filterSentiment);
    }
    if (filterSymbol) {
        filtered = filtered.filter(r => r.symbol === filterSymbol);
    }
    filtered.sort((a, b) => new Date(b.observed_at) - new Date(a.observed_at));
    if (!filtered.length) {
        tbody.innerHTML = `<tr><td colspan="10" class="muted">${esc(t('view.catalyst_correlations.empty.no_rows'))}</td></tr>`;
        return;
    }
    tbody.innerHTML = filtered.map(r => {
        const cls = r.signed_move_pct >= 0 ? 'pos' : 'neg';
        const peak = (r.peak - r.baseline) / r.baseline * 100;
        const trough = (r.trough - r.baseline) / r.baseline * 100;
        return `<tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
            <td>${esc(fmtDateTime(r.observed_at))}</td>
            <td><strong style="color:var(--accent)">${esc(r.symbol)}</strong></td>
            <td><span class="cc-sentiment cc-${esc(r.sentiment)}">${esc(r.sentiment)}</span></td>
            <td class="${cls}">${fmtPct(r.signed_move_pct)}</td>
            <td class="pos">${fmtPct(peak)}</td>
            <td class="neg">${fmtPct(trough)}</td>
            <td>${fmtLatency(r.cross_latency_ms)}</td>
            <td>${esc(kindShort(r.kind))}</td>
            <td>${esc(r.source)}</td>
            <td title="${esc(r.title)}">${esc(truncate(r.title, 80))}</td>
        </tr>`;
    }).join('');
}

function fmtPct(n) { return n == null ? '—' : (n >= 0 ? '+' : '') + n.toFixed(2) + '%'; }
function fmtLatency(ms) {
    if (ms == null) return '—';
    if (ms < 1000) return ms + 'ms';
    return (ms / 1000).toFixed(1) + 's';
}
function truncate(s, n) { return s.length > n ? s.slice(0, n - 1) + '…' : s; }
