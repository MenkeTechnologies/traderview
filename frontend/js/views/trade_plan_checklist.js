// Trade-Plan Checklist view — pre-trade discipline gate enforcer.
//
// Run a planned trade through 7 gates: thesis word count, stop set,
// target set, R-multiple ≥ min, target/stop direction, risk within
// max %. Big PASS/FAIL verdict + per-gate detail.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_CONFIG, validateInputs, buildBody, localEvaluate,
    gateLabel, gateCls, gateIcon, makeDemoData,
    fmtPct, fmtR,
} from '../_trade_plan_checklist_inputs.js';

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
let state = {
    plan: makeDemoData('good'),
    config: { ...DEFAULT_CONFIG },
};

export async function renderTradePlanChecklist(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.trade_plan_checklist.h1.trade_plan_checklist" class="view-title">// TRADE PLAN CHECKLIST</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.trade_plan_checklist.h2.planned_trade">Planned trade</h2>
            <label style="display:block;margin-bottom:6px"><span data-i18n="view.trade_plan_checklist.label.thesis">Thesis (free text)</span>
                <textarea id="tpc-thesis" rows="3" placeholder="Why does this work? Catalyst? Setup? Confirmation?"
                          data-i18n-placeholder="view.trade_plan_checklist.placeholder.thesis"
                          data-tip="view.trade_plan_checklist.tip.thesis">${esc(state.plan.thesis)}</textarea>
            </label>
            <div class="inline-form">
                <label><span data-i18n="view.trade_plan_checklist.label.side">Side</span>
                    <select id="tpc-side" data-tip="view.trade_plan_checklist.tip.side">
                        <option data-i18n="view.trade_plan_checklist.opt.long" value="long"  ${state.plan.is_long  ? 'selected' : ''}>Long</option>
                        <option data-i18n="view.trade_plan_checklist.opt.short" value="short" ${!state.plan.is_long ? 'selected' : ''}>Short</option>
                    </select></label>
                <label><span data-i18n="view.trade_plan_checklist.label.entry">Entry $</span>
                    <input id="tpc-entry" type="number" step="any" min="0" value="${state.plan.entry_price}" data-tip="view.trade_plan_checklist.tip.entry"></label>
                <label><span data-i18n="view.trade_plan_checklist.label.stop">Stop $ (blank = none)</span>
                    <input id="tpc-stop"  type="number" step="any" min="0" value="${state.plan.stop_price ?? ''}" data-tip="view.trade_plan_checklist.tip.stop"></label>
                <label><span data-i18n="view.trade_plan_checklist.label.target">Target $ (blank = none)</span>
                    <input id="tpc-tgt"   type="number" step="any" min="0" value="${state.plan.target_price ?? ''}" data-tip="view.trade_plan_checklist.tip.target"></label>
            </div>
            <div class="inline-form">
                <label><span data-i18n="view.trade_plan_checklist.label.risk">Risk $ (notional dollars on this trade)</span>
                    <input id="tpc-risk" type="number" step="any" min="0" value="${state.plan.risk_dollars}" data-tip="view.trade_plan_checklist.tip.risk"></label>
                <label><span data-i18n="view.trade_plan_checklist.label.equity">Account equity $</span>
                    <input id="tpc-eq" type="number" step="any" min="0" value="${state.plan.account_equity}" data-tip="view.trade_plan_checklist.tip.equity"></label>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.trade_plan_checklist.h2.gate_config">Gate config</h2>
            <div class="inline-form">
                <label><span data-i18n="view.trade_plan_checklist.label.min_words">Min thesis words</span>
                    <input id="tpc-mw" type="number" step="1" min="0" value="${state.config.min_thesis_words}" data-tip="view.trade_plan_checklist.tip.min_words"></label>
                <label><span data-i18n="view.trade_plan_checklist.label.min_r">Min R-multiple</span>
                    <input id="tpc-mr" type="number" step="any" min="0" value="${state.config.min_r_multiple}" data-tip="view.trade_plan_checklist.tip.min_r"></label>
                <label><span data-i18n="view.trade_plan_checklist.label.max_risk_pct">Max risk % (decimal — 0.02 = 2%)</span>
                    <input id="tpc-mrp" type="number" step="any" min="0" max="1" value="${state.config.max_risk_pct_per_trade}" data-tip="view.trade_plan_checklist.tip.max_risk_pct"></label>
                <button data-i18n="view.trade_plan_checklist.btn.evaluate" id="tpc-run" class="primary" type="button" data-tip="view.trade_plan_checklist.tip.run" data-shortcut="trade_plan_checklist_run">Evaluate</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.trade_plan_checklist.btn.demo_good_plan" id="tpc-demo-good"     class="secondary" type="button" data-tip="view.trade_plan_checklist.tip.demo_good">Demo: GOOD plan</button>
                <button data-i18n="view.trade_plan_checklist.btn.demo_missing_stop" id="tpc-demo-no-stop"  class="secondary" type="button" data-tip="view.trade_plan_checklist.tip.demo_no_stop">Demo: missing stop</button>
                <button data-i18n="view.trade_plan_checklist.btn.demo_weak_r" id="tpc-demo-weak-r"   class="secondary" type="button" data-tip="view.trade_plan_checklist.tip.demo_weak_r">Demo: weak R</button>
                <button data-i18n="view.trade_plan_checklist.btn.demo_oversize_risk" id="tpc-demo-oversize" class="secondary" type="button" data-tip="view.trade_plan_checklist.tip.demo_oversize">Demo: oversize risk</button>
                <button data-i18n="view.trade_plan_checklist.btn.demo_wrong_direction" id="tpc-demo-wrong"    class="secondary" type="button" data-tip="view.trade_plan_checklist.tip.demo_wrong">Demo: wrong direction</button>
                <button data-i18n="view.trade_plan_checklist.btn.demo_short_good" id="tpc-demo-short"    class="secondary" type="button" data-tip="view.trade_plan_checklist.tip.demo_short">Demo: SHORT (good)</button>
                <button data-i18n="view.trade_plan_checklist.btn.demo_no_thesis" id="tpc-demo-noth"     class="secondary" type="button" data-tip="view.trade_plan_checklist.tip.demo_noth">Demo: no thesis</button>
            </div>
        </div>

        <div id="tpc-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.trade_plan_checklist.h2.gate_results">Gate results</h2>
            <div id="tpc-gates"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.trade_plan_checklist.h2.gates_chart">Gate status (green pass / red fail)</h2>
            <div id="tpc-chart" style="width:100%;height:200px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.trade_plan_checklist.h2.rr_chart">R-multiple vs risk %</h2>
            <div id="tpc-rr-chart" style="width:100%;height:180px"></div>
        </div>

        <div id="tpc-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (kind) => {
        state.plan = makeDemoData(kind);
        document.getElementById('tpc-thesis').value = state.plan.thesis;
        document.getElementById('tpc-side').value = state.plan.is_long ? 'long' : 'short';
        document.getElementById('tpc-entry').value = state.plan.entry_price;
        document.getElementById('tpc-stop').value = state.plan.stop_price ?? '';
        document.getElementById('tpc-tgt').value = state.plan.target_price ?? '';
        document.getElementById('tpc-risk').value = state.plan.risk_dollars;
        document.getElementById('tpc-eq').value = state.plan.account_equity;
    };
    document.getElementById('tpc-demo-good').addEventListener('click',     () => loadDemo('good'));
    document.getElementById('tpc-demo-no-stop').addEventListener('click',  () => loadDemo('no-stop'));
    document.getElementById('tpc-demo-weak-r').addEventListener('click',   () => loadDemo('weak-r'));
    document.getElementById('tpc-demo-oversize').addEventListener('click', () => loadDemo('oversize'));
    document.getElementById('tpc-demo-wrong').addEventListener('click',    () => loadDemo('wrong-target'));
    document.getElementById('tpc-demo-short').addEventListener('click',    () => loadDemo('short-trade'));
    document.getElementById('tpc-demo-noth').addEventListener('click',     () => loadDemo('no-thesis'));
    document.getElementById('tpc-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
    readInputs(); void compute(tok);
}

