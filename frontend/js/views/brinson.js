// Brinson (1986) performance attribution view — decomposes active
// return into allocation / selection / interaction effects.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseInputsBlob, inputsToBlob, validateInputs, buildBody, localAnalyze,
    activeBadge, driverBadge, enrichSector,
    makeDemoInput,
    fmtPct, fmtPctSigned, fmtBps, fmtInt,
} from '../_brinson_inputs.js';

let state = { ...makeDemoInput('mixed') };

export async function renderBrinson(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.brinson.h1.title" class="view-title">// BRINSON ATTRIBUTION</h1>

        <div class="chart-panel" data-context-scope="brinson">
            <h2 data-i18n="view.brinson.h2.inputs">Sector inputs
                <small data-i18n="view.brinson.h2.inputs_hint" class="muted">(per line: sector port_w bench_w port_r bench_r — decimals or "5%")</small></h2>
            <textarea id="br-blob" rows="8"
                      data-tip="view.brinson.tip.inputs"
                      placeholder="Tech 0.30 0.20 0.12 0.08&#10;Energy 0.15 0.25 -0.03 0.01"
                      data-i18n-placeholder="view.brinson.placeholder.inputs">${esc(inputsToBlob(state.inputs))}</textarea>

            <div class="inline-form">
                <button data-i18n="view.brinson.btn.compute" id="br-run" class="primary"
                        data-tip="view.brinson.tip.compute" type="button">Analyze</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.brinson.btn.demo_identical"   id="br-demo-id"     class="secondary" type="button">Demo: identical (zero active)</button>
                <button data-i18n="view.brinson.btn.demo_allocation"  id="br-demo-alloc"  class="secondary" type="button">Demo: allocation win</button>
                <button data-i18n="view.brinson.btn.demo_selection"   id="br-demo-sel"    class="secondary" type="button">Demo: selection win</button>
                <button data-i18n="view.brinson.btn.demo_mixed"       id="br-demo-mix"    class="secondary" type="button">Demo: mixed 4-sector</button>
                <button data-i18n="view.brinson.btn.demo_losing"      id="br-demo-lose"   class="secondary" type="button">Demo: losing overweight</button>
                <button data-i18n="view.brinson.btn.demo_cash"        id="br-demo-cash"   class="secondary" type="button">Demo: cash-heavy (25% cash drag)</button>
                <button data-i18n="view.brinson.btn.demo_sector_bet"  id="br-demo-sb"     class="secondary" type="button">Demo: AI sector bet</button>
                <button data-i18n="view.brinson.btn.demo_all"         id="br-demo-all"    class="secondary" type="button">Demo: all 3 effects non-zero</button>
            </div>
            <p data-i18n="view.brinson.hint.about" class="muted">A_i = (w_p − w_b)·(r_b,i − r_b,total). S_i = w_b·(r_p − r_b). I_i = (w_p − w_b)·(r_p − r_b). Σ(A+S+I) = portfolio total − benchmark total. Allocation answers "did we overweight winning sectors?" Selection answers "did we pick better stocks within each sector?"</p>
        </div>

        <div id="br-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.brinson.h2.chart">Per-sector total effect (A + S + I)</h2>
            <div id="br-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.brinson.h2.table">Per-sector decomposition</h2>
            <div id="br-table"></div>
        </div>

        <div id="br-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('br-blob').value = inputsToBlob(state.inputs);
    };
    document.getElementById('br-demo-id').addEventListener('click',    () => { loadDemo('identical');         void compute(tok); });
    document.getElementById('br-demo-alloc').addEventListener('click', () => { loadDemo('allocation-win');    void compute(tok); });
    document.getElementById('br-demo-sel').addEventListener('click',   () => { loadDemo('selection-win');     void compute(tok); });
    document.getElementById('br-demo-mix').addEventListener('click',   () => { loadDemo('mixed');             void compute(tok); });
    document.getElementById('br-demo-lose').addEventListener('click',  () => { loadDemo('losing-overweight'); void compute(tok); });
    document.getElementById('br-demo-cash').addEventListener('click',  () => { loadDemo('cash-heavy');        void compute(tok); });
    document.getElementById('br-demo-sb').addEventListener('click',    () => { loadDemo('sector-bet');        void compute(tok); });
    document.getElementById('br-demo-all').addEventListener('click',   () => { loadDemo('all-effects');       void compute(tok); });
    document.getElementById('br-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseInputsBlob(document.getElementById('br-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.brinson.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.inputs = p.inputs;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localAnalyze(state.inputs);
    if (!local) { showErr(t('view.brinson.err.degenerate')); return; }
    renderSummary(local, true);
    renderChart(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.portfolioBrinsonAttribution(buildBody(state));
    } catch (e) {
        showErr(`${t('view.brinson.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) { showErr(t('view.brinson.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderTable(resp);
}

function renderSummary(report, pending) {
    const local = localAnalyze(state.inputs);
    const parityOk = !!local
        && Math.abs(local.total_active_return - report.total_active_return) < 1e-9
        && Math.abs(local.total_allocation - report.total_allocation) < 1e-9
        && Math.abs(local.total_selection - report.total_selection) < 1e-9
        && Math.abs(local.total_interaction - report.total_interaction) < 1e-9;
    const aBadge = activeBadge(report.total_active_return);
    const dBadge = driverBadge(report);
    const localTag = pending ? ` (${t('view.brinson.tag.local')})` : '';
    const checksum = report.total_allocation + report.total_selection + report.total_interaction;
    const checkOk = Math.abs(checksum - report.total_active_return) < 1e-9;
    document.getElementById('br-summary').innerHTML = [
        card(t('view.brinson.card.verdict'),     t(aBadge.key) + localTag, aBadge.cls),
        card(t('view.brinson.card.driver'),      t(dBadge.key)),
        card(t('view.brinson.card.sectors'),     fmtInt(report.per_sector.length)),
        card(t('view.brinson.card.port_total'),  fmtPctSigned(report.portfolio_total_return),
             report.portfolio_total_return > 0 ? 'pos' : report.portfolio_total_return < 0 ? 'neg' : ''),
        card(t('view.brinson.card.bench_total'), fmtPctSigned(report.benchmark_total_return),
             report.benchmark_total_return > 0 ? 'pos' : report.benchmark_total_return < 0 ? 'neg' : ''),
        card(t('view.brinson.card.active'),      fmtBps(report.total_active_return), aBadge.cls),
        card(t('view.brinson.card.allocation'),  fmtBps(report.total_allocation),
             report.total_allocation > 0 ? 'pos' : report.total_allocation < 0 ? 'neg' : ''),
        card(t('view.brinson.card.selection'),   fmtBps(report.total_selection),
             report.total_selection > 0 ? 'pos' : report.total_selection < 0 ? 'neg' : ''),
        card(t('view.brinson.card.interaction'), fmtBps(report.total_interaction),
             report.total_interaction > 0 ? 'pos' : report.total_interaction < 0 ? 'neg' : ''),
        card(t('view.brinson.card.identity'),
             checkOk ? t('view.brinson.tag.ok') : t('view.brinson.tag.diverged'),
             checkOk ? 'pos' : 'neg'),
        card(t('view.brinson.card.parity'),
             parityOk ? t('view.brinson.tag.ok') : t('view.brinson.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(report) {
    if (!window.uPlot) return;
    const el = document.getElementById('br-chart');
    if (!el) return;
    el.innerHTML = '';
    const rows = report.per_sector.map((eff, i) => enrichSector(state.inputs[i], eff));
    if (rows.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.brinson.empty">${esc(t('view.brinson.empty'))}</div>`;
        return;
    }
    rows.sort((a, b) => b.total_effect - a.total_effect);
    const xs = rows.map((_, i) => i);
    const allocs = rows.map(r => r.allocation_effect * 10_000);
    const sels   = rows.map(r => r.selection_effect * 10_000);
    const inters = rows.map(r => r.interaction_effect * 10_000);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 340,
        scales: { x: {}, y: {} },
        series: [
            { label: t('chart.series.rank') },
            { label: t('chart.series.alloc_bps'),  stroke: '#00e5ff', width: 1.5, points: { show: true, size: 5 } },
            { label: t('chart.series.select_bps'), stroke: '#ffd84a', width: 1.5, points: { show: true, size: 5 } },
            { label: t('chart.series.inter_bps'),  stroke: '#ff3860', width: 1.0, dash: [4, 4], points: { show: true, size: 4 } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => {
                  const i = Math.trunc(v);
                  return i >= 0 && i < rows.length ? rows[i].sector : '';
              }) },
            { stroke: '#aab', size: 60,
              values: (_u, splits) => splits.map(v => v.toFixed(0) + ' bps') },
        ],
        legend: { show: true },
    }, [xs, allocs, sels, inters], el);
}

