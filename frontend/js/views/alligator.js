// Williams Alligator view — Bill Williams chaos-theory trend indicator.
//
// Three smoothed MAs (jaw 13 / teeth 8 / lips 5) of the median price,
// each shifted forward (jaw +8, teeth +5, lips +3 bars). When they
// intertwine → "alligator sleeping" (no trade). When they fan out
// (lips above teeth above jaw, or inverse) → "alligator hunting" (trade
// with the trend).

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseBarBlob, validateInputs, buildBody,
    shiftLines, classifyPoint, biasBadge, biasCounts,
    makeDemoBars, medianPrices, fmtN, fmtPct,
} from '../_alligator_inputs.js';

import { t } from '../i18n.js';
let state = { barText: '' };

export async function renderAlligator(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.alligator.h1.williams_alligator" class="view-title">// WILLIAMS ALLIGATOR</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.alligator.h2.hl_bars">HL bars</h2>
            <p class="muted" data-i18n-html="view.alligator.help">Paste <code>high low</code> per line. Median price
                ((H+L)/2) drives the three SMMAs. Demo loads 50 bars cycling through
                sleep → uptrend → sleep → downtrend so all three biases are visible.</p>
            <textarea id="al-bars" rows="6" placeholder="100.5 99.5&#10;100.8 99.8&#10;..."></textarea>
            <div class="inline-form">
                <button data-i18n="view.alligator.btn.load_demo_50_bars_4_phases" id="al-demo" class="secondary" type="button">Load demo (50 bars, 4 phases)</button>
                <button data-i18n="view.alligator.btn.clear" id="al-clear" class="secondary" type="button">Clear</button>
                <button data-i18n="view.alligator.btn.compute" id="al-run" class="primary" type="button">Compute</button>
            </div>
        </div>

        <div id="al-errors" class="boot" style="display:none"></div>
        <div id="al-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.alligator.h2.median_price_alligator_jaw_teeth_lips">Median price + Alligator (jaw / teeth / lips)</h2>
            <div id="al-chart" style="height:320px"></div>
            <p data-i18n="view.alligator.hint.cyan_median_price_blue_jaw_13_smma_8_shift_red_tee" class="muted">Cyan = median price. Blue = jaw (13-SMMA, +8 shift).
                Red = teeth (8-SMMA, +5 shift). Green = lips (5-SMMA, +3 shift).
                Lines fan out (green above red above blue) = hunting up. Reverse stack =
                hunting down. Intertwined = sleeping.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.alligator.h2.bias_history_per_bar">Bias history per bar</h2>
            <div id="al-bias-strip"></div>
            <p data-i18n="view.alligator.hint.one_cell_per_bar_green_up_red_down_grey_sleeping_v" class="muted">One cell per bar — green up, red down, grey sleeping.
                Visualizes regime transitions across the full series at a glance.</p>
        </div>

        <div id="al-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('al-demo').addEventListener('click', () => {
        const b = makeDemoBars();
        document.getElementById('al-bars').value =
            b.map(x => `${x.high} ${x.low}`).join('\n');
    });
    document.getElementById('al-clear').addEventListener('click', () => {
        document.getElementById('al-bars').value = '';
    });
    document.getElementById('al-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.barText = document.getElementById('al-bars').value;
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('al-errors');
    errs.style.display = 'none';
    const { bars, errors } = parseBarBlob(state.barText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            t('common.parse_error_inline', { line: e.line_no, msg: e.message, raw: e.raw.slice(0, 80) })).join('<br>');
        const more = errors.length > 8 ? `<br>${esc(t('common.and_n_more', { n: errors.length - 8 }))}` : '';
        errs.innerHTML = `<strong>${esc(t('common.parse_errors_lead', { n: errors.length }))}</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (bars.length === 0) return;
    }
    const err = validateInputs(bars);
    if (err) { showErr(err); return; }
    let points;
    try {
        points = await api.barsAlligator(buildBody(bars));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e })); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(points, bars);
    renderChart(bars, points);
    renderBiasStrip(points);
}

function renderSummary(points, bars) {
    const counts = biasCounts(points);
    const validCount = (counts.up + counts.down + counts.sleeping);
    const sleepingPct = validCount > 0 ? counts.sleeping / validCount : 0;
    const lastPoint = (points || []).filter(p => p && (p.jaw || p.teeth || p.lips)).pop();
    const currentBias = lastPoint ? classifyPoint(lastPoint) : null;
    const badge = currentBias ? biasBadge(currentBias) : { label: '—', cls: '', hint: '' };
    document.getElementById('al-summary').innerHTML = [
        card(t('view.alligator.card.bars'),            String(bars.length)),
        card(t('view.alligator.card.points_computed'), String((points || []).length)),
        card(t('view.alligator.card.up_bars'),         String(counts.up),       counts.up ? 'pos' : ''),
        card(t('view.alligator.card.down_bars'),       String(counts.down),     counts.down ? 'neg' : ''),
        card(t('view.alligator.card.sleeping_bars'),   String(counts.sleeping)),
        card(t('view.alligator.card.sleeping'),      fmtPct(sleepingPct),     sleepingPct > 0.5 ? 'neg' : 'pos'),
        card(t('view.alligator.card.current_bias'),    badge.label, badge.cls),
        card(t('view.alligator.card.action'),          badge.hint),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderChart(bars, points) {
    if (!window.uPlot) return;
    const el = document.getElementById('al-chart');
    const xs = bars.map((_, i) => i);
    const median = medianPrices(bars);
    const totalBars = bars.length;
    // SHIFTS pushes each SMMA series forward — destination cells past
    // the chart end stay null since uPlot draws gaps for null.
    const { jaw, teeth, lips } = shiftLines(points, totalBars);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 320,
        scales: { x: {}, y: {} },
        series: [
            { label: t('chart.series.bar_num') },
            { label: t('chart.series.median'), stroke: '#00e5ff', width: 0.8,
              fill: '#00e5ff10', points: { show: false } },
            { label: t('chart.series.jaw_13'),   stroke: '#3b82f6', width: 1.5, points: { show: false } },
            { label: t('chart.series.teeth_8'),  stroke: '#ff3860', width: 1.5, points: { show: false } },
            { label: t('chart.series.lips_5'),   stroke: '#39ff14', width: 1.5, points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: true },
    }, [xs, median, jaw, teeth, lips], el);
}

function renderBiasStrip(points) {
    const wrap = document.getElementById('al-bias-strip');
    if (!Array.isArray(points) || points.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.alligator.empty.points">No points.</div>`;
        return;
    }
    // Color-coded cell per bar — green/red/grey by bias.
    wrap.innerHTML = `<div class="al-strip">${points.map((p, i) => {
        const bias = classifyPoint(p);
        const detail = p ? t('view.alligator.tip.bar_detail', { i, label: biasBadge(bias).label, jaw: fmtN(p.jaw), teeth: fmtN(p.teeth), lips: fmtN(p.lips) }) : t('view.alligator.tip.bar_empty', { i, label: biasBadge(bias).label });
        return `<div class="al-strip-cell al-bias-${bias}" title="${detail}"></div>`;
    }).join('')}</div>`;
}

function showErr(msg) {
    const el = document.getElementById('al-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('al-err').style.display = 'none'; }
