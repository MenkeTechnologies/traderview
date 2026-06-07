// Three-Line Break (TLB) chart view — Japanese trend-only chart with
// configurable N-line break reversal rule.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_NUM_LINES,
    parseCloses, closesToBlob, validateInputs, buildBody, localCompute,
    trendBadge, flipCount, finalRunLength, summarize, linesToPolyline,
    makeDemoInput,
    fmtUSD, fmtMove, fmtInt, dirLabelKey,
} from '../_three_line_break_inputs.js';

let state = { ...makeDemoInput('uptrend') };

export async function renderThreeLineBreak(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.tlb.h1.title" class="view-title">// THREE-LINE BREAK</h1>

        <div class="chart-panel" data-context-scope="tlb">
            <h2 data-i18n="view.tlb.h2.closes">Closes
                <small data-i18n="view.tlb.h2.closes_hint" class="muted">(comma/whitespace separated; # comments ignored)</small></h2>
            <textarea id="tlb-blob" rows="6"
                      data-tip="view.tlb.tip.closes"
                      placeholder="100, 102, 104, 106, 99, ...">${esc(closesToBlob(state.closes))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.tlb.label.num_lines">Break lines (N)</span>
                    <input id="tlb-n" type="number" step="1" min="1" value="${state.num_lines}" data-tip="view.tlb.tip.num_lines"></label>
                <button data-i18n="view.tlb.btn.compute" id="tlb-run" class="primary"
                        data-tip="view.tlb.tip.compute" data-shortcut="three_line_break_run" type="button">Build TLB</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.tlb.btn.demo_uptrend"   id="tlb-demo-up"      class="secondary" type="button" data-tip="view.tlb.tip.demo_up">Demo: pure uptrend</button>
                <button data-i18n="view.tlb.btn.demo_downtrend" id="tlb-demo-down"    class="secondary" type="button" data-tip="view.tlb.tip.demo_down">Demo: pure downtrend</button>
                <button data-i18n="view.tlb.btn.demo_small"     id="tlb-demo-small"   class="secondary" type="button" data-tip="view.tlb.tip.demo_small">Demo: small pullback (no flip)</button>
                <button data-i18n="view.tlb.btn.demo_deep"      id="tlb-demo-deep"    class="secondary" type="button" data-tip="view.tlb.tip.demo_deep">Demo: deep pullback (flips)</button>
                <button data-i18n="view.tlb.btn.demo_choppy"    id="tlb-demo-choppy"  class="secondary" type="button" data-tip="view.tlb.tip.demo_choppy">Demo: choppy</button>
                <button data-i18n="view.tlb.btn.demo_flat"      id="tlb-demo-flat"    class="secondary" type="button" data-tip="view.tlb.tip.demo_flat">Demo: flat (no lines)</button>
                <button data-i18n="view.tlb.btn.demo_two"       id="tlb-demo-two"     class="secondary" type="button" data-tip="view.tlb.tip.demo_two">Demo: 2-line break</button>
                <button data-i18n="view.tlb.btn.demo_five"      id="tlb-demo-five"    class="secondary" type="button" data-tip="view.tlb.tip.demo_five">Demo: 5-line break</button>
            </div>
            <p data-i18n="view.tlb.hint.about" class="muted">Continuation: close beyond prior line's close. Reversal requires breaking the OPEN of the last N lines (default 3). N=2 is sensitive, N=5 is slow.</p>
        </div>

        <div id="tlb-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.tlb.h2.chart">TLB line plot</h2>
            <div id="tlb-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.tlb.h2.move_chart">Per-line signed move (close − open)</h2>
            <div id="tlb-move-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.tlb.hint.move_chart" class="muted small">Signed per-line magnitude. Reveals which lines represented strong trend pushes vs small qualifying-but-weak moves. Orthogonal to the absolute polyline above. Yellow dashed = zero.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.tlb.h2.table">Line table</h2>
            <div id="tlb-table"></div>
        </div>

        <div id="tlb-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('tlb-blob').value = closesToBlob(state.closes);
        document.getElementById('tlb-n').value    = state.num_lines;
    };
    document.getElementById('tlb-demo-up').addEventListener('click',    () => { loadDemo('uptrend');       void compute(tok); });
    document.getElementById('tlb-demo-down').addEventListener('click',  () => { loadDemo('downtrend');     void compute(tok); });
    document.getElementById('tlb-demo-small').addEventListener('click', () => { loadDemo('small-pullback'); void compute(tok); });
    document.getElementById('tlb-demo-deep').addEventListener('click',  () => { loadDemo('deep-pullback');  void compute(tok); });
    document.getElementById('tlb-demo-choppy').addEventListener('click', () => { loadDemo('choppy');       void compute(tok); });
    document.getElementById('tlb-demo-flat').addEventListener('click',  () => { loadDemo('flat');          void compute(tok); });
    document.getElementById('tlb-demo-two').addEventListener('click',   () => { loadDemo('two-line');      void compute(tok); });
    document.getElementById('tlb-demo-five').addEventListener('click',  () => { loadDemo('five-line');     void compute(tok); });
    document.getElementById('tlb-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseCloses(document.getElementById('tlb-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.tlb.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        showToast(t('view.tlb.toast.parse_error', { n: p.errors.length }), { level: 'warning' });
        return;
    }
    hideErr();
    state.closes = p.closes;
    const n = parseInt(document.getElementById('tlb-n').value, 10);
    state.num_lines = Number.isInteger(n) && n >= 1 ? n : DEFAULT_NUM_LINES;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.tlb.toast.invalid'), { level: 'warning' }); return; }
    const local = localCompute(state.closes, state.num_lines);
    renderSummary(local, true);
    renderChart(local);
    renderMoveChart(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.chartsThreeLineBreak(buildBody(state));
    } catch (e) {
        showErr(`${t('view.tlb.err.api')}: ${e.message || e}`);
        showToast(t('view.tlb.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(resp);
    renderMoveChart(resp);
    renderTable(resp);
    const lines = Array.isArray(resp) ? resp.length : 0;
    const last = lines > 0 ? resp[lines - 1] : null;
    const dir = last ? String(last.direction) : '—';
    const flips = flipCount(resp);
    const level = dir === 'up' ? 'success' : dir === 'down' ? 'warning' : 'info';
    showToast(t('view.tlb.toast.built', { lines, dir, flips }), { level });
}

function renderSummary(lines, pending) {
    const local = localCompute(state.closes, state.num_lines);
    const parityOk = lines.length === local.length
        && lines.every((l, i) => l.direction === local[i].direction
            && Math.abs(l.open  - local[i].open)  < 1e-9
            && Math.abs(l.close - local[i].close) < 1e-9);
    const badge = trendBadge(lines);
    const s = summarize(lines);
    const localTag = pending ? ` (${t('view.tlb.tag.local')})` : '';
    document.getElementById('tlb-summary').innerHTML = [
        card(t('view.tlb.card.verdict'),     t(badge.key) + localTag, badge.cls),
        card(t('view.tlb.card.break_rule'),  fmtInt(state.num_lines) + '-line'),
        card(t('view.tlb.card.bars'),        fmtInt(state.closes.length)),
        card(t('view.tlb.card.lines'),       fmtInt(s.count)),
        card(t('view.tlb.card.ups'),         fmtInt(s.ups),
             s.ups > s.downs ? 'pos' : ''),
        card(t('view.tlb.card.downs'),       fmtInt(s.downs),
             s.downs > s.ups ? 'neg' : ''),
        card(t('view.tlb.card.avg_up'),      fmtUSD(s.avg_up)),
        card(t('view.tlb.card.avg_down'),    fmtUSD(s.avg_down)),
        card(t('view.tlb.card.flips'),       fmtInt(flipCount(lines))),
        card(t('view.tlb.card.run'),         fmtInt(finalRunLength(lines))
             + ' ' + (s.last_dir ? t(dirLabelKey(s.last_dir)).toLowerCase() : ''),
             s.last_dir === 'Up' ? 'pos' : s.last_dir === 'Down' ? 'neg' : ''),
        card(t('view.tlb.card.parity'),
             parityOk ? t('view.tlb.tag.ok') : t('view.tlb.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(lines) {
    if (!window.uPlot) return;
    const el = document.getElementById('tlb-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!lines || lines.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.tlb.empty">${esc(t('view.tlb.empty'))}</div>`;
        return;
    }
    const { xs, ys } = linesToPolyline(lines);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 340,
        scales: { x: { time: false,}, y: {} },
        series: [
            { label: t('chart.series.line') },
            { label: t('chart.series.tlb'), stroke: '#00e5ff', width: 1.5, points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => '#' + Math.trunc(v)) },
            { stroke: '#aab', size: 70 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderMoveChart(lines) {
    const el = document.getElementById('tlb-move-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const valid = (lines || []).filter(l =>
        Number.isFinite(Number(l.open)) && Number.isFinite(Number(l.close)));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.tlb.empty_move_chart">${esc(t('view.tlb.empty_move_chart'))}</div>`;
        return;
    }
    const ys = valid.map(l => Number(l.close) - Number(l.open));
    const xs = ys.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.tlb.chart.line_idx') },
            { label: t('view.tlb.chart.move'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 12, fill: '#b86bff', stroke: '#b86bff' } },
            { label: t('view.tlb.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => '#' + Math.trunc(v)) },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, ys, zero], el);
}

function renderTable(lines) {
    const wrap = document.getElementById('tlb-table');
    if (!lines || lines.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.tlb.empty">${esc(t('view.tlb.empty'))}</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.tlb.col.idx">#</th>
                <th data-i18n="view.tlb.col.dir">Direction</th>
                <th data-i18n="view.tlb.col.open">Open</th>
                <th data-i18n="view.tlb.col.close">Close</th>
                <th data-i18n="view.tlb.col.move">Move</th>
                <th data-i18n="view.tlb.col.src">Src idx</th>
            </tr></thead>
            <tbody>
                ${lines.map((l, i) => {
                    const move = l.close - l.open;
                    const cls = l.direction === 'Up' ? 'pos' : l.direction === 'Down' ? 'neg' : '';
                    return `<tr>
                        <td>${i + 1}</td>
                        <td data-i18n="${esc(dirLabelKey(l.direction))}" class="${cls}">${esc(t(dirLabelKey(l.direction)))}</td>
                        <td>${esc(fmtUSD(l.open))}</td>
                        <td>${esc(fmtUSD(l.close))}</td>
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
    const el = document.getElementById('tlb-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('tlb-err').style.display = 'none'; }
