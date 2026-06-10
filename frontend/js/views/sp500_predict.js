// S&P 500 inclusion predictor — score a list of symbols against the
// official S&P DJI methodology (market_cap ≥ $22.7B, public float
// ≥ 50%, $-vol/FFMC ≥ 1.0, US-domiciled, profitable 4 quarters).
// Names scoring 100 with passes_all=true are likely-add candidates
// for the next quarterly rebalance — ~6% inclusion-pop edge.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

// Hand-picked candidates commonly cited as borderline / next-in-line.
// These are mid-cap-to-large-cap US names NOT currently in S&P 500
// at the time of writing. The user can override the input list.
const DEFAULT_SYMS = 'PLTR,SMCI,DELL,COIN,RBLX,SNOW,DKNG,HOOD,WBD,UBER,DASH,RIVN,LCID,APP,ARM,SHOP,MELI,SPOT,MSTR,CELH,DDOG,TEAM,NET,CRWD,ZS,PANW,FTNT,DASH,BROS';

export async function renderSp500Predict(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.sp500_predict.title">// S&P 500 INCLUSION PREDICTOR</span></h1>
        <p class="muted small" data-i18n-html="view.sp500_predict.intro">
            Scores a user-supplied list of symbols against the official S&amp;P DJI
            methodology criteria — market cap ≥ $22.7B, public float ≥ 50%, annualised
            $-volume / FFMC ≥ 1.0, US-domiciled, trailing-4Q aggregate AND most-recent-
            quarter GAAP net income &gt; 0. Composite 0-100 + per-criterion pass/fail.
            Names scoring 100 with <code>passes_all</code> are mechanical-inclusion
            candidates for the next quarterly rebalance — historically a ~6%
            announcement-day pop + ~3% effective-date pop from passive-fund forced buying
            (Lynch &amp; Mendenhall 1997, Chen Noronha Singal 2004, follow-ups since).
            First scan can take 30-90s while Yahoo quoteSummary fetches each name.
        </p>
        <div class="chart-panel">
            <div class="sp-controls" style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <label style="flex:1 1 400px">
                    <span data-i18n="view.sp500_predict.label.symbols">symbols (comma-separated)</span>
                    <input type="text" id="sp-symbols" value="${DEFAULT_SYMS}" style="width:100%;text-transform:uppercase">
                </label>
                <label>
                    <span data-i18n="view.sp500_predict.label.min_mc">min market cap ($B)</span>
                    <input type="number" id="sp-min-mc" min="1" step="0.5" value="22.7" style="width:80px">
                </label>
                <button class="btn btn-sm primary" id="sp-scan" data-shortcut="r" data-i18n="view.sp500_predict.btn.scan">⚡ Scan</button>
                <span class="muted small" id="sp-meta"></span>
            </div>
            <table class="trades" id="sp-table">
                <thead><tr>
                    <th data-i18n="view.sp500_predict.th.rank">#</th>
                    <th data-i18n="view.sp500_predict.th.symbol">Symbol</th>
                    <th data-i18n="view.sp500_predict.th.composite">Composite</th>
                    <th data-i18n="view.sp500_predict.th.passes">All Pass?</th>
                    <th data-i18n="view.sp500_predict.th.market_cap">Market Cap</th>
                    <th data-i18n="view.sp500_predict.th.float">Float</th>
                    <th data-i18n="view.sp500_predict.th.liquidity">Liquidity</th>
                    <th data-i18n="view.sp500_predict.th.domicile">US Domicile</th>
                    <th data-i18n="view.sp500_predict.th.profitability">Profitable</th>
                </tr></thead>
                <tbody><tr><td colspan="9" class="muted" data-i18n="view.sp500_predict.empty.hint">Enter symbols and click Scan — fetches Yahoo quoteSummary per name (paced).</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelector('#sp-scan').addEventListener('click', () => runScan(mount));
}

async function runScan(mount) {
    const tbody = mount.querySelector('#sp-table tbody');
    const meta = mount.querySelector('#sp-meta');
    const symbols = mount.querySelector('#sp-symbols').value.trim();
    const minMcB = parseFloat(mount.querySelector('#sp-min-mc').value) || 22.7;
    if (!symbols) {
        tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(t('view.sp500_predict.empty.no_symbols'))}</td></tr>`;
        return;
    }
    tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(t('view.sp500_predict.status.scanning'))}</td></tr>`;
    if (meta) meta.textContent = '';
    try {
        const url = `/sp500-predict/scan?symbols=${encodeURIComponent(symbols)}&min_market_cap_usd=${minMcB * 1e9}`;
        const rows = await api.request(url);
        if (!rows.length) {
            tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(t('view.sp500_predict.empty.no_rows'))}</td></tr>`;
            return;
        }
        const passes = rows.filter(r => r.passes_all).length;
        if (meta) meta.textContent = t('view.sp500_predict.meta.summary')
            .replace('{n}', rows.length).replace('{p}', passes);
        tbody.innerHTML = rows.map((r, i) => {
            const compCls = r.composite >= 100 ? 'pos' : r.composite >= 80 ? '' : 'muted';
            const pickCrit = (name) => r.criteria.find(c => c.name === name);
            const cell = (c) => {
                if (!c) return `<td class="muted">—</td>`;
                const cls = c.passed ? 'pos' : c.partial_score >= 0.6 ? '' : 'neg';
                const pct = (c.partial_score * 100).toFixed(0) + '%';
                return `<td class="${cls}" title="${esc(c.detail)}">${pct}</td>`;
            };
            return `<tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
                <td class="muted">${i + 1}</td>
                <td><strong style="color:var(--accent)">${esc(r.symbol)}</strong></td>
                <td class="${compCls}"><strong>${r.composite.toFixed(1)}</strong></td>
                <td class="${r.passes_all ? 'pos' : 'muted'}">${r.passes_all ? '✓' : '✗'}</td>
                ${cell(pickCrit('market_cap'))}
                ${cell(pickCrit('public_float'))}
                ${cell(pickCrit('liquidity'))}
                ${cell(pickCrit('us_domicile'))}
                ${cell(pickCrit('profitability'))}
            </tr>`;
        }).join('');
    } catch (e) {
        tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(String(e))}</td></tr>`;
    }
}
