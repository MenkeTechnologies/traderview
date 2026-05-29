// Tick Imbalance Bar (TIB) view — López de Prado AFML signed-flow bar.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parsePrintsBlob, printsToBlob, validateInputs, buildBody, localCompute,
    flowBadge, tiltBadge, summarize,
    makeDemoInput,
    fmtUSD, fmtSigned, fmtMove, fmtInt, fmtVol,
} from '../_imbalance_bar_inputs.js';

let state = { ...makeDemoInput('uptrend') };

export async function renderImbalanceBar(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.imb_bar.h1.title" class="view-title">// TICK IMBALANCE BAR</h1>

        <div class="chart-panel" data-context-scope="imb-bar">
            <h2 data-i18n="view.imb_bar.h2.prints">Trade prints
                <small data-i18n="view.imb_bar.h2.prints_hint" class="muted">(per line: price size)</small></h2>
            <textarea id="ib-blob" rows="6"
                      data-tip="view.imb_bar.tip.prints"
                      placeholder="100.05 10&#10;100.10 20&#10;100.15 15">${esc(printsToBlob(state.prints))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.imb_bar.label.threshold">Imbalance threshold</span>
                    <input id="ib-thresh" type="number" step="any" min="0" value="${state.imbalance_threshold}"></label>
                <button data-i18n="view.imb_bar.btn.compute" id="ib-run" class="primary"
                        data-tip="view.imb_bar.tip.compute" type="button">Build bars</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.imb_bar.btn.demo_up"      id="ib-demo-up"      class="secondary" type="button">Demo: uptrend</button>
                <button data-i18n="view.imb_bar.btn.demo_down"    id="ib-demo-down"    class="secondary" type="button">Demo: downtrend</button>
                <button data-i18n="view.imb_bar.btn.demo_bal"     id="ib-demo-bal"     class="secondary" type="button">Demo: balanced (alternating)</button>
                <button data-i18n="view.imb_bar.btn.demo_flat"    id="ib-demo-flat"    class="secondary" type="button">Demo: flat (ties)</button>
                <button data-i18n="view.imb_bar.btn.demo_agg"     id="ib-demo-agg"     class="secondary" type="button">Demo: aggressive buy (size 100)</button>
                <button data-i18n="view.imb_bar.btn.demo_climax"  id="ib-demo-climax"  class="secondary" type="button">Demo: climax burst</button>
                <button data-i18n="view.imb_bar.btn.demo_partial" id="ib-demo-partial" class="secondary" type="button">Demo: partial (dropped)</button>
                <button data-i18n="view.imb_bar.btn.demo_tie"     id="ib-demo-tie"     class="secondary" type="button">Demo: tie-run (uses prior_sign)</button>
            </div>
            <p data-i18n="view.imb_bar.hint.about" class="muted">sign = +1 (uptick) / −1 (downtick) / prior_sign (tie). Bar closes when |Σ sign × size| ≥ imbalance_threshold. Trailing partial bars dropped. López de Prado AFML: better i.i.d. approximation than time bars.</p>
        </div>

        <div id="ib-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.imb_bar.h2.chart">Signed imbalance + cumulative close per bar</h2>
            <div id="ib-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.imb_bar.h2.table">Bars (tail — last 30)</h2>
            <div id="ib-table"></div>
        </div>

        <div id="ib-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('ib-blob').value  = printsToBlob(state.prints);
        document.getElementById('ib-thresh').value = state.imbalance_threshold;
    };
    document.getElementById('ib-demo-up').addEventListener('click',      () => { loadDemo('uptrend');         void compute(tok); });
    document.getElementById('ib-demo-down').addEventListener('click',    () => { loadDemo('downtrend');       void compute(tok); });
    document.getElementById('ib-demo-bal').addEventListener('click',     () => { loadDemo('balanced');        void compute(tok); });
    document.getElementById('ib-demo-flat').addEventListener('click',    () => { loadDemo('flat');            void compute(tok); });
    document.getElementById('ib-demo-agg').addEventListener('click',     () => { loadDemo('aggressive-buy'); void compute(tok); });
    document.getElementById('ib-demo-climax').addEventListener('click',  () => { loadDemo('climax-burst');   void compute(tok); });
    document.getElementById('ib-demo-partial').addEventListener('click', () => { loadDemo('partial-trail');  void compute(tok); });
    document.getElementById('ib-demo-tie').addEventListener('click',     () => { loadDemo('tie-runs');       void compute(tok); });
    document.getElementById('ib-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parsePrintsBlob(document.getElementById('ib-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.imb_bar.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.prints = p.prints;
    const th = Number(document.getElementById('ib-thresh').value);
    state.imbalance_threshold = Number.isFinite(th) && th > 0 ? th : 100;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.prints, state.imbalance_threshold);
    renderSummary(local, true);
    renderChart(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.chartsImbalanceBar(buildBody(state));
    } catch (e) {
        showErr(`${t('view.imb_bar.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(resp);
    renderTable(resp);
}

function renderSummary(bars, pending) {
    const local = localCompute(state.prints, state.imbalance_threshold);
    const parityOk = bars.length === local.length
        && bars.every((b, i) => Math.abs(b.imbalance - local[i].imbalance) < 1e-9
            && Math.abs(b.close - local[i].close) < 1e-9
            && b.tick_count === local[i].tick_count);
    const fBadge = flowBadge(bars);
    const tBadge = tiltBadge(bars);
    const s = summarize(bars);
    const localTag = pending ? ` (${t('view.imb_bar.tag.local')})` : '';
    document.getElementById('ib-summary').innerHTML = [
        card(t('view.imb_bar.card.verdict'),     t(fBadge.key) + localTag, fBadge.cls),
        card(t('view.imb_bar.card.tilt'),        t(tBadge.key), tBadge.cls),
        card(t('view.imb_bar.card.threshold'),   fmtVol(state.imbalance_threshold)),
        card(t('view.imb_bar.card.prints'),      fmtInt(state.prints.length)),
        card(t('view.imb_bar.card.bars'),        fmtInt(s.count)),
        card(t('view.imb_bar.card.buy_bars'),    fmtInt(s.buy_bars),
             s.buy_bars > s.sell_bars ? 'pos' : ''),
        card(t('view.imb_bar.card.sell_bars'),   fmtInt(s.sell_bars),
             s.sell_bars > s.buy_bars ? 'neg' : ''),
        card(t('view.imb_bar.card.max_abs_imb'), fmtVol(s.max_abs_imb)),
        card(t('view.imb_bar.card.total_vol'),   fmtVol(s.total_volume)),
        card(t('view.imb_bar.card.last_close'),  fmtUSD(s.last_close)),
        card(t('view.imb_bar.card.parity'),
             parityOk ? t('view.imb_bar.tag.ok') : t('view.imb_bar.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(bars) {
    if (!window.uPlot) return;
    const el = document.getElementById('ib-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!bars || bars.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.imb_bar.empty">${esc(t('view.imb_bar.empty'))}</div>`;
        return;
    }
    const xs = bars.map((_, i) => i);
    const imbs = bars.map(b => b.imbalance);
    const closes = bars.map(b => b.close);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 340,
        scales: { x: {}, y: {}, y2: { auto: true } },
        series: [
            { label: 'bar' },
            { label: 'imbalance', stroke: '#ffd84a', width: 1.5, points: { show: true, size: 5 } },
            { label: 'close',     stroke: '#00e5ff', width: 1.5, points: { show: false }, scale: 'y2' },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => '#' + Math.trunc(v)) },
            { stroke: '#aab', size: 60 },
            { side: 1, stroke: '#aab', size: 60, scale: 'y2',
              values: (_u, splits) => splits.map(v => '$' + v.toFixed(2)) },
        ],
        legend: { show: true },
    }, [xs, imbs, closes], el);
}

