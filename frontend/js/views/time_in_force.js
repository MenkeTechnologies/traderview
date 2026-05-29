// Time-in-Force view — single-order TIF validator. Covers DAY / GTC /
// IOC / FOK / GTD. Big verdict badge + reason text + cheat sheet.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    TIF_KINDS, validateInputs, buildBody, localEvaluate, actionBadge,
    makeDemoOrder, localDtToIsoUtc, isoUtcToLocalDt, isoToDate,
} from '../_time_in_force_inputs.js';

let state = makeDemoOrder('gtc-keep');

export async function renderTimeInForce(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// TIME IN FORCE</h1>

        <div class="chart-panel">
            <h2>Order</h2>
            <div class="inline-form">
                <label>TIF
                    <select id="tif-kind">
                        ${TIF_KINDS.map(k => `<option value="${k}" ${state.order.tif === k ? 'selected' : ''}>${k.toUpperCase()}</option>`).join('')}
                    </select></label>
                <label>Original qty
                    <input id="tif-oq" type="number" step="any" min="0" value="${state.order.original_qty}"></label>
                <label>Filled qty
                    <input id="tif-fq" type="number" step="any" min="0" value="${state.order.filled_qty}"></label>
            </div>
            <div class="inline-form">
                <label>Placed at <small class="muted">(your local time → UTC at the wire)</small>
                    <input id="tif-placed" type="datetime-local" value="${esc(isoUtcToLocalDt(state.order.placed_at))}"></label>
                <label>Good until (GTD only)
                    <input id="tif-good" type="date" value="${esc(state.good_until_in_order || '')}"></label>
            </div>
        </div>

        <div class="chart-panel">
            <h2>Clock + session</h2>
            <div class="inline-form">
                <label>Now
                    <input id="tif-now" type="datetime-local" value="${esc(isoUtcToLocalDt(state.now))}"></label>
                <label>Session open (UTC date)
                    <input id="tif-sess" type="date" value="${esc(state.session_open)}"></label>
                <button id="tif-now-snap" class="secondary" type="button">Snap "now" + session to current time</button>
                <button id="tif-run" class="primary" type="button">Evaluate</button>
            </div>
            <div class="inline-form">
                <button id="tif-demo-day-keep"    class="secondary" type="button">DAY → keep</button>
                <button id="tif-demo-day-cancel"  class="secondary" type="button">DAY → cancel (next session)</button>
                <button id="tif-demo-gtc-keep"    class="secondary" type="button">GTC → keep (30d old)</button>
                <button id="tif-demo-gtc-cancel"  class="secondary" type="button">GTC → cancel (>90d)</button>
                <button id="tif-demo-ioc"         class="secondary" type="button">IOC → cancel (partial)</button>
                <button id="tif-demo-fok-no"      class="secondary" type="button">FOK → cancel (no fill)</button>
                <button id="tif-demo-fok-partial" class="secondary" type="button">FOK → cancel (partial)</button>
                <button id="tif-demo-fok-done"    class="secondary" type="button">FOK → completed</button>
                <button id="tif-demo-gtd-keep"    class="secondary" type="button">GTD → keep (future date)</button>
                <button id="tif-demo-gtd-cancel"  class="secondary" type="button">GTD → cancel (past date)</button>
                <button id="tif-demo-gtd-miss"    class="secondary" type="button">GTD → cancel (no date)</button>
                <button id="tif-demo-completed"   class="secondary" type="button">→ completed (fully filled)</button>
            </div>
        </div>

        <div id="tif-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>TIF cheat sheet</h2>
            <table class="lq-table">
                <thead><tr><th>TIF</th><th>Semantics</th><th>Cancel trigger</th></tr></thead>
                <tbody>
                    <tr><td><strong>DAY</strong></td><td>Expires at session close.</td><td>session_open &gt; placed_date</td></tr>
                    <tr><td><strong>GTC</strong></td><td>Good Till Cancelled (with 90-day broker timeout).</td><td>age &gt; 90 days</td></tr>
                    <tr><td><strong>IOC</strong></td><td>Immediate Or Cancel: fill what's avail now, cancel rest.</td><td>any remaining qty</td></tr>
                    <tr><td><strong>FOK</strong></td><td>Fill Or Kill: all-or-nothing immediately.</td><td>partial / no fill</td></tr>
                    <tr><td><strong>GTD</strong></td><td>Good Till Date: expires after good_until.</td><td>session_open &gt; good_until OR missing date</td></tr>
                </tbody>
            </table>
        </div>

        <div id="tif-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (kind) => {
        const d = makeDemoOrder(kind);
        state = d;
        // GTD presets put good_until in the wrapper field; copy it onto order.
        if (state.good_until_in_order !== undefined) {
            state.order.good_until = state.good_until_in_order;
        }
        document.getElementById('tif-kind').value    = state.order.tif;
        document.getElementById('tif-oq').value      = state.order.original_qty;
        document.getElementById('tif-fq').value      = state.order.filled_qty;
        document.getElementById('tif-placed').value  = isoUtcToLocalDt(state.order.placed_at);
        document.getElementById('tif-good').value    = state.order.good_until || '';
        document.getElementById('tif-now').value     = isoUtcToLocalDt(state.now);
        document.getElementById('tif-sess').value    = state.session_open;
    };
    document.getElementById('tif-demo-day-keep').addEventListener('click',    () => loadDemo('day-keep'));
    document.getElementById('tif-demo-day-cancel').addEventListener('click',  () => loadDemo('day-cancel'));
    document.getElementById('tif-demo-gtc-keep').addEventListener('click',    () => loadDemo('gtc-keep'));
    document.getElementById('tif-demo-gtc-cancel').addEventListener('click',  () => loadDemo('gtc-cancel'));
    document.getElementById('tif-demo-ioc').addEventListener('click',         () => loadDemo('ioc-cancel'));
    document.getElementById('tif-demo-fok-no').addEventListener('click',      () => loadDemo('fok-no-fill'));
    document.getElementById('tif-demo-fok-partial').addEventListener('click', () => loadDemo('fok-partial'));
    document.getElementById('tif-demo-fok-done').addEventListener('click',    () => loadDemo('fok-completed'));
    document.getElementById('tif-demo-gtd-keep').addEventListener('click',    () => loadDemo('gtd-keep'));
    document.getElementById('tif-demo-gtd-cancel').addEventListener('click',  () => loadDemo('gtd-cancel'));
    document.getElementById('tif-demo-gtd-miss').addEventListener('click',    () => loadDemo('gtd-missing'));
    document.getElementById('tif-demo-completed').addEventListener('click',   () => loadDemo('completed'));
    document.getElementById('tif-now-snap').addEventListener('click', () => {
        const now = new Date();
        document.getElementById('tif-now').value  = isoUtcToLocalDt(now.toISOString());
        document.getElementById('tif-sess').value = isoToDate(now.toISOString());
    });
    document.getElementById('tif-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function readInputs() {
    const tif = document.getElementById('tif-kind').value;
    state = {
        order: {
            tif,
            original_qty: Number(document.getElementById('tif-oq').value),
            filled_qty:   Number(document.getElementById('tif-fq').value),
            placed_at:    localDtToIsoUtc(document.getElementById('tif-placed').value),
            good_until:   tif === 'gtd' ? (document.getElementById('tif-good').value || null) : null,
        },
        now:          localDtToIsoUtc(document.getElementById('tif-now').value),
        session_open: document.getElementById('tif-sess').value,
    };
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.order, state.now, state.session_open);
    if (err) { showErr(err); return; }
    const local = localEvaluate(state.order, state.now, state.session_open);
    renderSummary(local, true);
    let resp;
    try {
        resp = await api.discTimeInForce(buildBody(state.order, state.now, state.session_open));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
}

function renderSummary(verdict, pending) {
    const badge = actionBadge(verdict.action);
    const local = localEvaluate(state.order, state.now, state.session_open);
    const parityOk = verdict.action === local.action && verdict.reason === local.reason;
    document.getElementById('tif-summary').innerHTML = [
        card('Verdict',     badge.label + (pending ? ' (local)' : ''), badge.cls),
        card('Reason',      verdict.reason),
        card('TIF',         state.order.tif.toUpperCase()),
        card('Original qty', String(state.order.original_qty)),
        card('Filled qty',   String(state.order.filled_qty),
            state.order.filled_qty > 0 ? 'pos' : ''),
        card('Remaining qty', String(state.order.original_qty - state.order.filled_qty),
            (state.order.original_qty - state.order.filled_qty) > 0 ? 'neg' : 'pos'),
        card('Good until',   state.order.good_until || '—'),
        card('Local parity', parityOk ? 'OK' : `DIVERGED (${local.action}: ${local.reason})`,
            parityOk ? 'pos' : 'neg'),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('tif-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('tif-err').style.display = 'none'; }
