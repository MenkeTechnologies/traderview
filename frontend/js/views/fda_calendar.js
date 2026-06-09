// FDA Advisory Committee Calendar — Finnhub /fda-advisory-committee-calendar.
// Biotech catalyst gold: PDUFA dates + ad-com meetings.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

export async function renderFdaCalendar(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.fda_calendar.h1.title">// FDA ADVISORY CALENDAR</span></h1>
        <p class="muted small" data-i18n="view.fda_calendar.hint.intro">
            Upcoming FDA advisory committee meetings + PDUFA dates. Biotech runners
            often originate here — track ad-com votes for sympathy plays.
        </p>
        <div class="chart-panel">
            <button class="primary" id="fda-refresh" type="button" data-i18n="view.fda_calendar.btn.refresh">Refresh</button>
            <div id="fda-table" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('fda-refresh').addEventListener('click', () => void load(tok));
    await load(tok);
}

async function load(tok) {
    const el = document.getElementById('fda-table');
    if (el) el.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`;
    try {
        const data = await api.finnhubFdaCalendar();
        if (!viewIsCurrent(tok)) return;
        const rows = Array.isArray(data) ? data : (data?.data || []);
        if (!rows.length) {
            el.innerHTML = `<p class="muted" data-i18n="view.fda_calendar.empty">No upcoming FDA meetings.</p>`;
            return;
        }
        const sorted = [...rows].sort((a, b) =>
            String(a.eventDate || '').localeCompare(String(b.eventDate || '')));
        el.innerHTML = `<table class="trades">
            <thead><tr>
                <th data-i18n="view.fda_calendar.th.event_date">Event date</th>
                <th data-i18n="view.fda_calendar.th.event">Event / drug</th>
                <th data-i18n="view.fda_calendar.th.committee">Committee</th>
                <th data-i18n="view.fda_calendar.th.start">Start</th>
                <th data-i18n="view.fda_calendar.th.end">End</th>
            </tr></thead>
            <tbody>${sorted.map(r => `
                <tr>
                    <td><strong>${esc(r.eventDate || '—')}</strong></td>
                    <td>${esc(r.eventDescription || r.eventName || '—')}</td>
                    <td class="muted">${esc(r.committee || '—')}</td>
                    <td class="muted">${esc(r.startDate || '—')}</td>
                    <td class="muted">${esc(r.endDate || '—')}</td>
                </tr>
            `).join('')}</tbody>
        </table>`;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.fda_calendar.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.fda_calendar.toast.failed'), { level: 'error' });
    }
}
