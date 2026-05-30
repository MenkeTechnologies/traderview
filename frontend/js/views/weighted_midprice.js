// Weighted-midprice (Stoikov microprice) view — top-of-book imbalance
// → forecast of short-horizon midprice movement.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseQuotesBlob, quotesToBlob, validateInputs, buildBody,
    localSeries, summarize, imbalanceBadge,
    makeDemoInput,
    fmtUSD, fmtUSDSigned, fmtBps, fmtImb, fmtInt,
} from '../_weighted_midprice_inputs.js';

let state = { ...makeDemoInput('evolving-imbalance') };

export async function renderWeightedMidprice(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.wmp.h1.title" class="view-title">// WEIGHTED MIDPRICE</h1>

        <div class="chart-panel" data-context-scope="wmp">
            <h2 data-i18n="view.wmp.h2.quotes">Top-of-book quotes
                <small data-i18n="view.wmp.h2.quotes_hint" class="muted">(per line: bid_price bid_size ask_price ask_size)</small></h2>
            <textarea id="wmp-blob" rows="8"
                      data-tip="view.wmp.tip.quotes"
                      placeholder="100.00 100 100.10 100&#10;100.00 1000 100.10 100">${esc(quotesToBlob(state.quotes))}</textarea>

            <div class="inline-form">
                <button data-i18n="view.wmp.btn.compute" id="wmp-run" class="primary"
                        data-tip="view.wmp.tip.compute" data-shortcut="weighted_midprice_run" type="button">Compute microprice</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.wmp.btn.demo_balanced"  id="wmp-demo-bal"   class="secondary" type="button" data-tip="view.wmp.tip.demo_bal">Demo: balanced book</button>
                <button data-i18n="view.wmp.btn.demo_heavy_bid" id="wmp-demo-hb"    class="secondary" type="button" data-tip="view.wmp.tip.demo_hb">Demo: heavy bid (microp → ask)</button>
                <button data-i18n="view.wmp.btn.demo_heavy_ask" id="wmp-demo-ha"    class="secondary" type="button" data-tip="view.wmp.tip.demo_ha">Demo: heavy ask (microp → bid)</button>
                <button data-i18n="view.wmp.btn.demo_extreme_b" id="wmp-demo-eb"    class="secondary" type="button" data-tip="view.wmp.tip.demo_eb">Demo: extreme bid (1e6 vs 1)</button>
                <button data-i18n="view.wmp.btn.demo_extreme_a" id="wmp-demo-ea"    class="secondary" type="button" data-tip="view.wmp.tip.demo_ea">Demo: extreme ask (1 vs 1e6)</button>
                <button data-i18n="view.wmp.btn.demo_evolving"  id="wmp-demo-evo"   class="secondary" type="button" data-tip="view.wmp.tip.demo_evo">Demo: evolving imbalance</button>
                <button data-i18n="view.wmp.btn.demo_tight"     id="wmp-demo-tight" class="secondary" type="button" data-tip="view.wmp.tip.demo_tight">Demo: tight 1¢ spread</button>
                <button data-i18n="view.wmp.btn.demo_wide"      id="wmp-demo-wide"  class="secondary" type="button" data-tip="view.wmp.tip.demo_wide">Demo: wide 50¢ spread</button>
            </div>
            <p data-i18n="view.wmp.hint.about" class="muted">Stoikov (2017) microprice = (bid·ask_sz + ask·bid_sz) / (bid_sz + ask_sz). Biases toward the side with LESS size — the side likely to trade through soonest. Quote imbalance ∈ [−1, +1].</p>
        </div>

        <div id="wmp-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.wmp.h2.chart">Microprice deviation + imbalance over series</h2>
            <div id="wmp-chart" style="width:100%;height:320px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.wmp.h2.table">Per-snapshot breakdown</h2>
            <div id="wmp-table"></div>
        </div>

        <div id="wmp-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('wmp-blob').value = quotesToBlob(state.quotes);
    };
    document.getElementById('wmp-demo-bal').addEventListener('click',   () => { loadDemo('balanced');            void compute(tok); });
    document.getElementById('wmp-demo-hb').addEventListener('click',    () => { loadDemo('heavy-bid');           void compute(tok); });
    document.getElementById('wmp-demo-ha').addEventListener('click',    () => { loadDemo('heavy-ask');           void compute(tok); });
    document.getElementById('wmp-demo-eb').addEventListener('click',    () => { loadDemo('extreme-bid');         void compute(tok); });
    document.getElementById('wmp-demo-ea').addEventListener('click',    () => { loadDemo('extreme-ask');         void compute(tok); });
    document.getElementById('wmp-demo-evo').addEventListener('click',   () => { loadDemo('evolving-imbalance');  void compute(tok); });
    document.getElementById('wmp-demo-tight').addEventListener('click', () => { loadDemo('tight-spread');        void compute(tok); });
    document.getElementById('wmp-demo-wide').addEventListener('click',  () => { loadDemo('wide-spread');         void compute(tok); });
    document.getElementById('wmp-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseQuotesBlob(document.getElementById('wmp-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.wmp.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        showToast(t('view.wmp.toast.parse_error', { n: p.errors.length }), { level: 'warning' });
        return;
    }
    hideErr();
    state.quotes = p.quotes;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.wmp.toast.invalid'), { level: 'warning' }); return; }
    const local = localSeries(state.quotes);
    renderSummary(local, true);
    renderChart(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.microWeightedMidprice(buildBody(state));
    } catch (e) {
        showErr(`${t('view.wmp.err.api')}: ${e.message || e}`);
        showToast(t('view.wmp.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    const series = resp && resp.series ? resp.series : null;
    if (!series) {
        showErr(t('view.wmp.err.server_rejected'));
        showToast(t('view.wmp.toast.rejected'), { level: 'error' });
        return;
    }
    renderSummary(series, false);
    renderChart(series);
    renderTable(series);
    const last = series.length > 0 ? series[series.length - 1] : null;
    const imb = last ? Number(last.quote_imbalance) : 0;
    const imbStr = Number.isFinite(imb) ? imb.toFixed(2) : '—';
    const bias = imb > 0 ? 'BID-HEAVY' : imb < 0 ? 'ASK-HEAVY' : 'BALANCED';
    showToast(t('view.wmp.toast.computed', { n: series.length, imb: imbStr, bias }), { level: 'success' });
}

function renderSummary(series, pending) {
    const local = localSeries(state.quotes);
    const parityOk = series.length === local.length
        && series.every((r, i) => {
            if (r == null && local[i] == null) return true;
            if (r == null || local[i] == null) return false;
            return Math.abs(r.microprice - local[i].microprice) < 1e-9
                && Math.abs(r.quote_imbalance - local[i].quote_imbalance) < 1e-9;
        });
    const s = summarize(series);
    const last = series.length > 0 ? series[series.length - 1] : null;
    const lastBadge = imbalanceBadge(last);
    const localTag = pending ? ` (${t('view.wmp.tag.local')})` : '';
    document.getElementById('wmp-summary').innerHTML = [
        card(t('view.wmp.card.verdict'),     t(lastBadge.key) + localTag, lastBadge.cls),
        card(t('view.wmp.card.n'),           fmtInt(state.quotes.length)),
        card(t('view.wmp.card.valid'),       fmtInt(s.count)),
        card(t('view.wmp.card.last_micro'),  last ? fmtUSD(last.microprice) : '—'),
        card(t('view.wmp.card.last_mid'),    last ? fmtUSD(last.midpoint)   : '—'),
        card(t('view.wmp.card.last_dev'),    last ? fmtUSDSigned(last.microprice_minus_midpoint) : '—',
             last ? (last.microprice_minus_midpoint > 0 ? 'pos' : last.microprice_minus_midpoint < 0 ? 'neg' : '') : ''),
        card(t('view.wmp.card.last_imb'),    last ? fmtImb(last.quote_imbalance) : '—',
             last ? lastBadge.cls : ''),
        card(t('view.wmp.card.mean_dev'),    fmtUSDSigned(s.mean_dev)),
        card(t('view.wmp.card.max_abs_dev'), fmtUSD(s.max_abs_dev)),
        card(t('view.wmp.card.mean_spread'), fmtUSD(s.mean_spread)),
        card(t('view.wmp.card.last_rel'),    last ? fmtBps(last.relative_spread) : '—'),
        card(t('view.wmp.card.parity'),
             parityOk ? t('view.wmp.tag.ok') : t('view.wmp.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(series) {
    if (!window.uPlot) return;
    const el = document.getElementById('wmp-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!series || series.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.wmp.empty">${esc(t('view.wmp.empty'))}</div>`;
        return;
    }
    const xs = series.map((_, i) => i);
    const dev = series.map(r => r ? r.microprice_minus_midpoint : null);
    const imb = series.map(r => r ? r.quote_imbalance : null);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 320,
        scales: { x: {}, y: {}, y2: { auto: true } },
        series: [
            { label: t('chart.series.idx') },
            { label: t('view.weighted_midprice.series.dev'),       stroke: '#00e5ff', width: 1.5, points: { show: true, size: 4 } },
            { label: t('view.weighted_midprice.series.imbalance'), stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false }, scale: 'y2' },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 70 },
            { side: 1, stroke: '#aab', size: 50, scale: 'y2',
              values: (_u, splits) => splits.map(v => v.toFixed(2)) },
        ],
        legend: { show: true },
    }, [xs, dev, imb], el);
}

function renderTable(series) {
    const wrap = document.getElementById('wmp-table');
    if (!series || series.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.wmp.empty">${esc(t('view.wmp.empty'))}</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.wmp.col.idx">#</th>
                <th data-i18n="view.wmp.col.bid">Bid</th>
                <th data-i18n="view.wmp.col.bid_sz">Bid sz</th>
                <th data-i18n="view.wmp.col.ask">Ask</th>
                <th data-i18n="view.wmp.col.ask_sz">Ask sz</th>
                <th data-i18n="view.wmp.col.mid">Mid</th>
                <th data-i18n="view.wmp.col.micro">Microprice</th>
                <th data-i18n="view.wmp.col.dev">Dev</th>
                <th data-i18n="view.wmp.col.imb">Imbalance</th>
            </tr></thead>
            <tbody>
                ${series.map((r, i) => {
                    const q = state.quotes[i];
                    if (!r) return `<tr><td>${i + 1}</td><td colspan="8" class="muted">${esc(t('view.wmp.row_invalid'))}</td></tr>`;
                    const devCls = r.microprice_minus_midpoint > 0 ? 'pos' : r.microprice_minus_midpoint < 0 ? 'neg' : '';
                    const b = imbalanceBadge(r);
                    return `<tr>
                        <td>${i + 1}</td>
                        <td>${esc(fmtUSD(q.bid_price))}</td>
                        <td>${esc(fmtInt(q.bid_size))}</td>
                        <td>${esc(fmtUSD(q.ask_price))}</td>
                        <td>${esc(fmtInt(q.ask_size))}</td>
                        <td>${esc(fmtUSD(r.midpoint))}</td>
                        <td><strong>${esc(fmtUSD(r.microprice))}</strong></td>
                        <td class="${devCls}">${esc(fmtUSDSigned(r.microprice_minus_midpoint))}</td>
                        <td class="${b.cls}">${esc(fmtImb(r.quote_imbalance))}</td>
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
    const el = document.getElementById('wmp-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('wmp-err').style.display = 'none'; }
