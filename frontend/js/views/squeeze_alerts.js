// Squeeze Alerts view — audio + TTS firing when stocks squeeze.
//
// Workflow:
//   1. Paste tick stream (symbol ts_sec price volume) + ADV per symbol.
//   2. Configure thresholds (price %, volume ×, window, cooldown).
//   3. Toggle bell + TTS, set volume, set watchlist.
//   4. Click "Replay alerts" — engine scans the tick history and FIRES
//      audio + TTS for each squeeze event in real timing-order. Settings
//      persist in localStorage across reloads.
//
// Useful both for live monitoring (paste-in-fresh ticks every few sec) and
// for back-replay of historical sessions to tune the thresholds.

import { esc } from '../util.js';
import * as engine from '../_squeeze_alerts_inputs.js';
import * as audio from '../_audio_alerts.js';

import { t } from '../i18n.js';
let settings = engine.loadSettings();
let lastEvents = [];

export async function renderSqueezeAlerts(mount, _appState) {
    const caps = audio.audioCapabilities();
    mount.innerHTML = `
        <h1 data-i18n="view.squeeze_alerts.h1.squeeze_alerts_bell_tts" class="view-title">// SQUEEZE ALERTS · BELL + TTS</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.squeeze_alerts.h2.audio_capability">Audio capability</h2>
            <div class="cards">
                ${card(t('view.squeeze_alerts.card.web_audio'),   t(caps.audio ? 'common.status.available' : 'common.status.not_available'), caps.audio ? 'pos' : 'neg')}
                ${card(t('view.squeeze_alerts.card.speech_tts'), t(caps.tts   ? 'common.status.available' : 'common.status.not_available'), caps.tts   ? 'pos' : 'neg')}
            </div>
            <div class="inline-form" style="margin-top:8px">
                <button data-i18n="view.squeeze_alerts.btn.test_bell" id="sq-test-bell"  class="secondary" type="button" ${!caps.audio ? 'disabled' : ''}>🔔 Test bell</button>
                <button data-i18n="view.squeeze_alerts.btn.test_alarm" id="sq-test-alarm" class="secondary" type="button" ${!caps.audio ? 'disabled' : ''}>🚨 Test alarm</button>
                <button data-i18n="view.squeeze_alerts.btn.test_tts" id="sq-test-tts"   class="secondary" type="button" ${!caps.tts   ? 'disabled' : ''}>🗣️ Test TTS</button>
            </div>
            <p data-i18n="view.squeeze_alerts.hint.browser_autoplay_policy_audio_context_unlocks_afte" class="muted">Browser autoplay policy: audio context unlocks after your first
                click anywhere on the page. If "Test bell" fails silently, click any button on
                the page first then retry.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.squeeze_alerts.h2.settings">Settings</h2>
            <div class="inline-form">
                <label><span data-i18n="view.squeeze_alerts.label.price_threshold">Price threshold (% as decimal, 0.05 = 5%)</span>
                    <input id="sq-pt" type="number" step="any" min="0" value="${settings.price_threshold_pct}"></label>
                <label><span data-i18n="view.squeeze_alerts.label.volume_threshold">Volume threshold (× ADV in window)</span>
                    <input id="sq-vt" type="number" step="any" min="0" value="${settings.volume_threshold}"></label>
                <label><span data-i18n="view.squeeze_alerts.label.window_seconds">Window (seconds)</span>
                    <input id="sq-ws" type="number" step="1" min="1" value="${settings.window_seconds}"></label>
                <label><span data-i18n="view.squeeze_alerts.label.cooldown">Cooldown (seconds)</span>
                    <input id="sq-cs" type="number" step="1" min="0" value="${settings.cooldown_seconds}"></label>
            </div>
            <div class="inline-form">
                <label><input id="sq-bell" type="checkbox" ${settings.bell_enabled ? 'checked' : ''}>
                    <span data-i18n="view.squeeze_alerts.label.bell">Bell on alert</span></label>
                <label><input id="sq-tts"  type="checkbox" ${settings.tts_enabled  ? 'checked' : ''}>
                    <span data-i18n="view.squeeze_alerts.label.tts">TTS announcement</span></label>
                <label><input id="sq-alarm" type="checkbox" ${settings.use_alarm_for_critical ? 'checked' : ''}>
                    <span data-i18n="view.squeeze_alerts.label.alarm">Alarm chime for critical</span></label>
                <label><span data-i18n="view.squeeze_alerts.label.volume">Sound volume (0-1)</span>
                    <input id="sq-vol" type="number" step="0.05" min="0" max="1" value="${settings.sound_volume}"></label>
            </div>
            <div class="inline-form">
                <label><span data-i18n="view.squeeze_alerts.label.watchlist">Watchlist (comma-separated, blank = all symbols)</span>
                    <input id="sq-wl" type="text" value="${esc(settings.watchlist.join(','))}" style="min-width:300px"></label>
                <button data-i18n="view.squeeze_alerts.btn.save_settings" id="sq-save"    class="primary"   type="button">Save settings</button>
                <button data-i18n="view.squeeze_alerts.btn.reset_defaults" id="sq-default" class="secondary" type="button">Reset defaults</button>
            </div>
            <p data-i18n="view.squeeze_alerts.hint.all_settings_persist_in_localstorage_watchlist_gat" class="muted">All settings persist in localStorage. Watchlist gates which
                symbols can fire — leave blank to alert on every symbol that breaches.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.squeeze_alerts.h2.tick_feed">Tick feed</h2>
            <p class="muted" data-i18n="view.squeeze_alerts.hint.ticks">One tick per line: symbol unix_ts price volume. Demo loads 3 symbols over 30 minutes — AAPL ramps +8% in the last 5 minutes on 4× volume (normal alert), SMID gaps +12% on 6× volume (CRITICAL alarm), MSFT drifts (no alert).</p>
            <textarea id="sq-ticks" rows="6" placeholder="AAPL 1700000000 150.00 5000&#10;AAPL 1700000030 150.05 5500&#10;..."></textarea>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.squeeze_alerts.h2.adv_table">ADV table</h2>
            <p class="muted" data-i18n="view.squeeze_alerts.hint.adv">One per line: symbol adv_shares.</p>
            <textarea id="sq-adv" rows="3" placeholder="AAPL 50000000&#10;MSFT 25000000&#10;SMID 250000"></textarea>
            <div class="inline-form">
                <button data-i18n="view.squeeze_alerts.btn.load_demo_3_symbols_60_ticks_each_30_min_span" id="sq-demo" class="secondary" type="button">Load demo (3 symbols, 60 ticks each, 30 min span)</button>
                <button data-i18n="view.squeeze_alerts.btn.clear_feeds" id="sq-clear" class="secondary" type="button">Clear feeds</button>
                <button data-i18n="view.squeeze_alerts.btn.replay_alerts_audio_tts_will_fire" id="sq-replay" class="primary" type="button">▶ Replay alerts (audio + TTS will fire)</button>
                <button data-i18n="view.squeeze_alerts.btn.stop_tts" id="sq-stop-tts" class="secondary" type="button">⏹ Stop TTS</button>
            </div>
        </div>

        <div id="sq-errors" class="boot" style="display:none"></div>
        <div id="sq-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.squeeze_alerts.h2.detected_squeeze_events_chronological">Detected squeeze events (chronological)</h2>
            <div id="sq-events"></div>
        </div>
    `;

    // Wire test buttons.
    const opts = () => ({ volume: settings.sound_volume });
    document.getElementById('sq-test-bell').addEventListener('click',  () => audio.playBell(opts()));
    document.getElementById('sq-test-alarm').addEventListener('click', () => audio.playAlarm(opts()));
    document.getElementById('sq-test-tts').addEventListener('click',   () =>
        audio.speakAlert(t('view.squeeze_alerts.tts.test_phrase'),
            { volume: settings.sound_volume }));

    // Wire settings.
    document.getElementById('sq-save').addEventListener('click', () => {
        captureSettings();
        engine.saveSettings(settings);
        flash(t('view.squeeze_alerts.flash.saved'));
    });
    document.getElementById('sq-default').addEventListener('click', () => {
        settings = { ...engine.DEFAULT_SETTINGS };
        engine.saveSettings(settings);
        renderSqueezeAlerts(mount, _appState);
    });

    // Wire demo + replay.
    document.getElementById('sq-demo').addEventListener('click', () => {
        const { ticks, adv } = engine.makeDemoData();
        document.getElementById('sq-ticks').value =
            ticks.map(t => `${t.symbol} ${t.ts} ${t.price} ${t.volume}`).join('\n');
        document.getElementById('sq-adv').value =
            Object.entries(adv).map(([s, v]) => `${s} ${v}`).join('\n');
    });
    document.getElementById('sq-clear').addEventListener('click', () => {
        document.getElementById('sq-ticks').value = '';
        document.getElementById('sq-adv').value = '';
    });
    document.getElementById('sq-replay').addEventListener('click', () => {
        captureSettings();
        engine.saveSettings(settings);
        runReplay();
    });
    document.getElementById('sq-stop-tts').addEventListener('click', () => {
        if (typeof window !== 'undefined' && window.speechSynthesis) {
            window.speechSynthesis.cancel();
        }
    });
}

