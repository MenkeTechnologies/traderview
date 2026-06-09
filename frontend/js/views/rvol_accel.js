// Rel-vol acceleration — every tape print folds into a per-symbol
// 1-minute bucket. When ACCEL_LEN (3) consecutive bars strictly
// increase AND latest bar ≥ 5× baseline (mean of older bars), a fresh
// AccelEvent fires. TTS on every event.

import { wsUrl } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

let ws = null;
let viewTok = 0;
let voiceOn = true;
let minMultiple = 5;
const rows = new Map();

export async function renderRvolAccel(mount, _state) {
    viewTok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.rvol_accel.title">// REL-VOL ACCELERATION · LIVE</span>
            <span class="status-dot" id="ra-status" data-i18n-title="common.status.connecting" title="connecting">●</span>
            <label class="ra-voice-toggle">
                <input type="checkbox" id="ra-voice" ${voiceOn ? 'checked' : ''}>
                <span data-i18n="common.label.voice_alerts">voice alerts</span>
            </label>
        </h1>
        <p class="muted small" data-i18n-html="view.rvol_accel.intro">
            Every tape print folds into a per-symbol 1-minute bucket. When the last 3
            consecutive bars strictly increase in volume AND the latest bar is ≥ 5×
            the prior baseline (mean of older bars), a fresh acceleration event fires.
            Same-run duplicates are suppressed until the sequence breaks and re-forms.
        </p>
        <div class="chart-panel">
            <div class="ra-controls" style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <label class="ra-min-multiple">
                    <span data-i18n="view.rvol_accel.label.min_multiple">min multiple ×</span>
                    <input type="number" id="ra-min-multiple" min="1" step="0.5" value="${minMultiple}" style="width:80px">
                </label>
                <button class="btn btn-sm" id="ra-refresh" data-shortcut="r" data-i18n="common.btn.refresh">⟳ Refresh</button>
            </div>
            <table class="trades" id="ra-table">
                <thead><tr>
                    <th data-i18n="view.rvol_accel.th.observed">Observed</th>
                    <th data-i18n="view.rvol_accel.th.symbol">Symbol</th>
                    <th data-i18n="view.rvol_accel.th.multiple">Multiple ×</th>
                    <th data-i18n="view.rvol_accel.th.latest">Latest Vol</th>
                    <th data-i18n="view.rvol_accel.th.baseline">Baseline</th>
                    <th data-i18n="view.rvol_accel.th.run">Run (volumes)</th>
                </tr></thead>
                <tbody><tr><td colspan="6" class="muted" data-i18n="common.connecting">connecting…</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelector('#ra-voice').addEventListener('change', (e) => { voiceOn = e.target.checked; });
    mount.querySelector('#ra-min-multiple').addEventListener('change', (e) => {
        const v = parseFloat(e.target.value);
        if (Number.isFinite(v) && v >= 1) { minMultiple = v; render(); }
    });
    mount.querySelector('#ra-refresh').addEventListener('click', () => {
        rows.clear();
        connectWs(mount, viewTok);
    });
    connectWs(mount, viewTok);
}

function connectWs(mount, tok) {
    try { if (ws) { try { ws.close(); } catch (_) {} ws = null; } } catch (_) {}
    if (!viewIsCurrent(tok)) return;
    const dot = mount.querySelector('#ra-status');
    if (!dot) return;
    ws = new WebSocket(wsUrl('/api/ws/rvol-accel'));
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
            } else if (m.type === 'accel' && m.row) {
                addRow(m.row);
            }
            render();
        } catch (_) {}
    });
}

function rowKey(r) {
    return `${r.symbol}|${r.minute_started_at}`;
}

function addRow(r) {
    const k = rowKey(r);
    const fresh = !rows.has(k);
    rows.set(k, r);
    if (fresh && voiceOn) speak(r);
}

function speak(r) {
    try {
        const u = new SpeechSynthesisUtterance(
            `${spell(r.symbol)} volume acceleration. ${r.multiple.toFixed(1)} times baseline.`
        );
        u.rate = 1.1; u.pitch = 1.0; u.volume = 1.0;
        window.speechSynthesis.speak(u);
    } catch (_) {}
}

function spell(s) { return s.split('').join(' '); }

function render() {
    const tbody = document.querySelector('#ra-table tbody');
    if (!tbody) return;
    const filtered = Array.from(rows.values())
        .filter(r => r.multiple >= minMultiple)
        .sort((a, b) => new Date(b.observed_at) - new Date(a.observed_at));
    if (!filtered.length) {
        tbody.innerHTML = `<tr><td colspan="6" class="muted">${esc(t('view.rvol_accel.empty.no_rows'))}</td></tr>`;
        return;
    }
    tbody.innerHTML = filtered.map(r => `
        <tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
            <td>${esc(fmtDateTime(r.observed_at))}</td>
            <td><strong style="color:var(--accent)">${esc(r.symbol)}</strong></td>
            <td class="pos"><strong>${r.multiple.toFixed(2)}×</strong></td>
            <td>${fmtN(r.latest_volume)}</td>
            <td>${fmtN(r.baseline_volume)}</td>
            <td class="muted">${r.run.map(v => fmtN(v)).join(' → ')}</td>
        </tr>
    `).join('');
}

function fmtN(n) {
    if (n == null) return '—';
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(2) + 'M';
    if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
    return n.toFixed(0);
}
