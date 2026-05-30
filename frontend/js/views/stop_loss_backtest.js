// Single-method stop-loss backtester view. Replay a list of trades
// through one stop-loss strategy and report what would have happened.
// Pairs with the existing stop-loss-best-of view which compares N
// methods side-by-side.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    METHODS, DEFAULT_PARAMS, DEFAULT_SIDE_LONG,
    parseTradeBlob, validateInputs, buildBody, localSimulate, stopPriceFor,
    methodBadge, methodLabelKey, makeDemoTrades, makeDemoParams,
    fmtN, fmtSigned, fmtPct,
} from '../_stop_loss_backtest_inputs.js';

let state = {
    trades: makeDemoTrades('mixed'),
    params: { ...DEFAULT_PARAMS },
    side_long: DEFAULT_SIDE_LONG,
};

export async function renderStopLossBacktest(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.stop_loss_backtest.h1.title" class="view-title">// STOP-LOSS BACKTEST</h1>

        <div class="chart-panel" data-context-scope="stop-loss-backtest">
            <h2 data-i18n="view.stop_loss_backtest.h2.trades">Historical trades
                <small data-i18n="view.stop_loss_backtest.h2.trades_hint" class="muted">(per line: entry MAE MFE actual_exit — MAE/MFE are POSITIVE magnitudes)</small></h2>
            <textarea id="slb-trades" rows="8"
                      data-tip="view.stop_loss_backtest.tip.trades"
                      placeholder="100 3 8 106&#10;102 5 4 99">${esc(tradesToBlob(state.trades))}</textarea>

            <h2 data-i18n="view.stop_loss_backtest.h2.params">Stop method</h2>
            <div class="inline-form">
                <label><span data-i18n="view.stop_loss_backtest.label.method">Method</span>
                    <select id="slb-method">
                        ${METHODS.map(m => `<option value="${m}" ${state.params.method === m ? 'selected' : ''} data-i18n="${methodLabelKey(m)}">${m}</option>`).join('')}
                    </select></label>
                <label><span data-i18n="view.stop_loss_backtest.label.value">Value</span>
                    <input id="slb-value" type="number" step="any" value="${state.params.value}"></label>
                <label><span data-i18n="view.stop_loss_backtest.label.atr">ATR (only for atr_multiple)</span>
                    <input id="slb-atr" type="number" step="any" min="0" value="${state.params.atr}"></label>
                <label><span data-i18n="view.stop_loss_backtest.label.side">Side</span>
                    <select id="slb-side">
                        <option value="long"  ${state.side_long ? 'selected' : ''} data-i18n="view.stop_loss_backtest.side.long">long</option>
                        <option value="short" ${!state.side_long ? 'selected' : ''} data-i18n="view.stop_loss_backtest.side.short">short</option>
                    </select></label>
                <button data-i18n="view.stop_loss_backtest.btn.simulate" id="slb-run" class="primary"
                        data-tip="view.stop_loss_backtest.tip.simulate" type="button">Simulate</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.stop_loss_backtest.btn.demo_mixed"  id="slb-demo-mixed"  class="secondary" type="button">Demo: 10 mixed long</button>
                <button data-i18n="view.stop_loss_backtest.btn.demo_high"   id="slb-demo-high"   class="secondary" type="button">Demo: high MAE (tight stop kills)</button>
                <button data-i18n="view.stop_loss_backtest.btn.demo_low"    id="slb-demo-low"    class="secondary" type="button">Demo: low MAE (tight stop safe)</button>
                <button data-i18n="view.stop_loss_backtest.btn.demo_short"  id="slb-demo-short"  class="secondary" type="button">Demo: short side</button>
                <button data-i18n="view.stop_loss_backtest.btn.demo_loss"   id="slb-demo-loss"   class="secondary" type="button">Demo: all losers</button>
                <button data-i18n="view.stop_loss_backtest.btn.demo_win"    id="slb-demo-win"    class="secondary" type="button">Demo: all winners</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.stop_loss_backtest.btn.preset_none"      id="slb-p-none"   class="secondary" type="button">Preset: NO stop</button>
                <button data-i18n="view.stop_loss_backtest.btn.preset_tight_pct" id="slb-p-tight"  class="secondary" type="button">Preset: 2% tight</button>
                <button data-i18n="view.stop_loss_backtest.btn.preset_loose_pct" id="slb-p-loose"  class="secondary" type="button">Preset: 5% loose</button>
                <button data-i18n="view.stop_loss_backtest.btn.preset_dollar_1"  id="slb-p-d1"     class="secondary" type="button">Preset: $1 dollar</button>
                <button data-i18n="view.stop_loss_backtest.btn.preset_dollar_3"  id="slb-p-d3"     class="secondary" type="button">Preset: $3 dollar</button>
                <button data-i18n="view.stop_loss_backtest.btn.preset_atr_2x"    id="slb-p-atr2"   class="secondary" type="button">Preset: 2× ATR</button>
                <button data-i18n="view.stop_loss_backtest.btn.preset_atr_3x"    id="slb-p-atr3"   class="secondary" type="button">Preset: 3× ATR</button>
            </div>
            <p data-i18n="view.stop_loss_backtest.hint.about" class="muted">Replays each trade through the chosen stop rule. If MAE breaches the stop, exit at stop (realized = stop − entry). Otherwise exit at actual_exit. Same engine as the best-of view but compares one method instead of N.</p>
        </div>

        <div id="slb-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.stop_loss_backtest.h2.per_trade">Per-trade replay</h2>
            <div id="slb-table"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.stop_loss_backtest.h2.realized_chart">Realized P&L per trade (stop-hits in red)</h2>
            <div id="slb-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.stop_loss_backtest.h2.excursion_chart">MAE vs MFE per trade</h2>
            <div id="slb-exc-chart" style="width:100%;height:220px"></div>
        </div>

        <div id="slb-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemoTrades = (k) => {
        state.trades = makeDemoTrades(k);
        if (k === 'short-only') {
            state.side_long = false;
            document.getElementById('slb-side').value = 'short';
        }
        document.getElementById('slb-trades').value = tradesToBlob(state.trades);
    };
    document.getElementById('slb-demo-mixed').addEventListener('click', () => loadDemoTrades('mixed'));
    document.getElementById('slb-demo-high').addEventListener('click',  () => loadDemoTrades('high-mae'));
    document.getElementById('slb-demo-low').addEventListener('click',   () => loadDemoTrades('low-mae'));
    document.getElementById('slb-demo-short').addEventListener('click', () => loadDemoTrades('short-only'));
    document.getElementById('slb-demo-loss').addEventListener('click',  () => loadDemoTrades('all-losers'));
    document.getElementById('slb-demo-win').addEventListener('click',   () => loadDemoTrades('all-winners'));
    const loadPreset = (k) => {
        state.params = makeDemoParams(k);
        document.getElementById('slb-method').value = state.params.method;
        document.getElementById('slb-value').value  = state.params.value;
        document.getElementById('slb-atr').value    = state.params.atr;
    };
    document.getElementById('slb-p-none').addEventListener('click',  () => loadPreset('none'));
    document.getElementById('slb-p-tight').addEventListener('click', () => loadPreset('tight-pct'));
    document.getElementById('slb-p-loose').addEventListener('click', () => loadPreset('loose-pct'));
    document.getElementById('slb-p-d1').addEventListener('click',    () => loadPreset('dollar-1'));
    document.getElementById('slb-p-d3').addEventListener('click',    () => loadPreset('dollar-3'));
    document.getElementById('slb-p-atr2').addEventListener('click',  () => loadPreset('atr-2x'));
    document.getElementById('slb-p-atr3').addEventListener('click',  () => loadPreset('atr-3x'));
    document.getElementById('slb-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function tradesToBlob(trades) {
    return trades.map(t => `${t.entry} ${t.mae} ${t.mfe} ${t.actual_exit}`).join('\n');
}

