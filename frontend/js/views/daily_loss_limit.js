// Daily Loss Limit view — hard kill-switch tier checker.
//
// 4 states: OK / Warning (≥50%) / CutSize (≥75%) / KillSwitch (≥100%).
// Visualizes the loss against the binding limit as a progress bar with
// threshold markers, plus a state badge + action hint.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    validateInputs, buildBody, localEvaluate, stateBadge, decToNum,
    makeDemoData, fmtUSD, fmtUSDSigned, fmtPct,
} from '../_daily_loss_limit_inputs.js';

let state = { params: makeDemoData('cut-size') };

export async function renderDailyLossLimit(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// DAILY LOSS LIMIT</h1>

        <div class="chart-panel">
            <h2>Account</h2>
            <div class="inline-form">
                <label>Account equity ($)
                    <input id="dl-eq" type="number" step="any" min="0" value="${state.params.account_equity}"></label>
                <label>Today's P&L ($) — negative = loss
                    <input id="dl-pnl" type="number" step="any" value="${state.params.today_pnl}"></label>
            </div>
        </div>

        <div class="chart-panel">
            <h2>Limit config</h2>
            <div class="inline-form">
                <label>Max daily $ loss (0 = pct only)
                    <input id="dl-md" type="number" step="any" min="0" value="${state.params.max_daily_loss_dollars}"></label>
                <label>Max daily % loss (decimal; 0.02 = 2%)
                    <input id="dl-mp" type="number" step="any" min="0" max="1" value="${state.params.max_daily_loss_pct}"></label>
            </div>
            <div class="inline-form">
                <label>Warning threshold (decimal of binding limit)
                    <input id="dl-wt" type="number" step="any" min="0" max="5" value="${state.params.warning_threshold}"></label>
                <label>Cut-size threshold
                    <input id="dl-ct" type="number" step="any" min="0" max="5" value="${state.params.cut_size_threshold}"></label>
                <label>Kill threshold
                    <input id="dl-kt" type="number" step="any" min="0" max="5" value="${state.params.kill_threshold}"></label>
                <button id="dl-run" class="primary" type="button">Evaluate</button>
            </div>
            <div class="inline-form">
                <button id="dl-demo-ok"      class="secondary" type="button">Demo: OK (profit day)</button>
                <button id="dl-demo-warn"    class="secondary" type="button">Demo: WARNING (60%)</button>
                <button id="dl-demo-cut"     class="secondary" type="button">Demo: CUT SIZE (80%)</button>
                <button id="dl-demo-kill"    class="secondary" type="button">Demo: KILL SWITCH (over)</button>
                <button id="dl-demo-tight"   class="secondary" type="button">Demo: tight pct-binds</button>
            </div>
            <p class="muted">Binding limit = the SMALLER of (account_equity × max_pct) and (max_$ cap when set). Pct-only mode: set max_$ to 0.</p>
        </div>

        <div id="dl-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>Loss vs limit</h2>
            <div id="dl-bar"></div>
            <p class="muted">Track marks at warning (yellow) / cut-size (orange) / kill (red).
                Fill color reflects current state. 0% = no loss; 100% = at the binding cap.</p>
        </div>

        <div id="dl-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (kind) => {
        state.params = makeDemoData(kind);
        for (const [id, k] of [
            ['dl-eq', 'account_equity'],
            ['dl-pnl', 'today_pnl'],
            ['dl-md', 'max_daily_loss_dollars'],
            ['dl-mp', 'max_daily_loss_pct'],
            ['dl-wt', 'warning_threshold'],
            ['dl-ct', 'cut_size_threshold'],
            ['dl-kt', 'kill_threshold'],
        ]) {
            document.getElementById(id).value = state.params[k];
        }
    };
    document.getElementById('dl-demo-ok').addEventListener('click',    () => loadDemo('ok'));
    document.getElementById('dl-demo-warn').addEventListener('click',  () => loadDemo('warning'));
    document.getElementById('dl-demo-cut').addEventListener('click',   () => loadDemo('cut-size'));
    document.getElementById('dl-demo-kill').addEventListener('click',  () => loadDemo('kill'));
    document.getElementById('dl-demo-tight').addEventListener('click', () => loadDemo('tight'));
    document.getElementById('dl-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
    readInputs(); void compute(tok);
}

