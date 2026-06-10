// Analyst-sentiment revision tracker — pulls Finnhub
// /stock/recommendation monthly snapshots per symbol, collapses
// each month to a weighted sentiment score (SB·2 + B − S − SS·2)
// / total, anchors at the latest month, and ranks by composite
// of magnitude + acceleration. Womack 1996 — sell-side rating
// upgrades → ~3% abnormal returns over 30 days.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

const DEFAULT_SYMS = 'AAPL,MSFT,GOOG,META,AMZN,NVDA,TSLA,NFLX,AMD,INTC,CRM,ADBE,ORCL,IBM,JPM,BAC,WFC,XOM,CVX,WMT,COST,HD,LOW,NKE,DIS,UBER,SHOP,PYPL,SQ,SNOW';

export async function renderEarningsRevisions(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.earnings_revisions.title">// EARNINGS REVISION TRACKER</span></h1>
        <p class="muted small" data-i18n-html="view.earnings_revisions.intro">
            For each symbol, pulls Finnhub <code>/stock/recommendation</code> monthly
            snapshots, collapses each row's analyst counts into a weighted sentiment
            score <code>(2·SB + B − S − 2·SS) / total</code>, anchors at the latest
            month, then computes revision velocity: <code>rev_pct_30d</code> vs the
            prior month, <code>rev_pct_90d</code> vs three months ago, and an
            <strong>accelerating</strong> flag when the 30-day move dominates the 90-day
            move (≥40% of magnitude with the same sign). Composite score blends
            absolute magnitudes with the acceleration bonus. Source: <strong>Womack
            1996</strong> — sell-side recommendation upgrades predict ~3% abnormal
            returns over the next 30 days.
        </p>
        <div class="chart-panel">
            <div class="er-controls" style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <label style="flex:1 1 400px">
                    <span data-i18n="view.earnings_revisions.label.symbols">symbols (comma-separated)</span>
                    <input type="text" id="er-symbols" value="${DEFAULT_SYMS}" style="width:100%;text-transform:uppercase">
                </label>
                <button class="btn btn-sm primary" id="er-scan" data-shortcut="r" data-i18n="view.earnings_revisions.btn.scan">⚡ Scan</button>
                <span class="muted small" id="er-meta"></span>
            </div>
            <table class="trades" id="er-table">
                <thead><tr>
                    <th data-i18n="view.earnings_revisions.th.rank">#</th>
                    <th data-i18n="view.earnings_revisions.th.symbol">Symbol</th>
                    <th data-i18n="view.earnings_revisions.th.period">Month</th>
                    <th data-i18n="view.earnings_revisions.th.score">Score</th>
                    <th data-i18n="view.earnings_revisions.th.current">Sentiment</th>
                    <th data-i18n="view.earnings_revisions.th.rev_30">Rev 1mo %</th>
                    <th data-i18n="view.earnings_revisions.th.rev_90">Rev 3mo %</th>
                    <th data-i18n="view.earnings_revisions.th.accel">Accel?</th>
                    <th data-i18n="view.earnings_revisions.th.est_30">1mo ago</th>
                    <th data-i18n="view.earnings_revisions.th.est_90">3mo ago</th>
                </tr></thead>
                <tbody><tr><td colspan="10" class="muted" data-i18n="view.earnings_revisions.empty.hint">Enter symbols and click Scan — fetches Finnhub /stock/recommendation per name.</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelector('#er-scan').addEventListener('click', () => runScan(mount));
}

async function runScan(mount) {
    const tbody = mount.querySelector('#er-table tbody');
    const meta = mount.querySelector('#er-meta');
    const symbols = mount.querySelector('#er-symbols').value.trim();
    if (!symbols) {
        tbody.innerHTML = `<tr><td colspan="10" class="muted">${esc(t('view.earnings_revisions.empty.no_symbols'))}</td></tr>`;
        return;
    }
    tbody.innerHTML = `<tr><td colspan="10" class="muted">${esc(t('view.earnings_revisions.status.scanning'))}</td></tr>`;
    if (meta) meta.textContent = '';
    try {
        const rows = await api.request(`/earnings-revisions/scan?symbols=${encodeURIComponent(symbols)}`);
        if (!rows.length) {
            tbody.innerHTML = `<tr><td colspan="10" class="muted">${esc(t('view.earnings_revisions.empty.no_rows'))}</td></tr>`;
            return;
        }
        const accel = rows.filter(r => r.accelerating).length;
        if (meta) meta.textContent = t('view.earnings_revisions.meta.summary')
            .replace('{n}', rows.length).replace('{a}', accel);
        tbody.innerHTML = rows.map((r, i) => {
            const scoreCls = r.score >= 50 ? 'pos' : r.score >= 20 ? '' : 'muted';
            const r30Cls = r.rev_pct_30d == null ? 'muted' : r.rev_pct_30d >= 0 ? 'pos' : 'neg';
            const r90Cls = r.rev_pct_90d == null ? 'muted' : r.rev_pct_90d >= 0 ? 'pos' : 'neg';
            return `<tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
                <td class="muted">${i + 1}</td>
                <td><strong style="color:var(--accent)">${esc(r.symbol)}</strong></td>
                <td class="muted small">${esc(r.period)}</td>
                <td class="${scoreCls}"><strong>${r.score.toFixed(1)}</strong></td>
                <td>${r.current_estimate.toFixed(3)}</td>
                <td class="${r30Cls}">${fmtPct(r.rev_pct_30d)}</td>
                <td class="${r90Cls}">${fmtPct(r.rev_pct_90d)}</td>
                <td class="${r.accelerating ? 'pos' : 'muted'}">${r.accelerating ? '✓' : '—'}</td>
                <td class="muted small">${r.est_30d_ago == null ? '—' : r.est_30d_ago.toFixed(3)}</td>
                <td class="muted small">${r.est_90d_ago == null ? '—' : r.est_90d_ago.toFixed(3)}</td>
            </tr>`;
        }).join('');
    } catch (e) {
        tbody.innerHTML = `<tr><td colspan="10" class="muted">${esc(String(e))}</td></tr>`;
    }
}

function fmtPct(n) {
    if (n == null) return '—';
    return (n >= 0 ? '+' : '') + n.toFixed(2) + '%';
}
