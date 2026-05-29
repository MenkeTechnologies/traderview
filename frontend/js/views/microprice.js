// Microprice (Stoikov) calculator — quote-imbalance-adjusted fair mid.
//
// The user enters a single L1 snapshot (bid, ask, bid_size, ask_size).
// We show:
//   * Backend's microprice + midpoint + imbalance + bias_bps cards.
//   * An imbalance-sweep chart showing how the microprice moves across
//     all possible queue imbalances, with the current point marked.
//
// Use cases:
//   * Market makers: "is my quote at the fair mid or being adversely
//     selected because of the imbalance?"
//   * Liquidity takers: "where will the next tick likely print —
//     midpoint or microprice?"

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    buildBody, validateQuote, microprice as localMicroprice,
    imbalanceSweep, fmtPrice, fmtBps, fmtImbalance,
} from '../_microprice_inputs.js';

import { t } from '../i18n.js';
const DEFAULTS = { bid: 100.00, ask: 100.05, bid_size: 1500, ask_size: 400 };

let state = { quote: { ...DEFAULTS } };

export async function renderMicroprice(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.microprice.h1.microprice" class="view-title">// MICROPRICE</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.microprice.h2.l1_quote">L1 quote</h2>
            <div class="inline-form">
                <label><span data-i18n="view.microprice.label.bid">Bid</span>
                    <input id="mp-bid"      type="number" step="any" min="0" value="${state.quote.bid}"></label>
                <label><span data-i18n="view.microprice.label.ask">Ask</span>
                    <input id="mp-ask"      type="number" step="any" min="0" value="${state.quote.ask}"></label>
                <label><span data-i18n="view.microprice.label.bid_size">Bid size</span>
                    <input id="mp-bid-sz" type="number" step="1"   min="0" value="${state.quote.bid_size}"></label>
                <label><span data-i18n="view.microprice.label.ask_size">Ask size</span>
                    <input id="mp-ask-sz" type="number" step="1"   min="0" value="${state.quote.ask_size}"></label>
                <button data-i18n="view.microprice.btn.compute" id="mp-run" class="primary" type="button">Compute</button>
            </div>
            <p data-i18n="view.microprice.hint.microprice_bid_ask_size_total_ask_bid_size_total_w" class="muted">
                Microprice = bid · (ask_size / total) + ask · (bid_size / total). When the bid
                queue dwarfs the ask queue, the next print is likely to lift the offer →
                microprice biases toward the ask.
            </p>
        </div>

        <div id="mp-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.microprice.h2.imbalance_sweep">Imbalance sweep</h2>
            <div id="mp-chart" style="width:100%;height:300px"></div>
            <p data-i18n="view.microprice.hint.cyan_line_microprice_across_every_possible_imbalan" class="muted">
                Cyan line: microprice across every possible imbalance, given the current
                bid/ask spread. Orange marker: your current snapshot. The line interpolates
                linearly from bid (pure-ask-size queue, imbalance=0) to ask (pure-bid-size,
                imbalance=1).
            </p>
        </div>

        <div id="mp-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('mp-run').addEventListener('click', () => {
        readInputs();
        void compute(mount, tok);
    });
    void fmt;
}

function readInputs() {
    const get = id => document.getElementById(id).value;
    state.quote = {
        bid: Number(get('mp-bid')),
        ask: Number(get('mp-ask')),
        bid_size: Number(get('mp-bid-sz')),
        ask_size: Number(get('mp-ask-sz')),
    };
}

