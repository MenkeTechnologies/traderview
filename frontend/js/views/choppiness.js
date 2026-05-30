// Choppiness Index view — E.W. Dreiss's trend-vs-consolidation oscillator.
// Range 0–100. > 61.8 = choppy, < 38.2 = trending.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseBarBlob, validateInputs, buildBody, localCompute,
    regimeBadge, regimeBuckets, lastRegimeSwitch, makeDemoBars,
    fmtN, fmtPct,
} from '../_choppiness_inputs.js';

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
let state = {
    bars: makeDemoBars('trend-then-chop'),
    period: 14,
};

export async function renderChoppiness(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.choppiness.h1.choppiness_index" class="view-title">// CHOPPINESS INDEX</h1>

        <div class="chart-panel">
            <h2><span data-i18n="view.choppiness.h2.paste_bars">Paste OHLC bars (per line:</span> <code>high low close</code>)</h2>
            <textarea id="cp-blob" rows="6" placeholder="100.5 99.5 100.0&#10;100.6 99.4 100.1&#10;..." data-tip="view.choppiness.tip.blob">${esc(barsToBlob(state.bars))}</textarea>
            <div class="inline-form">
                <label><span data-i18n="view.choppiness.label.period">Lookback period</span>
                    <input id="cp-per" type="number" step="1" min="2" max="200" value="${state.period}" data-tip="view.choppiness.tip.period"></label>
                <button data-i18n="view.choppiness.btn.compute" id="cp-run" class="primary" type="button" data-tip="view.choppiness.tip.run" data-shortcut="choppiness_run">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.choppiness.btn.demo_trending_up" id="cp-demo-trend-up"   class="secondary" type="button" data-tip="view.choppiness.tip.demo_trend_up">Demo: trending up</button>
                <button data-i18n="view.choppiness.btn.demo_trending_down" id="cp-demo-trend-dn"   class="secondary" type="button" data-tip="view.choppiness.tip.demo_trend_dn">Demo: trending down</button>
                <button data-i18n="view.choppiness.btn.demo_choppy" id="cp-demo-choppy"     class="secondary" type="button" data-tip="view.choppiness.tip.demo_choppy">Demo: choppy</button>
                <button data-i18n="view.choppiness.btn.demo_mixed_drift" id="cp-demo-mixed"      class="secondary" type="button" data-tip="view.choppiness.tip.demo_mixed">Demo: mixed drift</button>
                <button data-i18n="view.choppiness.btn.demo_trend_chop_switch" id="cp-demo-switch"     class="secondary" type="button" data-tip="view.choppiness.tip.demo_switch">Demo: trend → chop switch</button>
            </div>
            <p data-i18n="view.choppiness.hint.formula_ci_100_log10_tr_max_h_min_l_log10_period_d" class="muted">Formula: CI = 100 × log10(ΣTR / (max H − min L)) / log10(period). Default period 14. Reference bands: 61.8 (choppy line), 38.2 (trending line).</p>
        </div>

        <div id="cp-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.choppiness.h2.close_choppiness_index">Close + Choppiness Index</h2>
            <div id="cp-chart" style="height:380px"></div>
            <p data-i18n="view.choppiness.hint.cyan_close_left_axis_yellow_ci_right_axis_0_100_re" class="muted">Cyan = close (left axis). Yellow = CI (right axis 0–100). Red dashed = 61.8 (chop), green dashed = 38.2 (trend).</p>
        </div>

        <div id="cp-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (kind) => {
        state.bars = makeDemoBars(kind);
        document.getElementById('cp-blob').value = barsToBlob(state.bars);
    };
    document.getElementById('cp-demo-trend-up').addEventListener('click', () => loadDemo('trending-up'));
    document.getElementById('cp-demo-trend-dn').addEventListener('click', () => loadDemo('trending-down'));
    document.getElementById('cp-demo-choppy').addEventListener('click',   () => loadDemo('choppy'));
    document.getElementById('cp-demo-mixed').addEventListener('click',    () => loadDemo('mixed'));
    document.getElementById('cp-demo-switch').addEventListener('click',   () => loadDemo('trend-then-chop'));
    document.getElementById('cp-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function barsToBlob(bars) {
    return bars.map(b => `${b.high} ${b.low} ${b.close}`).join('\n');
}

