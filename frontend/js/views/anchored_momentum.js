// Anchored Momentum view — ROC vs a chosen anchor bar with optional WMA smoothing.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_SMOOTH,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    momentumBadge, summarize,
    makeDemoInput,
    fmtPctSigned, fmtUSD, fmtInt,
} from '../_anchored_momentum_inputs.js';

let state = { ...makeDemoInput('post-earnings-rally') };

export async function renderAnchoredMomentum(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.anch_mom.h1.title" class="view-title">// ANCHORED MOMENTUM</h1>

        <div class="chart-panel" data-context-scope="anchored-momentum">
            <h2 data-i18n="view.anch_mom.h2.closes">Closes
                <small data-i18n="view.anch_mom.h2.closes_hint" class="muted">(one per token; "NaN" tolerated; comments + blanks ignored)</small></h2>
            <textarea id="am-blob" rows="6"
                      data-tip="view.anch_mom.tip.closes"
                      placeholder="100.0&#10;100.5&#10;101.2&#10;...">${esc(closesToBlob(state.closes))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.anch_mom.label.anchor">Anchor bar index</span>
                    <input id="am-anchor" type="number" step="1" min="0" value="${state.anchor}"></label>
                <label><span data-i18n="view.anch_mom.label.smooth">Smooth period</span>
                    <input id="am-smooth" type="number" step="1" min="1" value="${state.smooth_period}"></label>
                <button data-i18n="view.anch_mom.btn.compute" id="am-run" class="primary"
                        data-tip="view.anch_mom.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.anch_mom.btn.demo_rally"    id="am-demo-rally"  class="secondary" type="button">Demo: post-earnings rally</button>
                <button data-i18n="view.anch_mom.btn.demo_crash"    id="am-demo-crash"  class="secondary" type="button">Demo: post-news crash</button>
                <button data-i18n="view.anch_mom.btn.demo_flat"     id="am-demo-flat"   class="secondary" type="button">Demo: flat after anchor</button>
                <button data-i18n="view.anch_mom.btn.demo_clipped"  id="am-demo-clip"   class="secondary" type="button">Demo: pre-anchor clipped</button>
                <button data-i18n="view.anch_mom.btn.demo_raw"      id="am-demo-raw"    class="secondary" type="button">Demo: raw only (smooth=1)</button>
                <button data-i18n="view.anch_mom.btn.demo_long"     id="am-demo-long"   class="secondary" type="button">Demo: long smoothing (10)</button>
                <button data-i18n="view.anch_mom.btn.demo_nan"      id="am-demo-nan"    class="secondary" type="button">Demo: NaN gap</button>
                <button data-i18n="view.anch_mom.btn.demo_fomc"     id="am-demo-fomc"   class="secondary" type="button">Demo: FOMC volatility</button>
            </div>
            <p data-i18n="view.anch_mom.hint.about" class="muted">raw_i = (close_i − close_anchor) / close_anchor for i ≥ anchor. Smoothed = WMA(raw, smooth_period) with linear weights 1..N. smooth_period=1 → raw series. NaN bars block any window they fall in.</p>
        </div>

        <div id="am-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.anch_mom.h2.chart">Smoothed anchored momentum %</h2>
            <div id="am-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.anch_mom.h2.table">Per-bar momentum (tail — last 30)</h2>
            <div id="am-table"></div>
        </div>

        <div id="am-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('am-blob').value   = closesToBlob(state.closes);
        document.getElementById('am-anchor').value = state.anchor;
        document.getElementById('am-smooth').value = state.smooth_period;
    };
    document.getElementById('am-demo-rally').addEventListener('click', () => { loadDemo('post-earnings-rally'); void compute(tok); });
    document.getElementById('am-demo-crash').addEventListener('click', () => { loadDemo('post-news-crash');     void compute(tok); });
    document.getElementById('am-demo-flat').addEventListener('click',  () => { loadDemo('flat-after-anchor');   void compute(tok); });
    document.getElementById('am-demo-clip').addEventListener('click',  () => { loadDemo('pre-anchor-clipped'); void compute(tok); });
    document.getElementById('am-demo-raw').addEventListener('click',   () => { loadDemo('raw-only');            void compute(tok); });
    document.getElementById('am-demo-long').addEventListener('click',  () => { loadDemo('long-smoothing');      void compute(tok); });
    document.getElementById('am-demo-nan').addEventListener('click',   () => { loadDemo('with-nan-gap');        void compute(tok); });
    document.getElementById('am-demo-fomc').addEventListener('click',  () => { loadDemo('fomc-volatile');       void compute(tok); });
    document.getElementById('am-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseClosesBlob(document.getElementById('am-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.anch_mom.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.closes = p.closes;
    const anchor = parseInt(document.getElementById('am-anchor').value, 10);
    const smooth = parseInt(document.getElementById('am-smooth').value, 10);
    state.anchor = Number.isInteger(anchor) && anchor >= 0 ? anchor : 0;
    state.smooth_period = Number.isInteger(smooth) && smooth >= 1 ? smooth : DEFAULT_SMOOTH;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.closes, state.anchor, state.smooth_period);
    renderSummary(local, true);
    renderChart(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.anlyAnchoredMomentum(buildBody(state));
    } catch (e) {
        showErr(`${t('view.anch_mom.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(resp);
    renderTable(resp);
}

function renderSummary(series, pending) {
    const local = localCompute(state.closes, state.anchor, state.smooth_period);
    const parityOk = series.length === local.length
        && series.every((v, i) => {
            if (v == null && local[i] == null) return true;
            if (v == null || local[i] == null) return false;
            return Math.abs(v - local[i]) < 1e-9;
        });
    const s = summarize(series);
    const badge = momentumBadge(s.last);
    const localTag = pending ? ` (${t('view.anch_mom.tag.local')})` : '';
    const anchorPrice = state.closes[state.anchor];
    document.getElementById('am-summary').innerHTML = [
        card(t('view.anch_mom.card.verdict'),    t(badge.key) + localTag, badge.cls),
        card(t('view.anch_mom.card.anchor_idx'), '#' + (state.anchor + 1)),
        card(t('view.anch_mom.card.anchor_px'),  fmtUSD(anchorPrice)),
        card(t('view.anch_mom.card.smooth'),     fmtInt(state.smooth_period)),
        card(t('view.anch_mom.card.bars'),       fmtInt(s.count)),
        card(t('view.anch_mom.card.populated'),  fmtInt(s.populated)),
        card(t('view.anch_mom.card.last'),       fmtPctSigned(s.last), badge.cls),
        card(t('view.anch_mom.card.mean'),       fmtPctSigned(s.mean)),
        card(t('view.anch_mom.card.min'),        fmtPctSigned(s.min)),
        card(t('view.anch_mom.card.max'),        fmtPctSigned(s.max)),
        card(t('view.anch_mom.card.parity'),
             parityOk ? t('view.anch_mom.tag.ok') : t('view.anch_mom.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(series) {
    if (!window.uPlot) return;
    const el = document.getElementById('am-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!series || series.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.anch_mom.empty">${esc(t('view.anch_mom.empty'))}</div>`;
        return;
    }
    const xs = series.map((_, i) => i);
    // Convert to pct points for nicer chart axis.
    const pct = series.map(v => v == null ? null : v * 100);
    // Anchor marker: single point at anchor index, y=0.
    const anchor_marker = series.map((_, i) => i === state.anchor ? 0 : null);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 340,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('chart.series.bar') },
            { label: t('chart.series.momentum_'), stroke: '#00e5ff', width: 1.5, points: { show: false } },
            { label: t('chart.series.anchor'),     stroke: '#ffd84a', width: 0,   points: { show: true, size: 8 } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60,
              values: (_u, splits) => splits.map(v => (v >= 0 ? '+' : '') + v.toFixed(1) + '%') },
        ],
        legend: { show: true },
    }, [xs, pct, anchor_marker], el);
}

