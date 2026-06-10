// Greenblatt magic formula value scorer. Ranks S&P-universe stocks by
// combined (earnings yield + ROIC) ranking — Joel Greenblatt's "Little
// Book That Beats the Market" (2005). Buy the top 20-30, hold for a
// year, rebalance annually.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderMagicFormula(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.magic_formula.title">// MAGIC FORMULA · GREENBLATT VALUE SCORER</span></h1>
        <p class="muted small" data-i18n-html="view.magic_formula.intro">
            Joel Greenblatt's magic formula ranks stocks on two metrics combined:
            <strong>earnings yield</strong> = EBIT / Enterprise Value (how cheap)
            and <strong>ROIC</strong> = EBIT / Invested Capital (how good).
            Lower combined rank = better candidate (cheap + high quality).
            The book recommends holding the top 20-30 for one year, rebalancing
            annually. Universe: top S&P names from the heatmap UNIVERSE mapping.
            Yahoo quoteSummary fetches one symbol per request — keep
            <code>max_symbols</code> ≤ 50 to avoid rate-limits.
        </p>
        <div class="chart-panel">
            <div style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <label>
                    <span class="muted small" data-i18n="view.magic_formula.field.max">Max symbols</span>
                    <input type="number" id="mf-max" step="5" min="1" max="250" value="50" style="width:80px">
                </label>
                <button class="btn btn-sm primary" id="mf-run" data-shortcut="r" data-i18n="view.magic_formula.btn.run">⚡ Score Universe</button>
                <span class="muted small" id="mf-meta"></span>
            </div>
            <div id="mf-result"></div>
        </div>
    `;
    mount.querySelector('#mf-run').addEventListener('click', () => runScore(mount));
}

async function runScore(mount) {
    const result = mount.querySelector('#mf-result');
    const meta = mount.querySelector('#mf-meta');
    const maxSymbols = parseInt(mount.querySelector('#mf-max').value, 10) || 50;
    result.innerHTML = `<p class="muted">${esc(t('view.magic_formula.status.scoring'))}</p>`;
    if (meta) meta.textContent = '';
    try {
        const r = await api(`/magic-formula/rank?max_symbols=${maxSymbols}`);
        if (!r || !r.scored || !r.scored.length) {
            result.innerHTML = `<p class="muted">${esc(t('view.magic_formula.empty.no_data'))}</p>`;
            return;
        }
        if (meta) meta.textContent = t('view.magic_formula.meta.summary')
            .replace('{u}', r.universe_size).replace('{s}', r.scored.length).replace('{e}', r.errors.length);
        result.innerHTML = `
            <h2>${esc(t('view.magic_formula.h2.top_ranked'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.magic_formula.th.rank">Combined Rank</th>
                    <th data-i18n="view.magic_formula.th.symbol">Symbol</th>
                    <th data-i18n="view.magic_formula.th.ey">EY %</th>
                    <th data-i18n="view.magic_formula.th.roic">ROIC %</th>
                    <th data-i18n="view.magic_formula.th.ey_rank">EY Rank</th>
                    <th data-i18n="view.magic_formula.th.roic_rank">ROIC Rank</th>
                </tr></thead>
                <tbody>${r.scored.map(s => {
                    const eyCls = s.earnings_yield_pct >= 10 ? 'pos' : s.earnings_yield_pct >= 5 ? '' : 'muted';
                    const roicCls = s.roic_pct >= 20 ? 'pos' : s.roic_pct >= 10 ? '' : 'muted';
                    return `<tr>
                        <td><strong>${s.combined_rank}</strong></td>
                        <td><strong>${esc(s.symbol)}</strong></td>
                        <td class="${eyCls}">${s.earnings_yield_pct.toFixed(2)}</td>
                        <td class="${roicCls}">${s.roic_pct.toFixed(2)}</td>
                        <td>${s.earnings_yield_rank}</td>
                        <td>${s.roic_rank}</td>
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
