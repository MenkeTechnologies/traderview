// Insider cluster scoring — surfaces symbols where multiple distinct
// insiders bought open-market within the trailing 30 days. Cohen
// Malloy Pomorski 2012: opportunistic clusters of 3+ insiders predict
// ~12% alpha over the next 12 months. Highest-edge insider signal.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderInsiderClusters(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.insider_clusters.title">// INSIDER CLUSTER SCORING (30-DAY)</span></h1>
        <p class="muted small" data-i18n-html="view.insider_clusters.intro">
            Riding on the live <code>insider_stream</code> Form-4 firehose, this view
            groups buys per symbol over the trailing 30 days and ranks by a composite
            of distinct-insider-count + role-mix (officers weighted 3×, directors 1.5×,
            10%-owners 1×) + recency-weighted $ value (7-day half-life). Cohen, Malloy
            &amp; Pomorski 2012 documented that opportunistic clusters of ≥3 insiders
            predict ~12% alpha over the next 12 months — one of the largest
            single-signal edges available from free data. Sells are excluded from this
            ranking; the existing Insider Form 4 view shows them separately.
        </p>
        <div class="chart-panel">
            <div class="ic-controls" style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <button class="btn btn-sm" id="ic-refresh" data-shortcut="r" data-i18n="common.btn.refresh">⟳ Refresh</button>
                <span class="muted small" id="ic-meta"></span>
            </div>
            <table class="trades" id="ic-table">
                <thead><tr>
                    <th data-i18n="view.insider_clusters.th.rank">#</th>
                    <th data-i18n="view.insider_clusters.th.symbol">Symbol</th>
                    <th data-i18n="view.insider_clusters.th.score">Score</th>
                    <th data-i18n="view.insider_clusters.th.insiders">Distinct Insiders</th>
                    <th data-i18n="view.insider_clusters.th.roles">Officer/Dir/10%</th>
                    <th data-i18n="view.insider_clusters.th.buys">Buys</th>
                    <th data-i18n="view.insider_clusters.th.dollars">Total $</th>
                    <th data-i18n="view.insider_clusters.th.recency_dollars">Recency-Weighted $</th>
                    <th data-i18n="view.insider_clusters.th.spread">Span</th>
                    <th data-i18n="view.insider_clusters.th.names">Insiders</th>
                </tr></thead>
                <tbody><tr><td colspan="10" class="muted" data-i18n="common.loading">loading…</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelector('#ic-refresh').addEventListener('click', () => fetchAndRender(mount));
    fetchAndRender(mount);
}

async function fetchAndRender(mount) {
    const tbody = mount.querySelector('#ic-table tbody');
    const meta = mount.querySelector('#ic-meta');
    if (!tbody) return;
    tbody.innerHTML = `<tr><td colspan="10" class="muted">${esc(t('common.loading'))}</td></tr>`;
    try {
        const rows = await api.request('/insider-clusters/ranked?limit=50');
        if (!rows.length) {
            tbody.innerHTML = `<tr><td colspan="10" class="muted">${esc(t('view.insider_clusters.empty.no_rows'))}</td></tr>`;
            if (meta) meta.textContent = '';
            return;
        }
        const top3 = rows.filter(r => r.insider_count >= 3).length;
        if (meta) meta.textContent = t('view.insider_clusters.meta.summary')
            .replace('{n}', rows.length).replace('{c}', top3);
        tbody.innerHTML = rows.map((r, i) => {
            const scoreCls = r.insider_count >= 3 ? 'pos' : r.score >= 10 ? '' : 'muted';
            const spanDays = Math.round((new Date(r.latest_buy) - new Date(r.earliest_buy)) / 86_400_000);
            return `<tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
                <td class="muted">${i + 1}</td>
                <td><strong style="color:var(--accent)">${esc(r.symbol)}</strong></td>
                <td class="${scoreCls}"><strong>${r.score.toFixed(1)}</strong></td>
                <td class="${r.insider_count >= 3 ? 'pos' : ''}">${r.insider_count}</td>
                <td class="muted small">${r.officer_count}/${r.director_count}/${r.ten_pct_owner_count}</td>
                <td>${r.buy_count}</td>
                <td>${fmtDollar(r.total_dollars)}</td>
                <td>${fmtDollar(r.recency_weighted_dollars)}</td>
                <td class="muted small">${spanDays}d</td>
                <td class="muted small" title="${esc(r.insiders.join(', '))}">${esc(truncate(r.insiders.join(', '), 60))}</td>
            </tr>`;
        }).join('');
    } catch (e) {
        tbody.innerHTML = `<tr><td colspan="10" class="muted">${esc(String(e))}</td></tr>`;
    }
}

function fmtDollar(n) {
    if (n == null) return '—';
    const abs = Math.abs(n);
    if (abs >= 1_000_000_000) return '$' + (abs / 1_000_000_000).toFixed(2) + 'B';
    if (abs >= 1_000_000) return '$' + (abs / 1_000_000).toFixed(2) + 'M';
    if (abs >= 1_000) return '$' + (abs / 1_000).toFixed(0) + 'K';
    return '$' + abs.toFixed(0);
}
function truncate(s, n) { return s.length > n ? s.slice(0, n - 1) + '…' : s; }
