// DeMarker view — Tom DeMark's bounded [0, 1] overbought/oversold oscillator.
//
// Pure momentum extreme detector. Cuts at 0.70 (overbought) and 0.30
// (oversold). Crossovers used for counter-trend setup alerts.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseBarBlob, validateInputs, buildBody,
    OB_THRESHOLD, OS_THRESHOLD, regimeOf, regimeBadge,
    regimeCounts, detectCrossings, latestValue,
    makeDemoBars, fmtN, fmtPct,
} from '../_demarker_inputs.js';

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
let state = { barText: '', period: 14 };

export async function renderDemarker(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.demarker.h1.demarker_oscillator" class="view-title">// DEMARKER OSCILLATOR</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.demarker.h2.hl_bars">HL bars</h2>
            <p class="muted" data-i18n-html="view.demarker.help">Paste <code>high low</code> per line. Demo loads 60 bars
                cycling through uptrend → chop → downtrend so OB and OS readings both fire.</p>
            <textarea id="dm-bars" rows="6" placeholder="100.5 99.5&#10;100.8 99.8&#10;..." data-tip="view.demarker.tip.bars"></textarea>
            <div class="inline-form">
                <label><span data-i18n="view.demarker.label.period">Period</span>
                    <input id="dm-period" type="number" step="1" min="2" value="${state.period}" data-tip="view.demarker.tip.period"></label>
                <button data-i18n="view.demarker.btn.load_demo_60_bars_ob_os_cycle" id="dm-demo" class="secondary" type="button" data-tip="view.demarker.tip.demo" data-shortcut="demarker_demo">Load demo (60 bars, OB+OS cycle)</button>
                <button data-i18n="view.demarker.btn.clear" id="dm-clear" class="secondary" type="button" data-tip="view.demarker.tip.clear">Clear</button>
                <button data-i18n="view.demarker.btn.compute" id="dm-run" class="primary" type="button" data-tip="view.demarker.tip.run" data-shortcut="demarker_run">Compute</button>
            </div>
            <p data-i18n="view.demarker.hint.bounded_0_1_0_70_overbought_setup_for_short_mean_r" class="muted">Bounded [0, 1]. ≥0.70 = overbought (setup for short / mean-reversion).
                ≤0.30 = oversold (setup for long / mean-reversion). Crossovers from neutral into
                an extreme region are surfaced as event alerts.</p>
        </div>

        <div id="dm-errors" class="boot" style="display:none"></div>
        <div id="dm-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>${esc(t('view.demarker.h2.series', { period: state.period }))}</h2>
            <div id="dm-chart" style="height:280px"></div>
            <p data-i18n="view.demarker.hint.cyan_demarker_red_dashed_0_70_ob_threshold_green_d" class="muted">Cyan = DeMarker. Red dashed = 0.70 OB threshold. Green dashed =
                0.30 OS threshold. Yellow = 0.50 mid.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.demarker.h2.crossing_events">Crossing events</h2>
            <div id="dm-events"></div>
        </div>

        <div id="dm-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('dm-demo').addEventListener('click', () => {
        const b = makeDemoBars();
        document.getElementById('dm-bars').value =
            b.map(x => `${x.high} ${x.low}`).join('\n');
        showToast(t('view.demarker.toast.demo_loaded', { n: b.length }), { level: 'info' });
    });
    document.getElementById('dm-clear').addEventListener('click', () => {
        document.getElementById('dm-bars').value = '';
        showToast(t('view.demarker.toast.cleared'), { level: 'info' });
    });
    document.getElementById('dm-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.barText = document.getElementById('dm-bars').value;
    state.period = parseInt(document.getElementById('dm-period').value, 10);
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('dm-errors');
    errs.style.display = 'none';
    const { bars, errors } = parseBarBlob(state.barText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            t('common.parse_error_inline', { line: e.line_no, msg: e.message, raw: e.raw.slice(0, 80) })).join('<br>');
        const more = errors.length > 8 ? `<br>${esc(t('common.and_n_more', { n: errors.length - 8 }))}` : '';
        errs.innerHTML = `<strong>${esc(t('common.parse_errors_lead', { n: errors.length }))}</strong><br>${head}${more}`;
        errs.style.display = 'block';
        showToast(t('view.demarker.toast.parse_error', { n: errors.length }), { level: 'warning' });
        if (bars.length === 0) return;
    }
    const err = validateInputs(bars, state.period);
    if (err) { showErr(err); showToast(t('view.demarker.toast.invalid'), { level: 'warning' }); return; }

    let values;
    try {
        values = await api.anlyDemarker(buildBody(bars, state.period));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        showToast(t('view.demarker.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    // Backend returns Option<f64> as null for warmup → keep that as null for chart.
    const numeric = (values || []).map(v => v == null ? null : Number(v));
    renderSummary(numeric, bars);
    renderChart(numeric);
    renderEvents(numeric);
    const latest = latestValue(numeric);
    const regime = regimeOf(latest.value);
    showToast(t('view.demarker.toast.computed', { bars: bars.length, regime }), { level: 'success' });
}

function renderSummary(values, bars) {
    const counts = regimeCounts(values);
    const finite = values.filter(Number.isFinite).length;
    const obPct = finite > 0 ? counts.overbought / finite : 0;
    const osPct = finite > 0 ? counts.oversold   / finite : 0;
    const latest = latestValue(values);
    const reg = regimeOf(latest.value);
    const badge = regimeBadge(reg);
    document.getElementById('dm-summary').innerHTML = [
        card(t('view.demarker.card.bars'),          String(bars.length)),
        card(t('view.demarker.card.finite_values'), String(finite)),
        card(t('view.demarker.card.overbought'),    `${counts.overbought} · ${fmtPct(obPct)}`, counts.overbought ? 'neg' : ''),
        card(t('view.demarker.card.oversold'),      `${counts.oversold} · ${fmtPct(osPct)}`,   counts.oversold ? 'pos' : ''),
        card(t('view.demarker.card.neutral'),       String(counts.neutral)),
        card(t('view.demarker.card.latest_value'),  fmtN(latest.value), badge.cls),
        card(t('view.demarker.card.latest_regime'), badge.label, badge.cls),
        card(t('view.demarker.card.action'),        badge.hint),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderChart(values) {
    if (!window.uPlot) return;
    const xs = values.map((_, i) => i);
    const obYs  = xs.map(() => OB_THRESHOLD);
    const osYs  = xs.map(() => OS_THRESHOLD);
    const midYs = xs.map(() => 0.5);
    const el = document.getElementById('dm-chart');
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 280,
        scales: { x: {}, y: { range: [0, 1] } },
        series: [
            { label: t('chart.series.bar_num') },
            { label: t('chart.series.demarker'), stroke: '#00e5ff', width: 1.5,
              fill: '#00e5ff14', points: { show: false } },
            { label: t('chart.series.ob_070'),  stroke: '#ff3860', width: 1.0,
              dash: [4, 4], points: { show: false } },
            { label: t('chart.series.mid_050'), stroke: '#ffd84a', width: 0.8,
              dash: [2, 4], points: { show: false } },
            { label: t('chart.series.os_030'),  stroke: '#39ff14', width: 1.0,
              dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 40 }],
        legend: { show: true },
    }, [xs, values, obYs, midYs, osYs], el);
}

function renderEvents(values) {
    const wrap = document.getElementById('dm-events');
    const events = detectCrossings(values);
    if (!events.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.demarker.empty.crossings">No OB/OS crossings detected.</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th><th data-i18n="view.demarker.th.bar_idx">Bar idx</th><th data-i18n="view.demarker.th.regime">Regime</th><th data-i18n="view.demarker.th.demarker_value">DeMarker value</th><th data-i18n="view.demarker.th.action">Action</th>
            </tr></thead>
            <tbody>
                ${events.map((e, i) => {
                    const b = regimeBadge(e.regime);
                    return `<tr>
                        <td>${i + 1}</td>
                        <td>${e.bar_index}</td>
                        <td class="${b.cls}">${esc(b.label)}</td>
                        <td>${esc(fmtN(e.value))}</td>
                        <td>${esc(b.hint)}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
}

function showErr(msg) {
    const el = document.getElementById('dm-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('dm-err').style.display = 'none'; }
