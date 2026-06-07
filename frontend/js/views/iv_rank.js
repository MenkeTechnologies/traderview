// IV Rank view — the "is option premium cheap or expensive?" gauge.
//
// Two numbers a trader checks before opening any option position:
//   IV Rank      — current IV linearly placed in 52w low-high range.
//   IV Percentile — fraction of trailing days IV ≤ current.
//
// When rank ≫ percentile the underlying IV series is skewed (one
// earnings spike), and the percentile is the honest read.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseHistory, validateInputs, buildBody,
    rankEnvironment, rankVsPercentileNote,
    makeDemoHistory, fmtIv, fmtRank,
} from '../_iv_rank_inputs.js';

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
let state = { currentIv: 0.30, historyText: '' };

export async function renderIvRank(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.iv_rank.h1.iv_rank" class="view-title">// IV RANK</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.iv_rank.h2.current_implied_vol">Current implied vol</h2>
            <div class="inline-form">
                <label><span data-i18n="view.iv_rank.label.current_iv">Current IV (decimal; 0.25 = 25%)</span>
                    <input id="iv-cur" type="number" step="0.01" min="0" value="${state.currentIv}" data-tip="view.iv_rank.tip.current"></label>
                <button data-i18n="view.iv_rank.btn.compute" data-tip="view.iv_rank.tip.compute" data-shortcut="iv_rank_compute" id="iv-run" class="primary" type="button">Compute</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.iv_rank.h2.iv_history_trailing_252_trading_days">IV history (trailing ~252 trading days)</h2>
            <p data-i18n="view.iv_rank.hint.paste_one_iv_value_per_line_decimal_whitespace_com" class="muted">Paste one IV value per line (decimal). Whitespace, commas
                and # comments are tolerated.</p>
            <textarea id="iv-hist" rows="6" placeholder="0.22&#10;0.24&#10;0.21&#10;# comment&#10;0.26" data-tip="view.iv_rank.tip.history"></textarea>
            <div class="inline-form">
                <button data-i18n="view.iv_rank.btn.load_demo_252_days_w_earnings_spike" data-tip="view.iv_rank.tip.demo" data-shortcut="iv_rank_demo" id="iv-demo" class="secondary" type="button">Load demo (252 days w/ earnings spike)</button>
                <button data-i18n="view.iv_rank.btn.clear" data-tip="view.iv_rank.tip.clear" id="iv-clear" class="secondary" type="button">Clear</button>
            </div>
        </div>

        <div id="iv-errors" class="boot" style="display:none"></div>
        <div id="iv-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.iv_rank.h2.iv_history_bands">IV history + bands</h2>
            <div id="iv-chart" style="height:280px"></div>
            <p data-i18n="view.iv_rank.hint.cyan_iv_series_orange_dashed_current_iv_green_52w_" class="muted">Cyan = IV series. Orange dashed = current IV. Green = 52w low.
                Red = 52w high. Magenta = IV percentile reference (current IV).</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.iv_rank.h2.rank_vs_percentile_gauge">Rank vs Percentile gauge</h2>
            <div id="iv-gauges"></div>
            <p class="muted" id="iv-gauge-note">—</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.iv_rank.h2.vol_of_vol_chart">Rolling 20-day IV stddev (vol-of-vol)</h2>
            <div id="iv-vov-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.iv_rank.hint.vol_of_vol" class="muted">Rolling 20-day stddev of the IV series. High = IV itself is moving fast (regime shift; earnings approach; vol crush risk). Low = IV is steady and rangebound. Yellow dashed = median across the window.</p>
        </div>

        <div id="iv-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('iv-demo').addEventListener('click', () => {
        const hist = makeDemoHistory(42);
        document.getElementById('iv-hist').value = hist.join('\n');
    });
    document.getElementById('iv-clear').addEventListener('click', () => {
        document.getElementById('iv-hist').value = '';
    });
    document.getElementById('iv-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.currentIv = Number(document.getElementById('iv-cur').value);
    state.historyText = document.getElementById('iv-hist').value;
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('iv-errors');
    errs.style.display = 'none';
    const { value: history, errors } = parseHistory(state.historyText);
    if (errors.length) {
        const head = errors.slice(0, 6).map(e =>
            t('common.parse_error_inline', { line: e.line_no, msg: e.message, raw: e.raw.slice(0, 80) })).join('<br>');
        const more = errors.length > 6 ? `<br>… and ${errors.length - 6} more.` : '';
        errs.innerHTML = `<strong>${esc(t('common.parse_errors_lead', { n: errors.length }))}</strong><br>${head}${more}`;
        errs.style.display = 'block';
    }
    const err = validateInputs(state.currentIv, history);
    if (err) { showErr(err); showToast(err, { level: 'warning' }); return; }
    let res;
    try {
        res = await api.optCalcIvRank(buildBody(state.currentIv, history));
    } catch (e) {
        const m = t("common.error.api", { msg: e.message || e });
        showErr(m); showToast(m, { level: 'error' }); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(res);
    renderChart(history, res);
    renderVolOfVolChart(history);
    renderGauges(res);
    showToast(t('view.iv_rank.toast.done', {
        rank: fmtRank(res.iv_rank),
        pct: fmtRank(res.iv_percentile),
    }), { level: 'success' });
}

function renderSummary(r) {
    const env = rankEnvironment(r.iv_rank);
    const note = rankVsPercentileNote(r.iv_rank, r.iv_percentile);
    document.getElementById('iv-summary').innerHTML = [
        card(t('view.iv_rank.card.current_iv'),  fmtIv(r.current_iv)),
        card(t('view.iv_rank.card.52w_low'),     fmtIv(r.low_52w)),
        card(t('view.iv_rank.card.52w_high'),    fmtIv(r.high_52w)),
        card(t('view.iv_rank.card.iv_rank'),     fmtRank(r.iv_rank), env.cls),
        card(t('view.iv_rank.card.iv_ile'),     fmtRank(r.iv_percentile)),
        card(t('view.iv_rank.card.environment'), env.label, env.cls),
        card(t('view.iv_rank.card.action_hint'), env.hint),
        card(t('view.iv_rank.card.observations'), String(r.observations)),
    ].join('');
    document.getElementById('iv-gauge-note').textContent = note;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderChart(history, r) {
    if (!window.uPlot) return;
    const el = document.getElementById('iv-chart');
    const xs = history.map((_, i) => i);
    const lowYs  = xs.map(() => r.low_52w);
    const highYs = xs.map(() => r.high_52w);
    const curYs  = xs.map(() => r.current_iv);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 280,
        scales: { x: { time: false,}, y: {} },
        series: [
            { label: t('chart.series.day_') },
            { label: t('chart.series.iv'),       stroke: '#00e5ff', width: 1.2,
              fill: '#00e5ff14', points: { show: false } },
            { label: t('chart.series.current'),  stroke: '#ff9f1a', width: 1.0,
              dash: [4, 4], points: { show: false } },
            { label: t('chart.series.52w_low'),  stroke: '#39ff14', width: 1.0,
              dash: [2, 4], points: { show: false } },
            { label: t('chart.series.52w_high'), stroke: '#ff3860', width: 1.0,
              dash: [2, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: true },
    }, [xs, history, curYs, lowYs, highYs], el);
}

function renderVolOfVolChart(history) {
    const el = document.getElementById('iv-vov-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const win = 20;
    if (!Array.isArray(history) || history.length < win + 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.iv_rank.empty_vov_chart">${esc(t('view.iv_rank.empty_vov_chart'))}</div>`;
        return;
    }
    const sds = [];
    for (let i = win - 1; i < history.length; i++) {
        let mean = 0;
        for (let k = i - win + 1; k <= i; k++) mean += history[k];
        mean /= win;
        let v = 0;
        for (let k = i - win + 1; k <= i; k++) v += (history[k] - mean) ** 2;
        sds.push(Math.sqrt(v / win));
    }
    const sorted = sds.slice().sort((a, b) => a - b);
    const median = sorted.length % 2 === 1
        ? sorted[(sorted.length - 1) >> 1]
        : (sorted[sorted.length / 2 - 1] + sorted[sorted.length / 2]) / 2;
    const xs = sds.map((_, i) => i + win);
    const med = xs.map(() => median);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('chart.series.day_') },
            { label: t('view.iv_rank.chart.stddev'),
              stroke: '#b86bff', width: 1.5, points: { show: false } },
            { label: t('view.iv_rank.chart.median'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: true },
    }, [xs, sds, med], el);
}

function renderGauges(r) {
    const wrap = document.getElementById('iv-gauges');
    wrap.innerHTML = `
        <div class="is-bar-row">
            <div class="is-bar-label" data-i18n="view.iv_rank.bar.iv_rank">IV Rank</div>
            <div class="is-bar-track">
                <div class="is-bar-midline iv-q1"></div>
                <div class="is-bar-midline iv-q3"></div>
                <div class="iv-gauge-fill ${gaugeColorClass(r.iv_rank)}"
                     data-bar-pct="${r.iv_rank.toFixed(2)}"></div>
            </div>
            <div class="is-bar-value">${esc(fmtRank(r.iv_rank))}</div>
        </div>
        <div class="is-bar-row">
            <div class="is-bar-label" data-i18n="view.iv_rank.bar.iv_percentile">IV Percentile</div>
            <div class="is-bar-track">
                <div class="is-bar-midline iv-q1"></div>
                <div class="is-bar-midline iv-q3"></div>
                <div class="iv-gauge-fill ${gaugeColorClass(r.iv_percentile)}"
                     data-bar-pct="${r.iv_percentile.toFixed(2)}"></div>
            </div>
            <div class="is-bar-value">${esc(fmtRank(r.iv_percentile))}</div>
        </div>
    `;
    requestAnimationFrame(() => {
        wrap.querySelectorAll('.iv-gauge-fill').forEach(el => {
            const pct = Number(el.dataset.barPct);
            if (Number.isFinite(pct)) el.style.width = pct + '%';
        });
    });
}

function gaugeColorClass(v) {
    if (!Number.isFinite(v)) return 'iv-gauge-mid';
    if (v < 25) return 'iv-gauge-low';
    if (v > 75) return 'iv-gauge-high';
    return 'iv-gauge-mid';
}

function showErr(msg) {
    const el = document.getElementById('iv-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('iv-err').style.display = 'none'; }
