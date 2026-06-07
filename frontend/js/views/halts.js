// Halt scanner — live WebSocket stream of Nasdaq halts with reason codes,
// TTS voice alerts on every new halt.

import { api, wsUrl } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

let ws = null;
let voiceOn = true;
let viewTok = 0;
const halts = new Map(); // dedupe key → halt

export async function renderHalts(mount, _state) {
    viewTok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.halts.title">// HALT SCANNER · LIVE</span>
            <span class="status-dot" id="halt-status" data-i18n-title="common.status.connecting" title="connecting">●</span>
            <label class="halt-voice-toggle">
                <input type="checkbox" id="halt-voice" ${voiceOn ? 'checked' : ''}>
                <span data-i18n="common.label.voice_alerts">voice alerts</span>
            </label>
        </h1>
        <p class="muted small" data-i18n-html="view.halts.intro">
            Polling <code>nasdaqtrader.com/rss.aspx?feed=tradehalts</code> every 3 seconds.
            New halts speak the symbol + reason via Web Speech API.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.halts.h2.active_halts_live">Active halts (live)</h2>
            <table class="trades" id="halts-table">
                <thead><tr>
                    <th data-i18n="view.halts.th.time">Time</th><th data-i18n="view.halts.th.symbol">Symbol</th><th data-i18n="view.halts.th.issue">Issue</th>
                    <th data-i18n="view.halts.th.reason">Reason</th><th data-i18n="view.halts.th.resume_quote">Resume Quote</th><th data-i18n="view.halts.th.resume_trade">Resume Trade</th>
                </tr></thead>
                <tbody><tr><td colspan="6" class="muted" data-i18n="common.connecting">connecting…</td></tr></tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.halts.h2.reason_chart">Halt count by reason code</h2>
            <div id="halts-chart" style="width:100%;height:240px"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.halts.h2.category_chart">Halt count by category (news / volatility / circuit / regulatory / ipo / resume / other)</h2>
            <div id="halts-category-chart" style="width:100%;height:220px"></div>
        </div>
    `;
    mount.querySelector('#halt-voice').addEventListener('change', (e) => {
        voiceOn = e.target.checked;
    });
    connectWs(mount, viewTok);
}

function connectWs(mount, tok) {
    try {
        if (ws) { try { ws.close(); } catch (_) {} ws = null; }
    } catch (_) {}
    if (!viewIsCurrent(tok)) return;     // view changed before we got here
    const dot = mount.querySelector('#halt-status');
    if (!dot) return;                    // DOM gone — don't open an orphan WS
    halts.clear();
    ws = new WebSocket(wsUrl('/api/ws/halts'));
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
        tbody.innerHTML = `<tr><td colspan="6" class="muted">${esc(t('view.halts.empty.no_feed'))}</td></tr>`;
        return;
    }
    tbody.innerHTML = all.map(h => `
        <tr data-context-scope="symbol-row" data-symbol="${esc(h.symbol)}">
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
    renderReasonChart(all);
    renderCategoryChart(all);
}

function renderCategoryChart(all) {
    const el = document.getElementById('halts-category-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!all || all.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.halts.empty_category_chart">${esc(t('view.halts.empty_category_chart'))}</div>`;
        return;
    }
    const counts = new Map();
    for (const h of all) {
        const cat = reasonClass(h.reason_code);
        counts.set(cat, (counts.get(cat) || 0) + 1);
    }
    const pairs = Array.from(counts.entries()).sort((a, b) => b[1] - a[1]);
    const labels = pairs.map(([k]) => k);
    const ys = pairs.map(([, n]) => n);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.halts.chart.category_idx') },
            { label: t('view.halts.chart.category_count'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 14, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderReasonChart(all) {
    const el = document.getElementById('halts-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!all || all.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.halts.empty_chart">${esc(t('view.halts.empty_chart'))}</div>`;
        return;
    }
    const counts = new Map();
    for (const h of all) {
        const code = h.reason_code || '?';
        counts.set(code, (counts.get(code) || 0) + 1);
    }
    const pairs = Array.from(counts.entries()).sort((a, b) => b[1] - a[1]);
    const labels = pairs.map(([c]) => c);
    const ys = pairs.map(([, n]) => n);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.halts.chart.reason_idx') },
            { label: t('view.halts.chart.count'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
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