function readInputs() {
    const p = parseTradeBlob(document.getElementById('slb-trades').value);
    if (p.errors.length) {
        showErr(`${t('view.stop_loss_backtest.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.trades = p.trades;
    state.params = {
        method: document.getElementById('slb-method').value,
        value:  Number(document.getElementById('slb-value').value),
        atr:    Number(document.getElementById('slb-atr').value),
    };
    state.side_long = document.getElementById('slb-side').value === 'long';
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.trades, state.params, state.side_long);
    if (err) { showErr(err); return; }
    const local = localSimulate(state.trades, state.params, state.side_long);
    renderSummary(local, true);
    renderTable(local);
    renderRealizedChart();
    renderExcursionChart();
    let resp;
    try {
        resp = await api.discStopLossBacktest(buildBody(state.trades, state.params, state.side_long));
    } catch (e) {
        showErr(`${t('view.stop_loss_backtest.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderTable(resp);
    renderRealizedChart();
    renderExcursionChart();
}

function renderExcursionChart() {
    const el = document.getElementById('slb-exc-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (state.trades || []).filter(tr =>
        Number.isFinite(Number(tr.mae)) && Number.isFinite(Number(tr.mfe)));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.stop_loss_backtest.empty_exc_chart">${esc(t('view.stop_loss_backtest.empty_exc_chart'))}</div>`;
        return;
    }
    const xs = rows.map((_, i) => i + 1);
    const mae = rows.map(tr => -Number(tr.mae));
    const mfe = rows.map(tr => Number(tr.mfe));
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.stop_loss_backtest.chart.trade') },
            { label: t('view.stop_loss_backtest.chart.mae_neg'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 10, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.stop_loss_backtest.chart.mfe'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 10, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.stop_loss_backtest.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [ { stroke: '#aab', size: 28 }, { stroke: '#aab', size: 56 } ],
        legend: { show: true },
    }, [xs, mae, mfe, zero], el);
}

function renderRealizedChart() {
    const el = document.getElementById('slb-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (state.trades || []).map((tr, i) => {
        const stop = stopPriceFor(tr, state.params, state.side_long);
        const maePrice = state.side_long ? tr.entry - tr.mae : tr.entry + tr.mae;
        const hit = state.side_long ? maePrice <= stop : maePrice >= stop;
        const realized = hit
            ? (state.side_long ? stop - tr.entry : tr.entry - stop)
            : (state.side_long ? tr.actual_exit - tr.entry : tr.entry - tr.actual_exit);
        return { idx: i + 1, realized, hit };
    }).filter(r => Number.isFinite(r.realized));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.stop_loss_backtest.empty_chart">${esc(t('view.stop_loss_backtest.empty_chart'))}</div>`;
        return;
    }
    const xs = rows.map(r => r.idx);
    const winY  = rows.map(r => !r.hit && r.realized >= 0 ? r.realized : null);
    const loseY = rows.map(r => !r.hit && r.realized <  0 ? r.realized : null);
    const stopY = rows.map(r =>  r.hit ? r.realized : null);
    const zero  = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.stop_loss_backtest.chart.trade') },
            { label: t('view.stop_loss_backtest.chart.win'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 10, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.stop_loss_backtest.chart.lose'),
              stroke: '#ffb84a', width: 0,
              points: { show: true, size: 10, fill: '#ffb84a', stroke: '#ffb84a' } },
            { label: t('view.stop_loss_backtest.chart.stop'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.stop_loss_backtest.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [ { stroke: '#aab', size: 28 }, { stroke: '#aab', size: 56 } ],
        legend: { show: true },
    }, [xs, winY, loseY, stopY, zero], el);
}

function renderSummary(report, pending) {
    const n = state.trades.length;
    const badge = methodBadge(report, n);
    const local = localSimulate(state.trades, state.params, state.side_long);
    const parityOk = Math.abs(report.total_realized - local.total_realized) < 1e-6
                  && report.stopped_out_count === local.stopped_out_count;
    const localTag = pending ? ` (${t('view.stop_loss_backtest.tag.local')})` : '';
    const winRate = n > 0 ? report.winning_trades / n : 0;
    const stopRate = n > 0 ? report.stopped_out_count / n : 0;
    document.getElementById('slb-summary').innerHTML = [
        card(t('view.stop_loss_backtest.card.verdict'),       t(badge.key) + localTag, badge.cls),
        card(t('view.stop_loss_backtest.card.total_realized'),
             fmtSigned(report.total_realized),
             report.total_realized >= 0 ? 'pos' : 'neg'),
        card(t('view.stop_loss_backtest.card.avg_realized'),
             fmtSigned(report.avg_realized),
             report.avg_realized >= 0 ? 'pos' : 'neg'),
        card(t('view.stop_loss_backtest.card.trades'),        String(n)),
        card(t('view.stop_loss_backtest.card.wins'),
             `${report.winning_trades} / ${n}`,
             winRate >= 0.5 ? 'pos' : 'neg'),
        card(t('view.stop_loss_backtest.card.win_rate'),
             fmtPct(winRate),
             winRate >= 0.5 ? 'pos' : 'neg'),
        card(t('view.stop_loss_backtest.card.stops_hit'),
             `${report.stopped_out_count} / ${n}`,
             stopRate > 0.5 ? 'neg' : ''),
        card(t('view.stop_loss_backtest.card.method'),
             t(methodLabelKey(report.method))),
        card(t('view.stop_loss_backtest.card.parity'),
             parityOk ? t('view.stop_loss_backtest.tag.ok') : t('view.stop_loss_backtest.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderTable(report) {
    const wrap = document.getElementById('slb-table');
    if (!state.trades || state.trades.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.stop_loss_backtest.empty">${esc(t('view.stop_loss_backtest.empty'))}</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.stop_loss_backtest.col.idx">#</th>
                <th data-i18n="view.stop_loss_backtest.col.entry">Entry</th>
                <th data-i18n="view.stop_loss_backtest.col.mae">MAE</th>
                <th data-i18n="view.stop_loss_backtest.col.mfe">MFE</th>
                <th data-i18n="view.stop_loss_backtest.col.stop_price">Stop price</th>
                <th data-i18n="view.stop_loss_backtest.col.exit_price">Exit price</th>
                <th data-i18n="view.stop_loss_backtest.col.stopped">Stopped?</th>
                <th data-i18n="view.stop_loss_backtest.col.realized">Realized</th>
            </tr></thead>
            <tbody>
                ${state.trades.map((tr, i) => {
                    const stop = stopPriceFor(tr, state.params, state.side_long);
                    const maePrice = state.side_long ? tr.entry - tr.mae : tr.entry + tr.mae;
                    const hit = state.side_long ? maePrice <= stop : maePrice >= stop;
                    const exitPrice = hit ? stop : tr.actual_exit;
                    const realized = hit
                        ? (state.side_long ? stop - tr.entry : tr.entry - stop)
                        : (state.side_long ? tr.actual_exit - tr.entry : tr.entry - tr.actual_exit);
                    return `<tr class="${hit ? 'neg' : ''}">
                        <td>${i + 1}</td>
                        <td>${esc(fmtN(tr.entry))}</td>
                        <td>${esc(fmtN(tr.mae))}</td>
                        <td>${esc(fmtN(tr.mfe))}</td>
                        <td>${Number.isFinite(stop) ? esc(fmtN(stop)) : '—'}</td>
                        <td>${esc(fmtN(exitPrice))}</td>
                        <td class="${hit ? 'neg' : ''}">${hit ? '✓' : '·'}</td>
                        <td class="${realized >= 0 ? 'pos' : 'neg'}">${esc(fmtSigned(realized))}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
    void report;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('slb-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('slb-err').style.display = 'none'; }