function readInputs() {
    const get = id => Number(document.getElementById(id).value);
    state.params = {
        today_pnl: get('dl-pnl'),
        max_daily_loss_dollars: get('dl-md'),
        max_daily_loss_pct: get('dl-mp'),
        account_equity: get('dl-eq'),
        warning_threshold: get('dl-wt'),
        cut_size_threshold: get('dl-ct'),
        kill_threshold: get('dl-kt'),
    };
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.params);
    if (err) { showErr(err); return; }
    // Pre-flight render with local eval so the UI is responsive while
    // the network call settles.
    const local = localEvaluate(state.params);
    renderSummary({ state: local.state, pct_of_limit: String(local.pct),
                    binding_limit: String(local.limit), today_realized_loss: String(local.loss),
                    note: stateBadge(local.state).hint }, true);
    renderBar(local.pct, local.state);
    let resp;
    try {
        resp = await api.discDailyLossLimit(buildBody(state.params));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderBar(decToNum(resp.pct_of_limit), resp.state);
}

function renderSummary(r, pending) {
    const badge = stateBadge(r.state);
    const local = localEvaluate(state.params);
    const parityOk = r.state === local.state;
    document.getElementById('dl-summary').innerHTML = [
        card('State',          badge.label + (pending ? ' (local)' : ''), badge.cls),
        card('Action',         badge.hint),
        card('Today P&L',      fmtUSDSigned(state.params.today_pnl),
            state.params.today_pnl >= 0 ? 'pos' : 'neg'),
        card('Realized loss',  fmtUSD(decToNum(r.today_realized_loss))),
        card('Binding limit',  fmtUSD(decToNum(r.binding_limit))),
        card('% of limit',     fmtPct(decToNum(r.pct_of_limit)), badge.cls),
        card('Note',           r.note || badge.hint),
        card('Local check',    local.state.toUpperCase(), parityOk ? 'pos' : 'neg'),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderBar(pct, stateKey) {
    const wrap = document.getElementById('dl-bar');
    const clamped = Math.max(0, Math.min(1.5, pct));
    const fillPct = (Math.min(clamped, 1.0) * 100).toFixed(2);
    const fillCls =
        stateKey === 'kill_switch' ? 'dl-fill-kill' :
        stateKey === 'cut_size'    ? 'dl-fill-cut' :
        stateKey === 'warning'     ? 'dl-fill-warn' :
                                       'dl-fill-ok';
    const wPct = (state.params.warning_threshold * 100).toFixed(2);
    const cPct = (state.params.cut_size_threshold * 100).toFixed(2);
    const kPct = (state.params.kill_threshold * 100).toFixed(2);
    wrap.innerHTML = `
        <div class="dl-bar-track">
            <div class="dl-bar-fill ${fillCls}" data-pct="${fillPct}"></div>
            <div class="dl-bar-mark dl-mark-warn" data-pct="${wPct}"></div>
            <div class="dl-bar-mark dl-mark-cut"  data-pct="${cPct}"></div>
            <div class="dl-bar-mark dl-mark-kill" data-pct="${kPct}"></div>
            <div class="dl-bar-label">${esc(fmtPct(pct))} of binding limit ${esc(fmtUSD(localEvaluate(state.params).limit))}</div>
        </div>
        <div class="dl-bar-legend">
            <span class="dl-legend-warn">▎ warning ${wPct}%</span>
            <span class="dl-legend-cut">▎ cut ${cPct}%</span>
            <span class="dl-legend-kill">▎ kill ${kPct}%</span>
        </div>
    `;
    requestAnimationFrame(() => {
        wrap.querySelectorAll('.dl-bar-fill').forEach(el => {
            el.style.width = el.dataset.pct + '%';
        });
        wrap.querySelectorAll('.dl-bar-mark').forEach(el => {
            el.style.left = el.dataset.pct + '%';
        });
    });
}

function showErr(msg) {
    const el = document.getElementById('dl-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('dl-err').style.display = 'none'; }
