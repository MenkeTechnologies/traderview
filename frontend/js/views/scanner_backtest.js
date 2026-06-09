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
                <button class="btn btn-sm primary" id="sb-run-all" data-shortcut="r" data-i18n="view.scanner_backtest.btn.run_all">⚡ Run All Scanners</button>
                <button class="btn btn-sm" id="sb-run" data-i18n="view.scanner_backtest.btn.run">PEAD only</button>
                <button class="btn btn-sm" id="sb-run-insider" data-i18n="view.scanner_backtest.btn.run_insider">Insider clusters only</button>
                <label style="display:flex;align-items:center;gap:6px">
                    <input type="checkbox" id="sb-friction">
                    <span class="muted small" data-i18n="view.scanner_backtest.field.friction">Friction-adjusted (slippage + SEC fee)</span>
                </label>
                <button class="btn btn-sm" id="sb-walk-pead" data-i18n="view.scanner_backtest.btn.walk_pead">📊 Walk-forward PEAD</button>
                <button class="btn btn-sm" id="sb-walk-insider" data-i18n="view.scanner_backtest.btn.walk_insider">📊 Walk-forward Insider</button>
                <button class="btn btn-sm" id="sb-stab-pead" data-i18n="view.scanner_backtest.btn.stability_pead">📈 PEAD Stability (Q1-Q4)</button>
                <span class="muted small" id="sb-meta"></span>
            </div>
            <div id="sb-all"></div>
            <div id="sb-result"></div>
        </div>
    `;
    mount.querySelector('#sb-run-all').addEventListener('click', () => runAll(mount));
    mount.querySelector('#sb-run').addEventListener('click', () => runScan(mount, 'pead'));
    mount.querySelector('#sb-run-insider').addEventListener('click', () => runScan(mount, 'insider-clusters'));
    mount.querySelector('#sb-walk-pead').addEventListener('click', () => runWalk(mount, 'pead'));
    mount.querySelector('#sb-walk-insider').addEventListener('click', () => runWalk(mount, 'insider-clusters'));
    mount.querySelector('#sb-stab-pead').addEventListener('click', () => runStability(mount));
}

async function runStability(mount) {
    const result = mount.querySelector('#sb-result');
    const meta = mount.querySelector('#sb-meta');
    const days = parseInt(mount.querySelector('#sb-days').value, 10) || 365;
    result.innerHTML = `<p class="muted">${esc(t('view.scanner_backtest.status.stability'))}</p>`;
    if (meta) meta.textContent = '';
    try {
        const r = await api(`/scanner-backtest/pead/stability?days=${days}&horizon=20`);
        if (!r) {
            result.innerHTML = `<p class="muted">${esc(t('view.scanner_backtest.empty.no_data'))}</p>`;
            return;
        }
        const ratio = r.q4_vs_q1_sharpe_ratio;
        const ratioCls = ratio == null ? 'muted'
            : ratio >= 0.7 ? 'pos' : ratio >= 0.3 ? '' : 'neg';
        const ratioStr = ratio == null ? '—' : ratio.toFixed(2);
        const fh = r.full_window_horizon_stats || {};
        result.innerHTML = `
            <h2>${esc(r.scanner.toUpperCase())} · ${esc(t('view.scanner_backtest.h2.stability'))} · ${r.horizon_days}d</h2>
            <p class="small">${esc(t('view.scanner_backtest.field.q4_vs_q1'))}:
                <strong class="${ratioCls}">${ratioStr}</strong>
                <span class="muted small">${esc(t('view.scanner_backtest.hint.q_ratio'))}</span></p>
            <h3 style="margin-bottom:0.25em">${esc(t('view.scanner_backtest.h3.full_window'))}</h3>
            ${fullHorizonRow(fh)}
            <h3 style="margin-bottom:0.25em">${esc(t('view.scanner_backtest.h3.per_quarter'))}</h3>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.scanner_backtest.th.quarter">Quarter</th>
                    <th data-i18n="view.scanner_backtest.th.n">N</th>
                    <th data-i18n="view.scanner_backtest.th.sharpe">Sharpe</th>
                    <th data-i18n="view.scanner_backtest.th.sharpe_ci">95% CI</th>
                    <th data-i18n="view.scanner_backtest.th.hit_rate">Hit %</th>
                    <th data-i18n="view.scanner_backtest.th.mean">Mean %</th>
                </tr></thead>
                <tbody>${(r.quarters || []).map(q => {
                    const hs = q.horizon_stats || {};
                    if (!hs.n) return `<tr><td>Q${q.quarter_index}</td><td>0</td><td class="muted">—</td><td class="muted">—</td><td class="muted">—</td><td class="muted">—</td></tr>`;
                    const sCls = hs.annualised_sharpe >= 1.0 ? 'pos'
                                : hs.annualised_sharpe >= 0.5 ? '' : 'muted';
                    return `<tr>
                        <td><strong>Q${q.quarter_index}</strong></td>
                        <td>${hs.n}</td>
                        <td class="${sCls}"><strong>${hs.annualised_sharpe.toFixed(2)}</strong></td>
                        <td class="muted small">[${hs.sharpe_ci_lo_95.toFixed(2)}, ${hs.sharpe_ci_hi_95.toFixed(2)}]</td>
                        <td>${hs.hit_rate_pct.toFixed(1)}%</td>
                        <td>${hs.mean_return_pct.toFixed(2)}%</td>
                    </tr>`;
                }).join('')}</tbody>
            </table>
            <p class="muted small">${esc(t('view.scanner_backtest.hint.stability_interpret'))}</p>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}

