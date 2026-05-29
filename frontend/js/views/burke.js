// Burke Ratio (1994) view — risk-adjusted return penalizing only drawdown variance.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_RISK_FREE, DEFAULT_PERIODS_PER_YEAR,
    parseEquityBlob, equityToBlob, validateInputs, buildBody, localCompute,
    drawdownEpisodes, ratioBadge, ddBadge, excessBadge, summarizeEquity,
    makeDemoInput,
    fmtRatio, fmtRatioSigned, fmtPct, fmtPctSigned, fmtPrice, fmtInt,
} from '../_burke_inputs.js';

let state = { ...makeDemoInput('steady-growth') };
let chart = null;

export async function renderBurke(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.burke.h1.title" class="view-title">// BURKE RATIO</h1>

        <div class="chart-panel" data-context-scope="burke">
            <h2 data-i18n="view.burke.h2.equity">Equity curve
                <small data-i18n="view.burke.h2.equity_hint" class="muted">(positive values; ≥ 2 obs)</small></h2>
            <textarea id="bk-blob" rows="6"
                      data-tip="view.burke.tip.equity"
                      placeholder="100, 100.5, 101.2, ...">${esc(equityToBlob(state.equity))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.burke.label.risk_free">Risk-free total return</span>
                    <input id="bk-rf" type="number" step="0.001" value="${state.risk_free_total}"></label>
                <label><span data-i18n="view.burke.label.periods">Periods / year</span>
                    <input id="bk-periods" type="number" step="1" min="1" value="${state.periods_per_year}"></label>
                <button data-i18n="view.burke.btn.compute" id="bk-run" class="primary"
                        data-tip="view.burke.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.burke.btn.demo_steady"  id="bk-d1" class="secondary" type="button">Demo: steady growth</button>
                <button data-i18n="view.burke.btn.demo_high"    id="bk-d2" class="secondary" type="button">Demo: high Sharpe</button>
                <button data-i18n="view.burke.btn.demo_vol"     id="bk-d3" class="secondary" type="button">Demo: volatile uptrend</button>
                <button data-i18n="view.burke.btn.demo_deep"    id="bk-d4" class="secondary" type="button">Demo: deep drawdown</button>
                <button data-i18n="view.burke.btn.demo_multi"   id="bk-d5" class="secondary" type="button">Demo: multi-drawdowns</button>
                <button data-i18n="view.burke.btn.demo_loss"    id="bk-d6" class="secondary" type="button">Demo: losing strategy</button>
                <button data-i18n="view.burke.btn.demo_monthly" id="bk-d7" class="secondary" type="button">Demo: monthly bars</button>
                <button data-i18n="view.burke.btn.demo_open"    id="bk-d8" class="secondary" type="button">Demo: open-ended DD</button>
            </div>
            <p data-i18n="view.burke.hint.about" class="muted">Burke = (R − Rf) / √(Σ DD²). Penalizes only drawdown variance (vs Sharpe's full vol). Modified Burke annualizes by × √periods_per_year. Per-trough drawdowns measured between each pair of consecutive new highs.</p>
        </div>

        <div id="bk-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.burke.h2.chart">Equity curve</h2>
            <div id="bk-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.burke.h2.episodes">Drawdown episodes</h2>
            <div id="bk-episodes"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.burke.h2.stats">Equity summary</h2>
            <div id="bk-stats"></div>
        </div>

        <div id="bk-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('bk-blob').value    = equityToBlob(state.equity);
        document.getElementById('bk-rf').value      = state.risk_free_total;
        document.getElementById('bk-periods').value = state.periods_per_year;
    };
    document.getElementById('bk-d1').addEventListener('click', () => { loadDemo('steady-growth');    void compute(tok); });
    document.getElementById('bk-d2').addEventListener('click', () => { loadDemo('high-sharpe');      void compute(tok); });
    document.getElementById('bk-d3').addEventListener('click', () => { loadDemo('volatile-uptrend'); void compute(tok); });
    document.getElementById('bk-d4').addEventListener('click', () => { loadDemo('deep-drawdown');    void compute(tok); });
    document.getElementById('bk-d5').addEventListener('click', () => { loadDemo('multi-drawdowns');  void compute(tok); });
    document.getElementById('bk-d6').addEventListener('click', () => { loadDemo('losing-strategy'); void compute(tok); });
    document.getElementById('bk-d7').addEventListener('click', () => { loadDemo('monthly');          void compute(tok); });
    document.getElementById('bk-d8').addEventListener('click', () => { loadDemo('one-big-dd');       void compute(tok); });
    document.getElementById('bk-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseEquityBlob(document.getElementById('bk-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.burke.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.equity = p.equity;
    const rfV = parseFloat(document.getElementById('bk-rf').value);
    const ppV = parseFloat(document.getElementById('bk-periods').value);
    state.risk_free_total  = Number.isFinite(rfV) ? rfV : DEFAULT_RISK_FREE;
    state.periods_per_year = Number.isFinite(ppV) && ppV > 0 ? ppV : DEFAULT_PERIODS_PER_YEAR;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.equity, state.risk_free_total, state.periods_per_year);
    if (!local) { showErr(t('view.burke.err.degenerate')); return; }
    renderSummary(local, true);
    renderChart();
    renderEpisodes();
    renderStats();
    let resp;
    try {
        resp = await api.anlyBurkeRatio(buildBody(state));
    } catch (e) {
        showErr(`${t('view.burke.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) { showErr(t('view.burke.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart();
    renderEpisodes();
    renderStats();
}

function renderSummary(report, pending) {
    const local = localCompute(state.equity, state.risk_free_total, state.periods_per_year);
    const parityOk = !!local
        && Math.abs(local.burke_ratio - report.burke_ratio) < 1e-9
        && Math.abs(local.modified_burke_ratio - report.modified_burke_ratio) < 1e-9
        && Math.abs(local.total_return - report.total_return) < 1e-9
        && Math.abs(local.sum_squared_drawdowns - report.sum_squared_drawdowns) < 1e-12
        && local.n_drawdowns === report.n_drawdowns;
    const rBadge = ratioBadge(report.modified_burke_ratio);
    const dBadge = ddBadge(report.sum_squared_drawdowns, report.n_drawdowns);
    const eBadge = excessBadge(report.total_return, state.risk_free_total);
    const localTag = pending ? ` (${t('view.burke.tag.local')})` : '';
    document.getElementById('bk-summary').innerHTML = [
        card(t('view.burke.card.verdict'),   t(rBadge.key) + localTag, rBadge.cls),
        card(t('view.burke.card.dd_intensity'), t(dBadge.key), dBadge.cls),
        card(t('view.burke.card.excess'),    t(eBadge.key), eBadge.cls),
        card(t('view.burke.card.burke'),     fmtRatio(report.burke_ratio)),
        card(t('view.burke.card.mod_burke'), fmtRatio(report.modified_burke_ratio),
             report.modified_burke_ratio > 0 ? 'pos' : 'neg'),
        card(t('view.burke.card.total_ret'), fmtPctSigned(report.total_return),
             report.total_return > 0 ? 'pos' : 'neg'),
        card(t('view.burke.card.n_dd'),      fmtInt(report.n_drawdowns)),
        card(t('view.burke.card.sum_sq_dd'), fmtRatio(report.sum_squared_drawdowns, 6)),
        card(t('view.burke.card.periods'),   fmtInt(state.periods_per_year)),
        card(t('view.burke.card.parity'),
             parityOk ? t('view.burke.tag.ok') : t('view.burke.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart() {
    const el = document.getElementById('bk-chart');
    if (!el || !window.uPlot) return;
    const xs = state.equity.map((_, i) => i);
    const data = [xs, state.equity];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    chart = new window.uPlot({
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false } },
        series: [
            { label: 'i' },
            { label: t('view.burke.series.equity'), stroke: '#1de9b6', width: 1.5 },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    }, data, el);
}

function renderEpisodes() {
    const wrap = document.getElementById('bk-episodes');
    const eps = drawdownEpisodes(state.equity);
    if (eps.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.burke.no_dd">${esc(t('view.burke.no_dd'))}</div>`;
        return;
    }
    const rows = eps.map((e, i) => `
        <tr>
            <td>#${i + 1}</td>
            <td>${fmtInt(e.peak_idx)}</td>
            <td>${esc(fmtPrice(e.peak_value))}</td>
            <td>${fmtInt(e.trough_idx)}</td>
            <td>${esc(fmtPrice(e.trough_value))}</td>
            <td class="neg">${esc(fmtPct(e.drawdown_pct))}</td>
            <td>${e.recovery_idx == null ? esc(t('view.burke.no_recovery')) : fmtInt(e.recovery_idx)}</td>
        </tr>`).join('');
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th>
                <th data-i18n="view.burke.col.peak_idx">Peak idx</th>
                <th data-i18n="view.burke.col.peak_val">Peak value</th>
                <th data-i18n="view.burke.col.trough_idx">Trough idx</th>
                <th data-i18n="view.burke.col.trough_val">Trough value</th>
                <th data-i18n="view.burke.col.dd_pct">Drawdown %</th>
                <th data-i18n="view.burke.col.recovery">Recovery idx</th>
            </tr></thead>
            <tbody>${rows}</tbody>
        </table>
    `;
}

function renderStats() {
    const wrap = document.getElementById('bk-stats');
    if (!state.equity.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.burke.empty">${esc(t('view.burke.empty'))}</div>`;
        return;
    }
    const s = summarizeEquity(state.equity);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.burke.col.metric">Metric</th>
                <th data-i18n="view.burke.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.burke.row.count">Points</td>     <td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.burke.row.start">Start</td>      <td>${esc(fmtPrice(s.start))}</td></tr>
                <tr><td data-i18n="view.burke.row.end">End</td>          <td>${esc(fmtPrice(s.end))}</td></tr>
                <tr><td data-i18n="view.burke.row.min">Min</td>          <td>${esc(fmtPrice(s.min))}</td></tr>
                <tr><td data-i18n="view.burke.row.max">Max</td>          <td>${esc(fmtPrice(s.max))}</td></tr>
                <tr><td data-i18n="view.burke.row.peak_trough">Peak→trough</td>
                    <td class="neg">${esc(fmtPct(s.peak_to_trough))}</td></tr>
            </tbody>
        </table>
    `;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('bk-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('bk-err').style.display = 'none'; }