function readInputs() {
    const numOrNull = (id) => {
        const raw = document.getElementById(id).value.trim();
        if (!raw) return null;
        const n = Number(raw);
        return Number.isFinite(n) ? n : null;
    };
    state.plan = {
        thesis: document.getElementById('tpc-thesis').value,
        is_long: document.getElementById('tpc-side').value === 'long',
        entry_price: Number(document.getElementById('tpc-entry').value),
        stop_price:  numOrNull('tpc-stop'),
        target_price: numOrNull('tpc-tgt'),
        risk_dollars: Number(document.getElementById('tpc-risk').value),
        account_equity: Number(document.getElementById('tpc-eq').value),
    };
    state.config = {
        min_thesis_words: parseInt(document.getElementById('tpc-mw').value, 10),
        min_r_multiple:   Number(document.getElementById('tpc-mr').value),
        max_risk_pct_per_trade: Number(document.getElementById('tpc-mrp').value),
    };
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.plan, state.config);
    if (err) { showErr(err); showToast(t('view.trade_plan_checklist.toast.invalid'), { level: 'warning' }); return; }

    const local = localEvaluate(state.plan, state.config);
    renderSummary(local, true);
    renderGates(local);
    renderGatesChart(local);
    renderRrChart(local);

    let resp;
    try {
        resp = await api.discTradePlanChecklist(buildBody(state.plan, state.config));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        showToast(t('view.trade_plan_checklist.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderGates(resp);
    renderGatesChart(resp);
    renderRrChart(resp);
    const pass = !!resp.passed;
    const gates = Array.isArray(resp.gates) ? resp.gates : [];
    const failed = gates.filter(g => g && !g.passed).length;
    showToast(t(pass ? 'view.trade_plan_checklist.toast.passed' : 'view.trade_plan_checklist.toast.failed', { failed }), { level: pass ? 'success' : 'warning' });
}

function renderRrChart(report) {
    const el = document.getElementById('tpc-rr-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const r = Number(report?.computed_r_multiple);
    const risk = Number(report?.risk_pct);
    if (!Number.isFinite(r) && !Number.isFinite(risk)) {
        el.innerHTML = `<div class="muted" data-i18n="view.trade_plan_checklist.empty_rr_chart">${esc(t('view.trade_plan_checklist.empty_rr_chart'))}</div>`;
        return;
    }
    const labels = [
        t('view.trade_plan_checklist.chart.r_mult'),
        t('view.trade_plan_checklist.chart.risk_pct'),
    ];
    const xs = [1, 2];
    const ry  = [Number.isFinite(r)    ? r          : null, null];
    const rpY = [null, Number.isFinite(risk) ? risk * 100 : null];
    const minR = xs.map(() => Number(state.config.min_r_multiple));
    const maxR = xs.map(() => Number(state.config.max_risk_pct_per_trade) * 100);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 160,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.trade_plan_checklist.chart.bucket') },
            { label: t('view.trade_plan_checklist.chart.r_mult'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 18, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.trade_plan_checklist.chart.risk_pct'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 18, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.trade_plan_checklist.chart.min_r'),
              stroke: '#7af0a8', width: 1.0, dash: [4, 4],
              points: { show: false } },
            { label: t('view.trade_plan_checklist.chart.max_risk'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ry, rpY, minR, maxR], el);
}

function renderGatesChart(report) {
    const el = document.getElementById('tpc-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = report?.gates || [];
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.trade_plan_checklist.empty_chart">${esc(t('view.trade_plan_checklist.empty_chart'))}</div>`;
        return;
    }
    const labels = rows.map(g => gateLabel(g.gate));
    const xs = labels.map((_, i) => i + 1);
    const passY = rows.map(g => g.passed ? 1 : null);
    const failY = rows.map(g => g.passed ? null : 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: {}, y: { range: () => [-0.3, 1.3] } },
        series: [
            { label: t('view.trade_plan_checklist.chart.gate') },
            { label: t('view.trade_plan_checklist.chart.pass'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 16, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.trade_plan_checklist.chart.fail'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 16, fill: '#ff3860', stroke: '#ff3860' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40,
              values: (_u, splits) => splits.map(v => v === 1 ? 'pass' : v === 0 ? 'fail' : '') },
        ],
        legend: { show: true },
    }, [xs, passY, failY], el);
}

function renderSummary(r, pending) {
    const allCls = r.all_passed ? 'pos' : 'neg';
    const local = localEvaluate(state.plan, state.config);
    const parityOk = r.all_passed === local.all_passed;
    document.getElementById('tpc-summary').innerHTML = [
        card(t('view.trade_plan_checklist.card.verdict'), r.all_passed ? t('common.pass') : t('common.fail') + (pending ? t('common.suffix.local') : ''), allCls),
        card(t('view.trade_plan_checklist.card.gates_passed'), `${r.gates.filter(g => g.passed).length} / ${r.gates.length}`, allCls),
        card(t('view.trade_plan_checklist.card.r_multiple'), fmtR(r.computed_r_multiple),
            r.computed_r_multiple != null && r.computed_r_multiple >= state.config.min_r_multiple ? 'pos' : 'neg'),
        card(t('view.trade_plan_checklist.card.risk'), fmtPct(r.risk_pct),
            r.risk_pct <= state.config.max_risk_pct_per_trade ? 'pos' : 'neg'),
        card(t('view.trade_plan_checklist.card.side'),  state.plan.is_long ? t('common.long') : t('common.short'),
            state.plan.is_long ? 'pos' : 'neg'),
        card(t('view.trade_plan_checklist.card.local_parity'), parityOk ? t('common.ok') : t('common.diverged'), parityOk ? 'pos' : 'neg'),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderGates(report) {
    const wrap = document.getElementById('tpc-gates');
    const gates = report.gates || [];
    if (!gates.length) { wrap.innerHTML = `<div class="muted" data-i18n="view.trade_plan_checklist.empty.gates">No gates.</div>`; return; }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th><th data-i18n="view.trade_plan_checklist.th.gate">Gate</th><th data-i18n="view.trade_plan_checklist.th.pass">Pass?</th><th data-i18n="view.trade_plan_checklist.th.detail">Detail</th>
            </tr></thead>
            <tbody>
                ${gates.map((g, i) => `<tr>
                    <td>${i + 1}</td>
                    <td><strong>${esc(gateLabel(g.gate))}</strong></td>
                    <td class="${gateCls(g.passed)}">${gateIcon(g.passed)} ${t(g.passed ? 'common.pass' : 'common.fail')}</td>
                    <td class="muted">${esc(g.reason)}</td>
                </tr>`).join('')}
            </tbody>
        </table>
    `;
}

function showErr(msg) {
    const el = document.getElementById('tpc-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('tpc-err').style.display = 'none'; }
