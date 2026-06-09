// Today's Earnings Call Live — Finnhub /stock/earnings-call-live + per-symbol
// transcripts list. Pre-market BMO / after-close AMC catalyst calendar.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

let state = { lookbackDays: 7, transcriptSym: '' };

export async function renderEarningsCallLive(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.earnings_call.h1.title">// EARNINGS CALL LIVE</span></h1>
        <p class="muted small" data-i18n="view.earnings_call.hint.intro">
            Upcoming + live earnings calls. AMC reports + next morning's BMO reactions
            are prime squeeze territory. Also: per-symbol transcript history.
        </p>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.earnings_call.h2.upcoming">Upcoming earnings calls</h2>
                <div class="inline-form">
                    <label><span data-i18n="view.earnings_call.label.window">Window (days)</span>
                        <input type="number" id="ec-days" value="${state.lookbackDays}" min="1" max="30"></label>
                    <button class="primary" id="ec-load" type="button" data-i18n="view.earnings_call.btn.load">Load</button>
                </div>
                <div id="ec-list" style="margin-top:10px"></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.earnings_call.h2.transcripts">Transcript history</h2>
                <div class="inline-form">
                    <label><span data-i18n="view.earnings_call.label.symbol">Symbol</span>
                        <input type="text" id="ec-sym" value="${esc(state.transcriptSym)}" placeholder="AAPL"></label>
                    <button class="primary" id="ec-load-tr" type="button" data-i18n="view.earnings_call.btn.load_tr">Load</button>
                </div>
                <div id="ec-tr" style="margin-top:10px"></div>
            </div>
        </div>
    `;
    document.getElementById('ec-load').addEventListener('click', () => {
        state.lookbackDays = Number(document.getElementById('ec-days').value) || 7;
        void loadUpcoming(tok);
    });
    document.getElementById('ec-load-tr').addEventListener('click', () => {
        state.transcriptSym = document.getElementById('ec-sym').value.toUpperCase().trim();
        void loadTranscripts(tok);
    });
    await loadUpcoming(tok);
}

async function loadUpcoming(tok) {
    const el = document.getElementById('ec-list');
    if (el) el.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`;
    try {
        const to = new Date();
        to.setDate(to.getDate() + state.lookbackDays);
        const data = await api.finnhubEarningsCallLive(fmtDay(new Date()), fmtDay(to), undefined);
        if (!viewIsCurrent(tok)) return;
        const rows = data?.earningsCallLive || data?.data || (Array.isArray(data) ? data : []);
        if (!rows.length) {
            el.innerHTML = `<p class="muted" data-i18n="view.earnings_call.empty.upcoming">No upcoming earnings calls (may require premium).</p>`;
            return;
        }
        const sorted = [...rows].sort((a, b) =>
            String(a.startTime || a.atDate || '').localeCompare(String(b.startTime || b.atDate || '')));
        el.innerHTML = `<table class="trades">
            <thead><tr>
                <th data-i18n="view.earnings_call.th.date">Date / time</th>
                <th data-i18n="view.earnings_call.th.symbol">Symbol</th>
                <th data-i18n="view.earnings_call.th.timezone">Timezone</th>
                <th data-i18n="view.earnings_call.th.audio">Audio</th>
            </tr></thead>
            <tbody>${sorted.map(r => `
                <tr>
                    <td>${esc(r.startTime || r.atDate || '—')}</td>
                    <td><a class="link" href="#research/${esc(r.symbol || '')}">${esc(r.symbol || '—')}</a></td>
                    <td class="muted">${esc(r.timezone || '—')}</td>
                    <td>${r.audio
                        ? `<a class="link" href="${esc(r.audio)}" target="_blank" rel="noopener">listen</a>`
                        : '—'}</td>
                </tr>
            `).join('')}</tbody>
        </table>`;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.earnings_call.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.earnings_call.toast.failed'), { level: 'error' });
    }
}

async function loadTranscripts(tok) {
    if (!state.transcriptSym) return;
    const el = document.getElementById('ec-tr');
    if (el) el.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`;
    try {
        const data = await api.symbolTranscriptsList(state.transcriptSym);
        if (!viewIsCurrent(tok)) return;
        const rows = data?.transcripts || (Array.isArray(data) ? data : []);
        if (!rows.length) {
            el.innerHTML = `<p class="muted" data-i18n="view.earnings_call.empty.transcripts">No transcripts (premium endpoint).</p>`;
            return;
        }
        el.innerHTML = `<table class="trades">
            <thead><tr>
                <th data-i18n="view.earnings_call.th.year">Year</th>
                <th data-i18n="view.earnings_call.th.quarter">Q</th>
                <th data-i18n="view.earnings_call.th.title">Title</th>
                <th data-i18n="view.earnings_call.th.time">Time</th>
            </tr></thead>
            <tbody>${rows.slice(0, 50).map(r => `
                <tr>
                    <td>${r.year ?? '—'}</td>
                    <td>${r.quarter ?? '—'}</td>
                    <td>${esc(r.title || '—')}</td>
                    <td class="muted">${esc(r.time || '—')}</td>
                </tr>
            `).join('')}</tbody>
        </table>`;
    } catch (e) {
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.earnings_call.error.tr', { msg: e.message || e }))}</p>`;
    }
}

function fmtDay(d) {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
}
