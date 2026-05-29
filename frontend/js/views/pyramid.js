// Pyramid / Scale-In view — entry-management planning.
//
// Pyramid-Up: add to winners as price moves in your favor (Minervini /
// Zanger style).
// Scale-In: add as price moves against you within a planned ladder
// (mean-reversion / LEAPs style — risky without a hard total-risk cap).
// Visualizes per-tranche state evolution + avg-cost curve.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseTrancheBlob, validateInputs, buildBody,
    directionMisordered, decToNum, makeDemoData,
    avgCostSeries, fmtN, fmtInt, fmtUSD,
} from '../_pyramid_inputs.js';

import { t } from '../i18n.js';
let state = {
    kind: 'pyramid_up', side: 'long',
    initialQty: 100, initialEntry: 100,
    trancheText: '',
};

export async function renderPyramid(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.pyramid.h1.pyramid_entry_management" class="view-title">// PYRAMID · ENTRY MANAGEMENT</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.pyramid.h2.strategy">Strategy</h2>
            <div class="inline-form">
                <label><span data-i18n="view.pyramid.label.kind">Kind</span>
                    <select id="py-kind">
                        <option data-i18n="view.pyramid.opt.pyramid_up_add_to_winners" value="pyramid_up" ${state.kind === 'pyramid_up' ? 'selected' : ''}>Pyramid Up (add to winners)</option>
                        <option data-i18n="view.pyramid.opt.scale_in_avg_down_up_against" value="scale_in"   ${state.kind === 'scale_in'   ? 'selected' : ''}>Scale In (avg down / up against)</option>
                    </select></label>
                <label><span data-i18n="view.pyramid.label.side">Side</span>
                    <select id="py-side">
                        <option data-i18n="view.pyramid.opt.long" value="long"  ${state.side === 'long'  ? 'selected' : ''}>Long</option>
                        <option data-i18n="view.pyramid.opt.short" value="short" ${state.side === 'short' ? 'selected' : ''}>Short</option>
                    </select></label>
                <label><span data-i18n="view.pyramid.label.initial_qty">Initial qty</span>
                    <input id="py-iq" type="number" step="any" min="0" value="${state.initialQty}"></label>
                <label><span data-i18n="view.pyramid.label.initial_entry">Initial entry $</span>
                    <input id="py-ie" type="number" step="any" min="0" value="${state.initialEntry}"></label>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.pyramid.h2.tranche_ladder">Tranche ladder</h2>
            <p class="muted">One tranche per line: <code>trigger_price qty</code>.
                Pyramid-Up: tranches must move INTO profit. Scale-In: tranches must
                move AGAINST you within your planned ladder. Misordered tranches are
                flagged by both the local pre-flight and the backend.</p>
            <textarea id="py-tranches" rows="5" placeholder="105 75&#10;110 50&#10;115 25"></textarea>
            <div class="inline-form">
                <button data-i18n="view.pyramid.btn.demo_pyramid_up_long" id="py-demo-pu-long"  class="secondary" type="button">Demo: Pyramid Up Long</button>
                <button data-i18n="view.pyramid.btn.demo_pyramid_up_short" id="py-demo-pu-short" class="secondary" type="button">Demo: Pyramid Up Short</button>
                <button data-i18n="view.pyramid.btn.demo_scale_in_long" id="py-demo-si-long"  class="secondary" type="button">Demo: Scale In Long</button>
                <button data-i18n="view.pyramid.btn.demo_scale_in_short" id="py-demo-si-short" class="secondary" type="button">Demo: Scale In Short</button>
                <button data-i18n="view.pyramid.btn.build_plan" id="py-run" class="primary" type="button">Build plan</button>
            </div>
        </div>

        <div id="py-errors" class="boot" style="display:none"></div>
        <div id="py-misorder" class="boot" style="display:none;color:#ffd84a"></div>
        <div id="py-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.pyramid.h2.state_evolution">State evolution</h2>
            <div id="py-table"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.pyramid.h2.average_cost_curve">Average cost curve</h2>
            <div id="py-chart" style="height:260px"></div>
            <p data-i18n="view.pyramid.hint.cyan_avg_cost_per_share_contract_after_each_tranch" class="muted">Cyan = avg cost per share/contract after each tranche fires.
                Pyramid-Up curves AWAY from initial entry (avg cost rises with adds);
                Scale-In curves TOWARD initial entry (avg cost moves favorably).</p>
        </div>

        <div id="py-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    const loadDemo = (kind, side) => {
        const d = makeDemoData(kind, side);
        document.getElementById('py-kind').value = d.kind;
        document.getElementById('py-side').value = d.side;
        document.getElementById('py-iq').value = d.initial_qty;
        document.getElementById('py-ie').value = d.initial_entry;
        document.getElementById('py-tranches').value =
            d.tranches.map(t => `${t.trigger_price} ${t.qty}`).join('\n');
    };
    document.getElementById('py-demo-pu-long').addEventListener('click',  () => loadDemo('pyramid_up', 'long'));
    document.getElementById('py-demo-pu-short').addEventListener('click', () => loadDemo('pyramid_up', 'short'));
    document.getElementById('py-demo-si-long').addEventListener('click',  () => loadDemo('scale_in',   'long'));
    document.getElementById('py-demo-si-short').addEventListener('click', () => loadDemo('scale_in',   'short'));
    document.getElementById('py-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.kind = document.getElementById('py-kind').value;
    state.side = document.getElementById('py-side').value;
    state.initialQty = Number(document.getElementById('py-iq').value);
    state.initialEntry = Number(document.getElementById('py-ie').value);
    state.trancheText = document.getElementById('py-tranches').value;
}

