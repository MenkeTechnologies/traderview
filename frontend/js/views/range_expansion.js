// Range Expansion view — wide-range bar after consolidation detector.
//
// Linda Bradford Raschke / Larry Williams classic: when narrow-range bars
// (NR4/NR7-style compression) resolve, the resolution bar's range is
// typically much wider — and its direction confirms which way the spring
// uncoiled. The detector requires BOTH conditions: current bar ≥
// `min_expansion_atrs` × ATR AND at least one of the prior `lookback`
// bars had range < `prior_atr_max` × ATR.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseBarBlob, computeAtr, validateInputs, buildBody,
    dirBadge, eventMarkers, makeDemoBars, fmtN,
} from '../_range_expansion_inputs.js';

import { t } from '../i18n.js';
const DEFAULT_CFG = { lookback: 5, min_expansion_atrs: 1.5, prior_atr_max: 0.7 };
const DEFAULT_ATR_PERIOD = 14;

let state = { barText: '', atrPeriod: DEFAULT_ATR_PERIOD, config: { ...DEFAULT_CFG } };

export async function renderRangeExpansion(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.range_expansion.h1.range_expansion" class="view-title">// RANGE EXPANSION</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.range_expansion.h2.hlc_bars">HLC bars</h2>
            <p class="muted" data-i18n-html="view.range_expansion.help">Paste <code>high low close</code> per line. ATR is computed
                locally (Wilder smoothing). Demo loads 30 bars with engineered compression
                resolving UP and a second compression resolving DOWN.</p>
            <textarea id="re-bars" rows="6" placeholder="100.5 99.5 100.0&#10;100.8 99.8 100.3&#10;..."></textarea>
            <div class="inline-form">
                <button data-i18n="view.range_expansion.btn.load_demo_30_bars_2_expansions" id="re-demo" class="secondary" type="button">Load demo (30 bars, 2 expansions)</button>
                <button data-i18n="view.range_expansion.btn.clear" id="re-clear" class="secondary" type="button">Clear</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.range_expansion.h2.config">Config</h2>
            <div class="inline-form">
                <label><span data-i18n="view.range_expansion.label.atr_period">ATR period</span>
                    <input id="re-atr" type="number" step="1" min="1" value="${state.atrPeriod}"></label>
                <label><span data-i18n="view.range_expansion.label.lookback">Lookback bars</span>
                    <input id="re-lb"  type="number" step="1" min="1" value="${state.config.lookback}"></label>
                <label><span data-i18n="view.range_expansion.label.min_expansion">Min expansion (× ATR)</span>
                    <input id="re-min" type="number" step="0.1" min="0" value="${state.config.min_expansion_atrs}"></label>
                <label><span data-i18n="view.range_expansion.label.prior_atr_max">Prior ATR max (compression cap)</span>
                    <input id="re-prior" type="number" step="0.1" min="0" value="${state.config.prior_atr_max}"></label>
                <button data-i18n="view.range_expansion.btn.detect" id="re-run" class="primary" type="button">Detect</button>
            </div>
            <p data-i18n="view.range_expansion.hint.industry_defaults_raschke_atr_14_lookback_5_1_5_at" class="muted">Industry defaults (Raschke): ATR-14, lookback 5, ≥1.5× ATR for
                the expansion bar, &lt;0.7× ATR for at least one compression bar in the lookback.
                Prior-ATR-max must be &lt; min-expansion-atrs (compression then expansion).</p>
        </div>

        <div id="re-errors" class="boot" style="display:none"></div>
        <div id="re-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.range_expansion.h2.close_series_atr_expansion_markers">Close series + ATR + expansion markers</h2>
            <div id="re-chart" style="height:300px"></div>
            <p data-i18n="view.range_expansion.hint.cyan_close_yellow_atr_period_green_dot_above_bar_u" class="muted">Cyan = close. Yellow = ATR(period). Green dot above bar =
                UP expansion. Red dot below bar = DOWN expansion. Marker placement reveals
                direction at a glance.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.range_expansion.h2.event_log">Event log</h2>
            <div id="re-events"></div>
        </div>

        <div id="re-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('re-demo').addEventListener('click', () => {
        const b = makeDemoBars();
        document.getElementById('re-bars').value =
            b.map(x => `${x.high} ${x.low} ${x.close}`).join('\n');
    });
    document.getElementById('re-clear').addEventListener('click', () => {
        document.getElementById('re-bars').value = '';
    });
    document.getElementById('re-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.barText = document.getElementById('re-bars').value;
    state.atrPeriod = parseInt(document.getElementById('re-atr').value, 10);
    state.config = {
        lookback:           parseInt(document.getElementById('re-lb').value, 10),
        min_expansion_atrs: Number(document.getElementById('re-min').value),
        prior_atr_max:      Number(document.getElementById('re-prior').value),
    };
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('re-errors');
    errs.style.display = 'none';
    const { bars, errors } = parseBarBlob(state.barText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            `line ${e.line_no}: ${esc(e.message)} — ${esc(e.raw.slice(0, 80))}`).join('<br>');
        const more = errors.length > 8 ? `<br>… and ${errors.length - 8} more.` : '';
        errs.innerHTML = `<strong>${errors.length} parse error(s):</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (bars.length === 0) return;
    }
    const atr = computeAtr(bars, state.atrPeriod);
    const err = validateInputs(bars, atr, state.config);
    if (err) { showErr(err); return; }

    let report;
    try {
        report = await api.anlyRangeExpansion(buildBody(bars, atr, state.config));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e })); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(report, bars, atr);
    renderChart(bars, atr, report);
    renderEvents(report);
}

function renderSummary(report, bars, atr) {
    const events = report.events || [];
    const ups   = events.filter(e => e.direction === 'up').length;
    const downs = events.filter(e => e.direction === 'down').length;
    const last  = events[events.length - 1];
    const validAtr = atr.filter(Number.isFinite);
    const avgAtr = validAtr.length ? validAtr.reduce((a, b) => a + b, 0) / validAtr.length : NaN;
    document.getElementById('re-summary').innerHTML = [
        card(t('view.range_expansion.card.bars'),       String(bars.length)),
        card(t('view.range_expansion.card.events'),     String(report.n_events || 0)),
        card(t('view.range_expansion.card.up'),         String(ups),   ups   ? 'pos' : ''),
        card(t('view.range_expansion.card.down'),       String(downs), downs ? 'neg' : ''),
        card(t('view.range_expansion.card.avg_atr'),    fmtN(avgAtr)),
        card(t('view.range_expansion.card.last_event'), last
            ? `bar ${last.bar_index} ${dirBadge(last.direction).label} ${fmtN(last.range_atrs)}× ATR`
            : '—',
            last ? dirBadge(last.direction).cls : ''),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderChart(bars, atr, report) {
    if (!window.uPlot) return;
    const xs = bars.map((_, i) => i);
    const closes = bars.map(b => b.close);
    const { up, dn } = eventMarkers(report.events, bars);
    const el = document.getElementById('re-chart');
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 300,
        scales: { x: {}, y: {}, y_atr: {} },
        series: [
            { label: t('chart.series.bar_num') },
            { label: t('chart.series.close'), stroke: '#00e5ff', width: 1.2,
              fill: '#00e5ff14', points: { show: false } },
            { label: t('chart.series.atr'),   stroke: '#ffd84a', width: 1.0,
              points: { show: false }, scale: 'y_atr' },
            { label: t('chart.series.up'),    stroke: '#39ff14', width: 0,
              points: { show: true, size: 12, stroke: '#39ff14', fill: '#39ff14' } },
            { label: t('chart.series.down'),  stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, stroke: '#ff3860', fill: '#ff3860' } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 50 },
            { stroke: '#ffd84a', size: 40, scale: 'y_atr', side: 1 },
        ],
        legend: { show: true },
    }, [xs, closes, atr, up, dn], el);
}

function renderEvents(report) {
    const wrap = document.getElementById('re-events');
    const events = report.events || [];
    if (!events.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.range_expansion.empty.events">No range-expansion events at current config.</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th><th data-i18n="view.range_expansion.th.bar_idx">Bar idx</th><th data-i18n="view.range_expansion.th.direction">Direction</th>
                <th data-i18n="view.range_expansion.th.range_atr">Range × ATR</th><th data-i18n="view.range_expansion.th.compressed_bars_in_lookback">Compressed bars in lookback</th>
            </tr></thead>
            <tbody>
                ${events.map((e, i) => {
                    const d = dirBadge(e.direction);
                    return `<tr>
                        <td>${i + 1}</td>
                        <td>${e.bar_index}</td>
                        <td class="${d.cls}">${esc(d.label)}</td>
                        <td>${esc(fmtN(e.range_atrs))}</td>
                        <td>${e.compressed_bars_in_lookback}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
}

function showErr(msg) {
    const el = document.getElementById('re-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('re-err').style.display = 'none'; }
