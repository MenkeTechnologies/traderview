// Economic Calendar — Finnhub /calendar/economic consumer. Macro events
// (CPI, NFP, FOMC, GDP, etc.) with country / impact / forecast / actual.
// PREMIUM endpoint — gracefully fail on free-tier keys.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const HORIZONS = [
    { value: 7,   key: 'view.economic_calendar.horizon.next_7_days' },
    { value: 14,  key: 'view.economic_calendar.horizon.next_14_days' },
    { value: 30,  key: 'view.economic_calendar.horizon.next_30_days' },
];

let state = { horizon: 7 };

export async function renderEconomicCalendar(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.economic_calendar.h1.title">// ECONOMIC CALENDAR</span></h1>
        <p class="muted small" data-i18n="view.economic_calendar.hint.intro">
            Macro events (CPI, NFP, FOMC, GDP, jobless claims, retail sales).
            <strong>Premium endpoint</strong> — Finnhub free tier will return empty.
        </p>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.economic_calendar.label.horizon">Horizon</span>
                    <select id="ec-horizon">${HORIZONS.map(h =>
                        `<option value="${h.value}" ${h.value === state.horizon ? 'selected' : ''}>${esc(t(h.key))}</option>`
                    ).join('')}</select>
                </label>
                <button class="primary" id="ec-refresh" type="button" data-i18n="view.economic_calendar.btn.refresh">Refresh</button>
            </div>
            <div id="ec-table" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('ec-horizon').addEventListener('change', e => {
        state.horizon = Number(e.target.value);
        void load(tok);
    });
    document.getElementById('ec-refresh').addEventListener('click', () => void load(tok));
    await load(tok);
}

function fmtDay(d) {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
}

async function load(tok) {
    const el = document.getElementById('ec-table');
    if (el) el.innerHTML = `<div class="boot">${esc(t('common.loading'))}</div>`;
    try {
        const today = new Date();
        const to = new Date(today);
        to.setDate(to.getDate() + state.horizon);
        const data = await api.finnhubEconomicCalendar(fmtDay(today), fmtDay(to));
        if (!viewIsCurrent(tok)) return;
        const rows = data?.economicCalendar || [];
        if (!rows.length) {
            el.innerHTML = `<p class="muted" data-i18n="view.economic_calendar.empty">
                No events. Likely requires Finnhub Calendar API premium tier.
            </p>`;
            return;
        }
        el.innerHTML = `<table class="trades">
            <thead><tr>
                <th data-i18n="view.economic_calendar.th.time">Time</th>
                <th data-i18n="view.economic_calendar.th.country">Country</th>
                <th data-i18n="view.economic_calendar.th.event">Event</th>
                <th data-i18n="view.economic_calendar.th.impact">Impact</th>
                <th data-i18n="view.economic_calendar.th.actual">Actual</th>
                <th data-i18n="view.economic_calendar.th.estimate">Estimate</th>
                <th data-i18n="view.economic_calendar.th.prev">Previous</th>
            </tr></thead>
            <tbody>${rows.map(r => {
                const impact = (r.impact || '').toLowerCase();
                const impactCls = impact === 'high' ? 'neg' : impact === 'medium' ? '' : 'muted';
                return `<tr>
                    <td class="muted">${esc(r.time || '—')}</td>
                    <td>${esc(r.country || '—')}</td>
                    <td>${esc(r.event || '—')}</td>
                    <td class="${impactCls}">${esc(r.impact || '—')}</td>
                    <td>${r.actual ?? '—'}</td>
                    <td class="muted">${r.estimate ?? '—'}</td>
                    <td class="muted">${r.prev ?? '—'}</td>
                </tr>`;
            }).join('')}</tbody>
        </table>`;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.economic_calendar.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.economic_calendar.toast.failed'), { level: 'error' });
    }
}
