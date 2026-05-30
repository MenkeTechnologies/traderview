// Roll (1984) effective bid-ask spread view — covariance-based
// implicit-spread estimator from trade prices alone.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_WINDOW,
    parsePricesBlob, pricesToBlob, validateInputs, buildBody, localCompute,
    summarize, liquidityBadge, regimeBadge, spreadToBps,
    makeDemoInput,
    fmtUSD, fmtBps, fmtNum, fmtInt, fmtPct,
} from '../_roll_spread_inputs.js';

let state = { ...makeDemoInput('random-bounce') };

export async function renderRollSpread(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.roll.h1.title" class="view-title">// ROLL SPREAD (1984)</h1>

        <div class="chart-panel" data-context-scope="roll-spread">
            <h2 data-i18n="view.roll.h2.prices">Trade prices
                <small data-i18n="view.roll.h2.prices_hint" class="muted">(one price per token; comma/whitespace separated; # comments ignored)</small></h2>
            <textarea id="rs-blob" rows="6"
                      data-tip="view.roll.tip.prices"
                      placeholder="100.05, 99.95, 100.05, 99.95, ...">${esc(pricesToBlob(state.prices))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.roll.label.window">Window (bars)</span>
                    <input id="rs-window" type="number" step="1" min="3" value="${state.window}" data-tip="view.roll.tip.window"></label>
                <button data-i18n="view.roll.btn.compute" id="rs-run" class="primary"
                        data-tip="view.roll.tip.compute" data-shortcut="roll_spread_run" type="button">Compute spread</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.roll.btn.demo_random"  id="rs-demo-random" class="secondary" type="button" data-tip="view.roll.tip.demo_random">Demo: random bid/ask bounce (10 bp)</button>
                <button data-i18n="view.roll.btn.demo_tight"   id="rs-demo-tight"  class="secondary" type="button" data-tip="view.roll.tip.demo_tight">Demo: tight bounce (1 bp)</button>
                <button data-i18n="view.roll.btn.demo_wide"    id="rs-demo-wide"   class="secondary" type="button" data-tip="view.roll.tip.demo_wide">Demo: wide bounce (50 bp)</button>
                <button data-i18n="view.roll.btn.demo_trend"   id="rs-demo-trend"  class="secondary" type="button" data-tip="view.roll.tip.demo_trend">Demo: trending (spread=0)</button>
                <button data-i18n="view.roll.btn.demo_flat"    id="rs-demo-flat"   class="secondary" type="button" data-tip="view.roll.tip.demo_flat">Demo: flat market</button>
                <button data-i18n="view.roll.btn.demo_regime"  id="rs-demo-regime" class="secondary" type="button" data-tip="view.roll.tip.demo_regime">Demo: regime shift (bounce → trend)</button>
                <button data-i18n="view.roll.btn.demo_spotty"  id="rs-demo-spotty" class="secondary" type="button" data-tip="view.roll.tip.demo_spotty">Demo: spotty NaN gaps</button>
                <button data-i18n="view.roll.btn.demo_huge"    id="rs-demo-huge"   class="secondary" type="button" data-tip="view.roll.tip.demo_huge">Demo: window > n (all-null)</button>
            </div>
            <p data-i18n="view.roll.hint.about" class="muted">spread = 2·√(−cov(Δp_t, Δp_{t−1})). Negative serial covariance from bid/ask bouncing reveals the implicit spread; positive cov (trending) collapses estimate to 0. No quotes needed — works from trade prints alone.</p>
        </div>

        <div id="rs-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.roll.h2.chart">Rolling spread estimate</h2>
            <div id="rs-chart" style="width:100%;height:300px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.roll.h2.table">Per-bar spread (tail — last 30 bars)</h2>
            <div id="rs-table"></div>
        </div>

        <div id="rs-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('rs-blob').value = pricesToBlob(state.prices);
        document.getElementById('rs-window').value = state.window;
    };
    document.getElementById('rs-demo-random').addEventListener('click', () => { loadDemo('random-bounce'); void compute(tok); });
    document.getElementById('rs-demo-tight').addEventListener('click',  () => { loadDemo('tight-bounce');  void compute(tok); });
    document.getElementById('rs-demo-wide').addEventListener('click',   () => { loadDemo('wide-bounce');   void compute(tok); });
    document.getElementById('rs-demo-trend').addEventListener('click',  () => { loadDemo('trending');      void compute(tok); });
    document.getElementById('rs-demo-flat').addEventListener('click',   () => { loadDemo('flat');          void compute(tok); });
    document.getElementById('rs-demo-regime').addEventListener('click', () => { loadDemo('regime-shift');  void compute(tok); });
    document.getElementById('rs-demo-spotty').addEventListener('click', () => { loadDemo('spotty-nan');    void compute(tok); });
    document.getElementById('rs-demo-huge').addEventListener('click',   () => { loadDemo('huge-window');   void compute(tok); });
    document.getElementById('rs-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parsePricesBlob(document.getElementById('rs-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.roll.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        showToast(t('view.roll.toast.parse_error', { n: p.errors.length }), { level: 'warning' });
        return;
    }
    hideErr();
    state.prices = p.prices;
    const w = parseInt(document.getElementById('rs-window').value, 10);
    state.window = Number.isInteger(w) && w >= 3 ? w : DEFAULT_WINDOW;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.roll.toast.invalid'), { level: 'warning' }); return; }
    const local = localCompute(state.prices, state.window);
    renderSummary(local, true);
    renderChart(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.microRollSpread(buildBody(state));
    } catch (e) {
        showErr(`${t('view.roll.err.api')}: ${e.message || e}`);
        showToast(t('view.roll.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(resp);
    renderTable(resp);
    const s = summarize(resp);
    const lastPrice = state.prices.length > 0 ? state.prices[state.prices.length - 1] : NaN;
    const lastBps = spreadToBps(s.last, lastPrice);
    const bpsStr = Number.isFinite(lastBps) ? lastBps.toFixed(2) : '—';
    showToast(t('view.roll.toast.computed', { bps: bpsStr }), { level: 'success' });
}

function renderSummary(series, pending) {
    const local = localCompute(state.prices, state.window);
    const parityOk = series.length === local.length
        && series.every((v, i) => {
            if (v == null && local[i] == null) return true;
            if (v == null || local[i] == null) return false;
            return Math.abs(v - local[i]) < 1e-9;
        });
    const s = summarize(series);
    const lastPrice = state.prices.length > 0 ? state.prices[state.prices.length - 1] : NaN;
    const lastBadge = liquidityBadge(s.last, lastPrice);
    const regBadge  = regimeBadge(series, state.prices.length);
    const lastBps   = spreadToBps(s.last, lastPrice);
    const meanBps   = spreadToBps(s.mean, lastPrice);
    const localTag  = pending ? ` (${t('view.roll.tag.local')})` : '';
    document.getElementById('rs-summary').innerHTML = [
        card(t('view.roll.card.verdict'),    t(lastBadge.key) + localTag, lastBadge.cls),
        card(t('view.roll.card.regime'),     t(regBadge.key), regBadge.cls),
        card(t('view.roll.card.bars'),       fmtInt(state.prices.length)),
        card(t('view.roll.card.window'),     fmtInt(state.window)),
        card(t('view.roll.card.populated'),  fmtInt(s.count)),
        card(t('view.roll.card.last_spread'), fmtUSD(s.last), lastBadge.cls),
        card(t('view.roll.card.last_bps'),   fmtBps(lastBps), lastBadge.cls),
        card(t('view.roll.card.mean_spread'), fmtUSD(s.mean)),
        card(t('view.roll.card.mean_bps'),   fmtBps(meanBps)),
        card(t('view.roll.card.max_spread'), fmtUSD(s.max)),
        card(t('view.roll.card.zero_frac'),
             s.count > 0 ? fmtPct(s.zero_count / s.count) : '—'),
        card(t('view.roll.card.parity'),
             parityOk ? t('view.roll.tag.ok') : t('view.roll.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(series) {
    if (!window.uPlot) return;
    const el = document.getElementById('rs-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!series || series.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.roll.empty">${esc(t('view.roll.empty'))}</div>`;
        return;
    }
    const xs = series.map((_, i) => i);
    const ys = series.map(v => v == null ? null : v);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 300,
        scales: { x: {}, y: {} },
        series: [
            { label: t('chart.series.bar') },
            { label: t('chart.series.spread'), stroke: '#00e5ff', width: 1.5, points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 70 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderTable(series) {
    const wrap = document.getElementById('rs-table');
    if (!series || series.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.roll.empty">${esc(t('view.roll.empty'))}</div>`;
        return;
    }
    const start = Math.max(0, series.length - 30);
    const lastPriceForBps = state.prices.length > 0 ? state.prices[state.prices.length - 1] : NaN;
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.roll.col.bar">Bar</th>
                <th data-i18n="view.roll.col.price">Price</th>
                <th data-i18n="view.roll.col.spread">Spread ($)</th>
                <th data-i18n="view.roll.col.bps">Spread (bps)</th>
                <th data-i18n="view.roll.col.depth">Depth</th>
            </tr></thead>
            <tbody>
                ${series.slice(start).map((v, idx) => {
                    const i = start + idx;
                    const p = state.prices[i];
                    const bps = spreadToBps(v, lastPriceForBps);
                    const b = liquidityBadge(v, lastPriceForBps);
                    return `<tr>
                        <td>${i}</td>
                        <td>${esc(fmtUSD(p, 4))}</td>
                        <td class="${b.cls}">${esc(fmtNum(v))}</td>
                        <td class="${b.cls}">${esc(fmtBps(bps))}</td>
                        <td data-i18n="${esc(b.key)}" class="${b.cls}">${esc(t(b.key))}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('rs-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('rs-err').style.display = 'none'; }
