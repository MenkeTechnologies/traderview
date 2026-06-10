// Multi-signal confluence — meta-scanner ranking symbols by # of
// independent scanners agreeing × source weight × recency. The whole
// point: 10 scanners running in parallel produce noise, the
// intersection is where the actual edge lives.

import { api, wsUrl } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

let ws = null;
let viewTok = 0;
let voiceOn = true;
let minSources = 2;
const rows = new Map(); // symbol → row
let pollTimer = null;

export async function renderConfluence(mount, _state) {
    viewTok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.confluence.title">// CONFLUENCE · MULTI-SCANNER INTERSECTION · LIVE</span>
            <span class="status-dot" id="cf-status" data-i18n-title="common.status.connecting" title="connecting">●</span>
            <label class="cf-voice-toggle">
                <input type="checkbox" id="cf-voice" ${voiceOn ? 'checked' : ''}>
                <span data-i18n="common.label.voice_alerts">voice alerts</span>
            </label>
        </h1>
        <p class="muted small" data-i18n-html="view.confluence.intro">
            Subscribes to every scanner broadcast (after-hours, catalysts, catalyst
            correlations, gamma squeeze, halts, insider buys/sells, RVOL accel, sentiment
            velocity, squeeze detector, UOA) and ranks symbols by a confluence score:
            <code>Σ source_weight × recency_weight(half-life 4h) + diversity_bonus</code>.
            Diversity bonus kicks in from the 3rd distinct source. The top of this list
            is where the most edges agree right now — highest-conviction long candidates.
            Insider sells subtract from score. TTS fires when a row breaks score 10.
        </p>
        <div class="chart-panel">
            <div class="cf-controls" style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <label>
                    <span data-i18n="view.confluence.label.min_sources">min distinct sources</span>
                    <input type="number" id="cf-min-sources" min="1" max="11" step="1" value="${minSources}" style="width:60px">
                </label>
                <button class="btn btn-sm" id="cf-refresh" data-shortcut="r" data-i18n="common.btn.refresh">⟳ Refresh</button>
                <span class="muted small" id="cf-meta"></span>
            </div>
            <table class="trades" id="cf-table">
                <thead><tr>
                    <th data-i18n="view.confluence.th.rank">#</th>
                    <th data-i18n="view.confluence.th.symbol">Symbol</th>
                    <th data-i18n="view.confluence.th.score">Score</th>
                    <th data-i18n="view.confluence.th.sources">Sources</th>
                    <th data-i18n="view.confluence.th.events">Events</th>
                    <th data-i18n="view.confluence.th.last">Most Recent</th>
                </tr></thead>
                <tbody><tr><td colspan="6" class="muted" data-i18n="common.connecting">connecting…</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelector('#cf-voice').addEventListener('change', (e) => { voiceOn = e.target.checked; });
    mount.querySelector('#cf-min-sources').addEventListener('change', (e) => {
        const v = parseInt(e.target.value, 10);
        if (Number.isFinite(v) && v >= 1) { minSources = v; refresh(mount); }
    });
    mount.querySelector('#cf-refresh').addEventListener('click', () => refresh(mount));
    await refresh(mount);
    connectWs(mount, viewTok);
    // Poll every 30s to capture decay (recency weighting moves with wall-clock).
    pollTimer = setInterval(() => { if (viewIsCurrent(viewTok)) refresh(mount); }, 30_000);
}

async function refresh(mount) {
    try {
        const fresh = await api.request(`/confluence/ranked?limit=50&min_sources=${minSources}`);
        rows.clear();
        for (const r of fresh) rows.set(r.symbol, r);
        render(mount);
    } catch (e) {
        const tbody = mount.querySelector('#cf-table tbody');
        if (tbody) tbody.innerHTML = `<tr><td colspan="6" class="muted">${esc(String(e))}</td></tr>`;
    }
}

