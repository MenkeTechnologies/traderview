// Scanner backtest — measures which of the scanners ship real edge
// vs which are theoretical. First implementation: PEAD over the
// trailing 365 days using cached price_bars + earnings_events.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderScannerBacktest(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.scanner_backtest.title">// SCANNER BACKTEST</span></h1>
        <p class="muted small" data-i18n-html="view.scanner_backtest.intro">
            For each scanner, takes every historical signal it would have produced over
            the trailing window, looks up forward 1d / 5d / 20d / 60d returns from cached
            price_bars, and aggregates hit rate, mean / median return, annualised Sharpe,
            and max drawdown. <strong>PEAD</strong> is the first wired scanner —
            backtestable from the existing <code>earnings_events</code> table because all
            inputs are cached. Additional scanners (sector_timing, pairs_coint, vrp) port
            in follow-up commits. The goal: stop guessing which scanners have edge and
            start trading the ones with measurable Sharpe > 1.
        </p>
        <div class="chart-panel">
            <div class="sb-controls" style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <label>
                    <span data-i18n="view.scanner_backtest.label.days">lookback (days)</span>
                    <input type="number" id="sb-days" min="60" max="1825" step="30" value="365" style="width:80px">
                </label>
                <button class="btn btn-sm primary" id="sb-run" data-shortcut="r" data-i18n="view.scanner_backtest.btn.run">⚡ Run PEAD Backtest</button>
                <span class="muted small" id="sb-meta"></span>
            </div>
            <div id="sb-result"></div>
        </div>
    `;
    mount.querySelector('#sb-run').addEventListener('click', () => runScan(mount));
}

async function runScan(mount) {
    const result = mount.querySelector('#sb-result');
    const meta = mount.querySelector('#sb-meta');
    const days = parseInt(mount.querySelector('#sb-days').value, 10) || 365;
    result.innerHTML = `<p class="muted">${esc(t('view.scanner_backtest.status.running'))}</p>`;
    if (meta) meta.textContent = '';
    try {
        const r = await api(`/scanner-backtest/pead?days=${days}`);
        if (!r || !r.horizons || !r.horizons.length) {
            result.innerHTML = `<p class="muted">${esc(t('view.scanner_backtest.empty.no_data'))}</p>`;
            return;
        }
        if (meta) meta.textContent = t('view.scanner_backtest.meta.summary')
            .replace('{n}', r.samples_used).replace('{s}', r.scanner);
        result.innerHTML = `
            <h2>${esc(r.scanner.toUpperCase())} · ${r.samples_used} ${esc(t('view.scanner_backtest.field.samples'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.scanner_backtest.th.horizon">Horizon</th>
                    <th data-i18n="view.scanner_backtest.th.n">N</th>
                    <th data-i18n="view.scanner_backtest.th.hit_rate">Hit Rate</th>
                    <th data-i18n="view.scanner_backtest.th.mean">Mean %</th>
                    <th data-i18n="view.scanner_backtest.th.median">Median %</th>
                    <th data-i18n="view.scanner_backtest.th.stdev">Stdev %</th>
                    <th data-i18n="view.scanner_backtest.th.sharpe">Annualised Sharpe</th>
                    <th data-i18n="view.scanner_backtest.th.max_dd">Max DD %</th>
                    <th data-i18n="view.scanner_backtest.th.total_logret">Total log-return</th>
                </tr></thead>
                <tbody>
                ${r.horizons.map(h => {
                    const sharpeCls = h.annualised_sharpe >= 1.0 ? 'pos'
                                    : h.annualised_sharpe >= 0.5 ? '' : 'muted';
                    const meanCls = h.mean_return_pct >= 0 ? 'pos' : 'neg';
                    const hitCls = h.hit_rate_pct >= 55 ? 'pos'
                                 : h.hit_rate_pct >= 45 ? '' : 'neg';
                    return `<tr>
                        <td><strong>${h.horizon_days}d</strong></td>
                        <td>${h.n}</td>
                        <td class="${hitCls}">${h.hit_rate_pct.toFixed(1)}%</td>
                        <td class="${meanCls}">${h.mean_return_pct >= 0 ? '+' : ''}${h.mean_return_pct.toFixed(2)}%</td>
                        <td>${h.median_return_pct >= 0 ? '+' : ''}${h.median_return_pct.toFixed(2)}%</td>
                        <td>${h.stdev_pct.toFixed(2)}%</td>
                        <td class="${sharpeCls}"><strong>${h.annualised_sharpe.toFixed(2)}</strong></td>
                        <td class="neg">${h.max_drawdown_pct.toFixed(2)}%</td>
                        <td class="muted small">${h.total_logret_signed >= 0 ? '+' : ''}${(h.total_logret_signed * 100).toFixed(1)}%</td>
                    </tr>`;
                }).join('')}
                </tbody>
            </table>
            <p class="muted small" data-i18n-html="view.scanner_backtest.hint.interpret">
                <strong>Interpret:</strong> annualised Sharpe ≥ 1 on the longer horizons suggests a real
                edge worth capital. Hit rate ≥ 55% with positive mean is consistent.
                Max-DD shows the worst peak-to-trough — keep it bounded vs your risk budget.
            </p>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
