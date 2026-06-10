// S&P 500 Dividend Aristocrats (25+ yr) + Kings (50+ yr) tracker.
// Composite DGI score = yield + 0.5 × growth - 0.1 × (payout - 60) when
// payout > 60. Sorted by score descending.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderDividendAristocrats(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.dividend_aristocrats.title">// DIVIDEND ARISTOCRATS / KINGS</span></h1>
        <p class="muted small" data-i18n-html="view.dividend_aristocrats.intro">
            S&P 500 Dividend <strong>Aristocrats</strong> (25+ consecutive years of
            dividend increases) and <strong>Kings</strong> (50+ years). Each row pulls
            current yield, 5-year growth proxy (current vs avg yield), and payout
            ratio. <strong>Composite DGI score</strong> = yield + 0.5×growth -
            0.1×(payout - 60) when payout > 60. Sorted descending — highest scores
            are the best yield × growth × sustainability combinations.
        </p>
        <div class="chart-panel">
            <div style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <label>
                    <span class="muted small" data-i18n="view.dividend_aristocrats.field.max">Max symbols</span>
                    <input type="number" id="da-max" step="5" min="1" max="100" value="30" style="width:80px">
                </label>
                <button class="btn btn-sm primary" id="da-run" data-shortcut="r" data-i18n="view.dividend_aristocrats.btn.run">⚡ Rank Aristocrats</button>
                <span class="muted small" id="da-meta"></span>
            </div>
            <div id="da-result"></div>
        </div>
    `;
    mount.querySelector('#da-run').addEventListener('click', () => runRank(mount));
}

async function runRank(mount) {
    const result = mount.querySelector('#da-result');
    const meta = mount.querySelector('#da-meta');
    const max = parseInt(mount.querySelector('#da-max').value, 10) || 30;
    result.innerHTML = `<p class="muted">${esc(t('view.dividend_aristocrats.status.fetching'))}</p>`;
    if (meta) meta.textContent = '';
    try {
        const r = await api.request(`/dividend-aristocrats/rank?max_symbols=${max}`);
        const rows = r.rows || [];
        if (meta) meta.textContent = t('view.dividend_aristocrats.meta.summary')
            .replace('{n}', rows.length).replace('{e}', r.errors.length);
        if (!rows.length) {
            result.innerHTML = `<p class="muted">${esc(t('view.dividend_aristocrats.empty.no_data'))}</p>`;
            return;
        }
        result.innerHTML = `
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.dividend_aristocrats.th.symbol">Symbol</th>
                    <th data-i18n="view.dividend_aristocrats.th.kind">Kind</th>
                    <th data-i18n="view.dividend_aristocrats.th.score">DGI Score</th>
                    <th data-i18n="view.dividend_aristocrats.th.yield">Yield %</th>
                    <th data-i18n="view.dividend_aristocrats.th.growth">5y Growth Proxy %</th>
                    <th data-i18n="view.dividend_aristocrats.th.payout">Payout %</th>
                </tr></thead>
                <tbody>${rows.map(s => {
                    const kindBadge = s.kind === 'king' ? '👑 King' : '🎖️ Aristocrat';
                    const scoreCls = s.composite_score >= 5 ? 'pos' : s.composite_score >= 3 ? '' : 'muted';
                    const yldCls = (s.current_yield_pct || 0) >= 4 ? 'pos' : 'muted';
                    const payoutCls = (s.payout_ratio_pct || 0) <= 60 ? 'pos' : (s.payout_ratio_pct || 0) <= 80 ? '' : 'neg';
                    return `<tr>
                        <td><strong>${esc(s.symbol)}</strong></td>
                        <td class="muted small">${kindBadge}</td>
                        <td class="${scoreCls}"><strong>${s.composite_score.toFixed(2)}</strong></td>
                        <td class="${yldCls}">${s.current_yield_pct != null ? s.current_yield_pct.toFixed(2) + '%' : '—'}</td>
                        <td>${s.dividend_growth_5y_pct != null ? s.dividend_growth_5y_pct.toFixed(1) + '%' : '—'}</td>
                        <td class="${payoutCls}">${s.payout_ratio_pct != null ? s.payout_ratio_pct.toFixed(1) + '%' : '—'}</td>
                    </tr>`;
                }).join('')}</tbody>
            </table>
            ${r.errors.length ? `<details><summary class="muted small">${r.errors.length} fetch errors</summary>
                <ul>${r.errors.slice(0, 20).map(e => `<li class="muted small">${esc(e)}</li>`).join('')}</ul></details>` : ''}
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
