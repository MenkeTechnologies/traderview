// Catalyst Radar — live SEC EDGAR + PR Newswire / Business Wire /
// GlobeNewswire / AccessWire firehose with auto-extracted tickers and
// optional TTS announcements when a watchlist symbol is mentioned.

import { api, wsUrl } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

const catalysts = new Map();   // key → catalyst
let ws = null;
let voiceOn = true;
let viewTok = 0;
let watchSet = new Set();      // symbols user wants to be alerted on
const announced = new Set();

export async function renderCatalysts(mount, _state) {
    viewTok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.catalysts.title">// CATALYST RADAR · LIVE</span>
            <span class="status-dot" id="cat-status" data-i18n-title="common.status.connecting" title="connecting">●</span>
            <label class="halt-voice-toggle">
                <input type="checkbox" id="cat-voice" ${voiceOn ? 'checked' : ''}>
                <span data-i18n="common.label.voice_alerts">voice alerts</span>
            </label>
        </h1>
        <p data-i18n="view.catalysts.hint.sec_edgar_every_6s_business_wire_pr_newswire_globe" class="muted small">
            SEC EDGAR (every 6s) + Business Wire / PR Newswire / GlobeNewswire /
            AccessWire (every 30s). Tickers auto-extracted via NER.
            Filter for your watchlist on the left; voice alerts fire when
            any of those symbols appear in a fresh filing or press release.
        </p>

        <div class="chart-panel">
            <h2 data-i18n="view.catalysts.h2.voice_alert_filter_comma_sep_symbols">Voice alert filter (comma-sep symbols)</h2>
            <form id="cat-watch" class="inline-form">
                <input name="symbols" type="text" placeholder="GME,AMC,SPCE,..." style="min-width:360px">
                <button data-i18n="view.catalysts.btn.set" class="primary" type="submit">Set</button>
                <span class="muted small" id="cat-watch-state"></span>
            </form>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.catalysts.h2.live_feed">Live feed</h2>
            <table class="trades" id="cat-table">
                <thead><tr>
                    <th data-i18n="view.catalysts.th.time">Time</th><th data-i18n="view.catalysts.th.source">Source</th><th data-i18n="view.catalysts.th.form">Form</th>
                    <th data-i18n="view.catalysts.th.tickers">Tickers</th><th data-i18n="view.catalysts.th.headline">Headline</th>
                </tr></thead>
                <tbody><tr><td colspan="5" class="muted" data-i18n="common.connecting">connecting…</td></tr></tbody>
            </table>
        </div>
    `;

    mount.querySelector('#cat-voice').addEventListener('change', (e) => {
        voiceOn = e.target.checked;
    });
    mount.querySelector('#cat-watch').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const list = fd.get('symbols').split(',').map(s => s.trim().toUpperCase()).filter(Boolean);
        watchSet = new Set(list);
        const stateEl = mount.querySelector('#cat-watch-state');
        if (stateEl) stateEl.textContent = watchSet.size
            ? t('view.catalysts.watch.watching', { n: watchSet.size })
            : t('view.catalysts.watch.all_clear');
    });

    connectWs(mount, viewTok);
}

function connectWs(mount, tok) {
    try { if (ws) ws.close(); } catch (_) {}
    if (!viewIsCurrent(tok)) return;
    const dot = mount.querySelector('#cat-status');
    if (!dot) return;
    catalysts.clear();
    announced.clear();
    ws = new WebSocket(wsUrl('/api/ws/catalysts'));
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
                for (const c of (m.catalysts || [])) record(c, /*announce=*/false);
            } else if (m.type === 'catalyst') {
                record(m.catalyst, /*announce=*/true);
            }
            render();
        } catch (_) {}
    });
}

function record(c, announce) {
    const key = `${c.source}|${c.link || c.title}`;
    if (catalysts.has(key)) return;
    catalysts.set(key, c);
    if (announce && voiceOn && watchSet.size) {
        for (const t of c.tickers || []) {
            if (watchSet.has(t) && !announced.has(`${key}-${t}`)) {
                announced.add(`${key}-${t}`);
                speak(`${spell(t)}. ${c.form_type ? c.form_type + '.' : ''} ${c.source}.`);
                break;
            }
        }
    }
}

function speak(text) {
    try {
        const u = new SpeechSynthesisUtterance(text);
        u.rate = 1.15; u.volume = 1.0;
        window.speechSynthesis.speak(u);
    } catch (_) {}
}
function spell(s) { return s.split('').join(' '); }

function render() {
    const tbody = document.querySelector('#cat-table tbody');
    if (!tbody) return;
    const all = Array.from(catalysts.values())
        .sort((a, b) => new Date(b.fetched_at) - new Date(a.fetched_at))
        .slice(0, 200);
    if (!all.length) {
        tbody.innerHTML = `<tr><td colspan="5" class="muted">${esc(t('view.catalysts.empty.no_feed'))}</td></tr>`;
        return;
    }
    tbody.innerHTML = all.map(c => {
        const tickers = (c.tickers || []).map(t => {
            const cls = watchSet.has(t) ? 'tick-chip tick-watched' : 'tick-chip';
            return `<span class="${cls}">${esc(t)}</span>`;
        }).join('') || '<span class="muted small">—</span>';
        const linkHtml = c.link
            ? `<a href="${esc(c.link)}" target="_blank" rel="noopener noreferrer">${esc(c.title)}</a>`
            : esc(c.title);
        const formChip = c.form_type
            ? `<span class="halt-code halt-${c.kind === 'sec_filing' ? 'regulatory' : 'news'}">${esc(c.form_type)}</span>`
            : '';
        return `<tr>
            <td>${fmtDateTime(c.published_at || c.fetched_at)}</td>
            <td><span class="muted small">${esc(c.source)}</span></td>
            <td>${formChip}</td>
            <td>${tickers}</td>
            <td>${linkHtml}</td>
        </tr>`;
    }).join('');
}
