// Per-Symbol Slippage view — TCA roll-up across many trades.
//
// Companion to VWAP Slippage: that view drills into one trade vs VWAP;
// this view scans across all symbols to surface the *systematic* problem
// instruments where execution quality is consistently poor.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseRecordBlob, validateInputs, buildBody,
    executionGrade, worstSymbol, bestSymbol,
    makeDemoRecords, fmtBps, fmtPct, fmtN,
} from '../_per_symbol_slippage_inputs.js';

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
let state = { recordsText: '' };

export async function renderPerSymbolSlippage(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.per_symbol_slippage.h1.per_symbol_slippage" class="view-title">// PER-SYMBOL SLIPPAGE</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.per_symbol_slippage.h2.slippage_records">Slippage records</h2>
            <p class="muted" data-i18n-html="view.per_symbol_slippage.help">Paste <code>symbol slippage_bps</code> per line. Signed bps —
                positive = trader-favorable (beat benchmark). Demo loads 108 fills
                across 6 symbols spanning the full execution-quality spectrum from
                ETF to micro-cap.</p>
            <textarea id="ps-recs" rows="6" placeholder="AAPL -2.5&#10;SPY 7.0&#10;ILQD -28.0&#10;..." data-tip="view.per_symbol_slippage.tip.records"></textarea>
            <div class="inline-form">
                <button data-i18n="view.per_symbol_slippage.btn.load_demo_108_fills_6_symbols" data-tip="view.per_symbol_slippage.tip.demo" data-shortcut="per_symbol_slippage_demo" id="ps-demo" class="secondary" type="button">Load demo (108 fills, 6 symbols)</button>
                <button data-i18n="view.per_symbol_slippage.btn.clear" data-tip="view.per_symbol_slippage.tip.clear" id="ps-clear" class="secondary" type="button">Clear</button>
                <button data-i18n="view.per_symbol_slippage.btn.aggregate" data-tip="view.per_symbol_slippage.tip.aggregate" data-shortcut="per_symbol_slippage_run" id="ps-run" class="primary" type="button">Aggregate</button>
            </div>
        </div>

        <div id="ps-errors" class="boot" style="display:none"></div>
        <div id="ps-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.per_symbol_slippage.h2.per_symbol_breakdown">Per-symbol breakdown</h2>
            <div id="ps-table"></div>
            <p data-i18n="view.per_symbol_slippage.hint.sorted_worst_mean_first_poor_terrible_rows_are_whe" class="muted">Sorted worst-mean-first. POOR / TERRIBLE rows are where
                your sizing is bigger than the book can absorb — shrink position there
                or stop trading the name.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.per_symbol_slippage.h2.mean_chart">Mean slippage (bps) per symbol</h2>
            <div id="ps-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.per_symbol_slippage.h2.stdev_chart">Slippage stdev (consistency) per symbol</h2>
            <div id="ps-stdev-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.per_symbol_slippage.hint.stdev" class="muted small">Per-symbol stdev of slippage bps. Reveals which names have wildly variable execution — a 0-mean / 30-stdev name is just as bad as a -10-mean / 5-stdev one. Stable execution names sit at the bottom.</p>
        </div>

        <div id="ps-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('ps-demo').addEventListener('click', () => {
        const recs = makeDemoRecords(42);
        document.getElementById('ps-recs').value =
            recs.map(r => `${r.symbol} ${r.slippage_bps}`).join('\n');
    });
    document.getElementById('ps-clear').addEventListener('click', () => {
        document.getElementById('ps-recs').value = '';
    });
    document.getElementById('ps-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.recordsText = document.getElementById('ps-recs').value;
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('ps-errors');
    errs.style.display = 'none';
    const { records, errors } = parseRecordBlob(state.recordsText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            t('common.parse_error_inline', { line: e.line_no, msg: e.message, raw: e.raw.slice(0, 80) })).join('<br>');
        const more = errors.length > 8 ? `<br>${esc(t('common.and_n_more', { n: errors.length - 8 }))}` : '';
        errs.innerHTML = `<strong>${esc(t('common.parse_errors_lead', { n: errors.length }))}</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (records.length === 0) return;
    }
    const err = validateInputs(records);
    if (err) { showErr(err); showToast(err, { level: 'warning' }); return; }
    let report;
    try {
        report = await api.microPerSymbolSlippage(buildBody(records));
    } catch (e) {
        const m = t("common.error.api", { msg: e.message || e });
        showErr(m); showToast(m, { level: 'error' }); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(report, records);
    renderTable(report);
    renderMeanChart(report);
    renderStdevChart(report);
    const worst = worstSymbol(report);
    showToast(t('view.per_symbol_slippage.toast.done', {
        symbols: (report || []).length,
        worst: worst ? worst.symbol : '—',
    }), { level: 'success' });
}

function renderMeanChart(report) {
    const el = document.getElementById('ps-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const valid = (report || []).filter(r => Number.isFinite(Number(r.mean_bps)));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.per_symbol_slippage.empty_chart">${esc(t('view.per_symbol_slippage.empty_chart'))}</div>`;
        return;
    }
    valid.sort((a, b) => Number(a.mean_bps) - Number(b.mean_bps));
    const labels = valid.map(r => r.symbol);
    const ys = valid.map(r => Number(r.mean_bps));
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.per_symbol_slippage.chart.symbol_idx') },
            { label: t('view.per_symbol_slippage.chart.mean_bps'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.per_symbol_slippage.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ys, zero], el);
}