function captureSettings() {
    settings.price_threshold_pct = Number(document.getElementById('sq-pt').value);
    settings.volume_threshold   = Number(document.getElementById('sq-vt').value);
    settings.window_seconds     = parseInt(document.getElementById('sq-ws').value, 10);
    settings.cooldown_seconds   = parseInt(document.getElementById('sq-cs').value, 10);
    settings.bell_enabled       = document.getElementById('sq-bell').checked;
    settings.tts_enabled        = document.getElementById('sq-tts').checked;
    settings.use_alarm_for_critical = document.getElementById('sq-alarm').checked;
    settings.sound_volume       = Math.max(0, Math.min(1, Number(document.getElementById('sq-vol').value)));
    settings.watchlist = String(document.getElementById('sq-wl').value || '')
        .split(/[\s,]+/).filter(Boolean).map(s => s.toUpperCase());
}

async function runReplay() {
    hideErrors();
    const tickText = document.getElementById('sq-ticks').value;
    const advText  = document.getElementById('sq-adv').value;
    const { ticks, errors: te } = engine.parseTickBlob(tickText);
    const { adv, errors: ae } = engine.parseAdvBlob(advText);
    const errs = [...te.map(e => ({ ...e, src: 'ticks' })), ...ae.map(e => ({ ...e, src: 'adv' }))];
    if (errs.length) {
        const head = errs.slice(0, 8).map(e =>
            t('common.parse_error_inline_src', { src: e.src, line: e.line_no, msg: e.message, raw: e.raw.slice(0, 80) })).join('<br>');
        const el = document.getElementById('sq-errors');
        el.innerHTML = `<strong>${errs.length} parse error(s):</strong><br>${head}`;
        el.style.display = 'block';
        if (!ticks.length || !Object.keys(adv).length) return;
    }
    const events = engine.detectSqueezes(ticks, adv, settings);
    lastEvents = events;
    renderSummary(ticks, events);
    renderEvents(events);

    // Fire audio in chronological order, paced by real-time intervals
    // BETWEEN events (capped to keep replay snappy). Each event triggers:
    //   bell (or alarm if critical+enabled), then a TTS announcement.
    if (!events.length) return;
    const baseTs = events[0].ts;
    const now = Date.now();
    for (const ev of events) {
        // Schedule N ms from now where N = (ev.ts - baseTs) capped at 8s.
        const offsetMs = Math.min((ev.ts - baseTs) * 200, 8000);  // 0.2× speed cap
        setTimeout(() => fireAudioFor(ev), offsetMs);
        void now;
    }
}