async function compute(tok) {
    hideErr();
    document.getElementById('py-misorder').style.display = 'none';
    const errs = document.getElementById('py-errors');
    errs.style.display = 'none';

    const { tranches, errors } = parseTrancheBlob(state.trancheText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            `line ${e.line_no}: ${esc(e.message)} — ${esc(e.raw.slice(0, 80))}`).join('<br>');
        const more = errors.length > 8 ? `<br>… and ${errors.length - 8} more.` : '';
        errs.innerHTML = `<strong>${errors.length} parse error(s):</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (tranches.length === 0) return;
    }
    const input = {
        kind: state.kind, side: state.side,
        initial_qty: state.initialQty, initial_entry: state.initialEntry,
        tranches,
    };
    const vErr = validateInputs(input);
    if (vErr) { showErr(vErr); return; }

    // Local pre-flight: warn before round-trip if misordered.
    if (directionMisordered(state.kind, state.side, state.initialEntry, tranches)) {
        const el = document.getElementById('py-misorder');
        el.innerHTML = `<strong>⚠ Pre-flight:</strong> at least one tranche violates the ${state.kind.replace('_', '-')} ${state.side} direction rule. Backend will also flag this.`;
        el.style.display = 'block';
    }

    let report;
    try {
        report = await api.discPyramidPlan(buildBody(input));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(report);
    renderTable(report);
    renderChart(report);
}

function renderSummary(r) {
    const finalQty   = decToNum(r.final_qty);
    const finalAvg   = decToNum(r.final_avg_cost);
    const finalNotnl = decToNum(r.final_notional);
    document.getElementById('py-summary').innerHTML = [
        card(t('view.pyramid.card.states'),         String((r.states || []).length)),
        card(t('view.pyramid.card.final_qty'),      fmtInt(finalQty)),
        card(t('view.pyramid.card.final_avg_cost'), fmtN(finalAvg)),
        card(t('view.pyramid.card.final_notional'), fmtUSD(finalNotnl)),
        card(t('view.pyramid.card.plan_misordered'), r.plan_misordered ? 'YES' : 'NO',
            r.plan_misordered ? 'neg' : 'pos'),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderTable(report) {
    const wrap = document.getElementById('py-table');
    const states = report.states || [];
    if (!states.length) {
        wrap.innerHTML = '<div class="muted">No states.</div>';
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.pyramid.th.label">Label</th><th data-i18n="view.pyramid.th.trigger">Trigger $</th><th data-i18n="view.pyramid.th.added_qty">Added qty</th>
                <th data-i18n="view.pyramid.th.total_qty">Total qty</th><th data-i18n="view.pyramid.th.avg_cost">Avg cost</th><th data-i18n="view.pyramid.th.notional">Notional</th>
            </tr></thead>
            <tbody>
                ${states.map(s => `<tr>
                    <td>${esc(s.label)}</td>
                    <td>${esc(fmtN(decToNum(s.trigger_price)))}</td>
                    <td>${esc(fmtInt(decToNum(s.added_qty)))}</td>
                    <td>${esc(fmtInt(decToNum(s.total_qty)))}</td>
                    <td>${esc(fmtN(decToNum(s.avg_cost)))}</td>
                    <td>${esc(fmtUSD(decToNum(s.notional)))}</td>
                </tr>`).join('')}
            </tbody>
        </table>
    `;
}

function renderChart(report) {
    if (!window.uPlot) return;
    const el = document.getElementById('py-chart');
    const { xs, ys } = avgCostSeries(report);
    const triggerYs = (report.states || []).map(s => decToNum(s.trigger_price));
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 260,
        scales: { x: {}, y: {} },
        series: [
            { label: 'state #' },
            { label: 'avg cost',    stroke: '#00e5ff', width: 1.5,
              fill: '#00e5ff14', points: { show: true, size: 8 } },
            { label: 'trigger $',   stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: true, size: 5 } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: true },
    }, [xs, ys, triggerYs], el);
}

function showErr(msg) {
    const el = document.getElementById('py-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('py-err').style.display = 'none'; }
