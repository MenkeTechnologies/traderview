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
            <label style="display:block;margin-bottom:6px">Thesis (free text)
                <textarea id="tpc-thesis" rows="3" placeholder="Why does this work? Catalyst? Setup? Confirmation?">${esc(state.plan.thesis)}</textarea>
            </label>
            <div class="inline-form">
                <label><span data-i18n="view.trade_plan_checklist.label.side">Side</span>
                    <select id="tpc-side">
                        <option data-i18n="view.trade_plan_checklist.opt.long" value="long"  ${state.plan.is_long  ? 'selected' : ''}>Long</option>
                        <option data-i18n="view.trade_plan_checklist.opt.short" value="short" ${!state.plan.is_long ? 'selected' : ''}>Short</option>
                    </select></label>
                <label><span data-i18n="view.trade_plan_checklist.label.entry">Entry $</span>
                    <input id="tpc-entry" type="number" step="any" min="0" value="${state.plan.entry_price}"></label>
                <label><span data-i18n="view.trade_plan_checklist.label.stop">Stop $ (blank = none)</span>
                    <input id="tpc-stop"  type="number" step="any" min="0" value="${state.plan.stop_price ?? ''}"></label>
                <label><span data-i18n="view.trade_plan_checklist.label.target">Target $ (blank = none)</span>
                    <input id="tpc-tgt"   type="number" step="any" min="0" value="${state.plan.target_price ?? ''}"></label>
            </div>
            <div class="inline-form">
                <label><span data-i18n="view.trade_plan_checklist.label.risk">Risk $ (notional dollars on this trade)</span>
                    <input id="tpc-risk" type="number" step="any" min="0" value="${state.plan.risk_dollars}"></label>
                <label><span data-i18n="view.trade_plan_checklist.label.equity">Account equity $</span>
                    <input id="tpc-eq" type="number" step="any" min="0" value="${state.plan.account_equity}"></label>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.trade_plan_checklist.h2.gate_config">Gate config</h2>
            <div class="inline-form">
                <label><span data-i18n="view.trade_plan_checklist.label.min_words">Min thesis words</span>
                    <input id="tpc-mw" type="number" step="1" min="0" value="${state.config.min_thesis_words}"></label>
                <label><span data-i18n="view.trade_plan_checklist.label.min_r">Min R-multiple</span>
                    <input id="tpc-mr" type="number" step="any" min="0" value="${state.config.min_r_multiple}"></label>
                <label><span data-i18n="view.trade_plan_checklist.label.max_risk_pct">Max risk % (decimal — 0.02 = 2%)</span>
                    <input id="tpc-mrp" type="number" step="any" min="0" max="1" value="${state.config.max_risk_pct_per_trade}"></label>
                <button data-i18n="view.trade_plan_checklist.btn.evaluate" id="tpc-run" class="primary" type="button">Evaluate</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.trade_plan_checklist.btn.demo_good_plan" id="tpc-demo-good"     class="secondary" type="button">Demo: GOOD plan</button>
                <button data-i18n="view.trade_plan_checklist.btn.demo_missing_stop" id="tpc-demo-no-stop"  class="secondary" type="button">Demo: missing stop</button>
                <button data-i18n="view.trade_plan_checklist.btn.demo_weak_r" id="tpc-demo-weak-r"   class="secondary" type="button">Demo: weak R</button>
                <button data-i18n="view.trade_plan_checklist.btn.demo_oversize_risk" id="tpc-demo-oversize" class="secondary" type="button">Demo: oversize risk</button>
                <button data-i18n="view.trade_plan_checklist.btn.demo_wrong_direction" id="tpc-demo-wrong"    class="secondary" type="button">Demo: wrong direction</button>
                <button data-i18n="view.trade_plan_checklist.btn.demo_short_good" id="tpc-demo-short"    class="secondary" type="button">Demo: SHORT (good)</button>
                <button data-i18n="view.trade_plan_checklist.btn.demo_no_thesis" id="tpc-demo-noth"     class="secondary" type="button">Demo: no thesis</button>
            </div>
        </div>

        <div id="tpc-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.trade_plan_checklist.h2.gate_results">Gate results</h2>
            <div id="tpc-gates"></div>
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
    if (err) { showErr(err); return; }

    const local = localEvaluate(state.plan, state.config);
    renderSummary(local, true);
    renderGates(local);

    let resp;
    try {
        resp = await api.discTradePlanChecklist(buildBody(state.plan, state.config));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderGates(resp);
}

function renderSummary(r, pending) {
    const allCls = r.all_passed ? 'pos' : 'neg';
    const local = localEvaluate(state.plan, state.config);
    const parityOk = r.all_passed === local.all_passed;
    document.getElementById('tpc-summary').innerHTML = [
        card(t('view.trade_plan_checklist.card.verdict'), r.all_passed ? 'PASS' : 'FAIL' + (pending ? ' (local)' : ''), allCls),
        card(t('view.trade_plan_checklist.card.gates_passed'), `${r.gates.filter(g => g.passed).length} / ${r.gates.length}`, allCls),
        card(t('view.trade_plan_checklist.card.r_multiple'), fmtR(r.computed_r_multiple),
            r.computed_r_multiple != null && r.computed_r_multiple >= state.config.min_r_multiple ? 'pos' : 'neg'),
        card(t('view.trade_plan_checklist.card.risk'), fmtPct(r.risk_pct),
            r.risk_pct <= state.config.max_risk_pct_per_trade ? 'pos' : 'neg'),
        card(t('view.trade_plan_checklist.card.side'),  state.plan.is_long ? 'LONG' : 'SHORT',
            state.plan.is_long ? 'pos' : 'neg'),
        card(t('view.trade_plan_checklist.card.local_parity'), parityOk ? 'OK' : 'DIVERGED', parityOk ? 'pos' : 'neg'),
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
    if (!gates.length) { wrap.innerHTML = '<div class="muted">No gates.</div>'; return; }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th><th data-i18n="view.trade_plan_checklist.th.gate">Gate</th><th data-i18n="view.trade_plan_checklist.th.pass">Pass?</th><th data-i18n="view.trade_plan_checklist.th.detail">Detail</th>
            </tr></thead>
            <tbody>
                ${gates.map((g, i) => `<tr>
                    <td>${i + 1}</td>
                    <td><strong>${esc(gateLabel(g.gate))}</strong></td>
                    <td class="${gateCls(g.passed)}">${gateIcon(g.passed)} ${g.passed ? 'PASS' : 'FAIL'}</td>
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
