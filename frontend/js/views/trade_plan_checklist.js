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

let state = {
    plan: makeDemoData('good'),
    config: { ...DEFAULT_CONFIG },
};

export async function renderTradePlanChecklist(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// TRADE PLAN CHECKLIST</h1>

        <div class="chart-panel">
            <h2>Planned trade</h2>
            <label style="display:block;margin-bottom:6px">Thesis (free text)
                <textarea id="tpc-thesis" rows="3" placeholder="Why does this work? Catalyst? Setup? Confirmation?">${esc(state.plan.thesis)}</textarea>
            </label>
            <div class="inline-form">
                <label>Side
                    <select id="tpc-side">
                        <option value="long"  ${state.plan.is_long  ? 'selected' : ''}>Long</option>
                        <option value="short" ${!state.plan.is_long ? 'selected' : ''}>Short</option>
                    </select></label>
                <label>Entry $
                    <input id="tpc-entry" type="number" step="any" min="0" value="${state.plan.entry_price}"></label>
                <label>Stop $ (blank = none)
                    <input id="tpc-stop"  type="number" step="any" min="0" value="${state.plan.stop_price ?? ''}"></label>
                <label>Target $ (blank = none)
                    <input id="tpc-tgt"   type="number" step="any" min="0" value="${state.plan.target_price ?? ''}"></label>
            </div>
            <div class="inline-form">
                <label>Risk $ (notional dollars on this trade)
                    <input id="tpc-risk" type="number" step="any" min="0" value="${state.plan.risk_dollars}"></label>
                <label>Account equity $
                    <input id="tpc-eq" type="number" step="any" min="0" value="${state.plan.account_equity}"></label>
            </div>
        </div>

        <div class="chart-panel">
            <h2>Gate config</h2>
            <div class="inline-form">
                <label>Min thesis words
                    <input id="tpc-mw" type="number" step="1" min="0" value="${state.config.min_thesis_words}"></label>
                <label>Min R-multiple
                    <input id="tpc-mr" type="number" step="any" min="0" value="${state.config.min_r_multiple}"></label>
                <label>Max risk % (decimal — 0.02 = 2%)
                    <input id="tpc-mrp" type="number" step="any" min="0" max="1" value="${state.config.max_risk_pct_per_trade}"></label>
                <button id="tpc-run" class="primary" type="button">Evaluate</button>
            </div>
            <div class="inline-form">
                <button id="tpc-demo-good"     class="secondary" type="button">Demo: GOOD plan</button>
                <button id="tpc-demo-no-stop"  class="secondary" type="button">Demo: missing stop</button>
                <button id="tpc-demo-weak-r"   class="secondary" type="button">Demo: weak R</button>
                <button id="tpc-demo-oversize" class="secondary" type="button">Demo: oversize risk</button>
                <button id="tpc-demo-wrong"    class="secondary" type="button">Demo: wrong direction</button>
                <button id="tpc-demo-short"    class="secondary" type="button">Demo: SHORT (good)</button>
                <button id="tpc-demo-noth"     class="secondary" type="button">Demo: no thesis</button>
            </div>
        </div>

        <div id="tpc-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>Gate results</h2>
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
        card('Verdict', r.all_passed ? 'PASS' : 'FAIL' + (pending ? ' (local)' : ''), allCls),
        card('Gates passed', `${r.gates.filter(g => g.passed).length} / ${r.gates.length}`, allCls),
        card('R-multiple', fmtR(r.computed_r_multiple),
            r.computed_r_multiple != null && r.computed_r_multiple >= state.config.min_r_multiple ? 'pos' : 'neg'),
        card('Risk %', fmtPct(r.risk_pct),
            r.risk_pct <= state.config.max_risk_pct_per_trade ? 'pos' : 'neg'),
        card('Side',  state.plan.is_long ? 'LONG' : 'SHORT',
            state.plan.is_long ? 'pos' : 'neg'),
        card('Local parity', parityOk ? 'OK' : 'DIVERGED', parityOk ? 'pos' : 'neg'),
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
                <th>#</th><th>Gate</th><th>Pass?</th><th>Detail</th>
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
