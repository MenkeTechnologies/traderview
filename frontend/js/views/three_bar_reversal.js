// Three-Bar Reversal view — classic short-term key-reversal detector.
//
// Bullish: down bar → small/inside middle → up bar closing above bar1's high.
// Bearish: up bar → small/inside middle → down bar closing below bar1's low.
// Middle bar body must be ≤ 50% of bar1's body (fixed backend rule).

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseBarBlob, validateInputs, buildBody,
    kindBadge, eventMarkers, makeDemoBars, fmtN,
} from '../_three_bar_reversal_inputs.js';

import { t } from '../i18n.js';
let state = { barText: '' };

export async function renderThreeBarReversal(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.three_bar_reversal.h1.three_bar_reversal" class="view-title">// THREE-BAR REVERSAL</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.three_bar_reversal.h2.ohlc_bars">OHLC bars</h2>
            <p class="muted" data-i18n-html="view.three_bar_reversal.help">Paste <code>open high low close</code> per line.
                Detection rule (fixed): bar1 trend bar, small middle (body ≤ 50% of bar1's body),
                bar3 closes past bar1's opposite extreme. Demo includes one classic bullish
                pattern at bars 2-3-4 and one classic bearish pattern at bars 11-12-13.</p>
            <textarea id="tbr-bars" rows="6" placeholder="100 100.5 99.5 100.2&#10;100.2 100.5 99.8 100.0&#10;..."></textarea>
            <div class="inline-form">
                <button data-i18n="view.three_bar_reversal.btn.load_demo_14_bars_bull_bear_pattern" id="tbr-demo" class="secondary" type="button">Load demo (14 bars, bull + bear pattern)</button>
                <button data-i18n="view.three_bar_reversal.btn.clear" id="tbr-clear" class="secondary" type="button">Clear</button>
                <button data-i18n="view.three_bar_reversal.btn.detect" id="tbr-run" class="primary" type="button">Detect</button>
            </div>
        </div>

        <div id="tbr-errors" class="boot" style="display:none"></div>
        <div id="tbr-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.three_bar_reversal.h2.close_series_reversal_markers">Close series + reversal markers</h2>
            <div id="tbr-chart" style="height:280px"></div>
            <p data-i18n="view.three_bar_reversal.hint.cyan_close_green_dot_bullish_reversal_placed_below" class="muted">Cyan = close. Green dot = bullish reversal (placed below the
                third bar's low). Red dot = bearish reversal (placed above the third bar's high).</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.three_bar_reversal.h2.event_log">Event log</h2>
            <div id="tbr-events"></div>
        </div>

        <div id="tbr-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('tbr-demo').addEventListener('click', () => {
        const b = makeDemoBars();
        document.getElementById('tbr-bars').value =
            b.map(x => `${x.open} ${x.high} ${x.low} ${x.close}`).join('\n');
    });
    document.getElementById('tbr-clear').addEventListener('click', () => {
        document.getElementById('tbr-bars').value = '';
    });
    document.getElementById('tbr-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.barText = document.getElementById('tbr-bars').value;
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('tbr-errors');
    errs.style.display = 'none';
    const { bars, errors } = parseBarBlob(state.barText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            `line ${e.line_no}: ${esc(e.message)} — ${esc(e.raw.slice(0, 80))}`).join('<br>');
        const more = errors.length > 8 ? `<br>… and ${errors.length - 8} more.` : '';
        errs.innerHTML = `<strong>${errors.length} parse error(s):</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (bars.length < 3) return;
    }
    const err = validateInputs(bars);
    if (err) { showErr(err); return; }
    let report;
    try {
        report = await api.anlyThreeBarReversal(buildBody(bars));
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
    const bullish = events.filter(e => e.kind === 'bullish').length;
    const bearish = events.filter(e => e.kind === 'bearish').length;
    const last = events[events.length - 1];
    document.getElementById('tbr-summary').innerHTML = [
        card(t('view.three_bar_reversal.card.bars'),         String(bars.length)),
        card(t('view.three_bar_reversal.card.events'),       String(report.n_events || 0)),
        card(t('view.three_bar_reversal.card.bullish'),      String(bullish), bullish ? 'pos' : ''),
        card(t('view.three_bar_reversal.card.bearish'),      String(bearish), bearish ? 'neg' : ''),
        card(t('view.three_bar_reversal.card.last_event'),   last
            ? `bar ${last.bar_index} ${kindBadge(last.kind).label}`
            : '—',
            last ? kindBadge(last.kind).cls : ''),
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
    const xs = bars.map((_, i) => i);
    const closes = bars.map(b => b.close);
    const { up, dn } = eventMarkers(report.events, bars);
    const el = document.getElementById('tbr-chart');
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 280,
        scales: { x: {}, y: {} },
        series: [
            { label: 'bar #' },
            { label: 'close',    stroke: '#00e5ff', width: 1.2,
              fill: '#00e5ff14', points: { show: false } },
            { label: 'bullish',  stroke: '#39ff14', width: 0,
              points: { show: true, size: 12, stroke: '#39ff14', fill: '#39ff14' } },
            { label: 'bearish',  stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, stroke: '#ff3860', fill: '#ff3860' } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: true },
    }, [xs, closes, up, dn], el);
}

function renderEvents(report) {
    const wrap = document.getElementById('tbr-events');
    const events = report.events || [];
    if (!events.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.three_bar_reversal.empty.events">No three-bar reversal events.</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th><th data-i18n="view.three_bar_reversal.th.bar_idx">Bar idx</th><th data-i18n="view.three_bar_reversal.th.kind">Kind</th>
                <th data-i18n="view.three_bar_reversal.th.hint">Hint</th><th data-i18n="view.three_bar_reversal.th.bar_1_open">Bar 1 open</th><th data-i18n="view.three_bar_reversal.th.bar_3_close">Bar 3 close</th><th>Δ</th>
            </tr></thead>
            <tbody>
                ${events.map((e, i) => {
                    const k = kindBadge(e.kind);
                    const delta = (e.bar3_close ?? 0) - (e.bar1_open ?? 0);
                    return `<tr>
                        <td>${i + 1}</td>
                        <td>${e.bar_index}</td>
                        <td class="${k.cls}">${esc(k.label)}</td>
                        <td>${esc(k.hint)}</td>
                        <td>${esc(fmtN(e.bar1_open))}</td>
                        <td>${esc(fmtN(e.bar3_close))}</td>
                        <td class="${delta >= 0 ? 'pos' : 'neg'}">${esc(fmtN(delta))}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
}

function showErr(msg) {
    const el = document.getElementById('tbr-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('tbr-err').style.display = 'none'; }
