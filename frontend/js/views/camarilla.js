// Camarilla Pivot Points view — 8 intraday S/R levels from prior session.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseInputBlob, inputToBlob, validateInputs, buildBody, localCompute,
    zoneBadge, ruleBadge, widthBadge, nearestLevelInfo,
    makeDemoInput,
    fmtPrice, fmtPriceSigned, fmtPct, } from '../_camarilla_inputs.js';

let state = { ...makeDemoInput('standard-range') };

export async function renderCamarilla(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.cam.h1.title" class="view-title">// CAMARILLA PIVOTS</h1>

        <div class="chart-panel" data-context-scope="camarilla-pivots">
            <h2 data-i18n="view.cam.h2.session">Prior session OHLC
                <small data-i18n="view.cam.h2.session_hint" class="muted">(HIGH LOW CLOSE — optional CURRENT_PRICE for zone verdict)</small></h2>
            <textarea id="cm-blob" rows="3"
                      data-tip="view.cam.tip.session"
                      placeholder="110 100 105 106">${esc(inputToBlob(state))}</textarea>

            <div class="inline-form">
                <button data-i18n="view.cam.btn.compute" id="cm-run" class="primary"
                        data-tip="view.cam.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.cam.btn.demo_std"   id="cm-d1" class="secondary" type="button">Demo: standard range</button>
                <button data-i18n="view.cam.btn.demo_brkup" id="cm-d2" class="secondary" type="button">Demo: breakout long</button>
                <button data-i18n="view.cam.btn.demo_brkdn" id="cm-d3" class="secondary" type="button">Demo: breakdown short</button>
                <button data-i18n="view.cam.btn.demo_h3"    id="cm-d4" class="secondary" type="button">Demo: H3 short reversal</button>
                <button data-i18n="view.cam.btn.demo_l3"    id="cm-d5" class="secondary" type="button">Demo: L3 long reversal</button>
                <button data-i18n="view.cam.btn.demo_tight" id="cm-d6" class="secondary" type="button">Demo: tight range</button>
                <button data-i18n="view.cam.btn.demo_wide"  id="cm-d7" class="secondary" type="button">Demo: wide range</button>
                <button data-i18n="view.cam.btn.demo_flat"  id="cm-d8" class="secondary" type="button">Demo: flat session</button>
            </div>
            <p data-i18n="view.cam.hint.about" class="muted">8 intraday S/R levels derived from prior session H/L/C × 1.1 constant. Trade rules: L3/H3 = reversal zones (long L3, short H3); H4/L4 = breakout / breakdown triggers. Symmetric around close. Pivot = (H + L + C) / 3.</p>
        </div>

        <div id="cm-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.cam.h2.levels">Levels (high → low)</h2>
            <div id="cm-levels"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.cam.h2.levels_chart">Levels ladder vs current price</h2>
            <div id="cm-chart" style="width:100%;height:300px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.cam.h2.dist_chart">Signed distance to each level (current − level)</h2>
            <div id="cm-dist-chart" style="width:100%;height:220px"></div>
        </div>

        <div id="cm-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('cm-blob').value = inputToBlob(state);
    };
    document.getElementById('cm-d1').addEventListener('click', () => { loadDemo('standard-range');  void compute(tok); });
    document.getElementById('cm-d2').addEventListener('click', () => { loadDemo('breakout-long');   void compute(tok); });
    document.getElementById('cm-d3').addEventListener('click', () => { loadDemo('breakdown-short'); void compute(tok); });
    document.getElementById('cm-d4').addEventListener('click', () => { loadDemo('short-reversal'); void compute(tok); });
    document.getElementById('cm-d5').addEventListener('click', () => { loadDemo('long-reversal');   void compute(tok); });
    document.getElementById('cm-d6').addEventListener('click', () => { loadDemo('tight-range');     void compute(tok); });
    document.getElementById('cm-d7').addEventListener('click', () => { loadDemo('wide-range');      void compute(tok); });
    document.getElementById('cm-d8').addEventListener('click', () => { loadDemo('flat-session');    void compute(tok); });
    document.getElementById('cm-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseInputBlob(document.getElementById('cm-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.cam.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.session = p.session;
    state.current_price = p.current_price;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.session);
    if (!local) { showErr(t('view.cam.err.degenerate')); return; }
    renderSummary(local, true);
    renderLevels(local);
    renderLadderChart(local);
    renderDistChart(local);
    let resp;
    try {
        resp = await api.anlyCamarillaPivots(buildBody(state));
    } catch (e) {
        showErr(`${t('view.cam.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) { showErr(t('view.cam.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderLevels(resp);
    renderLadderChart(resp);
    renderDistChart(resp);
}

function renderSummary(levels, pending) {
    const local = localCompute(state.session);
    const parityOk = !!local
        && Math.abs(local.h4 - levels.h4) < 1e-9
        && Math.abs(local.h3 - levels.h3) < 1e-9
        && Math.abs(local.h2 - levels.h2) < 1e-9
        && Math.abs(local.h1 - levels.h1) < 1e-9
        && Math.abs(local.pivot - levels.pivot) < 1e-9
        && Math.abs(local.l1 - levels.l1) < 1e-9
        && Math.abs(local.l2 - levels.l2) < 1e-9
        && Math.abs(local.l3 - levels.l3) < 1e-9
        && Math.abs(local.l4 - levels.l4) < 1e-9;
    const cp = state.current_price != null ? state.current_price : state.session.close;
    const zBadge = zoneBadge(levels, cp);
    const rBadge = ruleBadge(levels, cp);
    const wBadge = widthBadge(levels);
    const near = nearestLevelInfo(levels, cp);
    const localTag = pending ? ` (${t('view.cam.tag.local')})` : '';
    document.getElementById('cm-summary').innerHTML = [
        card(t('view.cam.card.zone'),    t(zBadge.key) + localTag, zBadge.cls),
        card(t('view.cam.card.rule'),    t(rBadge.key), rBadge.cls),
        card(t('view.cam.card.width'),   t(wBadge.key), wBadge.cls),
        card(t('view.cam.card.current'), fmtPrice(cp, 2)),
        card(t('view.cam.card.pivot'),   fmtPrice(levels.pivot, 4)),
        card(t('view.cam.card.nearest_name'), near.name || '—'),
        card(t('view.cam.card.nearest_val'),  fmtPrice(near.value, 4)),
        card(t('view.cam.card.distance'),     fmtPriceSigned(near.distance, 4)),
        card(t('view.cam.card.distance_pct'), fmtPct(near.distance_pct)),
        card(t('view.cam.card.h_range'),
             fmtPrice(state.session.high - state.session.low, 4)),
        card(t('view.cam.card.parity'),
             parityOk ? t('view.cam.tag.ok') : t('view.cam.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderLevels(levels) {
    const wrap = document.getElementById('cm-levels');
    const cp = state.current_price != null ? state.current_price : state.session.close;
    const arr = [
        ['H4', levels.h4, 'view.cam.lvl.h4', 'pos'],
        ['H3', levels.h3, 'view.cam.lvl.h3', 'pos'],
        ['H2', levels.h2, 'view.cam.lvl.h2', 'pos'],
        ['H1', levels.h1, 'view.cam.lvl.h1', ''],
        ['Pivot', levels.pivot, 'view.cam.lvl.pivot', ''],
        ['L1', levels.l1, 'view.cam.lvl.l1', ''],
        ['L2', levels.l2, 'view.cam.lvl.l2', 'neg'],
        ['L3', levels.l3, 'view.cam.lvl.l3', 'neg'],
        ['L4', levels.l4, 'view.cam.lvl.l4', 'neg'],
    ];
    const rows = arr.map(([code, value, key, cls]) => {
        const distance = cp - value;
        const here = Math.abs(distance) < (state.session.high - state.session.low) * 0.005;
        return `<tr>
            <td data-i18n="${key}" class="${cls}">${esc(code)}</td>
            <td>${esc(fmtPrice(value, 4))}</td>
            <td class="${distance > 0 ? 'pos' : distance < 0 ? 'neg' : ''}">${esc(fmtPriceSigned(distance, 4))}</td>
            <td>${value !== 0 ? esc(fmtPct(distance / Math.abs(value))) : '—'}</td>
            <td>${here ? '◀' : ''}</td>
        </tr>`;
    }).join('');
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.cam.col.level">Level</th>
                <th data-i18n="view.cam.col.value">Price</th>
                <th data-i18n="view.cam.col.distance">Distance</th>
                <th data-i18n="view.cam.col.distance_pct">Δ%</th>
                <th data-i18n="view.cam.col.here">Here</th>
            </tr></thead>
            <tbody>${rows}</tbody>
        </table>
    `;
}

function renderLadderChart(levels) {
    const el = document.getElementById('cm-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const labels = [
        t('view.cam.lvl.h4'), t('view.cam.lvl.h3'), t('view.cam.lvl.h2'), t('view.cam.lvl.h1'),
        t('view.cam.lvl.pivot'),
        t('view.cam.lvl.l1'), t('view.cam.lvl.l2'), t('view.cam.lvl.l3'), t('view.cam.lvl.l4'),
    ];
    const values = [
        levels.h4, levels.h3, levels.h2, levels.h1,
        levels.pivot,
        levels.l1, levels.l2, levels.l3, levels.l4,
    ].map(v => Number.isFinite(v) ? v : null);
    if (values.every(v => v == null)) {
        el.innerHTML = `<div class="muted" data-i18n="view.cam.empty_chart">${esc(t('view.cam.empty_chart'))}</div>`;
        return;
    }
    const xs = labels.map((_, i) => i + 1);
    const cp = state.current_price != null ? state.current_price : state.session?.close;
    const cpLine = Number.isFinite(cp) ? xs.map(() => cp) : null;
    const series = [
        { label: t('view.cam.chart.level_idx') },
        { label: t('view.cam.chart.price'),
          stroke: '#00e5ff', width: 0,
          points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
    ];
    const data = [xs, values];
    if (cpLine) {
        series.push({ label: t('view.cam.chart.current'),
                      stroke: '#ffd84a', width: 1.5, dash: [4, 4],
                      points: { show: false } });
        data.push(cpLine);
    }
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 280,
        scales: { x: { time: false,}, y: { auto: true } },
        series,
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, data, el);
}

function renderDistChart(levels) {
    const el = document.getElementById('cm-dist-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const cp = state.current_price != null ? state.current_price : state.session?.close;
    if (!Number.isFinite(cp)) {
        el.innerHTML = `<div class="muted" data-i18n="view.cam.empty_dist_chart">${esc(t('view.cam.empty_dist_chart'))}</div>`;
        return;
    }
    const labels = [
        t('view.cam.lvl.h4'), t('view.cam.lvl.h3'), t('view.cam.lvl.h2'), t('view.cam.lvl.h1'),
        t('view.cam.lvl.pivot'),
        t('view.cam.lvl.l1'), t('view.cam.lvl.l2'), t('view.cam.lvl.l3'), t('view.cam.lvl.l4'),
    ];
    const values = [
        levels.h4, levels.h3, levels.h2, levels.h1,
        levels.pivot,
        levels.l1, levels.l2, levels.l3, levels.l4,
    ];
    const xs = labels.map((_, i) => i + 1);
    const dist = values.map(v => Number.isFinite(v) ? cp - v : null);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.cam.chart.level_idx') },
            { label: t('view.cam.chart.distance'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.cam.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, dist, zero], el);
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('cm-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('cm-err').style.display = 'none'; }
