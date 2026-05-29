// Active Share view — Cremers & Petajisto (2009) portfolio vs benchmark
// distance metric.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseWeightsBlob, weightsToBlob, validateInputs, buildBody, localCompute,
    styleBadge, sumBadge, enrich, stanceLabelKey,
    makeDemoInput,
    fmtPct, fmtPctSigned, fmtNum, fmtInt,
} from '../_active_share_inputs.js';

let state = { ...makeDemoInput('cremers-canonical') };

export async function renderActiveShare(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.act_share.h1.title" class="view-title">// ACTIVE SHARE</h1>

        <div class="chart-panel" data-context-scope="active-share">
            <h2 data-i18n="view.act_share.h2.weights">Portfolio vs benchmark weights
                <small data-i18n="view.act_share.h2.weights_hint" class="muted">(per line: symbol portfolio_w benchmark_w — decimals or "40%" form)</small></h2>
            <textarea id="as-blob" rows="8"
                      data-tip="view.act_share.tip.weights"
                      placeholder="AAPL 0.30 0.07&#10;MSFT 0.25 0.07">${esc(weightsToBlob(state.weights))}</textarea>

            <div class="inline-form">
                <button data-i18n="view.act_share.btn.compute" id="as-run" class="primary"
                        data-tip="view.act_share.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.act_share.btn.demo_identical"  id="as-demo-id"     class="secondary" type="button">Demo: identical (AS=0)</button>
                <button data-i18n="view.act_share.btn.demo_disjoint"   id="as-demo-disj"   class="secondary" type="button">Demo: disjoint (AS=1)</button>
                <button data-i18n="view.act_share.btn.demo_canonical"  id="as-demo-can"    class="secondary" type="button">Demo: Cremers canonical 50%</button>
                <button data-i18n="view.act_share.btn.demo_closet"     id="as-demo-closet" class="secondary" type="button">Demo: closet indexer</button>
                <button data-i18n="view.act_share.btn.demo_active"     id="as-demo-active" class="secondary" type="button">Demo: highly active</button>
                <button data-i18n="view.act_share.btn.demo_sector"     id="as-demo-sect"   class="secondary" type="button">Demo: sector bet</button>
                <button data-i18n="view.act_share.btn.demo_short"      id="as-demo-short"  class="secondary" type="button">Demo: long-only with missing names</button>
                <button data-i18n="view.act_share.btn.demo_unnorm"     id="as-demo-unnorm" class="secondary" type="button">Demo: unnormalized weights</button>
            </div>
            <p data-i18n="view.act_share.hint.about" class="muted">AS = ½·Σ|w_port − w_bench|. Range [0, 1]: 0 = closet indexer, 1 = disjoint from benchmark. Cremers & Petajisto found AS ≥ 0.60 correlates with managers who actually outperform their benchmark net of fees.</p>
        </div>

        <div id="as-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.act_share.h2.contrib">Per-name contribution to active share (|Δw|)</h2>
            <div id="as-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.act_share.h2.table">Per-name breakdown</h2>
            <div id="as-table"></div>
        </div>

        <div id="as-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('as-blob').value = weightsToBlob(state.weights);
    };
    document.getElementById('as-demo-id').addEventListener('click',     () => { loadDemo('identical');         void compute(tok); });
    document.getElementById('as-demo-disj').addEventListener('click',   () => { loadDemo('disjoint');          void compute(tok); });
    document.getElementById('as-demo-can').addEventListener('click',    () => { loadDemo('cremers-canonical'); void compute(tok); });
    document.getElementById('as-demo-closet').addEventListener('click', () => { loadDemo('closet-indexer');    void compute(tok); });
    document.getElementById('as-demo-active').addEventListener('click', () => { loadDemo('highly-active');     void compute(tok); });
    document.getElementById('as-demo-sect').addEventListener('click',   () => { loadDemo('sector-bet');        void compute(tok); });
    document.getElementById('as-demo-short').addEventListener('click',  () => { loadDemo('short-bet');         void compute(tok); });
    document.getElementById('as-demo-unnorm').addEventListener('click', () => { loadDemo('unnormalized');      void compute(tok); });
    document.getElementById('as-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseWeightsBlob(document.getElementById('as-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.act_share.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.weights = p.weights;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.weights);
    if (!local) { showErr(t('view.act_share.err.degenerate')); return; }
    renderSummary(local, true);
    renderChart();
    renderTable();
    let resp;
    try {
        resp = await api.portfolioActiveShare(buildBody(state));
    } catch (e) {
        showErr(`${t('view.act_share.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) { showErr(t('view.act_share.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart();
    renderTable();
}

function renderSummary(report, pending) {
    const local = localCompute(state.weights);
    const parityOk = !!local
        && Math.abs(local.active_share - report.active_share) < 1e-9
        && local.n_overweights === report.n_overweights
        && local.n_underweights === report.n_underweights;
    const sBadge = styleBadge(report.active_share);
    const pSumBadge = sumBadge(report.portfolio_weight_sum);
    const bSumBadge = sumBadge(report.benchmark_weight_sum);
    const localTag = pending ? ` (${t('view.act_share.tag.local')})` : '';
    document.getElementById('as-summary').innerHTML = [
        card(t('view.act_share.card.verdict'),    t(sBadge.key) + localTag, sBadge.cls),
        card(t('view.act_share.card.score'),      fmtNum(report.active_share, 4), sBadge.cls),
        card(t('view.act_share.card.score_pct'),  fmtPct(report.active_share), sBadge.cls),
        card(t('view.act_share.card.n_names'),    fmtInt(report.n_names)),
        card(t('view.act_share.card.over'),       fmtInt(report.n_overweights),
             report.n_overweights > 0 ? 'pos' : ''),
        card(t('view.act_share.card.under'),      fmtInt(report.n_underweights),
             report.n_underweights > 0 ? 'neg' : ''),
        card(t('view.act_share.card.p_sum'),      fmtPct(report.portfolio_weight_sum), pSumBadge.cls),
        card(t('view.act_share.card.b_sum'),      fmtPct(report.benchmark_weight_sum), bSumBadge.cls),
        card(t('view.act_share.card.p_sum_v'),    t(pSumBadge.key), pSumBadge.cls),
        card(t('view.act_share.card.b_sum_v'),    t(bSumBadge.key), bSumBadge.cls),
        card(t('view.act_share.card.parity'),
             parityOk ? t('view.act_share.tag.ok') : t('view.act_share.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart() {
    if (!window.uPlot) return;
    const el = document.getElementById('as-chart');
    if (!el) return;
    el.innerHTML = '';
    const enriched = state.weights.map(enrich);
    enriched.sort((a, b) => b.abs_diff - a.abs_diff);
    if (enriched.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.act_share.empty">${esc(t('view.act_share.empty'))}</div>`;
        return;
    }
    const xs = enriched.map((_, i) => i);
    const diffsPct = enriched.map(r => r.diff * 100);   // signed pct points
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 340,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('chart.series.rank') },
            { label: 'Δw (pp)', stroke: '#00e5ff', width: 1.5, points: { show: true, size: 5 } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => {
                  const i = Math.trunc(v);
                  return i >= 0 && i < enriched.length ? enriched[i].symbol : '';
              }) },
            { stroke: '#aab', size: 60,
              values: (_u, splits) => splits.map(v => v.toFixed(0) + 'pp') },
        ],
        legend: { show: true },
    }, [xs, diffsPct], el);
}

