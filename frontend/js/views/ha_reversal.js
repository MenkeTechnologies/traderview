// Heikin-Ashi Reversal view — color-flip detector on smoothed HA candles.
//
// Accepts raw OHLC, computes Heikin-Ashi locally, posts the HA series
// to the backend for flip detection, and overlays the detected
// reversal markers on the raw close + HA close chart.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseBarBlob, computeHeikinAshi, validateInputs, buildBody,
    dirBadge, strengthBadge, eventMarkers, makeDemoBars,
    fmtN, fmtPct,
} from '../_ha_reversal_inputs.js';

const DEFAULT_CFG = { min_body_ratio: 0.6, strong_streak: 3, weak_streak: 2 };

let state = { barText: '', config: { ...DEFAULT_CFG } };

export async function renderHaReversal(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// HEIKIN-ASHI REVERSAL</h1>

        <div class="chart-panel">
            <h2>OHLC bars</h2>
            <p class="muted">Paste <code>open high low close</code> per line. Bars
                are converted to Heikin-Ashi candles client-side; the backend
                detects color-flip reversal events on the HA series.
                Demo loads 30 bars with bull-bear-bull regime structure.</p>
            <textarea id="ha-bars" rows="6" placeholder="100.50 101.20 100.00 100.85&#10;100.85 101.50 100.40 101.30&#10;..."></textarea>
            <div class="inline-form">
                <button id="ha-demo" class="secondary" type="button">Load demo (30 bars, multi-regime)</button>
                <button id="ha-clear" class="secondary" type="button">Clear</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2>Flip config</h2>
            <div class="inline-form">
                <label>Min body ratio (Strong)
                    <input id="ha-body" type="number" step="0.01" min="0" max="1" value="${state.config.min_body_ratio}"></label>
                <label>Strong streak (≥ bars)
                    <input id="ha-strong" type="number" step="1" min="1" value="${state.config.strong_streak}"></label>
                <label>Weak streak (≥ bars)
                    <input id="ha-weak" type="number" step="1" min="1" value="${state.config.weak_streak}"></label>
                <button id="ha-run" class="primary" type="button">Detect</button>
            </div>
            <p class="muted">Strong flips need both a long prior same-color streak (default 3+)
                AND a decisive body (default ≥60% of bar range). Weak flips only need the
                shorter streak (default 2+) — earlier signal, less reliable.</p>
        </div>

        <div id="ha-errors" class="boot" style="display:none"></div>
        <div id="ha-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>Raw close + HA close + flip markers</h2>
            <div id="ha-chart" style="height:280px"></div>
            <p class="muted">Magenta = raw close. Cyan = HA close. Green dot = BEAR→BULL
                flip (placed below HA low). Red dot = BULL→BEAR flip (placed above HA high).</p>
        </div>

        <div class="chart-panel">
            <h2>Event log</h2>
            <div id="ha-events"></div>
        </div>

        <div id="ha-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('ha-demo').addEventListener('click', () => {
        const b = makeDemoBars(42);
        document.getElementById('ha-bars').value =
            b.map(x => `${x.open} ${x.high} ${x.low} ${x.close}`).join('\n');
    });
    document.getElementById('ha-clear').addEventListener('click', () => {
        document.getElementById('ha-bars').value = '';
    });
    document.getElementById('ha-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.barText = document.getElementById('ha-bars').value;
    state.config = {
        min_body_ratio: Number(document.getElementById('ha-body').value),
        strong_streak:  parseInt(document.getElementById('ha-strong').value, 10),
        weak_streak:    parseInt(document.getElementById('ha-weak').value, 10),
    };
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('ha-errors');
    errs.style.display = 'none';
    const { bars, errors } = parseBarBlob(state.barText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            `line ${e.line_no}: ${esc(e.message)} — ${esc(e.raw.slice(0, 80))}`).join('<br>');
        const more = errors.length > 8 ? `<br>… and ${errors.length - 8} more.` : '';
        errs.innerHTML = `<strong>${errors.length} parse error(s):</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (bars.length < 2) return;
    }
    const err = validateInputs(bars, state.config);
    if (err) { showErr(err); return; }

    let report;
    try {
        report = await api.anlyHaReversal(buildBody(bars, state.config));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(report, bars);
    renderChart(bars, report);
    renderEvents(report);
}

function renderSummary(report, bars) {
    const events = report.events || [];
    const strong = events.filter(e => e.strength === 'strong').length;
    const weak   = events.filter(e => e.strength === 'weak').length;
    const upFlips = events.filter(e => e.direction === 'bearish_to_bullish').length;
    const dnFlips = events.filter(e => e.direction === 'bullish_to_bearish').length;
    const last = events[events.length - 1];
    document.getElementById('ha-summary').innerHTML = [
        card('Bars',         String(bars.length)),
        card('Events',       String(report.n_events || 0)),
        card('Strong',       String(strong), strong ? 'pos' : ''),
        card('Weak',         String(weak)),
        card('BEAR → BULL',  String(upFlips), upFlips ? 'pos' : ''),
        card('BULL → BEAR',  String(dnFlips), dnFlips ? 'neg' : ''),
        card('Last event',   last
            ? `bar ${last.bar_index} ${dirBadge(last.direction).label} (${strengthBadge(last.strength).label})`
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

function renderChart(bars, report) {
    if (!window.uPlot) return;
    const haBars = computeHeikinAshi(bars);
    const xs = bars.map((_, i) => i);
    const rawClose = bars.map(b => b.close);
    const haClose = haBars.map(b => b.close);
    const { up, dn } = eventMarkers(report.events, haBars);
    const el = document.getElementById('ha-chart');
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 280,
        scales: { x: {}, y: {} },
        series: [
            { label: 'bar #' },
            { label: 'raw close', stroke: '#a06bff', width: 1.0,
              fill: '#a06bff10', points: { show: false } },
            { label: 'HA close',  stroke: '#00e5ff', width: 1.5,
              fill: '#00e5ff14', points: { show: false } },
            { label: 'BEAR→BULL', stroke: '#39ff14', width: 0,
              points: { show: true, size: 12, stroke: '#39ff14', fill: '#39ff14' } },
            { label: 'BULL→BEAR', stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, stroke: '#ff3860', fill: '#ff3860' } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: true },
    }, [xs, rawClose, haClose, up, dn], el);
}

function renderEvents(report) {
    const wrap = document.getElementById('ha-events');
    const events = report.events || [];
    if (!events.length) {
        wrap.innerHTML = '<div class="muted">No flip events at current config.</div>';
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th><th>Bar idx</th><th>Direction</th><th>Strength</th>
                <th>Prior streak</th><th>Body ratio</th>
            </tr></thead>
            <tbody>
                ${events.map((e, i) => {
                    const d = dirBadge(e.direction);
                    const s = strengthBadge(e.strength);
                    return `<tr>
                        <td>${i + 1}</td>
                        <td>${e.bar_index}</td>
                        <td class="${d.cls}">${esc(d.label)}</td>
                        <td class="${s.cls}">${esc(s.label)}</td>
                        <td>${e.prior_streak}</td>
                        <td>${esc(fmtPct(e.body_ratio))}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
    void fmtN;
}

function showErr(msg) {
    const el = document.getElementById('ha-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ha-err').style.display = 'none'; }
