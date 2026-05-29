// Balance of Power (Igor Livshin) view — bar-by-bar buyer/seller dominance.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_SMOOTHING,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    regimeBadge, lastCrossover, summarize,
    makeDemoInput,
    fmtBop, fmtUSD, fmtInt,
} from '../_balance_of_power_inputs.js';

let state = { ...makeDemoInput('strong-bull') };

export async function renderBalanceOfPower(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.bop.h1.title" class="view-title">// BALANCE OF POWER</h1>

        <div class="chart-panel" data-context-scope="bop">
            <h2 data-i18n="view.bop.h2.bars">Bars
                <small data-i18n="view.bop.h2.bars_hint" class="muted">(per line: open high low close)</small></h2>
            <textarea id="bp-blob" rows="6"
                      data-tip="view.bop.tip.bars"
                      placeholder="99 101 99 101&#10;101 101 99 99">${esc(barsToBlob(state.bars))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.bop.label.smoothing">Smoothing period</span>
                    <input id="bp-smooth" type="number" step="1" min="1" value="${state.smoothing_period}"></label>
                <button data-i18n="view.bop.btn.compute" id="bp-run" class="primary"
                        data-tip="view.bop.tip.compute" type="button">Compute BOP</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.bop.btn.demo_bull"     id="bp-demo-bull"   class="secondary" type="button">Demo: strong bull (marubozu)</button>
                <button data-i18n="view.bop.btn.demo_bear"     id="bp-demo-bear"   class="secondary" type="button">Demo: strong bear (marubozu)</button>
                <button data-i18n="view.bop.btn.demo_balanced" id="bp-demo-bal"    class="secondary" type="button">Demo: balanced doji</button>
                <button data-i18n="view.bop.btn.demo_chop"     id="bp-demo-chop"   class="secondary" type="button">Demo: choppy noise</button>
                <button data-i18n="view.bop.btn.demo_flip"     id="bp-demo-flip"   class="secondary" type="button">Demo: bull → bear flip</button>
                <button data-i18n="view.bop.btn.demo_zero"     id="bp-demo-zero"   class="secondary" type="button">Demo: zero-range dojis</button>
                <button data-i18n="view.bop.btn.demo_short"    id="bp-demo-short"  class="secondary" type="button">Demo: short smoothing (3)</button>
                <button data-i18n="view.bop.btn.demo_nosmooth" id="bp-demo-ns"     class="secondary" type="button">Demo: no smoothing (1)</button>
            </div>
            <p data-i18n="view.bop.hint.about" class="muted">BOP = (close − open) / (high − low) per bar. Clamped to [−1, +1]. Smoothed by SMA over smoothing_period bars. ≥ +0.5 = strong bull · ≤ −0.5 = strong bear · ±0.1 = balanced. Crossovers between raw and smoothed signal momentum shifts.</p>
        </div>

        <div id="bp-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.bop.h2.chart">Raw + smoothed BOP</h2>
            <div id="bp-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.bop.h2.table">Per-bar BOP (tail — last 30)</h2>
            <div id="bp-table"></div>
        </div>

        <div id="bp-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('bp-blob').value   = barsToBlob(state.bars);
        document.getElementById('bp-smooth').value = state.smoothing_period;
    };
    document.getElementById('bp-demo-bull').addEventListener('click',  () => { loadDemo('strong-bull');     void compute(tok); });
    document.getElementById('bp-demo-bear').addEventListener('click',  () => { loadDemo('strong-bear');     void compute(tok); });
    document.getElementById('bp-demo-bal').addEventListener('click',   () => { loadDemo('balanced');        void compute(tok); });
    document.getElementById('bp-demo-chop').addEventListener('click',  () => { loadDemo('choppy-noise');    void compute(tok); });
    document.getElementById('bp-demo-flip').addEventListener('click',  () => { loadDemo('bull-then-bear');  void compute(tok); });
    document.getElementById('bp-demo-zero').addEventListener('click',  () => { loadDemo('zero-range');      void compute(tok); });
    document.getElementById('bp-demo-short').addEventListener('click', () => { loadDemo('short-smoothing'); void compute(tok); });
    document.getElementById('bp-demo-ns').addEventListener('click',    () => { loadDemo('no-smoothing');    void compute(tok); });
    document.getElementById('bp-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseBarsBlob(document.getElementById('bp-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.bop.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.bars = p.bars;
    const sp = parseInt(document.getElementById('bp-smooth').value, 10);
    state.smoothing_period = Number.isInteger(sp) && sp >= 1 ? sp : DEFAULT_SMOOTHING;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.bars, state.smoothing_period);
    renderSummary(local, true);
    renderChart(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.anlyBalanceOfPower(buildBody(state));
    } catch (e) {
        showErr(`${t('view.bop.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(resp);
    renderTable(resp);
}

function renderSummary(report, pending) {
    const local = localCompute(state.bars, state.smoothing_period);
    const parityOk = report.raw_bop.length === local.raw_bop.length
        && report.raw_bop.every((v, i) => {
            if (v == null && local.raw_bop[i] == null) return true;
            if (v == null || local.raw_bop[i] == null) return false;
            return Math.abs(v - local.raw_bop[i]) < 1e-9;
        });
    const s = summarize(report);
    const badge = regimeBadge(s.last_smoothed);
    const xover = lastCrossover(report);
    const localTag = pending ? ` (${t('view.bop.tag.local')})` : '';
    const xoverLabel = xover
        ? `${xover.kind === 'bull' ? '▲' : '▼'} ${t(xover.kind === 'bull' ? 'view.bop.xover.bull' : 'view.bop.xover.bear')} @ ${xover.idx + 1}`
        : t('view.bop.tag.no_cross');
    const xoverCls = !xover ? '' : xover.kind === 'bull' ? 'pos' : 'neg';
    document.getElementById('bp-summary').innerHTML = [
        card(t('view.bop.card.verdict'),       t(badge.key) + localTag, badge.cls),
        card(t('view.bop.card.smoothing'),     fmtInt(state.smoothing_period)),
        card(t('view.bop.card.bars'),          fmtInt(s.count)),
        card(t('view.bop.card.populated'),     fmtInt(s.populated)),
        card(t('view.bop.card.last_raw'),      fmtBop(s.last_raw)),
        card(t('view.bop.card.last_smoothed'), fmtBop(s.last_smoothed), badge.cls),
        card(t('view.bop.card.mean_raw'),      fmtBop(s.mean_raw)),
        card(t('view.bop.card.bull_bars'),     fmtInt(s.bull_bars),
             s.bull_bars > s.bear_bars ? 'pos' : ''),
        card(t('view.bop.card.bear_bars'),     fmtInt(s.bear_bars),
             s.bear_bars > s.bull_bars ? 'neg' : ''),
        card(t('view.bop.card.last_xover'),    xoverLabel, xoverCls),
        card(t('view.bop.card.parity'),
             parityOk ? t('view.bop.tag.ok') : t('view.bop.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(report) {
    if (!window.uPlot) return;
    const el = document.getElementById('bp-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!report.raw_bop || report.raw_bop.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.bop.empty">${esc(t('view.bop.empty'))}</div>`;
        return;
    }
    const xs = report.raw_bop.map((_, i) => i);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 340,
        scales: { x: {}, y: { range: [-1.1, 1.1] } },
        series: [
            { label: 'bar' },
            { label: 'raw',      stroke: '#aab',     width: 1.0, dash: [2, 2], points: { show: false } },
            { label: 'smoothed', stroke: '#00e5ff', width: 1.5, points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, report.raw_bop, report.smoothed_bop], el);
}

function renderTable(report) {
    const wrap = document.getElementById('bp-table');
    const n = report.raw_bop?.length || 0;
    if (n === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.bop.empty">${esc(t('view.bop.empty'))}</div>`;
        return;
    }
    const start = Math.max(0, n - 30);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.bop.col.idx">#</th>
                <th data-i18n="view.bop.col.open">Open</th>
                <th data-i18n="view.bop.col.high">High</th>
                <th data-i18n="view.bop.col.low">Low</th>
                <th data-i18n="view.bop.col.close">Close</th>
                <th data-i18n="view.bop.col.raw">Raw BOP</th>
                <th data-i18n="view.bop.col.smoothed">Smoothed</th>
                <th data-i18n="view.bop.col.regime">Regime</th>
            </tr></thead>
            <tbody>
                ${Array.from({ length: n - start }, (_, k) => {
                    const i = start + k;
                    const b = state.bars[i];
                    const r = report.raw_bop[i];
                    const sm = report.smoothed_bop[i];
                    const rcls = r > 0 ? 'pos' : r < 0 ? 'neg' : '';
                    const scls = sm > 0 ? 'pos' : sm < 0 ? 'neg' : '';
                    const rb = regimeBadge(sm);
                    return `<tr>
                        <td>${i + 1}</td>
                        <td>${esc(fmtUSD(b?.open))}</td>
                        <td>${esc(fmtUSD(b?.high))}</td>
                        <td>${esc(fmtUSD(b?.low))}</td>
                        <td>${esc(fmtUSD(b?.close))}</td>
                        <td class="${rcls}">${esc(fmtBop(r))}</td>
                        <td class="${scls}">${esc(fmtBop(sm))}</td>
                        <td data-i18n="${esc(rb.key)}" class="${rb.cls}">${esc(t(rb.key))}</td>
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
    const el = document.getElementById('bp-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('bp-err').style.display = 'none'; }
