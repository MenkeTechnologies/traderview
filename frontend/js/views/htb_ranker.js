// Hard-to-borrow / squeeze-pressure ranker — composite of
// short_pct_float + days_to_cover + change_pct + inverse-float scored
// 0-100. Refreshes server-side every 30 min for top-30 most-active
// symbols. No WS — data updates monthly so polling on view-open is fine.

import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { t } from '../i18n.js';

export async function renderHtbRanker(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.htb_ranker.title">// HARD-TO-BORROW SQUEEZE RANKER</span></h1>
        <p class="muted small" data-i18n-html="view.htb_ranker.intro">
            Real borrow-fee data requires IB / Markit. This is the standard free-data
            proxy: composite of <code>short_pct_float</code> (30%) + <code>days_to_cover</code> (20%) +
            month-over-month change in shares short (30%) + inverse-float (20%), scored
            0-100. Top-30 most-active symbols refresh server-side every 30 min from
            Finnhub free tier.
        </p>
        <div class="chart-panel">
            <div class="htb-controls" style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <button class="btn btn-sm" id="htb-refresh" data-shortcut="r" data-i18n="common.btn.refresh">⟳ Refresh</button>
                <span class="muted small" id="htb-meta"></span>
            </div>
            <table class="trades" id="htb-table">
                <thead><tr>
                    <th data-i18n="view.htb_ranker.th.rank">#</th>
                    <th data-i18n="view.htb_ranker.th.symbol">Symbol</th>
                    <th data-i18n="view.htb_ranker.th.score">Score</th>
                    <th data-i18n="view.htb_ranker.th.short_pct_float">% Float</th>
                    <th data-i18n="view.htb_ranker.th.dtc">Days to Cover</th>
                    <th data-i18n="view.htb_ranker.th.change_pct">MoM Change %</th>
                    <th data-i18n="view.htb_ranker.th.float">Float (shares)</th>
                    <th data-i18n="view.htb_ranker.th.fetched">As of</th>
                </tr></thead>
                <tbody><tr><td colspan="8" class="muted" data-i18n="common.loading">loading…</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelector('#htb-refresh').addEventListener('click', () => fetchAndRender(mount));
    fetchAndRender(mount);
}

async function fetchAndRender(mount) {
    const tbody = mount.querySelector('#htb-table tbody');
    const meta = mount.querySelector('#htb-meta');
    try {
        const rows = await api.request('/htb-ranker/ranked?limit=50');
        if (!rows || !rows.length) {
            tbody.innerHTML = `<tr><td colspan="8" class="muted">${esc(t('view.htb_ranker.empty.no_rows'))}</td></tr>`;
            meta.textContent = '';
            return;
        }
        meta.textContent = t('view.htb_ranker.meta.count').replace('{n}', rows.length);
        tbody.innerHTML = rows.map((r, i) => `
            <tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
                <td class="muted">${i + 1}</td>
                <td><strong style="color:var(--accent)">${esc(r.symbol)}</strong></td>
                <td>${renderScore(r.score)}</td>
                <td>${fmtPct(r.short_pct_float == null ? null : r.short_pct_float * 100)}</td>
                <td>${r.days_to_cover == null ? '—' : r.days_to_cover.toFixed(2)}</td>
                <td class="${r.change_pct != null && r.change_pct >= 0 ? 'pos' : 'neg'}">${fmtPct(r.change_pct)}</td>
                <td>${fmtN(r.float)}</td>
                <td>${esc(fmtDateTime(r.fetched_at))}</td>
            </tr>
        `).join('');
    } catch (e) {
        tbody.innerHTML = `<tr><td colspan="8" class="muted">${esc(String(e))}</td></tr>`;
    }
}

function renderScore(s) {
    if (s == null) return '—';
    const cls = s >= 75 ? 'pos' : s >= 50 ? '' : 'muted';
    return `<span class="${cls}"><strong>${s.toFixed(1)}</strong></span>`;
}

function fmtPct(n) {
    if (n == null) return '—';
    return (n >= 0 ? '+' : '') + n.toFixed(2) + '%';
}

function fmtN(n) {
    if (n == null) return '—';
    if (n >= 1_000_000_000) return (n / 1_000_000_000).toFixed(2) + 'B';
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M';
    if (n >= 1_000) return (n / 1_000).toFixed(0) + 'K';
    return n.toFixed(0);
}
