// IV Backtest view — earnings-straddle long/short backtester.
//
// "Should I buy or sell the ATM straddle into this earnings print?"
//
// Given the current implied move (% of spot the market is pricing for
// the post-event jump) and a history of realized post-event moves, the
// backend computes per-quarter avg P&L for both directions plus a
// recommendation. The view visualizes the historical realized
// distribution with the implied move as a vertical reference line so
// the trader can see at a glance whether implied sits in the fat tail
// (cheap → long), in the body (neutral), or beyond the wing (rich → short).

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseRealized, validateInputs, buildBody,
    recommendationBadge, histogram, makeDemoData,
    fmtPct, fmtPnl, fmtWinRate,
} from '../_iv_backtest_inputs.js';

import { t } from '../i18n.js';
let state = { implied: 4.5, realizedText: '' };

export async function renderIvBacktest(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.iv_backtest.h1.iv_backtest_earnings_straddle" class="view-title">// IV BACKTEST · EARNINGS STRADDLE</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.iv_backtest.h2.event_setup">Event setup</h2>
            <div class="inline-form">
                <label><span data-i18n="view.iv_backtest.label.implied">Implied move (%) — what the ATM straddle is pricing for the event</span>
                    <input id="ib-imp" type="number" step="any" min="0" max="100" value="${state.implied}"></label>
                <button data-i18n="view.iv_backtest.btn.backtest" id="ib-run" class="primary" type="button">Backtest</button>
            </div>
            <p data-i18n="view.iv_backtest.hint.implied_move_atm_straddle_debit_spot_100_if_you_re" class="muted">Implied move ≈ ATM straddle debit / spot × 100.
                If you're not sure, options chains usually print it as "Expected Move."</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.iv_backtest.h2.historical_realized_post_event_moves">Historical realized post-event moves (%)</h2>
            <p data-i18n="view.iv_backtest.hint.one_signed_value_per_line_the_move_from_the_close_" class="muted">One signed value per line — the % move from the close before
                the event to the close after. Signs are kept for direction context but the
                backend computes on |realized|, since long-straddle P&amp;L is symmetric.</p>
            <textarea id="ib-real" rows="6" placeholder="7.2&#10;-8.5&#10;5.1&#10;..."></textarea>
            <div class="inline-form">
                <button data-i18n="view.iv_backtest.btn.load_demo_16_quarters_realized_implied" id="ib-demo" class="secondary" type="button">Load demo (16 quarters, realized &gt; implied)</button>
                <button data-i18n="view.iv_backtest.btn.clear" id="ib-clear" class="secondary" type="button">Clear</button>
            </div>
        </div>

        <div id="ib-errors" class="boot" style="display:none"></div>
        <div id="ib-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.iv_backtest.h2.realized_distribution_implied_reference">Realized distribution + implied reference</h2>
            <div id="ib-hist-chart" style="height:280px"></div>
            <p data-i18n="view.iv_backtest.hint.bars_histogram_of_realized_orange_dashed_implied_m" class="muted">Bars = histogram of |realized| %. Orange dashed = implied move.
                If most of the mass sits to the RIGHT of the dashed line, realized has
                historically beaten implied — the straddle was underpriced.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.iv_backtest.h2.per_event_pandl_long_straddle_1_of_premium">Per-event P&amp;L (long straddle, $1 of premium)</h2>
            <div id="ib-pnl-chart" style="height:220px"></div>
            <p data-i18n="view.iv_backtest.hint.each_dot_one_historical_event_s_long_straddle_pand" class="muted">Each dot = one historical event's long-straddle P&amp;L per $1 of premium.
                Positive = realized beat implied. Negative = implied was rich.</p>
        </div>

        <div id="ib-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('ib-demo').addEventListener('click', () => {
        const { implied_move_pct, realized_pcts } = makeDemoData();
        document.getElementById('ib-imp').value = implied_move_pct;
        document.getElementById('ib-real').value = realized_pcts.join('\n');
    });
    document.getElementById('ib-clear').addEventListener('click', () => {
        document.getElementById('ib-real').value = '';
    });
    document.getElementById('ib-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.implied = Number(document.getElementById('ib-imp').value);
    state.realizedText = document.getElementById('ib-real').value;
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('ib-errors');
    errs.style.display = 'none';
    const { value: realized, errors } = parseRealized(state.realizedText);
    if (errors.length) {
        const head = errors.slice(0, 6).map(e =>
            `line ${e.line_no}: ${esc(e.message)} — ${esc(e.raw.slice(0, 80))}`).join('<br>');
        const more = errors.length > 6 ? `<br>… and ${errors.length - 6} more.` : '';
        errs.innerHTML = `<strong>${errors.length} parse error(s):</strong><br>${head}${more}`;
        errs.style.display = 'block';
    }
    const err = validateInputs(state.implied, realized);
    if (err) { showErr(err); return; }
    let res;
    try {
        res = await api.optCalcIvBacktest(buildBody(state.implied, realized));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e })); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(res);
    renderHistogram(realized, res.implied_move_pct);
    renderPnlSeries(realized, res.implied_move_pct);
}