function renderStdevChart(report) {
    const el = document.getElementById('ps-stdev-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const valid = (report || []).filter(r => Number.isFinite(Number(r.stdev_bps)));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.per_symbol_slippage.empty_stdev_chart">${esc(t('view.per_symbol_slippage.empty_stdev_chart'))}</div>`;
        return;
    }
    valid.sort((a, b) => Number(b.stdev_bps) - Number(a.stdev_bps));
    const labels = valid.map(r => r.symbol);
    const ys = valid.map(r => Number(r.stdev_bps));
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.per_symbol_slippage.chart.symbol_idx') },
            { label: t('view.per_symbol_slippage.chart.stdev_bps'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 12, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderSummary(report, records) {
    const worst = worstSymbol(report);
    const best  = bestSymbol(report);
    const allBps = records.map(r => r.slippage_bps).filter(Number.isFinite);
    const aggMean = allBps.length ? allBps.reduce((a, b) => a + b, 0) / allBps.length : NaN;
    const beatRate = allBps.length ? allBps.filter(v => v > 0).length / allBps.length : NaN;
    document.getElementById('ps-summary').innerHTML = [
        card(t('view.per_symbol_slippage.card.symbols'),         String(report.length)),
        card(t('view.per_symbol_slippage.card.total_fills'),     String(records.length)),
        card(t('view.per_symbol_slippage.card.mean_slip'),       fmtBps(aggMean), aggMean > 0 ? 'pos' : aggMean < 0 ? 'neg' : ''),
        card(t('view.per_symbol_slippage.card.beat_rate'),       fmtPct(beatRate), beatRate >= 0.5 ? 'pos' : 'neg'),
        card(t('view.per_symbol_slippage.card.worst_symbol'),    worst ? `${worst.symbol} ${fmtBps(worst.mean_bps)}` : '—', 'neg'),
        card(t('view.per_symbol_slippage.card.best_symbol'),     best ? `${best.symbol} ${fmtBps(best.mean_bps)}`   : '—', 'pos'),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderTable(report) {
    const wrap = document.getElementById('ps-table');
    if (!report.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.per_symbol_slippage.empty.symbols">No symbols.</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.per_symbol_slippage.th.symbol">Symbol</th><th data-i18n="view.per_symbol_slippage.th.grade">Grade</th><th data-i18n="view.per_symbol_slippage.th.trades">Trades</th>
                <th data-i18n="view.per_symbol_slippage.th.mean">Mean</th><th data-i18n="view.per_symbol_slippage.th.median">Median</th><th data-i18n="view.per_symbol_slippage.th.stdev">Stdev</th>
                <th data-i18n="view.per_symbol_slippage.th.worst">Worst</th><th data-i18n="view.per_symbol_slippage.th.best">Best</th><th data-i18n="view.per_symbol_slippage.th.beat_rate">Beat rate</th>
            </tr></thead>
            <tbody>
                ${report.map(r => {
                    const grade = executionGrade(r.mean_bps);
                    return `<tr>
                        <td>${esc(r.symbol)}</td>
                        <td class="${grade.cls}">${esc(grade.label)}</td>
                        <td>${esc(fmtN(r.trade_count))}</td>
                        <td class="${r.mean_bps >= 0 ? 'pos' : 'neg'}">${esc(fmtBps(r.mean_bps))}</td>
                        <td class="${r.median_bps >= 0 ? 'pos' : 'neg'}">${esc(fmtBps(r.median_bps))}</td>
                        <td>${esc(fmtBps(r.stdev_bps))}</td>
                        <td class="neg">${esc(fmtBps(r.worst_bps))}</td>
                        <td class="pos">${esc(fmtBps(r.best_bps))}</td>
                        <td>${esc(fmtPct(r.beat_rate))}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
}

function showErr(msg) {
    const el = document.getElementById('ps-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ps-err').style.display = 'none'; }
