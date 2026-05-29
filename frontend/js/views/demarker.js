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

let state = { barText: '', period: 14 };

export async function renderDemarker(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// DEMARKER OSCILLATOR</h1>

        <div class="chart-panel">
            <h2>HL bars</h2>
            <p class="muted">Paste <code>high low</code> per line. Demo loads 60 bars
                cycling through uptrend → chop → downtrend so OB and OS readings both fire.</p>
            <textarea id="dm-bars" rows="6" placeholder="100.5 99.5&#10;100.8 99.8&#10;..."></textarea>
            <div class="inline-form">
                <label>Period
                    <input id="dm-period" type="number" step="1" min="2" value="${state.period}"></label>
                <button id="dm-demo" class="secondary" type="button">Load demo (60 bars, OB+OS cycle)</button>
                <button id="dm-clear" class="secondary" type="button">Clear</button>
                <button id="dm-run" class="primary" type="button">Compute</button>
            </div>
            <p class="muted">Bounded [0, 1]. ≥0.70 = overbought (setup for short / mean-reversion).
                ≤0.30 = oversold (setup for long / mean-reversion). Crossovers from neutral into
                an extreme region are surfaced as event alerts.</p>
        </div>

        <div id="dm-errors" class="boot" style="display:none"></div>
        <div id="dm-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>DeMarker(${state.period}) series</h2>
            <div id="dm-chart" style="height:280px"></div>
            <p class="muted">Cyan = DeMarker. Red dashed = 0.70 OB threshold. Green dashed =
                0.30 OS threshold. Yellow = 0.50 mid.</p>
        </div>

        <div class="chart-panel">
            <h2>Crossing events</h2>
            <div id="dm-events"></div>
        </div>

        <div id="dm-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('dm-demo').addEventListener('click', () => {
        const b = makeDemoBars();
        document.getElementById('dm-bars').value =
            b.map(x => `${x.high} ${x.low}`).join('\n');
    });
    document.getElementById('dm-clear').addEventListener('click', () => {
        document.getElementById('dm-bars').value = '';
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
            `line ${e.line_no}: ${esc(e.message)} — ${esc(e.raw.slice(0, 80))}`).join('<br>');
        const more = errors.length > 8 ? `<br>… and ${errors.length - 8} more.` : '';
        errs.innerHTML = `<strong>${errors.length} parse error(s):</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (bars.length === 0) return;
    }
    const err = validateInputs(bars, state.period);
    if (err) { showErr(err); return; }

    let values;
    try {
        values = await api.anlyDemarker(buildBody(bars, state.period));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    // Backend returns Option<f64> as null for warmup → keep that as null for chart.
    const numeric = (values || []).map(v => v == null ? null : Number(v));
    renderSummary(numeric, bars);
    renderChart(numeric);
    renderEvents(numeric);
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
        card('Bars',          String(bars.length)),
        card('Finite values', String(finite)),
        card('Overbought',    `${counts.overbought} · ${fmtPct(obPct)}`, counts.overbought ? 'neg' : ''),
        card('Oversold',      `${counts.oversold} · ${fmtPct(osPct)}`,   counts.oversold ? 'pos' : ''),
        card('Neutral',       String(counts.neutral)),
        card('Latest value',  fmtN(latest.value), badge.cls),
        card('Latest regime', badge.label, badge.cls),
        card('Action',        badge.hint),
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
            { label: 'bar #' },
            { label: 'DeMarker', stroke: '#00e5ff', width: 1.5,
              fill: '#00e5ff14', points: { show: false } },
            { label: 'OB 0.70',  stroke: '#ff3860', width: 1.0,
              dash: [4, 4], points: { show: false } },
            { label: 'mid 0.50', stroke: '#ffd84a', width: 0.8,
              dash: [2, 4], points: { show: false } },
            { label: 'OS 0.30',  stroke: '#39ff14', width: 1.0,
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
        wrap.innerHTML = '<div class="muted">No OB/OS crossings detected.</div>';
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th><th>Bar idx</th><th>Regime</th><th>DeMarker value</th><th>Action</th>
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