function renderTable(report) {
    const wrap = document.getElementById('br-table');
    if (!report.per_sector || report.per_sector.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.brinson.empty">${esc(t('view.brinson.empty'))}</div>`;
        return;
    }
    const rows = report.per_sector.map((eff, i) => enrichSector(state.inputs[i], eff));
    rows.sort((a, b) => Math.abs(b.total_effect) - Math.abs(a.total_effect));
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.brinson.col.rank">#</th>
                <th data-i18n="view.brinson.col.sector">Sector</th>
                <th data-i18n="view.brinson.col.dw">Δw</th>
                <th data-i18n="view.brinson.col.dr">Δr</th>
                <th data-i18n="view.brinson.col.alloc">Allocation</th>
                <th data-i18n="view.brinson.col.sel">Selection</th>
                <th data-i18n="view.brinson.col.inter">Interaction</th>
                <th data-i18n="view.brinson.col.total">Total</th>
            </tr></thead>
            <tbody>
                ${rows.map((r, i) => {
                    const tCls = r.total_effect > 0 ? 'pos' : r.total_effect < 0 ? 'neg' : '';
                    const aCls = r.allocation_effect > 0 ? 'pos' : r.allocation_effect < 0 ? 'neg' : '';
                    const sCls = r.selection_effect > 0 ? 'pos' : r.selection_effect < 0 ? 'neg' : '';
                    const iCls = r.interaction_effect > 0 ? 'pos' : r.interaction_effect < 0 ? 'neg' : '';
                    return `<tr>
                        <td>${i + 1}</td>
                        <td><strong>${esc(r.sector)}</strong></td>
                        <td>${esc(fmtPctSigned(r.weight_diff))}</td>
                        <td>${esc(fmtPctSigned(r.return_diff))}</td>
                        <td class="${aCls}">${esc(fmtBps(r.allocation_effect))}</td>
                        <td class="${sCls}">${esc(fmtBps(r.selection_effect))}</td>
                        <td class="${iCls}">${esc(fmtBps(r.interaction_effect))}</td>
                        <td class="${tCls}"><strong>${esc(fmtBps(r.total_effect))}</strong></td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
    void fmtPct;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('br-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('br-err').style.display = 'none'; }
