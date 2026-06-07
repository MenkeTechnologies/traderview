// Market Status + Holidays — live exchange state + upcoming closures.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const EXCHANGES = ['US', 'L', 'HK', 'T', 'F', 'PA', 'AS', 'BR', 'TO', 'SS', 'SI', 'AX'];
let state = { exchange: 'US' };

export async function renderMarketStatus(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.market_status.h1.title">// MARKET STATUS</span></h1>
        <p class="muted small" data-i18n="view.market_status.hint.intro">
            Live exchange status + upcoming holiday closures. Useful for confirming
            "is the market open right now?" + planning around half-days / FOMC.
        </p>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.market_status.label.exchange">Exchange</span>
                    <select id="ms-exchange">${EXCHANGES.map(e =>
                        `<option value="${e}" ${e === state.exchange ? 'selected' : ''}>${e}</option>`
                    ).join('')}</select>
                </label>
                <button class="primary" id="ms-refresh" type="button" data-i18n="view.market_status.btn.refresh">Refresh</button>
            </div>
        </div>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.market_status.h2.status">Current status</h2>
                <div id="ms-status"></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.market_status.h2.holidays">Upcoming holidays</h2>
                <div id="ms-holidays"></div>
            </div>
        </div>
    `;
    document.getElementById('ms-exchange').addEventListener('change', e => {
        state.exchange = e.target.value;
        void load(tok);
    });
    document.getElementById('ms-refresh').addEventListener('click', () => void load(tok));
    await load(tok);
}

async function load(tok) {
    const [statusEl, holEl] = ['ms-status', 'ms-holidays'].map(id => document.getElementById(id));
    statusEl && (statusEl.innerHTML = `<div class="boot">${esc(t('common.loading'))}</div>`);
    holEl && (holEl.innerHTML = `<div class="boot">${esc(t('common.loading'))}</div>`);
    try {
        const [st, hol] = await Promise.all([
            api.finnhubMarketStatus(state.exchange).catch(() => null),
            api.finnhubMarketHoliday(state.exchange).catch(() => null),
        ]);
        if (!viewIsCurrent(tok)) return;
        renderStatus(statusEl, st);
        renderHolidays(holEl, hol);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        showToast(t('view.market_status.toast.failed'), { level: 'error' });
    }
}

function renderStatus(el, s) {
    if (!el) return;
    if (!s || typeof s !== 'object') {
        el.innerHTML = `<p class="muted" data-i18n="view.market_status.empty.status">No status data.</p>`;
        return;
    }
    // Backend returns `{available: false, reason: ...}` when the
    // configured Finnhub plan can't reach this exchange. Render a
    // clean "not on this plan" message instead of treating it as
    // closed-with-blank-fields.
    if (s.available === false) {
        const reasonKey = s.reason === 'premium-required'
            ? 'view.market_status.unavailable.premium'
            : 'view.market_status.unavailable.upstream';
        el.innerHTML = `<p class="muted">${esc(t(reasonKey, { exchange: s.exchange || '' }))}</p>`;
        return;
    }
    const isOpen = s.isOpen === true;
    const cls = isOpen ? 'pos' : 'neg';
    const label = isOpen
        ? t('view.market_status.status.open')
        : t('view.market_status.status.closed');
    const sess = (s.session || '').replace(/_/g, ' ');
    el.innerHTML = `
        <div class="cards">
            <div class="card ${cls}">
                <div class="label" data-i18n="view.market_status.card.status">Status</div>
                <div class="value">${esc(label)}</div>
            </div>
            <div class="card">
                <div class="label" data-i18n="view.market_status.card.session">Session</div>
                <div class="value">${esc(sess || '—')}</div>
            </div>
            <div class="card">
                <div class="label" data-i18n="view.market_status.card.holiday">Holiday</div>
                <div class="value">${esc(s.holiday || '—')}</div>
            </div>
            <div class="card">
                <div class="label" data-i18n="view.market_status.card.timezone">Timezone</div>
                <div class="value">${esc(s.timezone || '—')}</div>
            </div>
            <div class="card">
                <div class="label" data-i18n="view.market_status.card.exchange">Exchange</div>
                <div class="value">${esc(s.exchange || '—')}</div>
            </div>
        </div>
    `;
}

function renderHolidays(el, h) {
    if (!el) return;
    // Backend returns `{available:false}` on plan-restricted exchanges —
    // render the same "not on this plan" hint as the status panel so the
    // user gets one consistent message per exchange.
    if (h && h.available === false) {
        const reasonKey = h.reason === 'premium-required'
            ? 'view.market_status.unavailable.premium'
            : 'view.market_status.unavailable.upstream';
        el.innerHTML = `<p class="muted">${esc(t(reasonKey, { exchange: h.exchange || '' }))}</p>`;
        return;
    }
    const rows = h?.data || (Array.isArray(h) ? h : []);
    if (!rows.length) {
        el.innerHTML = `<p class="muted" data-i18n="view.market_status.empty.holidays">No upcoming holidays.</p>`;
        return;
    }
    const sorted = [...rows].sort((a, b) =>
        String(a.atDate || '').localeCompare(String(b.atDate || '')));
    el.innerHTML = `<table class="trades">
        <thead><tr>
            <th data-i18n="view.market_status.th.date">Date</th>
            <th data-i18n="view.market_status.th.event">Event</th>
            <th data-i18n="view.market_status.th.trading_hour">Trading hour</th>
        </tr></thead>
        <tbody>${sorted.map(r => `
            <tr>
                <td>${esc(r.atDate || '—')}</td>
                <td>${esc(r.eventName || '—')}</td>
                <td class="muted">${esc(r.tradingHour || '—')}</td>
            </tr>
        `).join('')}</tbody>
    </table>`;
}