function renderTable(bars) {
    const wrap = document.getElementById('ib-table');
    if (!bars || bars.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.imb_bar.empty">${esc(t('view.imb_bar.empty'))}</div>`;
        return;
    }
    const start = Math.max(0, bars.length - 30);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.imb_bar.col.idx">#</th>
                <th data-i18n="view.imb_bar.col.open">Open</th>
                <th data-i18n="view.imb_bar.col.high">High</th>
                <th data-i18n="view.imb_bar.col.low">Low</th>
                <th data-i18n="view.imb_bar.col.close">Close</th>
                <th data-i18n="view.imb_bar.col.move">Move</th>
                <th data-i18n="view.imb_bar.col.volume">Volume</th>
                <th data-i18n="view.imb_bar.col.imb">Imbalance</th>
                <th data-i18n="view.imb_bar.col.ticks">Ticks</th>
            </tr></thead>
            <tbody>
                ${bars.slice(start).map((b, k) => {
                    const i = start + k;
                    const move = b.close - b.open;
                    const mcls = move > 0 ? 'pos' : move < 0 ? 'neg' : '';
                    const icls = b.imbalance > 0 ? 'pos' : b.imbalance < 0 ? 'neg' : '';
                    return `<tr>
                        <td>${i + 1}</td>
                        <td>${esc(fmtUSD(b.open))}</td>
                        <td>${esc(fmtUSD(b.high))}</td>
                        <td>${esc(fmtUSD(b.low))}</td>
                        <td><strong>${esc(fmtUSD(b.close))}</strong></td>
                        <td class="${mcls}">${esc(fmtMove(move))}</td>
                        <td>${esc(fmtVol(b.volume))}</td>
                        <td class="${icls}">${esc(fmtSigned(b.imbalance))}</td>
                        <td>${esc(fmtInt(b.tick_count))}</td>
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
    const el = document.getElementById('ib-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ib-err').style.display = 'none'; }