function renderSummary(r) {
    const badge = recommendationBadge(r.recommendation, r.edge_pct);
    document.getElementById('ib-summary').innerHTML = [
        card(t('view.iv_backtest.card.samples'),          String(r.samples)),
        card(t('view.iv_backtest.card.implied'),          fmtPct(r.implied_move_pct)),
        card(t('view.iv_backtest.card.median_realized'),  fmtPct(r.median_realized_pct), r.median_realized_pct > r.implied_move_pct ? 'pos' : 'neg'),
        card(t('view.iv_backtest.card.avg_realized'),     fmtPct(r.avg_realized_pct)),
        card(t('view.iv_backtest.card.long_pnl_per_1'),  fmtPnl(r.long_avg_pnl), r.long_avg_pnl >= 0 ? 'pos' : 'neg'),
        card(t('view.iv_backtest.card.long_win_rate'),    fmtWinRate(r.long_win_rate)),
        card(t('view.iv_backtest.card.short_pnl_per_1'), fmtPnl(r.short_avg_pnl), r.short_avg_pnl >= 0 ? 'pos' : 'neg'),
        card(t('view.iv_backtest.card.recommendation'),   badge.label, badge.cls),
        card(t('view.iv_backtest.card.action'),           badge.hint),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderHistogram(realized, impliedPct) {
    if (!window.uPlot) return;
    const { centers, counts } = histogram(realized, 20);
    const xs = centers;
    const ys = counts;
    const impliedYs = xs.map(() => Math.max(...counts) * 1.1);  // tall reference bar at implied
    const el = document.getElementById('ib-hist-chart');
    // Find the bin nearest the implied — we'll emit a single-point series
    // anchored to it for the dashed reference line.
    const impliedX = xs.map(c => c);  // identity; we'll overlay a separate vertical via second series
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 280,
        scales: { x: {}, y: {} },
        series: [
            { label: '|realized| %' },
            { label: t('chart.series.count'), stroke: '#00e5ff', width: 1.5,
              fill: '#00e5ff33', points: { show: true, size: 4 } },
            { label: 'implied', stroke: '#ff9f1a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 40 }],
        legend: { show: true },
    }, [xs, ys, xs.map(c => Math.abs(c - impliedPct) < (xs[1] - xs[0]) / 2 ? Math.max(...counts) : null)], el);
    void impliedX; void impliedYs;
}

function renderPnlSeries(realized, impliedPct) {
    if (!window.uPlot) return;
    const xs = realized.map((_, i) => i + 1);
    const abs = realized.map(v => Math.abs(v));
    const pnl = abs.map(r => impliedPct > 0 ? r / impliedPct - 1 : 0);
    const zeroYs = xs.map(() => 0);
    const el = document.getElementById('ib-pnl-chart');
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: {} },
        series: [
            { label: 'event #' },
            { label: 'long P&L / $1', stroke: '#39ff14', width: 1.2,
              fill: '#39ff141A', points: { show: true, size: 5 } },
            { label: 'breakeven', stroke: '#aab', width: 1.0, dash: [2, 4],
              points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: true },
    }, [xs, pnl, zeroYs], el);
}

function showErr(msg) {
    const el = document.getElementById('ib-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ib-err').style.display = 'none'; }
