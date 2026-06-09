// IV term-structure scanner — ranks symbols by front-vs-back IV
// inversion. Positive inversion = calendar-spread candidate (sell
// rich front-month, buy relatively-underpriced back-month).

import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { t } from '../i18n.js';

export async function renderIvTerm(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.iv_term.title">// IV TERM STRUCTURE · CALENDAR-SPREAD CANDIDATES</span></h1>
        <p class="muted small" data-i18n-html="view.iv_term.intro">
            For each top-30 most-active symbol, pulls ATM call IV from every available
            expiration (up to 10) and computes the term-structure shape. <strong>Inversion
            score</strong> = front-2-expiry avg IV − back-2-expiry avg IV. Above ~5pp
            (≥0.05) signals an event-priced front-month → classic <em>calendar spread</em>
            entry: sell the rich front-month, buy the relatively-underpriced back-month.
            P&amp;L crystallises when the front-month IV decays to the long-term level.
            Refreshes every 4 hours.
        </p>
        <div class="chart-panel">
            <div class="iv-controls" style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <button class="btn btn-sm" id="iv-refresh" data-shortcut="r" data-i18n="common.btn.refresh">⟳ Refresh</button>
                <span class="muted small" id="iv-meta"></span>
            </div>
            <table class="trades" id="iv-table">
                <thead><tr>
                    <th data-i18n="view.iv_term.th.rank">#</th>
                    <th data-i18n="view.iv_term.th.symbol">Symbol</th>
                    <th data-i18n="view.iv_term.th.inversion">Inversion</th>
                    <th data-i18n="view.iv_term.th.front_iv">Front IV</th>
                    <th data-i18n="view.iv_term.th.back_iv">Back IV</th>
                    <th data-i18n="view.iv_term.th.slope">Slope (pp/d)</th>
                    <th data-i18n="view.iv_term.th.points">Expirations</th>
                    <th data-i18n="view.iv_term.th.recommendation">Recommendation</th>
                    <th data-i18n="view.iv_term.th.observed">Observed</th>
                </tr></thead>
                <tbody><tr><td colspan="9" class="muted" data-i18n="common.loading">loading…</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelector('#iv-refresh').addEventListener('click', () => fetchAndRender(mount));
    fetchAndRender(mount);
}

async function fetchAndRender(mount) {
    const tbody = mount.querySelector('#iv-table tbody');
    const meta = mount.querySelector('#iv-meta');
    if (!tbody) return;
    tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(t('common.loading'))}</td></tr>`;
    try {
        const rows = await api('/iv-term/ranked?limit=30');
        if (!rows.length) {
            tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(t('view.iv_term.empty.no_rows'))}</td></tr>`;
            if (meta) meta.textContent = '';
            return;
        }
        if (meta) meta.textContent = t('view.iv_term.meta.summary').replace('{n}', rows.length);
        tbody.innerHTML = rows.map((r, i) => {
            const invCls = r.inversion_score >= 0.05 ? 'pos' : r.inversion_score >= 0 ? '' : 'neg';
            const recCls = r.recommendation === 'calendar_long_back_short_front' ? 'pos' : 'muted';
            return `<tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
                <td class="muted">${i + 1}</td>
                <td><strong style="color:var(--accent)">${esc(r.symbol)}</strong></td>
                <td class="${invCls}"><strong>${fmtPP(r.inversion_score)}</strong></td>
                <td>${fmtVol(r.front_avg_iv)}</td>
                <td>${fmtVol(r.back_avg_iv)}</td>
                <td>${fmtSlope(r.slope_pct_per_day)}</td>
                <td>${r.points.length}</td>
                <td class="${recCls}">${esc(r.recommendation)}</td>
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
function fmtPP(v) {
    if (v == null) return '—';
    const s = (v * 100).toFixed(1);
    return (v >= 0 ? '+' : '') + s + 'pp';
}
function fmtSlope(v) {
    if (v == null) return '—';
    return (v * 10_000).toFixed(2) + ' bp/d';
}
