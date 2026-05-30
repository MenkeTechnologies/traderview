// OI Change view — open-interest surge / unwind alerter across an options chain.
//
// "Where is institutional positioning building today?" Compares each
// strike's current OI to its rolling baseline; emits an alert when
// pct_change exceeds threshold AND current OI ≥ min_oi (suppresses
// micro-strike noise). Surge on the call side = upside positioning;
// surge on the put side = downside hedge build.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseSnapshotBlob, validateInputs, buildBody,
    alertTier, flowDirection, summarize,
    makeDemoSnapshots,
    fmtN, fmtInt, fmtPct, fmtSignedInt,
} from '../_oi_change_inputs.js';

import { t } from '../i18n.js';
let state = { snapshotsText: '', pctThreshold: 0.25, minOi: 1000 };

export async function renderOiChange(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.oi_change.h1.oi_change_positioning_surge_alerter" class="view-title">// OI CHANGE · POSITIONING SURGE ALERTER</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.oi_change.h2.strike_level_oi_snapshots">Strike-level OI snapshots</h2>
            <p class="muted" data-i18n="view.oi_change.hint.format">One line per strike: strike call_oi put_oi call_baseline put_baseline. Baseline = trailing 20-day average (or your own reference). Demo loads an 8-strike chain with engineered surges on the 510 call and 470 put.</p>
            <textarea id="oi-snap" rows="8" placeholder="500 25000 6000 24000 6200&#10;510 32000 3000 12000 3100&#10;..."></textarea>
            <div class="inline-form">
                <label><span data-i18n="view.oi_change.label.pct_threshold">Pct threshold (e.g. 0.25 = 25%)</span>
                    <input id="oi-pct" type="number" step="any" min="0" value="${state.pctThreshold}"></label>
                <label><span data-i18n="view.oi_change.label.min_oi">Min OI (suppress micro-strike noise)</span>
                    <input id="oi-min" type="number" step="1" min="0" value="${state.minOi}"></label>
                <button data-i18n="view.oi_change.btn.load_demo_8_strikes_surge_call_put" id="oi-demo" class="secondary" type="button">Load demo (8 strikes, surge call+put)</button>
                <button data-i18n="view.oi_change.btn.clear" id="oi-clear" class="secondary" type="button">Clear</button>
                <button data-i18n="view.oi_change.btn.analyze" id="oi-run" class="primary" type="button">Analyze</button>
            </div>
        </div>

        <div id="oi-errors" class="boot" style="display:none"></div>
        <div id="oi-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.oi_change.h2.call_side_oi_alerts">Call-side OI alerts</h2>
            <div id="oi-calls"></div>
            <p data-i18n="view.oi_change.hint.sorted_biggest_absolute_change_first_surge_strong_" class="muted">Sorted biggest absolute change first. SURGE/STRONG rows
                are where upside positioning is concentrating today.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.oi_change.h2.put_side_oi_alerts">Put-side OI alerts</h2>
            <div id="oi-puts"></div>
            <p data-i18n="view.oi_change.hint.sorted_biggest_absolute_change_first_surge_on_puts" class="muted">Sorted biggest absolute change first. SURGE on puts =
                institutional hedging or directional bearish bets.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.oi_change.h2.pct_chart">Call vs put OI % change by strike</h2>
            <div id="oi-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.oi_change.h2.abs_chart">Call vs put Δ OI (absolute contracts) by strike</h2>
            <div id="oi-abs-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.oi_change.hint.abs_chart" class="muted small">Absolute contract change per strike. A 1000% spike on a 50-OI strike is noise; a 30% increase on a 100k-OI strike is massive. This view filters % illusions.</p>
        </div>

        <div id="oi-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('oi-demo').addEventListener('click', () => {
        const snaps = makeDemoSnapshots();
        document.getElementById('oi-snap').value =
            snaps.map(s => `${s.strike} ${s.call_oi} ${s.put_oi} ${s.call_oi_baseline} ${s.put_oi_baseline}`).join('\n');
    });
    document.getElementById('oi-clear').addEventListener('click', () => {
        document.getElementById('oi-snap').value = '';
    });
    document.getElementById('oi-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.snapshotsText = document.getElementById('oi-snap').value;
    state.pctThreshold = Number(document.getElementById('oi-pct').value);
    state.minOi = parseInt(document.getElementById('oi-min').value, 10);
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('oi-errors');
    errs.style.display = 'none';
    const { snapshots, errors } = parseSnapshotBlob(state.snapshotsText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            t('common.parse_error_inline', { line: e.line_no, msg: e.message, raw: e.raw.slice(0, 80) })).join('<br>');
        const more = errors.length > 8 ? `<br>${esc(t('common.and_n_more', { n: errors.length - 8 }))}` : '';
        errs.innerHTML = `<strong>${esc(t('common.parse_errors_lead', { n: errors.length }))}</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (snapshots.length === 0) return;
    }
    const err = validateInputs(snapshots, state.pctThreshold, state.minOi);
    if (err) { showErr(err); return; }
    let report;
    try {
        report = await api.optCalcOiChange(buildBody(snapshots, state.pctThreshold, state.minOi));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e })); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(report, snapshots);
    renderTable('oi-calls', report.call_alerts || []);
    renderTable('oi-puts',  report.put_alerts  || []);
    renderOiPctChart(report);
    renderOiAbsChart(report);
}

function renderOiAbsChart(report) {
    const el = document.getElementById('oi-abs-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const calls = (report.call_alerts || []).filter(a => Number.isFinite(Number(a.abs_change)));
    const puts  = (report.put_alerts  || []).filter(a => Number.isFinite(Number(a.abs_change)));
    if (calls.length + puts.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.oi_change.empty_abs_chart">${esc(t('view.oi_change.empty_abs_chart'))}</div>`;
        return;
    }
    const strikes = Array.from(new Set([...calls, ...puts].map(a => Number(a.strike)))).sort((a, b) => a - b);
    const callMap = new Map(calls.map(a => [Number(a.strike), Number(a.abs_change)]));
    const putMap  = new Map(puts.map(a => [Number(a.strike), Number(a.abs_change)]));
    const callYs = strikes.map(s => callMap.has(s) ? callMap.get(s) : null);
    const putYs  = strikes.map(s => putMap.has(s)  ? putMap.get(s)  : null);
    const zero = strikes.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { auto: true }, y: { auto: true } },
        series: [
            { label: t('view.oi_change.chart.strike') },
            { label: t('view.oi_change.chart.calls_abs'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.oi_change.chart.puts_abs'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 12, fill: '#b86bff', stroke: '#b86bff' } },
            { label: t('view.oi_change.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 60 }],
        legend: { show: true },
    }, [strikes, callYs, putYs, zero], el);
}

function renderOiPctChart(report) {
    const el = document.getElementById('oi-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const calls = (report.call_alerts || []).filter(a => Number.isFinite(Number(a.pct_change)));
    const puts  = (report.put_alerts  || []).filter(a => Number.isFinite(Number(a.pct_change)));
    if (calls.length + puts.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.oi_change.empty_chart">${esc(t('view.oi_change.empty_chart'))}</div>`;
        return;
    }
    const strikes = Array.from(new Set([...calls, ...puts].map(a => Number(a.strike)))).sort((a, b) => a - b);
    const callMap = new Map(calls.map(a => [Number(a.strike), Number(a.pct_change) * 100]));
    const putMap  = new Map(puts.map(a => [Number(a.strike), Number(a.pct_change) * 100]));
    const callYs = strikes.map(s => callMap.has(s) ? callMap.get(s) : null);
    const putYs  = strikes.map(s => putMap.has(s)  ? putMap.get(s)  : null);
    const zero = strikes.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { auto: true }, y: { auto: true } },
        series: [
            { label: t('view.oi_change.chart.strike') },
            { label: t('view.oi_change.chart.calls'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.oi_change.chart.puts'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.oi_change.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [ { stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 } ],
        legend: { show: true },
    }, [strikes, callYs, putYs, zero], el);
}

function renderSummary(report, snapshots) {
    const s = summarize(report);
    const callPutSkew = s.netCallChange - s.netPutChange;
    document.getElementById('oi-summary').innerHTML = [
        card(t('view.oi_change.card.strikes_scanned'),  String(snapshots.length)),
        card(t('view.oi_change.card.call_alerts'),      String(s.totalCallAlerts), s.totalCallAlerts ? 'pos' : ''),
        card(t('view.oi_change.card.put_alerts'),       String(s.totalPutAlerts),  s.totalPutAlerts ? 'neg' : ''),
        card(t('view.oi_change.card.net_call_oi'),     fmtSignedInt(s.netCallChange), s.netCallChange >= 0 ? 'pos' : 'neg'),
        card(t('view.oi_change.card.net_put_oi'),      fmtSignedInt(s.netPutChange),  s.netPutChange >= 0 ? 'neg' : 'pos'),
        card(t('view.oi_change.card.hot_call_strike'),  s.maxCallStrike != null ? fmtN(s.maxCallStrike) : '—', 'pos'),
        card(t('view.oi_change.card.hot_put_strike'),   s.maxPutStrike  != null ? fmtN(s.maxPutStrike)  : '—', 'neg'),
        card(t('view.oi_change.card.net_positioning'),  fmtSignedInt(callPutSkew),
            callPutSkew > 0 ? 'pos' : callPutSkew < 0 ? 'neg' : ''),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderTable(elId, alerts) {
    const wrap = document.getElementById(elId);
    if (!alerts.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.oi_change.empty.alerts">No alerts.</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.oi_change.th.strike">Strike</th><th data-i18n="view.oi_change.th.tier">Tier</th><th data-i18n="view.oi_change.th.flow">Flow</th>
                <th data-i18n="view.oi_change.th.current_oi">Current OI</th><th data-i18n="view.oi_change.th.baseline">Baseline</th>
                <th data-i18n="view.oi_change.th.oi">Δ OI</th><th>Δ %</th>
            </tr></thead>
            <tbody>
                ${alerts.map(a => {
                    const tier = alertTier(a);
                    const flow = flowDirection(a.abs_change);
                    return `<tr>
                        <td>${esc(fmtN(a.strike))}</td>
                        <td class="${tier.cls}">${esc(tier.label)}</td>
                        <td class="${flow.cls}">${esc(flow.label)}</td>
                        <td>${esc(fmtInt(a.current_oi))}</td>
                        <td>${esc(fmtInt(a.baseline_oi))}</td>
                        <td class="${a.abs_change >= 0 ? 'neg' : 'pos'}">${esc(fmtSignedInt(a.abs_change))}</td>
                        <td class="${a.pct_change >= 0 ? 'neg' : 'pos'}">${esc(fmtPct(a.pct_change))}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
}

function showErr(msg) {
    const el = document.getElementById('oi-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('oi-err').style.display = 'none'; }
