// Round-number levels view — emits round-price S/R levels in a price
// window, weighted by "roundness," with nearest above/below + ATR distance.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    WEIGHTS, MAX_INTEGER_SCAN,
    validateInputs, buildBody, localDetect,
    weightBadge, weightLabelKey, pinningBadge,
    makeDemoInput, fmtUSD, fmtUSDSigned, fmtAtrs,
} from '../_round_levels_inputs.js';

let state = makeDemoInput('aapl-near-180');

export async function renderRoundLevels(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.round_levels.h1.title" class="view-title">// ROUND-NUMBER LEVELS</h1>

        <div class="chart-panel" data-context-scope="round-levels">
            <h2 data-i18n="view.round_levels.h2.inputs">Inputs</h2>
            <div class="inline-form">
                <label><span data-i18n="view.round_levels.label.price">Current price ($)</span>
                    <input id="rl-price" type="number" step="any" min="0" value="${state.current_price}" data-tip="view.round_levels.tip.price"></label>
                <label><span data-i18n="view.round_levels.label.atr">ATR ($, optional)</span>
                    <input id="rl-atr" type="number" step="any" min="0"
                           placeholder="leave blank to skip ATR-distance" data-i18n-placeholder="view.round_levels.placeholder.atr_blank"
                           value="${state.atr == null ? '' : state.atr}" data-tip="view.round_levels.tip.atr"></label>
                <label><span data-i18n="view.round_levels.label.window">Window ($ either side)</span>
                    <input id="rl-window" type="number" step="any" min="0" value="${state.config.window}" data-tip="view.round_levels.tip.window"></label>
                <label><span data-i18n="view.round_levels.label.min_weight">Min weight</span>
                    <select id="rl-min-weight" data-tip="view.round_levels.tip.min_weight">
                        ${WEIGHTS.map(w => `<option value="${w}" ${w === state.config.min_weight ? 'selected' : ''}
                            data-i18n="${weightLabelKey(w)}">${esc(t(weightLabelKey(w)))}</option>`).join('')}
                    </select></label>
                <button data-i18n="view.round_levels.btn.detect" id="rl-run" class="primary"
                        data-tip="view.round_levels.tip.detect" data-shortcut="round_levels_run" type="button">Detect levels</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.round_levels.btn.demo_aapl"   id="rl-demo-aapl"   class="secondary" type="button" data-tip="view.round_levels.tip.demo_aapl">Demo: AAPL ~180</button>
                <button data-i18n="view.round_levels.btn.demo_spy"    id="rl-demo-spy"    class="secondary" type="button" data-tip="view.round_levels.tip.demo_spy">Demo: SPY ~500</button>
                <button data-i18n="view.round_levels.btn.demo_tsla"   id="rl-demo-tsla"   class="secondary" type="button" data-tip="view.round_levels.tip.demo_tsla">Demo: TSLA ~250 (medium+)</button>
                <button data-i18n="view.round_levels.btn.demo_btc"    id="rl-demo-btc"    class="secondary" type="button" data-tip="view.round_levels.tip.demo_btc">Demo: BTC ~100k (major-only)</button>
                <button data-i18n="view.round_levels.btn.demo_penny"  id="rl-demo-penny"  class="secondary" type="button" data-tip="view.round_levels.tip.demo_penny">Demo: penny ~3</button>
                <button data-i18n="view.round_levels.btn.demo_pinned" id="rl-demo-pinned" class="secondary" type="button" data-tip="view.round_levels.tip.demo_pinned">Demo: pinned at $100</button>
                <button data-i18n="view.round_levels.btn.demo_major"  id="rl-demo-major"  class="secondary" type="button" data-tip="view.round_levels.tip.demo_major">Demo: major-only ($175 ±100)</button>
                <button data-i18n="view.round_levels.btn.demo_noatr"  id="rl-demo-noatr"  class="secondary" type="button" data-tip="view.round_levels.tip.demo_noatr">Demo: no ATR ($250)</button>
            </div>
            <p data-i18n="view.round_levels.hint.about" class="muted">Major = ÷1000, ÷500, ÷100. Medium = ÷50, ÷25. Minor = any other integer. Window > ${MAX_INTEGER_SCAN.toLocaleString()} integers short-circuits to empty (memory guard).</p>
        </div>

        <div id="rl-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.round_levels.h2.levels">Detected levels (sorted by distance from current price)</h2>
            <div id="rl-table"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.round_levels.h2.levels_chart">Levels by price (colored by weight)</h2>
            <div id="rl-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.round_levels.h2.dist_chart">Distance to current price ($) per level</h2>
            <div id="rl-dist-chart" style="width:100%;height:200px"></div>
        </div>

        <div id="rl-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('rl-price').value      = state.current_price;
        document.getElementById('rl-atr').value        = state.atr == null ? '' : state.atr;
        document.getElementById('rl-window').value     = state.config.window;
        document.getElementById('rl-min-weight').value = state.config.min_weight;
    };
    document.getElementById('rl-demo-aapl').addEventListener('click',   () => { loadDemo('aapl-near-180');  void compute(tok); });
    document.getElementById('rl-demo-spy').addEventListener('click',    () => { loadDemo('spy-near-500');   void compute(tok); });
    document.getElementById('rl-demo-tsla').addEventListener('click',   () => { loadDemo('tsla-near-250');  void compute(tok); });
    document.getElementById('rl-demo-btc').addEventListener('click',    () => { loadDemo('btc-near-100k');  void compute(tok); });
    document.getElementById('rl-demo-penny').addEventListener('click',  () => { loadDemo('penny-near-3');   void compute(tok); });
    document.getElementById('rl-demo-pinned').addEventListener('click', () => { loadDemo('pinned-at-100'); void compute(tok); });
    document.getElementById('rl-demo-major').addEventListener('click',  () => { loadDemo('major-only');    void compute(tok); });
    document.getElementById('rl-demo-noatr').addEventListener('click',  () => { loadDemo('no-atr');        void compute(tok); });
    document.getElementById('rl-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const atrRaw = document.getElementById('rl-atr').value;
    state.current_price    = Number(document.getElementById('rl-price').value);
    state.atr              = atrRaw === '' ? null : Number(atrRaw);
    state.config.window    = Number(document.getElementById('rl-window').value);
    state.config.min_weight = document.getElementById('rl-min-weight').value;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.round_levels.toast.invalid'), { level: 'warning' }); return; }
    const local = localDetect(state.current_price, state.atr, state.config);
    renderSummary(local, true);
    renderTable(local);
    renderLevelsChart(local);
    renderDistChart(local);
    let resp;
    try {
        resp = await api.chartsRoundLevels(buildBody(state));
    } catch (e) {
        showErr(`${t('view.round_levels.err.api')}: ${e.message || e}`);
        showToast(t('view.round_levels.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderTable(resp);
    renderLevelsChart(resp);
    renderDistChart(resp);
    const n = Array.isArray(resp.levels) ? resp.levels.length : 0;
    const pinned = !!resp.is_pinned;
    const level = pinned ? 'warning' : 'success';
    showToast(t('view.round_levels.toast.detected', { n, pinned: pinned ? 'PINNED' : 'free' }), { level });
}

function renderDistChart(report) {
    const el = document.getElementById('rl-dist-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (report?.levels || []).filter(l => Number.isFinite(Number(l.distance)));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.round_levels.empty_dist_chart">${esc(t('view.round_levels.empty_dist_chart'))}</div>`;
        return;
    }
    rows.sort((a, b) => Math.abs(a.distance) - Math.abs(b.distance));
    const labels = rows.map(l => `$${Number(l.price).toFixed(2)}`);
    const xs = labels.map((_, i) => i + 1);
    const aboveY = rows.map(l => Number(l.distance) > 0 ? Number(l.distance) : null);
    const belowY = rows.map(l => Number(l.distance) < 0 ? Number(l.distance) : null);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.round_levels.chart.level_px') },
            { label: t('view.round_levels.chart.above_cur'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.round_levels.chart.below_cur'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.round_levels.chart.current'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 56 },
        ],
        legend: { show: true },
    }, [xs, aboveY, belowY, zero], el);
}

function renderSummary(report, pending) {
    const local = localDetect(state.current_price, state.atr, state.config);
    const badge = pinningBadge(report.nearest_above, report.nearest_below, state.current_price);
    const parityOk = report.levels.length === local.levels.length
                  && ((report.nearest_above && local.nearest_above && report.nearest_above.price === local.nearest_above.price)
                      || (!report.nearest_above && !local.nearest_above))
                  && ((report.nearest_below && local.nearest_below && report.nearest_below.price === local.nearest_below.price)
                      || (!report.nearest_below && !local.nearest_below));
    const localTag = pending ? ` (${t('view.round_levels.tag.local')})` : '';
    const majorCount  = report.levels.filter(l => l.weight === 'major').length;
    const mediumCount = report.levels.filter(l => l.weight === 'medium').length;
    const minorCount  = report.levels.filter(l => l.weight === 'minor').length;
    const above = report.nearest_above;
    const below = report.nearest_below;
    document.getElementById('rl-summary').innerHTML = [
        card(t('view.round_levels.card.verdict'),     t(badge.key) + localTag, badge.cls),
        card(t('view.round_levels.card.current'),     fmtUSD(state.current_price)),
        card(t('view.round_levels.card.count'),       String(report.levels.length)),
        card(t('view.round_levels.card.major'),       String(majorCount),  majorCount  > 0 ? 'neg' : ''),
        card(t('view.round_levels.card.medium'),      String(mediumCount)),
        card(t('view.round_levels.card.minor'),       String(minorCount)),
        card(t('view.round_levels.card.above'),
             above ? `${fmtUSD(above.price)}  ${fmtUSDSigned(above.distance)}` : t('view.round_levels.tag.none'),
             above ? 'pos' : ''),
        card(t('view.round_levels.card.above_atrs'),
             above ? fmtAtrs(above.distance_atrs) : '—'),
        card(t('view.round_levels.card.below'),
             below ? `${fmtUSD(below.price)}  ${fmtUSDSigned(below.distance)}` : t('view.round_levels.tag.none'),
             below ? 'neg' : ''),
        card(t('view.round_levels.card.below_atrs'),
             below ? fmtAtrs(below.distance_atrs) : '—'),
        card(t('view.round_levels.card.parity'),
             parityOk ? t('view.round_levels.tag.ok') : t('view.round_levels.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderLevelsChart(report) {
    const el = document.getElementById('rl-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (report?.levels || []).filter(l => Number.isFinite(Number(l.price)));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.round_levels.empty_chart">${esc(t('view.round_levels.empty_chart'))}</div>`;
        return;
    }
    rows.sort((a, b) => Number(a.price) - Number(b.price));
    const xs = rows.map((_, i) => i + 1);
    const major  = rows.map(l => l.weight === 'major'  ? Number(l.price) : null);
    const medium = rows.map(l => l.weight === 'medium' ? Number(l.price) : null);
    const minor  = rows.map(l => l.weight === 'minor'  ? Number(l.price) : null);
    const cur    = xs.map(() => Number(state.current_price));
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.round_levels.chart.idx') },
            { label: t('view.round_levels.chart.major'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 14, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.round_levels.chart.medium'),
              stroke: '#ffd84a', width: 0,
              points: { show: true, size: 12, fill: '#ffd84a', stroke: '#ffd84a' } },
            { label: t('view.round_levels.chart.minor'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 10, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.round_levels.chart.current'),
              stroke: '#7af0a8', width: 1.2, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [ { stroke: '#aab', size: 28 }, { stroke: '#aab', size: 56 } ],
        legend: { show: true },
    }, [xs, major, medium, minor, cur], el);
}

function renderTable(report) {
    const wrap = document.getElementById('rl-table');
    if (!report.levels || report.levels.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.round_levels.empty">${esc(t('view.round_levels.empty'))}</div>`;
        return;
    }
    const sorted = [...report.levels].sort((a, b) => Math.abs(a.distance) - Math.abs(b.distance));
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.round_levels.col.price">Price</th>
                <th data-i18n="view.round_levels.col.weight">Weight</th>
                <th data-i18n="view.round_levels.col.side">Side</th>
                <th data-i18n="view.round_levels.col.distance">Distance</th>
                <th data-i18n="view.round_levels.col.distance_atrs">Distance (ATRs)</th>
            </tr></thead>
            <tbody>
                ${sorted.map(l => {
                    const badge = weightBadge(l.weight);
                    const sideKey = l.distance > 0 ? 'view.round_levels.side.above'
                                  : l.distance < 0 ? 'view.round_levels.side.below'
                                  : 'view.round_levels.side.at';
                    const sideCls = l.distance > 0 ? 'pos' : l.distance < 0 ? 'neg' : '';
                    return `<tr>
                        <td><strong>${esc(fmtUSD(l.price))}</strong></td>
                        <td data-i18n="${esc(weightLabelKey(l.weight))}" class="${badge.cls}">${esc(t(badge.key))}</td>
                        <td data-i18n="${esc(sideKey)}" class="${sideCls}">${esc(t(sideKey))}</td>
                        <td class="${sideCls}">${esc(fmtUSDSigned(l.distance))}</td>
                        <td class="${sideCls}">${esc(fmtAtrs(l.distance_atrs))}</td>
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
    const el = document.getElementById('rl-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('rl-err').style.display = 'none'; }
