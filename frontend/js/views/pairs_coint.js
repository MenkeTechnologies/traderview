// Pairs cointegration scanner — stat-arb on the relative spread
// between cointegrated symbols. OLS hedge ratio + AR(1) on spread +
// half-life + z-score. |z| ≥ 2 entry signal with half-life ≤ 30d.
//
// Distinct from the existing `pairs` view (correlation matrix); this
// surfaces actionable stat-arb pairs ranked by entry signal strength.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

const DEFAULT_SYMS = 'AAPL,MSFT,GOOG,GOOGL,META,AMZN,NFLX,NVDA,AMD,INTC,XLF,XLK,KO,PEP,V,MA,GM,F,SPY,QQQ';

export async function renderPairsCoint(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.pairs_coint.title">// PAIRS COINTEGRATION SCANNER</span></h1>
        <p class="muted small" data-i18n-html="view.pairs_coint.intro">
            For every symbol-pair in the input list: OLS regression to find hedge ratio β,
            spread = y - β·x, AR(1) coefficient ρ on the spread, half-life of mean
            reversion = <code>-ln(2) / ln(ρ)</code>, and current z-score. Surfaces pairs
            with <strong>|z| ≥ 2</strong> AND <strong>half-life ≤ 30 days</strong> — the
            classic stat-arb entry signal. Default universe is liquid mega-caps + ETFs.
            Pure compute over daily bars (180-day lookback).
        </p>
        <div class="chart-panel">
            <div class="pairs-controls" style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <label style="flex:1 1 300px">
                    <span data-i18n="view.pairs_coint.label.symbols">symbols (comma-separated)</span>
                    <input type="text" id="pairs-coint-symbols" value="${DEFAULT_SYMS}" style="width:100%;text-transform:uppercase">
                </label>
                <label>
                    <span data-i18n="view.pairs_coint.label.days">days lookback</span>
                    <input type="number" id="pairs-coint-days" min="60" max="730" step="30" value="180" style="width:80px">
                </label>
                <button class="btn btn-sm primary" id="pairs-coint-scan" data-shortcut="r" data-i18n="view.pairs_coint.btn.scan">⚡ Scan</button>
                <span class="muted small" id="pairs-coint-meta"></span>
            </div>
            <table class="trades" id="pairs-coint-table">
                <thead><tr>
                    <th data-i18n="view.pairs_coint.th.rank">#</th>
                    <th data-i18n="view.pairs_coint.th.pair">Pair</th>
                    <th data-i18n="view.pairs_coint.th.beta">β</th>
                    <th data-i18n="view.pairs_coint.th.z">Current Z</th>
                    <th data-i18n="view.pairs_coint.th.rho">ρ</th>
                    <th data-i18n="view.pairs_coint.th.half_life">Half-life (d)</th>
                    <th data-i18n="view.pairs_coint.th.spread">Current Spread</th>
                    <th data-i18n="view.pairs_coint.th.mean">Mean Spread</th>
                    <th data-i18n="view.pairs_coint.th.n">N obs</th>
                </tr></thead>
                <tbody><tr><td colspan="9" class="muted" data-i18n="view.pairs_coint.empty.hint">Enter symbols and click Scan — first scan can take ~10-30s depending on bar-cache state.</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelector('#pairs-coint-scan').addEventListener('click', () => runScan(mount));
}

async function runScan(mount) {
    const tbody = mount.querySelector('#pairs-coint-table tbody');
    const meta = mount.querySelector('#pairs-coint-meta');
    if (!tbody) return;
    const symbols = mount.querySelector('#pairs-coint-symbols').value.trim();
    const days = parseInt(mount.querySelector('#pairs-coint-days').value, 10) || 180;
    if (!symbols) {
        tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(t('view.pairs_coint.empty.no_symbols'))}</td></tr>`;
        return;
    }
    tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(t('view.pairs_coint.status.scanning'))}</td></tr>`;
    if (meta) meta.textContent = '';
    try {
        const rows = await api(`/pairs-coint/scan?symbols=${encodeURIComponent(symbols)}&days=${days}`);
        if (!rows.length) {
            tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(t('view.pairs_coint.empty.no_rows'))}</td></tr>`;
            return;
        }
        if (meta) meta.textContent = t('view.pairs_coint.meta.summary').replace('{n}', rows.length);
        tbody.innerHTML = rows.map((r, i) => {
            const zCls = Math.abs(r.current_z) >= 2.5 ? 'pos' : '';
            const direction = r.current_z > 0 ? 'short A / long B' : 'long A / short B';
            return `<tr>
                <td class="muted">${i + 1}</td>
                <td><strong style="color:var(--accent)" data-context-scope="symbol-row" data-symbol="${esc(r.sym_a)}">${esc(r.sym_a)}</strong>
                    <span class="muted">/</span>
                    <strong style="color:var(--accent)" data-context-scope="symbol-row" data-symbol="${esc(r.sym_b)}">${esc(r.sym_b)}</strong>
                    <span class="muted small">(${esc(direction)})</span></td>
                <td>${r.beta.toFixed(3)}</td>
                <td class="${zCls}"><strong>${r.current_z >= 0 ? '+' : ''}${r.current_z.toFixed(2)}</strong></td>
                <td>${r.rho.toFixed(3)}</td>
                <td>${r.half_life_days.toFixed(1)}</td>
                <td>${r.current_spread.toFixed(3)}</td>
                <td class="muted">${r.mean_spread.toFixed(3)}</td>
                <td>${r.n_obs}</td>
            </tr>`;
        }).join('');
    } catch (e) {
        tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(String(e))}</td></tr>`;
    }
}
