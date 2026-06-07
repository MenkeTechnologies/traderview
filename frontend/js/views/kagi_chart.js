// Kagi chart view — directional price-action lines with reversal thresholds.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    KINDS,
    parseCloses, validateInputs, buildBody, localCompute,
    classifyYangYin, summarize, trendBadge, linesToPolyline,
    makeDemoInput,
    fmtUSD, fmtMove, fmtInt, fmtPct,
    dirLabelKey, yangYinLabelKey,
} from '../_kagi_chart_inputs.js';

let state = { ...makeDemoInput('breakout') };

export async function renderKagiChart(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.kagi.h1.title" class="view-title">// KAGI CHART</h1>

        <div class="chart-panel" data-context-scope="kagi">
            <h2 data-i18n="view.kagi.h2.closes">Closes
                <small data-i18n="view.kagi.h2.closes_hint" class="muted">(comma/whitespace separated; # comments ignored)</small></h2>
            <textarea id="kg-blob" rows="6"
                      data-tip="view.kagi.tip.closes"
                      placeholder="100, 101, 102, 100.5, 99, ...">${esc(state.closes.join(', '))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.kagi.label.reversal">Reversal</span>
                    <input id="kg-rev" type="number" step="0.01" min="0" value="${state.reversal}" data-tip="view.kagi.tip.reversal"></label>
                <label><span data-i18n="view.kagi.label.kind">Kind</span>
                    <select id="kg-kind" data-tip="view.kagi.tip.kind">
                        ${KINDS.map(k => `<option value="${k}" ${k === state.kind ? 'selected' : ''}
                            data-i18n="view.kagi.kind.${k}">${esc(k)}</option>`).join('')}
                    </select></label>
                <button data-i18n="view.kagi.btn.compute" id="kg-run" class="primary"
                        data-tip="view.kagi.tip.compute" data-shortcut="kagi_run" type="button">Build Kagi</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.kagi.btn.demo_uptrend"   id="kg-demo-up"      class="secondary" type="button" data-tip="view.kagi.tip.demo_up">Demo: pure uptrend</button>
                <button data-i18n="view.kagi.btn.demo_downtrend" id="kg-demo-down"    class="secondary" type="button" data-tip="view.kagi.tip.demo_down">Demo: pure downtrend</button>
                <button data-i18n="view.kagi.btn.demo_choppy"    id="kg-demo-choppy"  class="secondary" type="button" data-tip="view.kagi.tip.demo_choppy">Demo: choppy</button>
                <button data-i18n="view.kagi.btn.demo_breakout"  id="kg-demo-bo"      class="secondary" type="button" data-tip="view.kagi.tip.demo_bo">Demo: breakout from flat</button>
                <button data-i18n="view.kagi.btn.demo_flat"      id="kg-demo-flat"    class="secondary" type="button" data-tip="view.kagi.tip.demo_flat">Demo: flat (no reversal)</button>
                <button data-i18n="view.kagi.btn.demo_pct"       id="kg-demo-pct"     class="secondary" type="button" data-tip="view.kagi.tip.demo_pct">Demo: pct reversal</button>
                <button data-i18n="view.kagi.btn.demo_storm"     id="kg-demo-storm"   class="secondary" type="button" data-tip="view.kagi.tip.demo_storm">Demo: reversal storm</button>
                <button data-i18n="view.kagi.btn.demo_bull"      id="kg-demo-bull"    class="secondary" type="button" data-tip="view.kagi.tip.demo_bull">Demo: gentle bull</button>
            </div>
            <p data-i18n="view.kagi.hint.about" class="muted">Direction holds until price reverses by ≥ reversal against the running extreme. Yang (thick) = up line crossing prior peak. Yin (thin) = down line crossing prior trough. Time axis is line-count, not bars — characteristic of Kagi.</p>
        </div>

        <div id="kg-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.kagi.h2.chart">Kagi line plot</h2>
            <div id="kg-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.kagi.h2.moves_chart">Per-line signed move (up positive, down negative)</h2>
            <div id="kg-moves-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.kagi.hint.moves" class="muted small">Bar magnitude per kagi line, signed by direction. Reveals the line-level strength distribution that the cumulative polyline above obscures by stacking. Yellow dashed = zero reference.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.kagi.h2.table">Kagi lines</h2>
            <div id="kg-table"></div>
        </div>

        <div id="kg-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('kg-blob').value = state.closes.join(', ');
        document.getElementById('kg-rev').value  = state.reversal;
        document.getElementById('kg-kind').value = state.kind;
    };
    document.getElementById('kg-demo-up').addEventListener('click',     () => { loadDemo('uptrend');        void compute(tok); });
    document.getElementById('kg-demo-down').addEventListener('click',   () => { loadDemo('downtrend');      void compute(tok); });
    document.getElementById('kg-demo-choppy').addEventListener('click', () => { loadDemo('choppy');         void compute(tok); });
    document.getElementById('kg-demo-bo').addEventListener('click',     () => { loadDemo('breakout');       void compute(tok); });
    document.getElementById('kg-demo-flat').addEventListener('click',   () => { loadDemo('flat');           void compute(tok); });
    document.getElementById('kg-demo-pct').addEventListener('click',    () => { loadDemo('pct-reversal');   void compute(tok); });
    document.getElementById('kg-demo-storm').addEventListener('click',  () => { loadDemo('reversal-storm'); void compute(tok); });
    document.getElementById('kg-demo-bull').addEventListener('click',   () => { loadDemo('gentle-bull');    void compute(tok); });
    document.getElementById('kg-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseCloses(document.getElementById('kg-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.kagi.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        showToast(t('view.kagi.toast.parse_error', { n: p.errors.length }), { level: 'warning' });
        return;
    }
    hideErr();
    state.closes   = p.closes;
    state.reversal = Number(document.getElementById('kg-rev').value);
    state.kind     = document.getElementById('kg-kind').value;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.kagi.toast.invalid'), { level: 'warning' }); return; }
    const local = localCompute(state.closes, state.reversal, state.kind);
    renderSummary(local, true);
    renderChart(local);
    renderMovesChart(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.chartsKagi(buildBody(state));
    } catch (e) {
        showErr(`${t('view.kagi.err.api')}: ${e.message || e}`);
        showToast(t('view.kagi.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(resp);
    renderMovesChart(resp);
    renderTable(resp);
    const lines = Array.isArray(resp) ? resp.length : 0;
    const last = lines > 0 ? resp[lines - 1] : null;
    const dir = last ? String(last.direction) : '—';
    const level = dir === 'up' ? 'success' : dir === 'down' ? 'warning' : 'info';
    showToast(t('view.kagi.toast.built', { lines, dir }), { level });
}

function renderSummary(lines, pending) {
    const local = localCompute(state.closes, state.reversal, state.kind);
    const parityOk = lines.length === local.length
        && lines.every((l, i) => l.direction === local[i].direction
            && Math.abs(l.anchor_price - local[i].anchor_price) < 1e-9
            && Math.abs(l.end_price   - local[i].end_price)   < 1e-9);
    const badge = trendBadge(lines);
    const s = summarize(lines);
    const localTag = pending ? ` (${t('view.kagi.tag.local')})` : '';
    const threshDisplay = state.kind === 'pct'
        ? fmtPct(state.reversal)
        : fmtUSD(state.reversal);
    document.getElementById('kg-summary').innerHTML = [
        card(t('view.kagi.card.verdict'),   t(badge.key) + localTag, badge.cls),
        card(t('view.kagi.card.threshold'), threshDisplay),
        card(t('view.kagi.card.bars'),      fmtInt(state.closes.length)),
        card(t('view.kagi.card.lines'),     fmtInt(s.count)),
        card(t('view.kagi.card.ups'),       fmtInt(s.ups), s.ups > s.downs ? 'pos' : ''),
        card(t('view.kagi.card.downs'),     fmtInt(s.downs), s.downs > s.ups ? 'neg' : ''),
        card(t('view.kagi.card.avg_up'),    fmtUSD(s.avg_up)),
        card(t('view.kagi.card.avg_down'),  fmtUSD(s.avg_down)),
        card(t('view.kagi.card.last_dir'),
             s.last_dir ? t(dirLabelKey(s.last_dir)) : '—',
             s.last_dir === 'Up' ? 'pos' : s.last_dir === 'Down' ? 'neg' : ''),
        card(t('view.kagi.card.parity'),
             parityOk ? t('view.kagi.tag.ok') : t('view.kagi.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(lines) {
    if (!window.uPlot) return;
    const el = document.getElementById('kg-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!lines || lines.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.kagi.empty">${esc(t('view.kagi.empty'))}</div>`;
        return;
    }
    const { xs, ys } = linesToPolyline(lines);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 340,
        scales: { x: { time: false,}, y: {} },
        series: [
            { label: t('chart.series.line') },
            { label: t('chart.series.kagi'), stroke: '#00e5ff', width: 1.5, points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => '#' + Math.trunc(v)) },
            { stroke: '#aab', size: 70 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderMovesChart(lines) {
    const el = document.getElementById('kg-moves-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!lines || lines.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.kagi.empty_moves_chart">${esc(t('view.kagi.empty_moves_chart'))}</div>`;
        return;
    }
    const xs = lines.map((_, i) => i + 1);
    const moves = lines.map(l => Number(l.end_price) - Number(l.anchor_price));
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('chart.series.line') },
            { label: t('view.kagi.chart.signed_move'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 10, fill: '#b86bff', stroke: '#b86bff' } },
            { label: t('view.kagi.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => '#' + Math.trunc(v)) },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, moves, zero], el);
}

function renderTable(lines) {
    const wrap = document.getElementById('kg-table');
    if (!lines || lines.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.kagi.empty">${esc(t('view.kagi.empty'))}</div>`;
        return;
    }
    const yy = classifyYangYin(lines);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.kagi.col.idx">#</th>
                <th data-i18n="view.kagi.col.dir">Direction</th>
                <th data-i18n="view.kagi.col.yy">Yang/Yin</th>
                <th data-i18n="view.kagi.col.anchor">Anchor</th>
                <th data-i18n="view.kagi.col.end">End</th>
                <th data-i18n="view.kagi.col.move">Move</th>
                <th data-i18n="view.kagi.col.src">Src idx</th>
            </tr></thead>
            <tbody>
                ${lines.map((l, i) => {
                    const move = l.end_price - l.anchor_price;
                    const cls = l.direction === 'Up' ? 'pos' : l.direction === 'Down' ? 'neg' : '';
                    return `<tr>
                        <td>${i + 1}</td>
                        <td data-i18n="${esc(dirLabelKey(l.direction))}" class="${cls}">${esc(t(dirLabelKey(l.direction)))}</td>
                        <td data-i18n="${esc(yangYinLabelKey(yy[i]))}">${esc(t(yangYinLabelKey(yy[i])))}</td>
                        <td>${esc(fmtUSD(l.anchor_price))}</td>
                        <td>${esc(fmtUSD(l.end_price))}</td>
                        <td class="${cls}">${esc(fmtMove(move))}</td>
                        <td>${l.source_index}</td>
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
    const el = document.getElementById('kg-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('kg-err').style.display = 'none'; }
