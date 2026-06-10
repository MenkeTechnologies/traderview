// Vol-risk-premium (VRP) scanner — ranks symbols by IV/RV ratio.
// High ratio = overpriced premium (sell straddles/strangles edge).
// Low ratio = underpriced (buy premium, contrarian long-vol).

import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { t } from '../i18n.js';

let direction = 'sell'; // 'sell' | 'buy'

export async function renderVrp(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.vrp.title">// VOL RISK PREMIUM (IV ÷ RV)</span></h1>
        <p class="muted small" data-i18n-html="view.vrp.intro">
            For each of the top-30 most-active symbols, computes annualised 20-day
            realized vol from daily closes (Yahoo cache) and ATM call implied vol from
            the option expiration nearest 30 DTE. <strong>SELL</strong> direction
            sorts by descending IV/RV — overpriced premium, candidates for short
            straddles / iron condors / credit spreads. <strong>BUY</strong> direction
            ranks ascending — underpriced premium, contrarian long-vol setups.
            Historical SPX mean IV/RV ≈ 1.2-1.3; anything above ~1.6 for a single name
            is meaningfully rich. Refreshes every 60 min in the background.
        </p>
        <div class="chart-panel">
            <div class="vrp-controls" style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <div class="vrp-toggle" role="tablist" aria-label="direction">
                    <button class="btn btn-sm" data-dir="sell" data-i18n="view.vrp.btn.sell">Sell Premium</button>
                    <button class="btn btn-sm" data-dir="buy"  data-i18n="view.vrp.btn.buy">Buy Premium</button>
                </div>
                <button class="btn btn-sm" id="vrp-refresh" data-shortcut="r" data-i18n="common.btn.refresh">⟳ Refresh</button>
                <span class="muted small" id="vrp-meta"></span>
            </div>
            <table class="trades" id="vrp-table">
                <thead><tr>
                    <th data-i18n="view.vrp.th.rank">#</th>
                    <th data-i18n="view.vrp.th.symbol">Symbol</th>
                    <th data-i18n="view.vrp.th.ratio">IV / RV</th>
                    <th data-i18n="view.vrp.th.iv">Implied 30d</th>
                    <th data-i18n="view.vrp.th.rv">Realized 20d</th>
                    <th data-i18n="view.vrp.th.spread">Spread</th>
                    <th data-i18n="view.vrp.th.dte">DTE</th>
                    <th data-i18n="view.vrp.th.spot">Spot</th>
                    <th data-i18n="view.vrp.th.observed">Observed</th>
                </tr></thead>
                <tbody><tr><td colspan="9" class="muted" data-i18n="common.loading">loading…</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelectorAll('[data-dir]').forEach(b => {
        b.addEventListener('click', () => { direction = b.dataset.dir; applyToggleState(mount); fetchAndRender(mount); });
    });
    mount.querySelector('#vrp-refresh').addEventListener('click', () => fetchAndRender(mount));
    applyToggleState(mount);
    fetchAndRender(mount);
}

function applyToggleState(mount) {
    mount.querySelectorAll('[data-dir]').forEach(b => {
        b.classList.toggle('active', b.dataset.dir === direction);
    });
}

async function fetchAndRender(mount) {
    const tbody = mount.querySelector('#vrp-table tbody');
    const meta = mount.querySelector('#vrp-meta');
    if (!tbody) return;
    tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(t('common.loading'))}</td></tr>`;
    try {
        const rows = await api.request(`/vrp/ranked?direction=${direction}&limit=50`);
        if (!rows.length) {
            tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(t('view.vrp.empty.no_rows'))}</td></tr>`;
            if (meta) meta.textContent = '';
            return;
        }
        if (meta) meta.textContent = t('view.vrp.meta.summary').replace('{n}', rows.length);
        tbody.innerHTML = rows.map((r, i) => {
            const ratioCls = r.iv_rv_ratio >= 1.6 ? 'pos'
                          : r.iv_rv_ratio >= 1.2 ? '' : 'neg';
            return `<tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
                <td class="muted">${i + 1}</td>
                <td><strong style="color:var(--accent)">${esc(r.symbol)}</strong></td>
                <td class="${ratioCls}"><strong>${r.iv_rv_ratio.toFixed(2)}</strong></td>
                <td>${fmtVol(r.implied_vol)}</td>
                <td>${fmtVol(r.realized_vol_20d)}</td>
                <td>${(r.iv_rv_spread >= 0 ? '+' : '') + (r.iv_rv_spread * 100).toFixed(1) + 'pp'}</td>
                <td>${r.iv_dte_days}d</td>
                <td>${r.spot.toFixed(2)}</td>
                <td class="muted small">${esc(fmtDateTime(r.observed_at))}</td>
            </tr>`;
        }).join('');
    } catch (e) {
        tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(String(e))}</td></tr>`;
    }
}

function fmtVol(v) {
    if (v == null) return '—';
    return (v * 100).toFixed(1) + '%';
}
