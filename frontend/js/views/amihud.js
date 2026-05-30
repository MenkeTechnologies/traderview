// Amihud (2002) Illiquidity Ratio view — price impact per dollar traded.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_PERIOD,
    parsePairsBlob, pairsToBlob, validateInputs, buildBody, localCompute,
    summarize, liquidityBadge, trendBadge,
    makeDemoInput,
    fmtAmihud, fmtPct, fmtDV, fmtInt,
} from '../_amihud_inputs.js';

let state = { ...makeDemoInput('mid-cap') };

export async function renderAmihud(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.amihud.h1.title" class="view-title">// AMIHUD ILLIQUIDITY</h1>

        <div class="chart-panel" data-context-scope="amihud">
            <h2 data-i18n="view.amihud.h2.pairs">Returns + dollar volumes
                <small data-i18n="view.amihud.h2.pairs_hint" class="muted">(per line: return dollar_volume — return as 0.012 or "1.2%")</small></h2>
            <textarea id="am-blob" rows="6"
                      data-tip="view.amihud.tip.pairs"
                      placeholder="0.012 100000000&#10;-0.005 90000000">${esc(pairsToBlob(state.returns, state.dollar_volumes))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.amihud.label.period">Rolling period</span>
                    <input id="am-period" type="number" step="1" min="1" value="${state.period}"></label>
                <button data-i18n="view.amihud.btn.compute" id="am-run" class="primary"
                        data-tip="view.amihud.tip.compute" type="button">Compute Amihud</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.amihud.btn.demo_large"  id="am-demo-large"  class="secondary" type="button">Demo: large-cap ($10B/d)</button>
                <button data-i18n="view.amihud.btn.demo_mid"    id="am-demo-mid"    class="secondary" type="button">Demo: mid-cap ($100M/d)</button>
                <button data-i18n="view.amihud.btn.demo_small"  id="am-demo-small"  class="secondary" type="button">Demo: small-cap ($1M/d)</button>
                <button data-i18n="view.amihud.btn.demo_penny"  id="am-demo-penny"  class="secondary" type="button">Demo: penny ($50k/d)</button>
                <button data-i18n="view.amihud.btn.demo_shock"  id="am-demo-shock"  class="secondary" type="button">Demo: liquidity shock</button>
                <button data-i18n="view.amihud.btn.demo_recover" id="am-demo-recover" class="secondary" type="button">Demo: recovery (illiquid → liquid)</button>
                <button data-i18n="view.amihud.btn.demo_spotty" id="am-demo-spotty" class="secondary" type="button">Demo: spotty volume (NaN + zero)</button>
                <button data-i18n="view.amihud.btn.demo_short"  id="am-demo-short"  class="secondary" type="button">Demo: short period (5)</button>
            </div>
            <p data-i18n="view.amihud.hint.about" class="muted">illiq_t = |r_t|/dollar_vol_t · 10⁶. Higher = less liquid. Mean over rolling period bars. S&amp;P 500 large-caps ≈ 0.0001; penny stocks ≈ 1.0+.</p>
        </div>

        <div id="am-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.amihud.h2.chart">Rolling Amihud (log-scale friendly)</h2>
            <div id="am-chart" style="width:100%;height:320px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.amihud.h2.dvol_chart">Dollar volume per bar (liquidity input)</h2>
            <div id="am-dvol-chart" style="width:100%;height:220px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.amihud.h2.table">Per-bar Amihud (tail — last 30)</h2>
            <div id="am-table"></div>
        </div>

        <div id="am-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('am-blob').value   = pairsToBlob(state.returns, state.dollar_volumes);
        document.getElementById('am-period').value = state.period;
    };
    document.getElementById('am-demo-large').addEventListener('click',   () => { loadDemo('large-cap');         void compute(tok); });
    document.getElementById('am-demo-mid').addEventListener('click',     () => { loadDemo('mid-cap');           void compute(tok); });
    document.getElementById('am-demo-small').addEventListener('click',   () => { loadDemo('small-cap');         void compute(tok); });
    document.getElementById('am-demo-penny').addEventListener('click',   () => { loadDemo('penny-illiquid');    void compute(tok); });
    document.getElementById('am-demo-shock').addEventListener('click',   () => { loadDemo('liquidity-shock');   void compute(tok); });
    document.getElementById('am-demo-recover').addEventListener('click', () => { loadDemo('recovery');          void compute(tok); });
    document.getElementById('am-demo-spotty').addEventListener('click',  () => { loadDemo('spotty-volume');     void compute(tok); });
    document.getElementById('am-demo-short').addEventListener('click',   () => { loadDemo('short-period');      void compute(tok); });
    document.getElementById('am-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parsePairsBlob(document.getElementById('am-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.amihud.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.returns = p.returns;
    state.dollar_volumes = p.dollar_volumes;
    const period = parseInt(document.getElementById('am-period').value, 10);
    state.period = Number.isInteger(period) && period >= 1 ? period : DEFAULT_PERIOD;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.returns, state.dollar_volumes, state.period);
    renderSummary(local, true);
    renderChart(local);
    renderDvolChart();
    renderTable(local);
    let resp;
    try {
        resp = await api.anlyAmihudIlliquidity(buildBody(state));
    } catch (e) {
        showErr(`${t('view.amihud.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(resp);
    renderDvolChart();
    renderTable(resp);
}

function renderSummary(series, pending) {
    const local = localCompute(state.returns, state.dollar_volumes, state.period);
    const parityOk = series.length === local.length
        && series.every((v, i) => {
            if (v == null && local[i] == null) return true;
            if (v == null || local[i] == null) return false;
            return Math.abs(v - local[i]) < 1e-9;
        });
    const s = summarize(series);
    const lBadge = liquidityBadge(s.last);
    const tBadge = trendBadge(series);
    const localTag = pending ? ` (${t('view.amihud.tag.local')})` : '';
    document.getElementById('am-summary').innerHTML = [
        card(t('view.amihud.card.verdict'),    t(lBadge.key) + localTag, lBadge.cls),
        card(t('view.amihud.card.trend'),      t(tBadge.key), tBadge.cls),
        card(t('view.amihud.card.bars'),       fmtInt(state.returns.length)),
        card(t('view.amihud.card.period'),     fmtInt(state.period)),
        card(t('view.amihud.card.populated'),  fmtInt(s.populated)),
        card(t('view.amihud.card.last'),       fmtAmihud(s.last), lBadge.cls),
        card(t('view.amihud.card.mean'),       fmtAmihud(s.mean)),
        card(t('view.amihud.card.min'),        fmtAmihud(s.min)),
        card(t('view.amihud.card.max'),        fmtAmihud(s.max)),
        card(t('view.amihud.card.parity'),
             parityOk ? t('view.amihud.tag.ok') : t('view.amihud.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(series) {
    if (!window.uPlot) return;
    const el = document.getElementById('am-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!series || series.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.amihud.empty">${esc(t('view.amihud.empty'))}</div>`;
        return;
    }
    const xs = series.map((_, i) => i);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 320,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('chart.series.bar') },
            { label: t('chart.series.amihud'),
              stroke: '#00e5ff',
              width: 1.5,
              points: { show: false },
              value: (_u, v) => v == null ? '—' : fmtAmihud(v) },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 80,
              values: (_u, splits) => splits.map(v => fmtAmihud(v)) },
        ],
        legend: { show: true },
    }, [xs, series], el);
}

