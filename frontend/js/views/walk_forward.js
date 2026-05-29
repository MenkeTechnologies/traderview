// Walk-forward optimization — rolling IS/OOS sweep; exposes curve-fit collapse.
import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { applyUiI18n } from '../i18n.js';

const KINDS = [
    { id: 'sma_cross',         label: 'SMA crossover (6×6 grid)' },
    { id: 'rsi_reversion',     label: 'RSI mean reversion (5×4 grid)' },
    { id: 'bollinger_breakout',label: 'Bollinger breakout (5×4 grid)' },
    { id: 'macd_cross',        label: 'MACD crossover (1 combo)' },
];

export async function renderWalkForward(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.walk_forward.h1.walk_forward_optimization" class="view-title">// WALK-FORWARD OPTIMIZATION</h1>
        <p class="muted small" data-i18n="view.walk_forward.hint.intro">Rolling in-sample / out-of-sample sweep. For each window we (1) sweep the full preset grid on the IS slice, (2) pick the best params, (3) run those params on the OOS slice with the running equity. OOS slices stitch into one continuous equity curve. Walk-forward efficiency = avg_oos / avg_is. Ratios < 0.5 indicate the strategy is curve-fit (looks great in-sample, dies forward). Compare against the baseline — a single best fit on the entire series — to see how much of the headline backtest return is hindsight bias.</p>

        <div class="chart-panel">
            <form id="wf-form" class="inline-form">
                <input name="symbol" placeholder="symbol (SPY)" value="SPY" required style="text-transform:uppercase">
                <select name="kind">
                    ${KINDS.map(k => `<option value="${k.id}">${esc(k.label)}</option>`).join('')}
                </select>
                <label><span data-i18n="view.walk_forward.label.days">Days</span>
                    <input name="days" type="number" value="1825" style="width:90px"></label>
                <label><span data-i18n="view.walk_forward.label.is_bars">IS bars</span>
                    <input name="is_bars" type="number" value="252" style="width:80px"></label>
                <label><span data-i18n="view.walk_forward.label.oos_bars">OOS bars</span>
                    <input name="oos_bars" type="number" value="63" style="width:80px"></label>
                <label><span data-i18n="view.walk_forward.label.step_bars">Step bars</span>
                    <input name="step_bars" type="number" value="63" style="width:80px"></label>
                <label><span data-i18n="view.walk_forward.label.capital">Capital</span>
                    <input name="capital" type="number" value="10000" style="width:110px"></label>
                <label><span data-i18n="view.walk_forward.label.fee_trade">Fee/trade</span>
                    <input name="fee" type="number" step="any" value="1" style="width:80px"></label>
                <select name="metric">
                    <option data-i18n="view.walk_forward.opt.maximize_return" value="return" selected>Maximize return</option>
                    <option data-i18n="view.walk_forward.opt.maximize_sharpe" value="sharpe">Maximize Sharpe</option>
                </select>
                <button data-i18n="view.walk_forward.btn.run_walk_forward" class="primary" type="submit">Run walk-forward</button>
            </form>
        </div>

        <div id="wf-out"><p data-i18n="view.walk_forward.hint.pick_a_long_history_symbol_spy_qqq_aapl_and_run" class="muted small">Pick a long-history symbol (SPY, QQQ, AAPL) and run.</p></div>
    `;
    mount.querySelector('#wf-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = {
            symbol: fd.get('symbol').trim().toUpperCase(),
            kind: fd.get('kind'),
            days: Number(fd.get('days')) || 1825,
            is_bars: Number(fd.get('is_bars')) || 252,
            oos_bars: Number(fd.get('oos_bars')) || 63,
            step_bars: Number(fd.get('step_bars')) || 63,
            initial_capital: Number(fd.get('capital')) || 10_000,
            fee_per_trade: Number(fd.get('fee')) || 0,
            metric: fd.get('metric'),
        };
        const out = mount.querySelector('#wf-out');
        if (!out) return;
        out.innerHTML = `<div class="boot">running ${body.symbol}, sweeping grid…</div>`;
        try {
            const r = await api.walkForward(body);
            if (!viewIsCurrent(tok)) return;
            const outNow = mount.querySelector('#wf-out');
            if (outNow) renderResult(r, body, outNow, mount);
        } catch (err) {
            if (!viewIsCurrent(tok)) return;
            const outNow = mount.querySelector('#wf-out');
            if (outNow) outNow.innerHTML = `<p class="boot">${esc(err.message)}</p>`;
        }
    });
}

function renderResult(r, body, out, mount) {
    const wfe = r.walk_forward_efficiency;
    const wfeCls = wfe >= 0.7 ? 'pos' : (wfe >= 0.4 ? '' : 'neg');
    const wfeText = wfe >= 0.7 ? 'robust' : (wfe >= 0.4 ? 'marginal' : 'curve-fit');

    out.innerHTML = `
        <div class="cards">
            <div class="card"><div class="label" data-i18n="view.walk_forward.card.wfe">Walk-forward efficiency</div>
                <div class="value ${wfeCls}">${wfe.toFixed(2)}</div>
                <div class="small muted">${wfeText} (avg OOS / avg IS)</div></div>
            <div class="card"><div class="label" data-i18n="view.walk_forward.card.total_oos">Total OOS return</div>
                <div class="value ${r.total_oos_return_pct >= 0 ? 'pos' : 'neg'}">${r.total_oos_return_pct.toFixed(2)}%</div></div>
            <div class="card"><div class="label" data-i18n="view.walk_forward.card.avg_is">Avg IS return / window</div>
                <div class="value">${r.avg_is_return_pct.toFixed(2)}%</div></div>
            <div class="card"><div class="label" data-i18n="view.walk_forward.card.avg_oos">Avg OOS return / window</div>
                <div class="value ${r.avg_oos_return_pct >= 0 ? 'pos' : 'neg'}">${r.avg_oos_return_pct.toFixed(2)}%</div></div>
            <div class="card"><div class="label" data-i18n="view.walk_forward.card.windows_grid">Windows × grid</div>
                <div class="value">${r.windows.length} × ${r.grid_size}</div>
                <div class="small muted">= ${r.windows.length * r.grid_size} backtests</div></div>
            <div class="card"><div class="label" data-i18n="view.walk_forward.card.final_equity">Final equity</div>
                <div class="value">$${fmt(r.final_oos_equity)}</div>
                <div class="small muted">from $${fmt(body.initial_capital)}</div></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.walk_forward.h2.baseline_single_best_fit_entire_series_for_compari">Baseline (single best fit, entire series) — for comparison</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.walk_forward.card.baseline_params">Baseline params</div>
                    <div class="value small">${esc(JSON.stringify(r.baseline_params))}</div></div>
                <div class="card"><div class="label" data-i18n="view.walk_forward.card.baseline_return">Baseline return</div>
                    <div class="value ${r.baseline_summary.total_return_pct >= 0 ? 'pos' : 'neg'}">${r.baseline_summary.total_return_pct.toFixed(2)}%</div></div>
                <div class="card"><div class="label" data-i18n="view.walk_forward.card.baseline_sharpe">Baseline Sharpe</div>
                    <div class="value">${r.baseline_summary.sharpe_daily.toFixed(3)}</div></div>
                <div class="card"><div class="label" data-i18n="view.walk_forward.card.baseline_trades">Baseline trades</div>
                    <div class="value">${r.baseline_summary.trades}</div></div>
            </div>
            <p data-i18n="view.walk_forward.hint.if_baseline_total_oos_the_headline_strategy_is_ove" class="muted small">If baseline ≫ total_OOS, the headline strategy is over-fitted; the parameter set that won on the full history would not have survived rolling re-fits.</p>
        </div>

        <div class="chart-panel">
            <h2>${esc(t('view.walk_forward.h2.stitched_oos', { bars: r.oos_equity.length }))}</h2>
            <div id="wf-eq-svg"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.walk_forward.h2.per_window_is_vs_oos">Per-window IS vs OOS</h2>
            <table class="trades">
                <thead><tr>
                    <th>#</th><th data-i18n="view.walk_forward.th.is_range">IS range</th><th data-i18n="view.walk_forward.th.oos_range">OOS range</th><th data-i18n="view.walk_forward.th.chosen_params">Chosen params</th>
                    <th data-i18n="view.walk_forward.th.is_return">IS return</th><th data-i18n="view.walk_forward.th.is_sharpe">IS Sharpe</th>
                    <th data-i18n="view.walk_forward.th.oos_return">OOS return</th><th data-i18n="view.walk_forward.th.oos_sharpe">OOS Sharpe</th><th data-i18n="view.walk_forward.th.oos_trades">OOS trades</th><th data-i18n="view.walk_forward.th.equity_after">Equity after</th>
                </tr></thead>
                <tbody>
                    ${r.windows.map(w => `<tr>
                        <td>${w.idx + 1}</td>
                        <td class="small">${w.is_start.slice(0, 10)} → ${w.is_end.slice(0, 10)}</td>
                        <td class="small">${w.oos_start.slice(0, 10)} → ${w.oos_end.slice(0, 10)}</td>
                        <td class="small">${esc(JSON.stringify(w.chosen))}</td>
                        <td class="${w.is_return_pct >= 0 ? 'pos' : 'neg'}">${w.is_return_pct.toFixed(2)}%</td>
                        <td>${w.is_sharpe.toFixed(3)}</td>
                        <td class="${w.oos_return_pct >= 0 ? 'pos' : 'neg'}">${w.oos_return_pct.toFixed(2)}%</td>
                        <td>${w.oos_sharpe.toFixed(3)}</td>
                        <td>${w.oos_trades}</td>
                        <td>$${fmt(w.equity_after_window)}</td>
                    </tr>`).join('')}
                </tbody>
            </table>
        </div>
    `;
    try { applyUiI18n(out); } catch (_) {}
    renderEqSvg(r.oos_equity, body.initial_capital, mount);
}

function renderEqSvg(pts, capital, mount) {
    if (!pts.length) return;
    const w = 1000, h = 280, pad = 40;
    const eqs = pts.map(p => p.equity);
    const yMin = Math.min(capital, ...eqs);
    const yMax = Math.max(capital, ...eqs);
    const sx = (i) => pad + (i / Math.max(pts.length - 1, 1)) * (w - 2 * pad);
    const sy = (y) => h - pad - (y - yMin) / Math.max(yMax - yMin, 1e-9) * (h - 2 * pad);
    const path = pts.map((p, i) => (i ? 'L' : 'M') + sx(i) + ',' + sy(p.equity)).join(' ');
    const baseY = sy(capital);
    const eqEl = mount.querySelector('#wf-eq-svg');
    if (!eqEl) return;
    eqEl.innerHTML = `
        <svg viewBox="0 0 ${w} ${h}" width="100%" style="display:block;">
            <line x1="${pad}" y1="${h - pad}" x2="${w - pad}" y2="${h - pad}" stroke="#444"/>
            <line x1="${pad}" y1="${pad}" x2="${pad}" y2="${h - pad}" stroke="#444"/>
            <line x1="${pad}" y1="${baseY}" x2="${w - pad}" y2="${baseY}" stroke="#666" stroke-dasharray="3,3"/>
            <text x="${pad + 4}" y="${baseY - 4}" fill="#9aa0c8" font-size="10">starting capital $${fmt(capital)}</text>
            <path d="${path}" stroke="#00e5ff" stroke-width="2" fill="none"/>
            <text x="${w / 2}" y="${h - 10}" text-anchor="middle" fill="#9aa0c8" font-size="11">OOS bar index (stitched across ${pts.length} bars)</text>
        </svg>
    `;
}