function fireAudioFor(ev) {
    if (settings.bell_enabled) {
        if (ev.severity === 'critical' && settings.use_alarm_for_critical) {
            audio.playAlarm({ volume: settings.sound_volume });
        } else {
            audio.playBell({ volume: settings.sound_volume });
        }
    }
    if (settings.tts_enabled) {
        // Slight delay so the bell finishes before TTS speaks.
        setTimeout(() => audio.speakAlert(audio.formatSqueezeSpeech(ev), {
            volume: settings.sound_volume,
        }), 700);
    }
}

function renderSummary(ticks, events) {
    const symbols = new Set(events.map(e => e.symbol));
    const critical = events.filter(e => e.severity === 'critical').length;
    const lastEv = events[events.length - 1];
    document.getElementById('sq-summary').innerHTML = [
        card(t('view.squeeze_alerts.card.ticks'),            String(ticks.length)),
        card(t('view.squeeze_alerts.card.symbols_alerting'), String(symbols.size), symbols.size ? 'neg' : 'pos'),
        card(t('view.squeeze_alerts.card.events'),           String(events.length), events.length ? 'neg' : 'pos'),
        card(t('view.squeeze_alerts.card.critical'),         String(critical), critical ? 'neg' : ''),
        card(t('view.squeeze_alerts.card.last_event'),       lastEv
            ? `${lastEv.symbol} ${engine.fmtPct(lastEv.price_change_pct)} ${engine.fmtMult(lastEv.volume_multiplier)} @ ${engine.fmtTime(lastEv.ts)}`
            : '—',
            lastEv && lastEv.severity === 'critical' ? 'neg' : ''),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderEvents(events) {
    const wrap = document.getElementById('sq-events');
    if (!events.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.squeeze_alerts.empty.events">No squeezes detected at current thresholds.</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th><th data-i18n="view.squeeze_alerts.th.time">Time</th><th data-i18n="view.squeeze_alerts.th.symbol">Symbol</th>
                <th data-i18n="view.squeeze_alerts.th.price">Price Δ</th><th data-i18n="view.squeeze_alerts.th.vol">Vol ×</th><th data-i18n="view.squeeze_alerts.th.severity">Severity</th><th data-i18n="view.squeeze_alerts.th.tts_phrase">TTS phrase</th>
            </tr></thead>
            <tbody>
                ${events.map((e, i) => `
                    <tr data-context-scope="symbol-row" data-symbol="${esc(e.symbol)}">
                        <td>${i + 1}</td>
                        <td>${esc(engine.fmtTime(e.ts))}</td>
                        <td><strong>${esc(e.symbol)}</strong></td>
                        <td class="neg">${esc(engine.fmtPct(e.price_change_pct))}</td>
                        <td>${esc(engine.fmtMult(e.volume_multiplier))}</td>
                        <td class="${e.severity === 'critical' ? 'neg' : ''}">${esc(e.severity.toUpperCase())}</td>
                        <td class="muted">"${esc(audio.formatSqueezeSpeech(e))}"</td>
                    </tr>
                `).join('')}
            </tbody>
        </table>
    `;
}

function hideErrors() {
    const el = document.getElementById('sq-errors');
    if (el) el.style.display = 'none';
}

function flash(msg) {
    const el = document.getElementById('sq-errors');
    el.style.color = '#39ff14';
    el.style.display = 'block';
    el.textContent = msg;
    setTimeout(() => { el.style.display = 'none'; }, 2000);
}
