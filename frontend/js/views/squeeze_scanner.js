// Live Squeeze Scanner — consumes the /ws/squeeze WebSocket fed by the
// backend catalyst-aggregator + rolling-window detector. Shows:
//   * Top-N candidate symbols currently streamed (with score + last-source)
//   * Fired squeeze events in newest-first order (auto-scroll)
//   * Editable detector thresholds (%, vol burst ratio, windows, cooldown)
//   * Audio bell + TTS on every fire (reuses _audio_alerts.js)

import { api, wsUrl } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import * as audio from '../_audio_alerts.js';

let ws = null;
let liveEvents = [];
let candidates = [];
let cfg = null;
let unmounted = false;

const MAX_EVENTS = 200;

export async function renderSqueezeScanner(mount, _state) {
    unmounted = false;
    const tok = currentViewToken();

    // Load initial state in parallel — config + candidate snapshot.
    const [cfgInit, candInit] = await Promise.all([
        api.squeezeConfig().catch(() => null),
        api.squeezeCandidates().catch(() => []),
    ]);
    if (!viewIsCurrent(tok)) return;
    cfg = cfgInit || defaultConfig();
    candidates = Array.isArray(candInit) ? candInit : [];

    mount.innerHTML = `
        <h1 class="view-title">
            <span data-i18n="view.squeeze_scanner.h1.title">// LIVE SQUEEZE SCANNER</span>
            <span class="muted small" id="sq-conn-status">${esc(t('view.squeeze_scanner.status.connecting'))}</span>
        </h1>
        <p class="muted small" data-i18n="view.squeeze_scanner.hint.intro">
            Catalyst-driven candidate aggregator + rolling-window detector.
            Triggers on %change + volume burst within the configured window.
            Audio bell + TTS fires on every event.
        </p>

        <div class="chart-panel">
            <h2 data-i18n="view.squeeze_scanner.h2.config">Detector thresholds</h2>
            <form id="sq-cfg-form" class="inline-form">
                <label><span data-i18n="view.squeeze_scanner.label.pct_threshold">% change threshold</span>
                    <input type="number" step="0.1" name="pct_threshold" value="${cfg.pct_threshold}"></label>
                <label><span data-i18n="view.squeeze_scanner.label.pct_window">over (sec)</span>
                    <input type="number" step="1" name="pct_window_secs" value="${cfg.pct_window_secs}"></label>
                <label><span data-i18n="view.squeeze_scanner.label.burst_ratio">volume burst ×</span>
                    <input type="number" step="0.1" name="volume_burst_ratio" value="${cfg.volume_burst_ratio}"></label>
                <label><span data-i18n="view.squeeze_scanner.label.burst_window">over (sec)</span>
                    <input type="number" step="1" name="burst_window_secs" value="${cfg.burst_window_secs}"></label>
                <label><span data-i18n="view.squeeze_scanner.label.min_price">min price $</span>
                    <input type="number" step="0.1" name="min_price" value="${cfg.min_price}"></label>
                <label><span data-i18n="view.squeeze_scanner.label.min_burst_volume">min burst vol</span>
                    <input type="number" step="1000" name="min_burst_volume" value="${cfg.min_burst_volume}"></label>
                <label><span data-i18n="view.squeeze_scanner.label.cooldown">cooldown (sec)</span>
                    <input type="number" step="1" name="cooldown_secs" value="${cfg.cooldown_secs}"></label>
                <button type="submit" class="primary" data-i18n="view.squeeze_scanner.btn.save_config">Save thresholds</button>
            </form>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.squeeze_scanner.h2.candidates">
                Candidates currently streamed
                <span class="muted small" id="sq-cand-count"></span>
            </h2>
            <div id="sq-candidates"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.squeeze_scanner.h2.events">
                Squeeze events (newest first)
                <span class="muted small" id="sq-ev-count"></span>
            </h2>
            <div id="sq-events"></div>
        </div>
    `;

    renderCandidates();
    renderEvents();

    mount.querySelector('#sq-cfg-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = {
            pct_threshold:      Number(fd.get('pct_threshold')),
            pct_window_secs:    Number(fd.get('pct_window_secs')),
            volume_burst_ratio: Number(fd.get('volume_burst_ratio')),
            burst_window_secs:  Number(fd.get('burst_window_secs')),
            min_price:          Number(fd.get('min_price')),
            min_burst_volume:   Number(fd.get('min_burst_volume')),
            cooldown_secs:      Number(fd.get('cooldown_secs')),
        };
        try {
            await api.updateSqueezeConfig(body);
            cfg = body;
            showToast(t('view.squeeze_scanner.toast.config_saved'), { level: 'success' });
        } catch (err) {
            showToast(t('view.squeeze_scanner.toast.config_failed', { msg: err.message || err }), { level: 'error' });
        }
    });

    connectWs(tok);
    schedulePolls(tok);

    // Tear down WS + polling when the view unmounts.
    return () => {
        unmounted = true;
        if (ws) { try { ws.close(); } catch { /* ignore */ } ws = null; }
    };
}

