// Margin-call runway view — projects % price decline that triggers a
// margin call given account equity + position value + maintenance %.
//
// All user-facing strings via i18n.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_INPUTS, validateInputs, buildBody, localCompute,
    runwayBadge, projectionCurves, makeDemoInputs,
    fmtUSD, fmtUSDSigned, fmtPct, fmtMaintPct,
} from '../_margin_runway_inputs.js';

let state = { ...DEFAULT_INPUTS };

export async function renderMarginRunway(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.margin_runway.h1.title" class="view-title">// MARGIN RUNWAY</h1>

        <div class="chart-panel" data-context-scope="margin-runway">
            <h2 data-i18n="view.margin_runway.h2.account">Account</h2>
            <div class="inline-form">
                <label><span data-i18n="view.margin_runway.label.equity">Account equity ($)</span>
                    <input id="mr-eq" type="number" step="any" min="0" value="${state.account_equity}"></label>
                <label><span data-i18n="view.margin_runway.label.position">Position value ($)</span>
                    <input id="mr-pos" type="number" step="any" min="0" value="${state.position_value}"></label>
                <label><span data-i18n="view.margin_runway.label.maint">Maintenance req %  (decimal — 0.25 = 25%)</span>
                    <input id="mr-mp" type="number" step="any" min="0" max="0.99" value="${state.maintenance_req_pct}"></label>
                <button data-i18n="view.margin_runway.btn.compute" id="mr-run" class="primary" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.margin_runway.btn.demo_safe"        id="mr-demo-safe"     class="secondary" type="button">Demo: safe (100% runway)</button>
                <button data-i18n="view.margin_runway.btn.demo_moderate"    id="mr-demo-mod"      class="secondary" type="button">Demo: moderate (~33%)</button>
                <button data-i18n="view.margin_runway.btn.demo_tight"       id="mr-demo-tight"    class="secondary" type="button">Demo: tight (~7%)</button>
                <button data-i18n="view.margin_runway.btn.demo_critical"    id="mr-demo-crit"     class="secondary" type="button">Demo: critical (&lt;5%)</button>
                <button data-i18n="view.margin_runway.btn.demo_in_call"     id="mr-demo-call"     class="secondary" type="button">Demo: already in call</button>
                <button data-i18n="view.margin_runway.btn.demo_pdt"         id="mr-demo-pdt"      class="secondary" type="button">Demo: PDT 4× leveraged</button>
                <button data-i18n="view.margin_runway.btn.demo_concentrated" id="mr-demo-conc"    class="secondary" type="button">Demo: concentrated (50% maint)</button>
                <button data-i18n="view.margin_runway.btn.demo_cash"        id="mr-demo-cash"     class="secondary" type="button">Demo: cash only (no position)</button>
            </div>
            <p data-i18n="view.margin_runway.hint.about" class="muted">Runway % = how far the position can fall before broker issues a margin call. Reg-T retail accounts default to 25% maintenance; brokers often raise it on concentrated / volatile names.</p>
        </div>

        <div id="mr-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.margin_runway.h2.projection">Equity vs maintenance under hypothetical price decline</h2>
            <div id="mr-chart" style="height:340px"></div>
            <p data-i18n="view.margin_runway.hint.chart" class="muted">Cyan = equity after price drop. Yellow = maintenance requirement after drop. Margin call hits at the crossover (red dot). X axis: % price decline from current.</p>
        </div>

        <div id="mr-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (kind) => {
        state = makeDemoInputs(kind);
        document.getElementById('mr-eq').value  = state.account_equity;
        document.getElementById('mr-pos').value = state.position_value;
        document.getElementById('mr-mp').value  = state.maintenance_req_pct;
    };
    document.getElementById('mr-demo-safe').addEventListener('click',  () => loadDemo('safe'));
    document.getElementById('mr-demo-mod').addEventListener('click',   () => loadDemo('moderate'));
    document.getElementById('mr-demo-tight').addEventListener('click', () => loadDemo('tight'));
    document.getElementById('mr-demo-crit').addEventListener('click',  () => loadDemo('critical'));
    document.getElementById('mr-demo-call').addEventListener('click',  () => loadDemo('in-call'));
    document.getElementById('mr-demo-pdt').addEventListener('click',   () => loadDemo('pdt-leveraged'));
    document.getElementById('mr-demo-conc').addEventListener('click',  () => loadDemo('concentrated'));
    document.getElementById('mr-demo-cash').addEventListener('click',  () => loadDemo('cash-only'));
    document.getElementById('mr-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function readInputs() {
    state.account_equity      = Number(document.getElementById('mr-eq').value);
    state.position_value      = Number(document.getElementById('mr-pos').value);
    state.maintenance_req_pct = Number(document.getElementById('mr-mp').value);
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.account_equity, state.position_value, state.maintenance_req_pct);
    if (err) { showErr(err); return; }
    const local = localCompute(state.account_equity, state.position_value, state.maintenance_req_pct);
    renderSummary(local, true);
    renderChart(state.account_equity, state.position_value, state.maintenance_req_pct, local);
    let resp;
    try {
        resp = await api.calcMarginRunway(buildBody(
            state.account_equity, state.position_value, state.maintenance_req_pct));
    } catch (e) {
        showErr(`${t('view.margin_runway.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(state.account_equity, state.position_value, state.maintenance_req_pct, resp);
}

function renderSummary(report, pending) {
    const badge = runwayBadge(report);
    const local = localCompute(state.account_equity, state.position_value, state.maintenance_req_pct);
    const parityOk = Math.abs(report.runway_pct - local.runway_pct) < 1e-9
                  && report.already_in_margin_call === local.already_in_margin_call;
    const localTag = pending ? ` (${t('view.margin_runway.tag.local')})` : '';
    const maintDollars = report.position_value * report.maintenance_req_pct;
    document.getElementById('mr-summary').innerHTML = [
        card(t('view.margin_runway.card.verdict'),
             t(badge.key) + localTag, badge.cls || ''),
        card(t('view.margin_runway.card.runway'),
             report.already_in_margin_call ? t('view.margin_runway.tag.zero') : fmtPct(report.runway_pct, 2),
             badge.cls || ''),
        card(t('view.margin_runway.card.buffer'),
             fmtUSDSigned(report.equity_buffer_dollars),
             report.equity_buffer_dollars >= 0 ? 'pos' : 'neg'),
        card(t('view.margin_runway.card.equity'),
             fmtUSD(report.account_equity)),
        card(t('view.margin_runway.card.position'),
             fmtUSD(report.position_value)),
        card(t('view.margin_runway.card.maint_pct'),
             fmtMaintPct(report.maintenance_req_pct)),
        card(t('view.margin_runway.card.maint_dollars'),
             fmtUSD(maintDollars)),
        card(t('view.margin_runway.card.leverage'),
             report.account_equity > 0
               ? (report.position_value / report.account_equity).toFixed(2) + '×' : '—',
             (report.position_value / report.account_equity) > 2 ? 'neg' : ''),
        card(t('view.margin_runway.card.parity'),
             parityOk ? t('view.margin_runway.tag.ok') : t('view.margin_runway.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(equity, position, maintPct, report) {
    if (!window.uPlot) return;
    const el = document.getElementById('mr-chart');
    if (!el) return;
    el.innerHTML = '';
    // Project across enough range to definitely show the crossover.
    const range = Math.max(0.5, Math.min(1, (report.runway_pct || 0.3) * 2 + 0.1));
    const { xs, equityCurve, maintCurve } = projectionCurves(equity, position, maintPct, 60, range);
    // Marker at the actual margin-call point.
    const callPct = report.runway_pct;
    const callX = report.already_in_margin_call ? 0 : callPct;
    const callMarker = xs.map(() => null);
    if (callX >= 0 && callX <= range) {
        // Find nearest x bucket.
        let bestI = 0, bestD = Infinity;
        for (let i = 0; i < xs.length; i++) {
            const d = Math.abs(xs[i] - callX);
            if (d < bestD) { bestD = d; bestI = i; }
        }
        callMarker[bestI] = maintCurve[bestI];
    }
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 320,
        scales: { x: {}, y: {} },
        series: [
            { label: t('chart.series._decline') },
            { label: t('chart.series.equity'),      stroke: '#00e5ff', width: 1.5, points: { show: false } },
            { label: t('chart.series.maintenance'), stroke: '#ffd84a', width: 1.5, points: { show: false } },
            { label: t('chart.series.call_point'),  stroke: '#ff3860', width: 0,
              points: { show: true, size: 11, fill: '#ff3860', stroke: '#ff3860' } },
        ],
        axes: [
            { stroke: '#aab', size: 32,
              values: (_u, splits) => splits.map(v => (v * 100).toFixed(0) + '%') },
            { stroke: '#aab', size: 60,
              values: (_u, splits) => splits.map(v => fmtUSD(v, 0)) },
        ],
        legend: { show: true },
    }, [xs, equityCurve, maintCurve, callMarker], el);
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('mr-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('mr-err').style.display = 'none'; }