function renderTable() {
    const wrap = document.getElementById('as-table');
    if (!state.weights || state.weights.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.act_share.empty">${esc(t('view.act_share.empty'))}</div>`;
        return;
    }
    const enriched = state.weights.map(enrich);
    enriched.sort((a, b) => b.abs_diff - a.abs_diff);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.act_share.col.rank">#</th>
                <th data-i18n="view.act_share.col.symbol">Symbol</th>
                <th data-i18n="view.act_share.col.port">Portfolio</th>
                <th data-i18n="view.act_share.col.bench">Benchmark</th>
                <th data-i18n="view.act_share.col.diff">Δ</th>
                <th data-i18n="view.act_share.col.abs_diff">|Δ|</th>
                <th data-i18n="view.act_share.col.stance">Stance</th>
            </tr></thead>
            <tbody>
                ${enriched.map((r, i) => {
                    const stanceCls = r.stance === 'over' ? 'pos' : r.stance === 'under' ? 'neg' : '';
                    return `<tr>
                        <td>${i + 1}</td>
                        <td><strong>${esc(r.symbol)}</strong></td>
                        <td>${esc(fmtPct(r.portfolio_weight))}</td>
                        <td>${esc(fmtPct(r.benchmark_weight))}</td>
                        <td class="${stanceCls}">${esc(fmtPctSigned(r.diff))}</td>
                        <td>${esc(fmtPct(r.abs_diff))}</td>
                        <td data-i18n="${esc(stanceLabelKey(r.stance))}" class="${stanceCls}">${esc(t(stanceLabelKey(r.stance)))}</td>
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
    const el = document.getElementById('as-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('as-err').style.display = 'none'; }