function renderDvolChart() {
    if (!window.uPlot) return;
    const el = document.getElementById('am-dvol-chart');
    if (!el) return;
    el.innerHTML = '';
    const dvols = state.dollar_volumes;
    if (!Array.isArray(dvols) || dvols.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.amihud.empty_dvol">${esc(t('view.amihud.empty_dvol'))}</div>`;
        return;
    }
    const xs = dvols.map((_, i) => i);
    const ys = dvols.map(v => Number.isFinite(v) ? v : null);
    const mean = ys.reduce((s, v) => s + (v == null ? 0 : v), 0) / Math.max(1, ys.filter(v => v != null).length);
    const meanLine = xs.map(() => mean);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('chart.series.bar') },
            { label: t('view.amihud.chart.dvol'),
              stroke: '#7af0a8', width: 1.5,
              points: { show: true, size: 4, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.amihud.chart.dvol_mean'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, ys, meanLine], el);
}

function renderTable(series) {
    const wrap = document.getElementById('am-table');
    const n = series?.length || 0;
    if (n === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.amihud.empty">${esc(t('view.amihud.empty'))}</div>`;
        return;
    }
    const start = Math.max(0, n - 30);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.amihud.col.idx">#</th>
                <th data-i18n="view.amihud.col.return">Return</th>
                <th data-i18n="view.amihud.col.dv">Dollar vol</th>
                <th data-i18n="view.amihud.col.amihud">Amihud</th>
                <th data-i18n="view.amihud.col.verdict">Verdict</th>
            </tr></thead>
            <tbody>
                ${Array.from({ length: n - start }, (_, k) => {
                    const i = start + k;
                    const v = series[i];
                    const b = liquidityBadge(v);
                    return `<tr>
                        <td>${i + 1}</td>
                        <td>${esc(fmtPct(state.returns[i]))}</td>
                        <td>${esc(fmtDV(state.dollar_volumes[i]))}</td>
                        <td class="${b.cls}">${esc(fmtAmihud(v))}</td>
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
    const el = document.getElementById('am-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('am-err').style.display = 'none'; }