function defaultConfig() {
    return {
        pct_threshold: 1.5,
        pct_window_secs: 10,
        volume_burst_ratio: 3.0,
        burst_window_secs: 10,
        min_price: 1.0,
        min_burst_volume: 100000.0,
        cooldown_secs: 60,
    };
}

function connectWs(tok) {
    if (ws) { try { ws.close(); } catch { /* ignore */ } }
    const url = wsUrl('/api/ws/squeeze');
    ws = new WebSocket(url);
    setStatus('connecting');
    ws.onopen = () => setStatus('live');
    ws.onclose = () => {
        setStatus('reconnecting');
        if (unmounted || !viewIsCurrent(tok)) return;
        setTimeout(() => { if (!unmounted) connectWs(tok); }, 3000);
    };
    ws.onerror = () => { /* silent — close handler reconnects */ };
    ws.onmessage = (e) => {
        let msg;
        try { msg = JSON.parse(e.data); } catch { return; }
        if (msg.type === 'snapshot' && Array.isArray(msg.events)) {
            liveEvents = msg.events.slice(0, MAX_EVENTS);
            renderEvents();
        } else if (msg.type === 'event' && msg.event) {
            handleLiveFire(msg.event);
        }
    };
}

function handleLiveFire(ev) {
    liveEvents.unshift(ev);
    if (liveEvents.length > MAX_EVENTS) liveEvents.length = MAX_EVENTS;
    renderEvents();
    // Audio + TTS on every fire. Both are user-toggle-able via the existing
    // _audio_alerts module; this view just dispatches the trigger.
    try { audio.playBell({ volume: 0.5 }); } catch { /* audio unavailable */ }
    try {
        audio.speakAlert(
            t('view.squeeze_scanner.tts.fire', {
                symbol: ev.symbol,
                pct: ev.pct_change.toFixed(2),
            })
        );
    } catch { /* tts unavailable */ }
    showToast(
        t('view.squeeze_scanner.toast.fire', {
            symbol: ev.symbol,
            pct: ev.pct_change.toFixed(2),
            ratio: ev.burst_ratio.toFixed(1),
        }),
        { level: 'success' }
    );
}

function schedulePolls(tok) {
    // Candidate list refreshes every 15s. The aggregator reconciles every
    // 30s on the backend; polling at half that cadence keeps the UI fresh
    // without hammering.
    const tick = async () => {
        if (unmounted || !viewIsCurrent(tok)) return;
        try {
            const list = await api.squeezeCandidates();
            if (Array.isArray(list)) {
                candidates = list;
                renderCandidates();
            }
        } catch { /* ignore — next tick retries */ }
        setTimeout(tick, 15000);
    };
    setTimeout(tick, 15000);
}

