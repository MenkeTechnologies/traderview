// IPO lockup expiration tracker — surface upcoming forced-supply
// pressure events. ~180 days after IPO, insider holdings unlock and
// hit the tape. The event is mechanical so the edge persists.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderIpoLockups(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ipo_lockups.title">// IPO LOCKUP EXPIRATIONS · 60-DAY HORIZON</span></h1>
        <p class="muted small" data-i18n-html="view.ipo_lockups.intro">
            For every IPO in the trailing 200 days that's priced + tradeable, projects
            the standard 180-day lockup expiration. Surfaces ones expiring within the
            next 60 days. <strong>Field &amp; Hanka 2001</strong> and follow-ups document
            a measurable price drop in the days around lockup expiry from insider
            supply hitting the tape — a mechanical edge that persists because the
            constraint is structural. Unlocked-share estimate is conservative (3× IPO
            float; actual S-1 tranches typically 5-8×).
        </p>
        <div class="chart-panel">
            <div class="il-controls" style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <button class="btn btn-sm" id="il-refresh" data-shortcut="r" data-i18n="common.btn.refresh">⟳ Refresh</button>
                <span class="muted small" id="il-meta"></span>
            </div>
            <table class="trades" id="il-table">
                <thead><tr>
                    <th data-i18n="view.ipo_lockups.th.days">Days</th>
                    <th data-i18n="view.ipo_lockups.th.symbol">Symbol</th>
                    <th data-i18n="view.ipo_lockups.th.name">Name</th>
                    <th data-i18n="view.ipo_lockups.th.exchange">Exch</th>
                    <th data-i18n="view.ipo_lockups.th.ipo_date">IPO Date</th>
                    <th data-i18n="view.ipo_lockups.th.expires">Lockup Expires</th>
                    <th data-i18n="view.ipo_lockups.th.ipo_shares">IPO Shares</th>
                    <th data-i18n="view.ipo_lockups.th.unlocked">Est. Unlocked</th>
                    <th data-i18n="view.ipo_lockups.th.price">Price Range</th>
                </tr></thead>
                <tbody><tr><td colspan="9" class="muted" data-i18n="common.loading">loading…</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelector('#il-refresh').addEventListener('click', () => fetchAndRender(mount));
    fetchAndRender(mount);
}

async function fetchAndRender(mount) {
    const tbody = mount.querySelector('#il-table tbody');
    const meta = mount.querySelector('#il-meta');
    if (!tbody) return;
    tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(t('common.loading'))}</td></tr>`;
    try {
        const rows = await api.request('/ipo-lockups/upcoming');
        if (!rows.length) {
            tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(t('view.ipo_lockups.empty.no_rows'))}</td></tr>`;
            if (meta) meta.textContent = '';
            return;
        }
        if (meta) meta.textContent = t('view.ipo_lockups.meta.summary').replace('{n}', rows.length);
        tbody.innerHTML = rows.map(r => {
            const daysCls = r.days_until_expiry <= 7 ? 'neg' : r.days_until_expiry <= 30 ? '' : 'muted';
            return `<tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
                <td class="${daysCls}"><strong>${r.days_until_expiry}d</strong></td>
                <td><strong style="color:var(--accent)">${esc(r.symbol)}</strong></td>
                <td>${esc(r.name)}</td>
                <td class="muted small">${esc(r.exchange)}</td>
                <td class="muted small">${esc(r.ipo_date)}</td>
                <td><strong>${esc(r.lockup_expires_at)}</strong></td>
                <td>${fmtN(r.ipo_share_count)}</td>
                <td class="neg">${fmtN(r.estimated_unlocked_shares)} <span class="muted">(${r.float_multiple_estimate.toFixed(1)}×)</span></td>
                <td class="muted">${esc(r.ipo_price_range || '—')}</td>
            </tr>`;
        }).join('');
    } catch (e) {
        tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(String(e))}</td></tr>`;
    }
}

function fmtN(n) {
    if (n == null) return '—';
    if (n >= 1_000_000_000) return (n / 1_000_000_000).toFixed(2) + 'B';
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(2) + 'M';
    if (n >= 1_000) return (n / 1_000).toFixed(0) + 'K';
    return n.toLocaleString();
}
