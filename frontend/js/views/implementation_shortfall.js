// Implementation Shortfall view — institutional Transaction Cost Analysis.
//
// Inputs: 7 numbers about the order's lifecycle.
//   decision_mid       — mid when the trade was decided
//   arrival_mid        — mid when the order hit the market
//   vwap_fill          — realized volume-weighted average fill
//   final_mid          — mid at order close (post-cancel / completion)
//   half_spread        — (ask-bid)/2 at decision
//   intended_qty       — shares the trader meant to buy / sell
//   filled_qty         — shares actually filled
//
// Output: 4-component cost decomposition (spread / timing / impact /
// opportunity) + total $ + total bps. Visualized as a horizontal bar
// chart so the dominant component is obvious at a glance.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    validateInputs, buildBody, decompose, costSignClass,
    fillKind, fmtUSD, fmtBps, fmtPct, } from '../_implementation_shortfall_inputs.js';

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
const DEFAULTS = {
    direction:               'buy',
    decision_mid:            100.00,
    arrival_mid:             100.05,
    vwap_fill:               100.08,
    final_mid:               100.20,
    half_spread_at_decision: 0.02,
    intended_qty:            10_000,
    filled_qty:              9_500,
};

let state = { params: { ...DEFAULTS } };

export async function renderImplementationShortfall(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.implementation_shortfall.h1.implementation_shortfall" class="view-title">// IMPLEMENTATION SHORTFALL</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.implementation_shortfall.h2.order_lifecycle">Order lifecycle</h2>
            <div class="inline-form">
                <label><span data-i18n="view.implementation_shortfall.label.direction">Direction</span>
                    <select id="is-dir" data-tip="view.implementation_shortfall.tip.dir">
                        <option data-i18n="view.implementation_shortfall.opt.buy" value="buy"  ${state.params.direction === 'buy'  ? 'selected' : ''}>Buy</option>
                        <option data-i18n="view.implementation_shortfall.opt.sell" value="sell" ${state.params.direction === 'sell' ? 'selected' : ''}>Sell</option>
                    </select></label>
                <label><span data-i18n="view.implementation_shortfall.label.decision_mid">Decision mid</span>
                    <input id="is-dm" type="number" step="0.01" min="0" value="${state.params.decision_mid}" data-tip="view.implementation_shortfall.tip.decision_mid"></label>
                <label><span data-i18n="view.implementation_shortfall.label.arrival_mid">Arrival mid</span>
                    <input id="is-am" type="number" step="0.01" min="0" value="${state.params.arrival_mid}" data-tip="view.implementation_shortfall.tip.arrival_mid"></label>
                <label><span data-i18n="view.implementation_shortfall.label.vwap_fill">VWAP fill</span>
                    <input id="is-vw" type="number" step="0.01" min="0" value="${state.params.vwap_fill}" data-tip="view.implementation_shortfall.tip.vwap_fill"></label>
                <label><span data-i18n="view.implementation_shortfall.label.final_mid">Final mid</span>
                    <input id="is-fm" type="number" step="0.01" min="0" value="${state.params.final_mid}" data-tip="view.implementation_shortfall.tip.final_mid"></label>
                <label><span data-i18n="view.implementation_shortfall.label.half_spread">½-spread @ decision</span>
                    <input id="is-hs" type="number" step="0.01" min="0" value="${state.params.half_spread_at_decision}" data-tip="view.implementation_shortfall.tip.half_spread"></label>
                <label><span data-i18n="view.implementation_shortfall.label.intended_qty">Intended qty</span>
                    <input id="is-iq" type="number" step="1" min="1" value="${state.params.intended_qty}" data-tip="view.implementation_shortfall.tip.intended_qty"></label>
                <label><span data-i18n="view.implementation_shortfall.label.filled_qty">Filled qty</span>
                    <input id="is-fq" type="number" step="1" min="0" value="${state.params.filled_qty}" data-tip="view.implementation_shortfall.tip.filled_qty"></label>
                <button data-i18n="view.implementation_shortfall.btn.analyze" id="is-run" class="primary" type="button" data-tip="view.implementation_shortfall.tip.run" data-shortcut="implementation_shortfall_run">Analyze</button>
            </div>
            <p data-i18n="view.implementation_shortfall.hint.buy_convention_a_positive_cost_means_the_trader_pa" class="muted">
                Buy convention: a positive $ cost means the trader paid up.
                Negative = captured liquidity / favorable drift. Total in bps
                normalizes against intended notional (qty × decision_mid).
            </p>
        </div>

        <div id="is-summary" class="cards"></div>

        <div class="chart-panel"><h2 data-i18n="view.implementation_shortfall.h2.cost_attribution">Cost attribution</h2>
            <div id="is-bars"></div>
        </div>

        <div class="chart-panel"><h2 data-i18n="view.implementation_shortfall.h2.backend_note">Backend note</h2>
            <div id="is-note" class="muted">—</div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.implementation_shortfall.h2.component_chart">Cost components ($)</h2>
            <div id="is-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.implementation_shortfall.h2.component_bps_chart">Cost components (bps of intended notional)</h2>
            <div id="is-bps-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.implementation_shortfall.hint.bps_chart" class="muted small">Same components normalized as basis points of intended notional (qty × decision_mid). Order-size-invariant; lets you compare today's slice against any historical execution on a level field.</p>
        </div>

        <div id="is-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('is-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });

    readInputs();
    void compute(tok);
}