function connectWs(mount, tok) {
    try { if (ws) { try { ws.close(); } catch (_) {} ws = null; } } catch (_) {}
    if (!viewIsCurrent(tok)) return;
    const dot = mount.querySelector('#cf-status');
    if (!dot) return;
    ws = new WebSocket(wsUrl('/api/ws/confluence'));
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
            // Every fresh event triggers a refetch of the ranked snapshot.
            // The score itself depends on time-decay so refetching is the
            // correct semantics — incrementally patching client-side
            // would drift from server truth.
            if (m.type === 'event') { refresh(mount); }
        } catch (_) {}
    });
}

function render(mount) {
    const tbody = mount.querySelector('#cf-table tbody');
    if (!tbody) return;
    const meta = mount.querySelector('#cf-meta');
    const sorted = Array.from(rows.values()).sort((a, b) => b.score - a.score);
    if (!sorted.length) {
        tbody.innerHTML = `<tr><td colspan="6" class="muted">${esc(t('view.confluence.empty.no_rows'))}</td></tr>`;
        if (meta) meta.textContent = '';
        return;
    }
    if (meta) {
        const totalEvents = sorted.reduce((s, r) => s + r.event_count, 0);
        meta.textContent = t('view.confluence.meta.summary')
            .replace('{symbols}', sorted.length)
            .replace('{events}', totalEvents);
    }
    tbody.innerHTML = sorted.map((r, i) => {
        const scoreCls = r.score >= 10 ? 'pos' : r.score >= 5 ? '' : 'muted';
        const newest = r.events && r.events.length
            ? esc(fmtDateTime(r.events[0].observed_at))
            : '—';
        const sourceChips = (r.sources_hit || []).map(s =>
            `<span class="cf-chip cf-chip-${esc(s)}" title="${esc(sourceWeightLabel(s))}">${esc(sourceShort(s))}</span>`
        ).join(' ');
        return `<tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
            <td class="muted">${i + 1}</td>
            <td><strong style="color:var(--accent)">${esc(r.symbol)}</strong></td>
            <td class="${scoreCls}"><strong>${r.score.toFixed(2)}</strong></td>
            <td class="cf-sources">${sourceChips} <span class="muted small">(${r.distinct_sources})</span></td>
            <td>${r.event_count}</td>
            <td>${newest}</td>
        </tr>`;
    }).join('');
    maybeAnnounce(sorted[0]);
}

let lastAnnouncedKey = null;
function maybeAnnounce(top) {
    if (!voiceOn || !top || top.score < 10) return;
    const key = top.symbol;
    if (key === lastAnnouncedKey) return;
    lastAnnouncedKey = key;
    try {
        const u = new SpeechSynthesisUtterance(
            `${spell(top.symbol)} confluence ${top.score.toFixed(1)}. ${top.distinct_sources} sources.`
        );
        u.rate = 1.1; u.pitch = 1.0; u.volume = 1.0;
        window.speechSynthesis.speak(u);
    } catch (_) {}
}

function spell(s) { return s.split('').join(' '); }

function sourceShort(s) {
    return {
        'after_hours': 'AH',
        'catalyst': 'CAT',
        'catalyst_correlation': 'C+',
        'gamma_squeeze': 'GEX',
        'halt': 'HLT',
        'insider_buy': 'IB',
        'insider_sell': 'IS',
        'rvol_accel': 'RV',
        'sentiment_velocity': 'SV',
        'squeeze_detector': 'SQ',
        'uoa': 'UOA',
    }[s] || s;
}
function sourceWeightLabel(s) {
    const w = {
        'insider_buy': 3.0, 'catalyst_correlation': 2.5,
        'gamma_squeeze': 2.0, 'sentiment_velocity': 2.0, 'uoa': 2.0,
        'rvol_accel': 1.5, 'squeeze_detector': 1.5,
        'after_hours': 1.5, 'halt': 1.5, 'catalyst': 1.0,
        'insider_sell': -1.5,
    }[s];
    return `${s} (weight ${w >= 0 ? '+' : ''}${w?.toFixed?.(1) ?? '?'})`;
}