async function compute(mount, tok) {
    hideErr();
    const err = validateQuote(state.quote);
    if (err) { showErr(err); return; }

    // Local-first preview so the cards update instantly even if the
    // backend round-trip stalls. Backend response overrides on success.
    const total = state.quote.bid_size + state.quote.ask_size;
    const imbalance = state.quote.bid_size / total;
    const mp = localMicroprice(state.quote.bid, state.quote.ask, state.quote.bid_size, state.quote.ask_size);
    const midpoint = 0.5 * (state.quote.bid + state.quote.ask);
    const biasBps = midpoint > 0 ? ((mp - midpoint) / midpoint) * 10000 : 0;
    renderSummary({ microprice: mp, midpoint, imbalance, bias_bps: biasBps }, /*fromBackend=*/false);
    renderChart({ microprice: mp });

    let res;
    try {
        res = await api.anlyMicropriceStoikov(buildBody(state.quote));
        if (!Array.isArray(res) || res.length !== 1 || res[0] == null) {
            throw new Error(t('view.microprice.error.null'));
        }
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(res[0], /*fromBackend=*/true);
    renderChart(res[0]);
}

function renderSummary(bar, fromBackend) {
    const biasCls = bar.bias_bps > 0 ? 'pos' : (bar.bias_bps < 0 ? 'neg' : '');
    document.getElementById('mp-summary').innerHTML = [
        card(t('view.microprice.card.microprice'), fmtPrice(bar.microprice), '',
            `<div class="vc-row"><span class="muted" data-i18n="view.microprice.row.source">source</span>
                <strong>${esc(t(fromBackend ? 'view.microprice.source.backend' : 'view.microprice.source.local'))}</strong></div>`),
        card(t('view.microprice.card.midpoint'), fmtPrice(bar.midpoint), '',
            `<div class="vc-row"><span class="muted" data-i18n="view.microprice.row.spread">spread</span>
                <strong>${fmtPrice(state.quote.ask - state.quote.bid)}</strong></div>`),
        card(t('view.microprice.card.imbalance'), fmtImbalance(bar.imbalance), '',
            `<div class="vc-row"><span class="muted">${esc(t(bar.imbalance > 0.5 ? 'view.microprice.tilt.bid_heavy' : (bar.imbalance < 0.5 ? 'view.microprice.tilt.ask_heavy' : 'view.microprice.tilt.balanced')))}</span>
                <strong>${(bar.imbalance * 100).toFixed(1)}% bid / ${((1 - bar.imbalance) * 100).toFixed(1)}% ask</strong></div>`),
        card(t('view.microprice.card.bias_vs_midpoint'), fmtBps(bar.bias_bps), biasCls,
            `<div class="vc-row"><span class="muted" data-i18n="view.microprice.row.interp">interp</span>
                <strong>${biasInterp(bar.bias_bps)}</strong></div>`),
    ].join('');
}

function card(label, value, valueCls, body) {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${valueCls}">${esc(value)}</div>
        <div class="value mp-summary-value">${body}</div>
    </div>`;
}

function biasInterp(bps) {
    if (!Number.isFinite(bps)) return '—';
    const abs = Math.abs(bps);
    if (abs < 0.5) return 'flat (balanced queue)';
    return bps > 0
        ? 'leans toward ask (bid-side liquidity heavier)'
        : 'leans toward bid (ask-side liquidity heavier)';
}

function renderChart(bar) {
    const el = document.getElementById('mp-chart');
    if (!window.uPlot) { el.textContent = t('common.error.uplot_not_loaded'); return; }
    el.innerHTML = '';

    const { xs, ys } = imbalanceSweep(state.quote.bid, state.quote.ask, 101);
    if (xs.length === 0) {
        el.innerHTML = `<div class="boot">${esc(t('view.microprice.empty.need_bid_ask'))}</div>`;
        return;
    }
    // User's current point as a 1-element marker series (rendered as a
    // single point that uPlot will tooltip).
    const total = state.quote.bid_size + state.quote.ask_size;
    const currentImb = total > 0 ? state.quote.bid_size / total : 0.5;
    // Align the marker to the nearest x-tick so uPlot draws it on the curve.
    const markerYs = xs.map(x => Math.abs(x - currentImb) < (1 / (xs.length - 1) / 2) ? bar.microprice : null);

    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 300,
        scales: { x: {}, y: {} },
        series: [
            { label: 'imbalance' },
            { label: 'microprice', stroke: '#00e5ff', width: 2, points: { show: false } },
            { label: 'current', stroke: '#ff9f1a', width: 0,
              points: { show: true, size: 12, stroke: '#ff9f1a', fill: '#ff9f1a' } },
        ],
        axes: [
            { stroke: '#aab',
              values: (_, ticks) => ticks.map(t => `${(t * 100).toFixed(0)}%`) },
            { stroke: '#aab' },
        ],
    }, [xs, ys, markerYs], el);
}

function showErr(msg) {
    const el = document.getElementById('mp-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('mp-err').style.display = 'none'; }
