// IPO Calendar — Finnhub `/calendar/ipo` consumer. Lists upcoming IPOs
// with filed price range, shares, exchange. Filterable by date horizon.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const HORIZONS = [
    { value: 30,  key: 'view.ipo_calendar.horizon.next_30_days' },
    { value: 60,  key: 'view.ipo_calendar.horizon.next_60_days' },
    { value: 90,  key: 'view.ipo_calendar.horizon.next_90_days' },
    { value: 180, key: 'view.ipo_calendar.horizon.next_180_days' },
];

let state = { horizon: 60, rows: [] };

export async function renderIpoCalendar(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ipo_calendar.h1.title">// IPO CALENDAR</span></h1>
        <p class="muted small" data-i18n="view.ipo_calendar.hint.intro">
            Upcoming IPOs from Finnhub. Low-float runners on IPO day are
            classic squeeze candidates — watch the first 30 min after open
            for opportunistic plays.
        </p>

        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.ipo_calendar.label.horizon">Horizon</span>
                    <select id="ipo-horizon">${HORIZONS.map(h =>
                        `<option value="${h.value}" ${h.value === state.horizon ? 'selected' : ''}>${esc(t(h.key))}</option>`
                    ).join('')}</select>
                </label>
                <button class="primary" id="ipo-refresh" type="button"
                    data-i18n="view.ipo_calendar.btn.refresh">Refresh</button>
            </div>
            <div id="ipo-table" style="margin-top:10px"></div>
        </div>
    `;

    document.getElementById('ipo-horizon').addEventListener('change', e => {
        state.horizon = Number(e.target.value);
        void load(tok);
    });
    document.getElementById('ipo-refresh').addEventListener('click', () => void load(tok));

    await load(tok);
}

async function load(tok) {
    const el = document.getElementById('ipo-table');
    if (el) el.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`;
    try {
        const today = new Date();
        const to = new Date(today);
        to.setDate(to.getDate() + state.horizon);
        const data = await api.finnhubIpoCalendar(fmtDay(today), fmtDay(to));
        if (!viewIsCurrent(tok)) return;
        const rows = Array.isArray(data?.ipoCalendar) ? data.ipoCalendar : [];
        state.rows = rows;
        renderTable(rows);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (el) el.innerHTML = `<p class="muted neg" data-i18n="view.ipo_calendar.error.load">
            ${esc(t('view.ipo_calendar.error.load_msg', { msg: e.message || e }))}
        </p>`;
        showToast(t('view.ipo_calendar.toast.load_failed'), { level: 'error' });
    }
}

function renderTable(rows) {
    const el = document.getElementById('ipo-table');
    if (!el) return;
    if (!rows.length) {
        el.innerHTML = `<p class="muted" data-i18n="view.ipo_calendar.empty">
            No upcoming IPOs in this window.
        </p>`;
        return;
    }
    el.innerHTML = `<table class="trades">
        <thead><tr>
            <th data-i18n="view.ipo_calendar.th.date">Date</th>
            <th data-i18n="view.ipo_calendar.th.symbol">Symbol</th>
            <th data-i18n="view.ipo_calendar.th.name">Name</th>
            <th data-i18n="view.ipo_calendar.th.exchange">Exchange</th>
            <th data-i18n="view.ipo_calendar.th.price">Filed price</th>
            <th data-i18n="view.ipo_calendar.th.shares">Shares</th>
            <th data-i18n="view.ipo_calendar.th.status">Status</th>
        </tr></thead>
        <tbody>${rows.map(r => `
            <tr>
                <td>${esc(r.date || '—')}</td>
                <td><a class="link" href="#research/${esc(r.symbol || '')}">${esc(r.symbol || '—')}</a></td>
                <td>${esc(r.name || '—')}</td>
                <td class="muted">${esc(r.exchange || '—')}</td>
                <td>${esc(r.price || '—')}</td>
                <td>${r.numberOfShares ? r.numberOfShares.toLocaleString() : '—'}</td>
                <td class="muted">${esc(r.status || '—')}</td>
            </tr>
        `).join('')}</tbody>
    </table>`;
}

function fmtDay(d) {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
}
