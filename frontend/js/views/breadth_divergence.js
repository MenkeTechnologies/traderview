// Breadth-divergence detector — every 5 min the backend samples
// (SPY price, breadth composite_score) into a rolling 60-sample
// window and flags BEARISH (price up + breadth weak) or BULLISH
// (price down + breadth strong) regimes. TTS on every regime cross.

import { api, wsUrl } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

let ws = null;
let viewTok = 0;
let voiceOn = true;
const rows = new Map(); // (date|kind) → event

export async function renderBreadthDivergence(mount, _state) {
    viewTok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.breadth_divergence.title">// BREADTH DIVERGENCE · LIVE</span>
            <span class="status-dot" id="bd-status" data-i18n-title="common.status.connecting" title="connecting">●</span>
            <label class="bd-voice-toggle">
                <input type="checkbox" id="bd-voice" ${voiceOn ? 'checked' : ''}>
                <span data-i18n="common.label.voice_alerts">voice alerts</span>
            </label>
        </h1>
        <p class="muted small" data-i18n-html="view.breadth_divergence.intro">
            Samples (SPY price, breadth composite score) every 5 min into a 60-sample rolling
            window (~5 hrs of intraday). Flags <strong>BEARISH</strong> when price drifts up ≥1%
            while breadth averages ≤ -15 (rally on weak internals), or <strong>BULLISH</strong> when
            price drifts down ≥1% while breadth averages ≥ +15 (selling on strong internals).
            Regime changes only — no spam on persistent state.
        </p>
        <div class="chart-panel">
            <div class="bd-current">
                <span data-i18n="view.breadth_divergence.label.current_regime">current regime</span>:
                <strong id="bd-current-regime">—</strong>
                <button class="btn btn-sm" id="bd-refresh" data-shortcut="r" data-i18n="common.btn.refresh">⟳ Refresh</button>
            </div>
            <table class="trades" id="bd-table">
                <thead><tr>
                    <th data-i18n="view.breadth_divergence.th.started">Started</th>
                    <th data-i18n="view.breadth_divergence.th.kind">Regime</th>
                    <th data-i18n="view.breadth_divergence.th.spy_change">SPY Δ %</th>
                    <th data-i18n="view.breadth_divergence.th.breadth_avg">Breadth Avg</th>
                    <th data-i18n="view.breadth_divergence.th.samples">Samples</th>
                    <th data-i18n="view.breadth_divergence.th.window">Window (min)</th>
                </tr></thead>
                <tbody><tr><td colspan="6" class="muted" data-i18n="common.connecting">connecting…</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelector('#bd-voice').addEventListener('change', (e) => { voiceOn = e.target.checked; });
    mount.querySelector('#bd-refresh').addEventListener('click', async () => {
        await refreshCurrent(mount);
        connectWs(mount, viewTok);
    });
    await refreshCurrent(mount);
    connectWs(mount, viewTok);
}

async function refreshCurrent(mount) {
    try {
        const cur = await api.request('/breadth-divergence/current');
        const el = mount.querySelector('#bd-current-regime');
        if (el) {
            if (cur && cur.regime) {
                el.textContent = cur.regime;
                el.className = cur.regime === 'bearish' ? 'neg' : 'pos';
            } else {
                el.textContent = t('view.breadth_divergence.label.none');
                el.className = 'muted';
            }
        }
    } catch (_) {}
}

function connectWs(mount, tok) {
    try { if (ws) { try { ws.close(); } catch (_) {} ws = null; } } catch (_) {}
    if (!viewIsCurrent(tok)) return;
    const dot = mount.querySelector('#bd-status');
    if (!dot) return;
    ws = new WebSocket(wsUrl('/api/ws/breadth-divergence'));
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
            } else if (m.type === 'divergence' && m.row) {
                addRow(m.row);
            }
            render(mount);
        } catch (_) {}
    });
}

function rowKey(r) {
    return `${r.started_at}|${r.kind}`;
}

function addRow(r) {
    const k = rowKey(r);
    const fresh = !rows.has(k);
    rows.set(k, r);
    if (fresh && voiceOn) speak(r);
}

function speak(r) {
    try {
        const dir = r.kind === 'bullish' ? 'bullish' : 'bearish';
        const u = new SpeechSynthesisUtterance(
            `${dir} breadth divergence. SPY ${r.spy_change_pct.toFixed(1)} percent over ${r.window_minutes} minutes.`
        );
        u.rate = 1.1; u.pitch = 1.0; u.volume = 1.0;
        window.speechSynthesis.speak(u);
    } catch (_) {}
}

function render(mount) {
    const tbody = mount.querySelector('#bd-table tbody');
    if (!tbody) return;
    const all = Array.from(rows.values())
        .sort((a, b) => new Date(b.started_at) - new Date(a.started_at));
    if (!all.length) {
        tbody.innerHTML = `<tr><td colspan="6" class="muted">${esc(t('view.breadth_divergence.empty.no_rows'))}</td></tr>`;
        return;
    }
    tbody.innerHTML = all.map(r => {
        const kindCls = r.kind === 'bullish' ? 'pos' : 'neg';
        const spyCls = r.spy_change_pct >= 0 ? 'pos' : 'neg';
        const breadthCls = r.breadth_avg >= 0 ? 'pos' : 'neg';
        return `<tr>
            <td>${esc(fmtDateTime(r.started_at))}</td>
            <td class="${kindCls}"><strong>${esc(r.kind)}</strong></td>
            <td class="${spyCls}">${fmtPct(r.spy_change_pct)}</td>
            <td class="${breadthCls}">${r.breadth_avg.toFixed(1)}</td>
            <td>${r.samples_used}</td>
            <td>${r.window_minutes}</td>
        </tr>`;
    }).join('');
}

function fmtPct(n) {
    if (n == null) return '—';
    return (n >= 0 ? '+' : '') + n.toFixed(2) + '%';
}
