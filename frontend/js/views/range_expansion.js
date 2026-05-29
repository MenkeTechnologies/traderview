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

const DEFAULT_CFG = { lookback: 5, min_expansion_atrs: 1.5, prior_atr_max: 0.7 };
const DEFAULT_ATR_PERIOD = 14;

let state = { barText: '', atrPeriod: DEFAULT_ATR_PERIOD, config: { ...DEFAULT_CFG } };

export async function renderRangeExpansion(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// RANGE EXPANSION</h1>

        <div class="chart-panel">
            <h2>HLC bars</h2>
            <p class="muted">Paste <code>high low close</code> per line. ATR is computed
                locally (Wilder smoothing). Demo loads 30 bars with engineered compression
                resolving UP and a second compression resolving DOWN.</p>
            <textarea id="re-bars" rows="6" placeholder="100.5 99.5 100.0&#10;100.8 99.8 100.3&#10;..."></textarea>
            <div class="inline-form">
                <button id="re-demo" class="secondary" type="button">Load demo (30 bars, 2 expansions)</button>
                <button id="re-clear" class="secondary" type="button">Clear</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2>Config</h2>
            <div class="inline-form">
                <label>ATR period
                    <input id="re-atr" type="number" step="1" min="1" value="${state.atrPeriod}"></label>
                <label>Lookback bars
                    <input id="re-lb"  type="number" step="1" min="1" value="${state.config.lookback}"></label>
                <label>Min expansion (× ATR)
                    <input id="re-min" type="number" step="0.1" min="0" value="${state.config.min_expansion_atrs}"></label>
                <label>Prior ATR max (compression cap)
                    <input id="re-prior" type="number" step="0.1" min="0" value="${state.config.prior_atr_max}"></label>
                <button id="re-run" class="primary" type="button">Detect</button>
            </div>
            <p class="muted">Industry defaults (Raschke): ATR-14, lookback 5, ≥1.5× ATR for
                the expansion bar, &lt;0.7× ATR for at least one compression bar in the lookback.
                Prior-ATR-max must be &lt; min-expansion-atrs (compression then expansion).</p>
        </div>

        <div id="re-errors" class="boot" style="display:none"></div>
        <div id="re-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>Close series + ATR + expansion markers</h2>
            <div id="re-chart" style="height:300px"></div>
            <p class="muted">Cyan = close. Yellow = ATR(period). Green dot above bar =
                UP expansion. Red dot below bar = DOWN expansion. Marker placement reveals
                direction at a glance.</p>
        </div>

        <div class="chart-panel">
            <h2>Event log</h2>
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
        showErr(`API error: ${e.message || e}`); return;
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
        card('Bars',       String(bars.length)),
        card('Events',     String(report.n_events || 0)),
        card('UP',         String(ups),   ups   ? 'pos' : ''),
        card('DOWN',       String(downs), downs ? 'neg' : ''),
        card('Avg ATR',    fmtN(avgAtr)),
        card('Last event', last
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
            { label: 'bar #' },
            { label: 'close', stroke: '#00e5ff', width: 1.2,
              fill: '#00e5ff14', points: { show: false } },
            { label: 'ATR',   stroke: '#ffd84a', width: 1.0,
              points: { show: false }, scale: 'y_atr' },
            { label: 'UP',    stroke: '#39ff14', width: 0,
              points: { show: true, size: 12, stroke: '#39ff14', fill: '#39ff14' } },
            { label: 'DOWN',  stroke: '#ff3860', width: 0,
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
        wrap.innerHTML = '<div class="muted">No range-expansion events at current config.</div>';
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th><th>Bar idx</th><th>Direction</th>
                <th>Range × ATR</th><th>Compressed bars in lookback</th>
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