function fullHorizonRow(h) {
    if (!h || !h.n) return `<p class="muted small">${esc(t('view.scanner_backtest.empty.no_data'))}</p>`;
    const sCls = h.annualised_sharpe >= 1.0 ? 'pos' : h.annualised_sharpe >= 0.5 ? '' : 'muted';
    return `<p>
        N=${h.n} · Sharpe <strong class="${sCls}">${h.annualised_sharpe.toFixed(2)}</strong>
        · 95% CI <span class="muted small">[${h.sharpe_ci_lo_95.toFixed(2)}, ${h.sharpe_ci_hi_95.toFixed(2)}]</span>
        · SE ${h.sharpe_se.toFixed(2)}
    </p>`;
}

async function runWalk(mount, scanner) {
    const result = mount.querySelector('#sb-result');
    const meta = mount.querySelector('#sb-meta');
    const days = parseInt(mount.querySelector('#sb-days').value, 10) || 365;
    result.innerHTML = `<p class="muted">${esc(t('view.scanner_backtest.status.walking'))}</p>`;
    if (meta) meta.textContent = '';
    try {
        const r = await api(`/scanner-backtest/${scanner}/walk-forward?days=${days}&train_pct=70`);
        if (!r) {
            result.innerHTML = `<p class="muted">${esc(t('view.scanner_backtest.empty.no_data'))}</p>`;
            return;
        }
        const ratio = r.oos_to_is_sharpe_ratio_20d;
        const ratioCls = ratio == null ? 'muted'
            : ratio >= 0.7 ? 'pos' : ratio >= 0.3 ? '' : 'neg';
        const ratioStr = ratio == null ? '—' : ratio.toFixed(2);
        if (meta) meta.textContent = t('view.scanner_backtest.meta.walk_summary')
            .replace('{s}', r.scanner).replace('{t}', r.train_samples_used).replace('{x}', r.test_samples_used);
        result.innerHTML = `
            <h2>${esc(r.scanner.toUpperCase())} · ${esc(t('view.scanner_backtest.h2.walk_forward'))} (${r.train_pct}% train)</h2>
            <p class="small">${esc(t('view.scanner_backtest.field.oos_ratio'))}:
                <strong class="${ratioCls}">${ratioStr}</strong>
                <span class="muted small">${esc(t('view.scanner_backtest.hint.ratio_threshold'))}</span></p>
            <h3 style="margin-bottom:0.25em">${esc(t('view.scanner_backtest.h3.train'))} · ${r.train_samples_used} ${esc(t('view.scanner_backtest.field.samples'))}</h3>
            ${horizonTable(r.train_horizons)}
            <h3 style="margin-bottom:0.25em">${esc(t('view.scanner_backtest.h3.test'))} · ${r.test_samples_used} ${esc(t('view.scanner_backtest.field.samples'))}</h3>
            ${horizonTable(r.test_horizons)}
            <p class="muted small">${esc(t('view.scanner_backtest.hint.walk_interpret'))}</p>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}

function horizonTable(horizons) {
    return `
        <table class="trades">
            <thead><tr>
                <th>Horizon</th><th>N</th><th>Hit %</th>
                <th>Mean %</th><th>Stdev %</th><th>Sharpe</th><th>Max DD %</th>
            </tr></thead>
            <tbody>${(horizons || []).map(h => {
                if (!h || h.n === 0) return '';
                const sharpeCls = h.annualised_sharpe >= 1.0 ? 'pos'
                                : h.annualised_sharpe >= 0.5 ? '' : 'muted';
                return `<tr>
                    <td><strong>${h.horizon_days}d</strong></td>
                    <td>${h.n}</td>
                    <td>${h.hit_rate_pct.toFixed(1)}%</td>
                    <td>${h.mean_return_pct.toFixed(2)}%</td>
                    <td>${h.stdev_pct.toFixed(2)}%</td>
                    <td class="${sharpeCls}"><strong>${h.annualised_sharpe.toFixed(2)}</strong></td>
                    <td class="neg">${h.max_drawdown_pct.toFixed(2)}%</td>
                </tr>`;
            }).join('')}</tbody>
        </table>`;
}

async function runAll(mount) {
    const all = mount.querySelector('#sb-all');
    const meta = mount.querySelector('#sb-meta');
    const days = parseInt(mount.querySelector('#sb-days').value, 10) || 365;
    const friction = mount.querySelector('#sb-friction').checked ? 'true' : 'false';
    all.innerHTML = `<p class="muted">${esc(t('view.scanner_backtest.status.running_all'))}</p>`;
    if (meta) meta.textContent = '';
    try {
        const r = await api(`/scanner-backtest/all?days=${days}&friction=${friction}`);
        const scanners = r.scanners || [];
        if (!scanners.length) {
            all.innerHTML = `<p class="muted">${esc(t('view.scanner_backtest.empty.no_data'))}</p>`;
            return;
        }
        if (meta) meta.textContent = t('view.scanner_backtest.meta.all_summary').replace('{n}', scanners.length).replace('{d}', days);
        const horizonList = scanners[0].horizons?.map(h => h.horizon_days) || [1, 5, 20, 60];
        all.innerHTML = `
            <h2 style="margin-top:0">${esc(t('view.scanner_backtest.h2.all'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.scanner_backtest.th.scanner">Scanner</th>
                    <th data-i18n="view.scanner_backtest.th.samples">Samples</th>
                    ${horizonList.map(d => `<th>${d}d Sharpe</th>`).join('')}
                    ${horizonList.map(d => `<th>${d}d Hit%</th>`).join('')}
                    <th data-i18n="view.scanner_backtest.th.notes">Notes</th>
                </tr></thead>
                <tbody>
                ${scanners.map(s => {
                    if (s.error) {
                        return `<tr><td><strong>${esc(s.scanner)}</strong></td>
                            <td>0</td>
                            ${horizonList.map(_ => '<td class="muted">—</td>').join('')}
                            ${horizonList.map(_ => '<td class="muted">—</td>').join('')}
                            <td class="neg small">${esc(s.error)}</td></tr>`;
                    }
                    return `<tr>
                        <td><strong>${esc(s.scanner)}</strong></td>
                        <td>${s.samples_used}</td>
                        ${horizonList.map(d => {
                            const h = s.horizons.find(x => x.horizon_days === d);
                            if (!h || h.n === 0) return '<td class="muted">—</td>';
                            const cls = h.annualised_sharpe >= 1.0 ? 'pos'
                                       : h.annualised_sharpe >= 0.5 ? '' : 'muted';
                            return `<td class="${cls}"><strong>${h.annualised_sharpe.toFixed(2)}</strong></td>`;
                        }).join('')}
                        ${horizonList.map(d => {
                            const h = s.horizons.find(x => x.horizon_days === d);
                            if (!h || h.n === 0) return '<td class="muted">—</td>';
                            const cls = h.hit_rate_pct >= 55 ? 'pos' : h.hit_rate_pct >= 45 ? '' : 'neg';
                            return `<td class="${cls}">${h.hit_rate_pct.toFixed(0)}%</td>`;
                        }).join('')}
                        <td class="muted small"></td>
                    </tr>`;
                }).join('')}
                </tbody>
            </table>
            <p class="muted small" data-i18n="view.scanner_backtest.hint.all">Sorted by 20d Sharpe descending — the horizon Kelly defaults to. Scanners with no data (no historical events cached or no price_bars) appear with —.</p>
        `;
    } catch (e) {
        all.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}

async function runScan(mount, scanner) {
    const result = mount.querySelector('#sb-result');
    const meta = mount.querySelector('#sb-meta');
    const days = parseInt(mount.querySelector('#sb-days').value, 10) || 365;
    result.innerHTML = `<p class="muted">${esc(t('view.scanner_backtest.status.running'))}</p>`;
    if (meta) meta.textContent = '';
    try {
        const r = await api(`/scanner-backtest/${scanner}?days=${days}`);
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