function readInputs() {
    const parsed = parseBarBlob(document.getElementById('cp-blob').value);
    if (parsed.errors.length) {
        showErr(t("common.error.parse_errors", { summary: parsed.errors.slice(0, 3).map(e => `[] `).join("; ") }));
        showToast(t('view.choppiness.toast.parse_error', { n: parsed.errors.length }), { level: 'warning' });
        return;
    }
    hideErr();
    state.bars   = parsed.bars;
    state.period = parseInt(document.getElementById('cp-per').value, 10);
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.bars, state.period);
    if (err) { showErr(err); showToast(t('view.choppiness.toast.invalid'), { level: 'warning' }); return; }
    const local = localCompute(state.bars, state.period);
    renderSummary(local, true);
    renderChart(state.bars, local);
    let resp;
    try {
        resp = await api.anlyChoppiness(buildBody(state.bars, state.period));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        showToast(t('view.choppiness.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(state.bars, resp);
    const regime = String(resp.regime || '');
    const latest = resp.latest == null ? '—' : Number(resp.latest).toFixed(1);
    const level = regime === 'trending' ? 'success' : regime === 'choppy' ? 'warning' : 'info';
    showToast(t('view.choppiness.toast.computed', { regime, ci: latest }), { level });
}

function renderSummary(report, pending) {
    const local = localCompute(state.bars, state.period);
    const parity = reportEq(report, local);
    const badge = regimeBadge(report.regime);
    const buckets = regimeBuckets(report.series);
    const totalEvaluated = buckets.trending + buckets.mixed + buckets.choppy;
    const switchEvt = lastRegimeSwitch(report.series);
    const labelKey = `view.choppiness.regime.${report.regime}.label`;
    const hintKey  = `view.choppiness.regime.${report.regime}.hint`;
    const labelTr  = (() => { const v = t(labelKey); return (v && v !== labelKey) ? v : badge.label; })();
    const hintTr   = (() => { const v = t(hintKey);  return (v && v !== hintKey)  ? v : badge.hint;  })();
    document.getElementById('cp-summary').innerHTML = [
        card(t('view.choppiness.card.regime'),         labelTr + (pending ? t('common.suffix.local') : ''), badge.cls),
        card(t('view.choppiness.card.action'),         hintTr),
        card(t('view.choppiness.card.latest_ci'),      report.latest == null ? '—' : fmtN(report.latest, 2),
            badge.cls),
        card(t('view.choppiness.card.note'),           report.note),
        card(t('view.choppiness.card.bars_trending'), totalEvaluated > 0 ? fmtPct(buckets.trending / totalEvaluated) : '—',
            buckets.trending > buckets.choppy ? 'pos' : ''),
        card(t('view.choppiness.card.bars_mixed'),    totalEvaluated > 0 ? fmtPct(buckets.mixed / totalEvaluated) : '—'),
        card(t('view.choppiness.card.bars_choppy'),   totalEvaluated > 0 ? fmtPct(buckets.choppy / totalEvaluated) : '—',
            buckets.choppy > buckets.trending ? 'neg' : ''),
        card(t('view.choppiness.card.warmup_bars'),    String(buckets.warmup)),
        card(t('view.choppiness.card.last_switch'),    switchEvt
            ? t('view.choppiness.switch.bar', { bar: switchEvt.switchedAt, from: switchEvt.fromRegime, to: switchEvt.toRegime })
            : t('view.choppiness.switch.none')),
        card(t('view.choppiness.card.local_parity'),   parity ? t('common.ok') : t('common.diverged'), parity ? 'pos' : 'neg'),
    ].join('');
}

function reportEq(a, b) {
    if (!a || !b) return false;
    if (a.regime !== b.regime) return false;
    const al = a.latest, bl = b.latest;
    if (al == null && bl == null) return true;
    if (al == null || bl == null) return false;
    return Math.abs(al - bl) < 1e-6;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderChart(bars, report) {
    if (!window.uPlot) return;
    const el = document.getElementById('cp-chart');
    if (!el) return;
    el.innerHTML = '';
    const xs = bars.map((_, i) => i);
    const closes = bars.map(b => b.close);
    const ci = report.series.slice();
    const chopBand  = new Array(bars.length).fill(61.8);
    const trendBand = new Array(bars.length).fill(38.2);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 360,
        scales: { x: {}, y: {}, y_ci: { range: [0, 100] } },
        series: [
            { label: t('chart.series.bar_num') },
            { label: t('chart.series.close'),         stroke: '#00e5ff', width: 1.5, points: { show: false } },
            { label: t('chart.series.ci'),            stroke: '#ffd84a', width: 1.5, scale: 'y_ci',
              points: { show: false } },
            { label: t('chart.series.618_choppy'), stroke: '#ff3860', width: 1.0, dash: [4, 4],
              scale: 'y_ci', points: { show: false } },
            { label: t('chart.series.382_trend'),  stroke: '#23d18b', width: 1.0, dash: [4, 4],
              scale: 'y_ci', points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60 },
            { stroke: '#ffd84a', size: 50, scale: 'y_ci', side: 1 },
        ],
        legend: { show: true },
    }, [xs, closes, ci, chopBand, trendBand], el);
}

function showErr(msg) {
    const el = document.getElementById('cp-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('cp-err').style.display = 'none'; }