function readInputs() {
    const get = id => document.getElementById(id).value;
    state.params = {
        direction:               get('is-dir'),
        decision_mid:            Number(get('is-dm')),
        arrival_mid:             Number(get('is-am')),
        vwap_fill:               Number(get('is-vw')),
        final_mid:               Number(get('is-fm')),
        half_spread_at_decision: Number(get('is-hs')),
        intended_qty:            Number(get('is-iq')),
        filled_qty:              Number(get('is-fq')),
    };
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.params);
    if (err) { showErr(err); showToast(t('view.implementation_shortfall.toast.invalid'), { level: 'warning' }); return; }
    let res;
    try {
        res = await api.microImplementationShortfall(buildBody(state.params));
        if (!res) throw new Error(t('view.implementation_shortfall.error.null'));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        showToast(t('view.implementation_shortfall.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(res);
    renderBars(res);
    renderComponentChart(res);
    renderComponentBpsChart(res);
    document.getElementById('is-note').textContent = res.note || '—';
    showToast(t('view.implementation_shortfall.toast.analyzed', { bps: (res.total_bps ?? 0).toFixed(2) }), { level: 'success' });
}

function renderComponentChart(report) {
    const el = document.getElementById('is-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const items = decompose(report);
    if (!items.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.implementation_shortfall.empty_chart">${esc(t('view.implementation_shortfall.empty_chart'))}</div>`;
        return;
    }
    const labels = items.map(it => it.label);
    const ys = items.map(it => Number(it.value));
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.implementation_shortfall.chart.component_idx') },
            { label: t('view.implementation_shortfall.chart.cost'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 16, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.implementation_shortfall.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, ys, zero], el);
}

function renderComponentBpsChart(report) {
    const el = document.getElementById('is-bps-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const items = decompose(report);
    const notional = (Number(state.params.intended_qty) || 0)
                   * (Number(state.params.decision_mid) || 0);
    if (!items.length || notional <= 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.implementation_shortfall.empty_bps_chart">${esc(t('view.implementation_shortfall.empty_bps_chart'))}</div>`;
        return;
    }
    const labels = items.map(it => it.label);
    const ys = items.map(it => Number(it.value) / notional * 10_000);
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.implementation_shortfall.chart.component_idx') },
            { label: t('view.implementation_shortfall.chart.cost_bps'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 16, fill: '#b86bff', stroke: '#b86bff' } },
            { label: t('view.implementation_shortfall.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, ys, zero], el);
}

function renderSummary(r) {
    const fill = fillKind(state.params.intended_qty, state.params.filled_qty);
    const fillPct = state.params.filled_qty / state.params.intended_qty;
    document.getElementById('is-summary').innerHTML = [
        card(t('view.implementation_shortfall.card.direction'),     state.params.direction.toUpperCase()),
        card(t('view.implementation_shortfall.card.fill'),          fill.toUpperCase() + ' · ' + fmtPct(fillPct), fill === 'full' ? 'pos' : 'neg'),
        card(t('view.implementation_shortfall.card.spread_d'),      fmtUSD(r.spread_cost),      costSignClass(r.spread_cost)),
        card(t('view.implementation_shortfall.card.timing_d'),      fmtUSD(r.timing_cost),      costSignClass(r.timing_cost)),
        card(t('view.implementation_shortfall.card.impact_d'),      fmtUSD(r.impact_cost),      costSignClass(r.impact_cost)),
        card(t('view.implementation_shortfall.card.opportunity_d'), fmtUSD(r.opportunity_cost), costSignClass(r.opportunity_cost)),
        card(t('view.implementation_shortfall.card.total_d'),       fmtUSD(r.total_dollars),    costSignClass(r.total_dollars)),
        card(t('view.implementation_shortfall.card.total_bps'),     fmtBps(r.total_bps),        costSignClass(r.total_bps)),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderBars(report) {
    const wrap = document.getElementById('is-bars');
    const items = decompose(report);
    const maxAbs = Math.max(...items.map(it => Math.abs(it.value)), 1e-9);
    // Horizontal divergent bars: 50% midline, left = negative (captured),
    // right = positive (paid). Color is a static CSS class (is-fill-<key>);
    // width/side is set after insertion via rAF so release-WebKit picks it up.
    wrap.innerHTML = items.map(it => `
        <div class="is-bar-row">
            <div class="is-bar-label">${esc(it.label)}</div>
            <div class="is-bar-track">
                <div class="is-bar-midline"></div>
                <div class="is-bar-fill is-fill-${esc(it.key)} is-fill-${it.value >= 0 ? 'pos' : 'neg'}"
                     data-bar-pct="${(Math.abs(it.value) / maxAbs * 50).toFixed(2)}"></div>
            </div>
            <div class="is-bar-value ${costSignClass(it.value)}">${esc(fmtUSD(it.value))} · ${esc(fmtPct(it.share))}</div>
        </div>
    `).join('');
    requestAnimationFrame(() => {
        wrap.querySelectorAll('.is-bar-fill').forEach(el => {
            const pct = Number(el.dataset.barPct);
            if (Number.isFinite(pct)) el.style.width = pct + '%';
        });
    });
}

function showErr(msg) {
    const el = document.getElementById('is-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('is-err').style.display = 'none'; }