function renderCandidates() {
    const root = document.getElementById('sq-candidates');
    const counter = document.getElementById('sq-cand-count');
    if (!root) return;
    if (counter) counter.textContent = candidates.length
        ? `(${candidates.length})`
        : '';
    if (!candidates.length) {
        root.innerHTML = `<p class="muted" data-i18n="view.squeeze_scanner.empty.no_candidates">
            No candidates yet. Waiting for catalysts (SEC filings, PR wires, halts) to populate the list.
        </p>`;
        return;
    }
    root.innerHTML = `<table class="trades">
        <thead><tr>
            <th data-i18n="view.squeeze_scanner.th.symbol">Symbol</th>
            <th data-i18n="view.squeeze_scanner.th.score">Score</th>
            <th data-i18n="view.squeeze_scanner.th.source">Last source</th>
            <th data-i18n="view.squeeze_scanner.th.hits"># hits</th>
            <th data-i18n="view.squeeze_scanner.th.title">Title</th>
            <th data-i18n="view.squeeze_scanner.th.seen">Seen</th>
        </tr></thead>
        <tbody>${candidates.map(c => `
            <tr>
                <td><a class="link" href="#research/${esc(c.symbol)}">${esc(c.symbol)}</a></td>
                <td>${c.score.toFixed(1)}</td>
                <td>${esc(c.last_source.replace(/_/g, ' '))}</td>
                <td>${c.hit_count}</td>
                <td class="muted">${esc((c.last_title || '').slice(0, 80))}</td>
                <td class="muted">${esc(timeAgo(c.last_seen))}</td>
            </tr>
        `).join('')}</tbody>
    </table>`;
}

function renderEvents() {
    const root = document.getElementById('sq-events');
    const counter = document.getElementById('sq-ev-count');
    if (!root) return;
    if (counter) counter.textContent = liveEvents.length
        ? `(${liveEvents.length})`
        : '';
    if (!liveEvents.length) {
        root.innerHTML = `<p class="muted" data-i18n="view.squeeze_scanner.empty.no_events">
            No squeeze events yet. Detector is armed and watching.
        </p>`;
        return;
    }
    root.innerHTML = `<table class="trades">
        <thead><tr>
            <th data-i18n="view.squeeze_scanner.th.time">Time</th>
            <th data-i18n="view.squeeze_scanner.th.symbol_2">Symbol</th>
            <th data-i18n="view.squeeze_scanner.th.price">Price</th>
            <th data-i18n="view.squeeze_scanner.th.pct">% chg</th>
            <th data-i18n="view.squeeze_scanner.th.window">Window</th>
            <th data-i18n="view.squeeze_scanner.th.burst_vol">Burst vol</th>
            <th data-i18n="view.squeeze_scanner.th.baseline">Baseline</th>
            <th data-i18n="view.squeeze_scanner.th.ratio">Ratio</th>
        </tr></thead>
        <tbody>${liveEvents.map(ev => `
            <tr>
                <td class="muted">${esc(new Date(ev.fired_at).toLocaleTimeString(undefined, { hour12: false }))}</td>
                <td><a class="link" href="#research/${esc(ev.symbol)}">${esc(ev.symbol)}</a></td>
                <td>$${ev.price.toFixed(2)}</td>
                <td class="${ev.pct_change >= 0 ? 'pos' : 'neg'}">${ev.pct_change >= 0 ? '+' : ''}${ev.pct_change.toFixed(2)}%</td>
                <td>${ev.pct_window_secs}s</td>
                <td>${fmtVol(ev.burst_volume)}</td>
                <td class="muted">${fmtVol(ev.baseline_volume)}</td>
                <td class="pos">${ev.burst_ratio.toFixed(1)}×</td>
            </tr>
        `).join('')}</tbody>
    </table>`;
}

function setStatus(state) {
    const el = document.getElementById('sq-conn-status');
    if (!el) return;
    const label = state === 'live'
        ? t('view.squeeze_scanner.status.live')
        : state === 'reconnecting'
            ? t('view.squeeze_scanner.status.reconnecting')
            : t('view.squeeze_scanner.status.connecting');
    el.textContent = '· ' + label;
    el.classList.toggle('pos', state === 'live');
    el.classList.toggle('neg', state === 'reconnecting');
}

function fmtVol(v) {
    if (!Number.isFinite(v)) return '—';
    if (v >= 1e9) return (v / 1e9).toFixed(2) + 'B';
    if (v >= 1e6) return (v / 1e6).toFixed(2) + 'M';
    if (v >= 1e3) return (v / 1e3).toFixed(1) + 'k';
    return v.toFixed(0);
}

function timeAgo(iso) {
    if (!iso) return '';
    const ms = Date.now() - new Date(iso).getTime();
    if (!Number.isFinite(ms) || ms < 0) return '';
    const s = Math.floor(ms / 1000);
    if (s < 60)   return `${s}s ago`;
    if (s < 3600) return `${Math.floor(s / 60)}m ago`;
    return `${Math.floor(s / 3600)}h ago`;
}