function renderTable(series) {
    const wrap = document.getElementById('am-table');
    const n = series?.length || 0;
    if (n === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.anch_mom.empty">${esc(t('view.anch_mom.empty'))}</div>`;
        return;
    }
    const start = Math.max(0, n - 30);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.anch_mom.col.idx">#</th>
                <th data-i18n="view.anch_mom.col.close">Close</th>
                <th data-i18n="view.anch_mom.col.smoothed">Smoothed momentum</th>
                <th data-i18n="view.anch_mom.col.tag">Tag</th>
            </tr></thead>
            <tbody>
                ${Array.from({ length: n - start }, (_, k) => {
                    const i = start + k;
                    const v = series[i];
                    const cls = v == null ? '' : v > 0 ? 'pos' : v < 0 ? 'neg' : '';
                    const isAnchor = i === state.anchor;
                    const tag = isAnchor ? t('view.anch_mom.tag.anchor') : '';
                    return `<tr>
                        <td>${i + 1}${isAnchor ? ' ⚓' : ''}</td>
                        <td>${esc(fmtUSD(state.closes[i]))}</td>
                        <td class="${cls}">${esc(fmtPctSigned(v))}</td>
                        <td>${esc(tag)}</td>
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
    const el = document.getElementById('am-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('am-err').style.display = 'none'; }
