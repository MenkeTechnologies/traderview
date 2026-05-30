// Time-in-Force view — single-order TIF validator. Covers DAY / GTC /
// IOC / FOK / GTD. Big verdict badge + reason text + cheat sheet.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    TIF_KINDS, validateInputs, buildBody, localEvaluate, actionBadge,
    makeDemoOrder, localDtToIsoUtc, isoUtcToLocalDt, isoToDate,
} from '../_time_in_force_inputs.js';

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
let state = makeDemoOrder('gtc-keep');

export async function renderTimeInForce(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.time_in_force.h1.time_in_force" class="view-title">// TIME IN FORCE</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.time_in_force.h2.order">Order</h2>
            <div class="inline-form">
                <label><span data-i18n="view.time_in_force.label.tif">TIF</span>
                    <select id="tif-kind" data-tip="view.time_in_force.tip.tif">
                        ${TIF_KINDS.map(k => `<option value="${k}" ${state.order.tif === k ? 'selected' : ''}>${k.toUpperCase()}</option>`).join('')}
                    </select></label>
                <label><span data-i18n="view.time_in_force.label.orig_qty">Original qty</span>
                    <input id="tif-oq" type="number" step="any" min="0" value="${state.order.original_qty}" data-tip="view.time_in_force.tip.orig_qty"></label>
                <label><span data-i18n="view.time_in_force.label.filled_qty">Filled qty</span>
                    <input id="tif-fq" type="number" step="any" min="0" value="${state.order.filled_qty}" data-tip="view.time_in_force.tip.filled_qty"></label>
            </div>
            <div class="inline-form">
                <label><span data-i18n="view.time_in_force.label.placed_at">Placed at</span> <small class="muted" data-i18n="view.time_in_force.label.placed_at_hint">(your local time → UTC at the wire)</small>
                    <input id="tif-placed" type="datetime-local" value="${esc(isoUtcToLocalDt(state.order.placed_at))}" data-tip="view.time_in_force.tip.placed"></label>
                <label><span data-i18n="view.time_in_force.label.good_until">Good until (GTD only)</span>
                    <input id="tif-good" type="date" value="${esc(state.good_until_in_order || '')}" data-tip="view.time_in_force.tip.good_until"></label>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.time_in_force.h2.clock_session">Clock + session</h2>
            <div class="inline-form">
                <label><span data-i18n="view.time_in_force.label.now">Now</span>
                    <input id="tif-now" type="datetime-local" value="${esc(isoUtcToLocalDt(state.now))}" data-tip="view.time_in_force.tip.now"></label>
                <label><span data-i18n="view.time_in_force.label.session_open">Session open (UTC date)</span>
                    <input id="tif-sess" type="date" value="${esc(state.session_open)}" data-tip="view.time_in_force.tip.session"></label>
                <button data-i18n="view.time_in_force.btn.snap_now_session_to_current_time" id="tif-now-snap" class="secondary" type="button" data-tip="view.time_in_force.tip.snap_now" data-shortcut="time_in_force_snap_now">Snap "now" + session to current time</button>
                <button data-i18n="view.time_in_force.btn.evaluate" id="tif-run" class="primary" type="button" data-tip="view.time_in_force.tip.run" data-shortcut="time_in_force_run">Evaluate</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.time_in_force.btn.day_keep" id="tif-demo-day-keep"    class="secondary" type="button">DAY → keep</button>
                <button data-i18n="view.time_in_force.btn.day_cancel_next_session" id="tif-demo-day-cancel"  class="secondary" type="button">DAY → cancel (next session)</button>
                <button data-i18n="view.time_in_force.btn.gtc_keep_30d_old" id="tif-demo-gtc-keep"    class="secondary" type="button">GTC → keep (30d old)</button>
                <button data-i18n="view.time_in_force.btn.gtc_cancel_old" id="tif-demo-gtc-cancel"  class="secondary" type="button">GTC → cancel (>90d)</button>
                <button data-i18n="view.time_in_force.btn.ioc_cancel_partial" id="tif-demo-ioc"         class="secondary" type="button">IOC → cancel (partial)</button>
                <button data-i18n="view.time_in_force.btn.fok_cancel_no_fill" id="tif-demo-fok-no"      class="secondary" type="button">FOK → cancel (no fill)</button>
                <button data-i18n="view.time_in_force.btn.fok_cancel_partial" id="tif-demo-fok-partial" class="secondary" type="button">FOK → cancel (partial)</button>
                <button data-i18n="view.time_in_force.btn.fok_completed" id="tif-demo-fok-done"    class="secondary" type="button">FOK → completed</button>
                <button data-i18n="view.time_in_force.btn.gtd_keep_future_date" id="tif-demo-gtd-keep"    class="secondary" type="button">GTD → keep (future date)</button>
                <button data-i18n="view.time_in_force.btn.gtd_cancel_past_date" id="tif-demo-gtd-cancel"  class="secondary" type="button">GTD → cancel (past date)</button>
                <button data-i18n="view.time_in_force.btn.gtd_cancel_no_date" id="tif-demo-gtd-miss"    class="secondary" type="button">GTD → cancel (no date)</button>
                <button data-i18n="view.time_in_force.btn.completed_fully_filled" id="tif-demo-completed"   class="secondary" type="button">→ completed (fully filled)</button>
            </div>
        </div>

        <div id="tif-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.time_in_force.h2.tif_cheat_sheet">TIF cheat sheet</h2>
            <table class="lq-table">
                <thead><tr><th data-i18n="view.time_in_force.th.tif">TIF</th><th data-i18n="view.time_in_force.th.semantics">Semantics</th><th data-i18n="view.time_in_force.th.cancel_trigger">Cancel trigger</th></tr></thead>
                <tbody>
                    <tr data-i18n-html="view.time_in_force.cheat.day"><td><strong>DAY</strong></td><td>Expires at session close.</td><td>session_open &gt; placed_date</td></tr>
                    <tr data-i18n-html="view.time_in_force.cheat.gtc"><td><strong>GTC</strong></td><td>Good Till Cancelled (with 90-day broker timeout).</td><td>age &gt; 90 days</td></tr>
                    <tr data-i18n-html="view.time_in_force.cheat.ioc"><td><strong>IOC</strong></td><td>Immediate Or Cancel: fill what's avail now, cancel rest.</td><td>any remaining qty</td></tr>
                    <tr data-i18n-html="view.time_in_force.cheat.fok"><td><strong>FOK</strong></td><td>Fill Or Kill: all-or-nothing immediately.</td><td>partial / no fill</td></tr>
                    <tr data-i18n-html="view.time_in_force.cheat.gtd"><td><strong>GTD</strong></td><td>Good Till Date: expires after good_until.</td><td>session_open &gt; good_until OR missing date</td></tr>
                </tbody>
            </table>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.time_in_force.h2.qty_chart">Order quantities: original / filled / remaining</h2>
            <div id="tif-chart" style="width:100%;height:200px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.time_in_force.h2.age_chart">Order age (days since placed) vs cutoffs</h2>
            <div id="tif-age-chart" style="width:100%;height:180px"></div>
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
    if (err) { showErr(err); showToast(t('view.time_in_force.toast.invalid'), { level: 'warning' }); return; }
    const local = localEvaluate(state.order, state.now, state.session_open);
    renderSummary(local, true);
    let resp;
    try {
        resp = await api.discTimeInForce(buildBody(state.order, state.now, state.session_open));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        showToast(t('view.time_in_force.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderQtyChart();
    renderAgeChart();
    const action = String(resp.action || '').toUpperCase();
    const level = action === 'CANCEL' ? 'warning' : action === 'COMPLETED' ? 'info' : 'success';
    showToast(t('view.time_in_force.toast.evaluated', { tif: state.order.tif.toUpperCase(), action }), { level });
}

function renderAgeChart() {
    const el = document.getElementById('tif-age-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const placed = state.order.placed_at ? new Date(state.order.placed_at).getTime() : NaN;
    const now = state.now ? new Date(state.now).getTime() : NaN;
    if (!Number.isFinite(placed) || !Number.isFinite(now)) {
        el.innerHTML = `<div class="muted" data-i18n="view.time_in_force.empty_age_chart">${esc(t('view.time_in_force.empty_age_chart'))}</div>`;
        return;
    }
    const ageDays = Math.max(0, (now - placed) / 86400000);
    const labels = [
        t('view.time_in_force.chart.age'),
    ];
    const xs = [1];
    const ys = [ageDays];
    const dayCut = xs.map(() => 1);    // DAY cutoff = 1 day
    const gtcCut = xs.map(() => 90);   // GTC cutoff = 90 days
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 160,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.time_in_force.chart.bucket') },
            { label: t('view.time_in_force.chart.age'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 22, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.time_in_force.chart.day_cutoff'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
            { label: t('view.time_in_force.chart.gtc_cutoff'),
              stroke: '#ff3860', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys, dayCut, gtcCut], el);
}

function renderQtyChart() {
    const el = document.getElementById('tif-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const orig   = Number(state.order.original_qty);
    const filled = Number(state.order.filled_qty);
    const remain = Math.max(0, orig - filled);
    if (!Number.isFinite(orig) || orig <= 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.time_in_force.empty_chart">${esc(t('view.time_in_force.empty_chart'))}</div>`;
        return;
    }
    const labels = [
        t('view.time_in_force.chart.original'),
        t('view.time_in_force.chart.filled'),
        t('view.time_in_force.chart.remaining'),
    ];
    const ys = [orig, filled, remain];
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.time_in_force.chart.bucket') },
            { label: t('view.time_in_force.chart.qty'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 16, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderSummary(verdict, pending) {
    const badge = actionBadge(verdict.action);
    const local = localEvaluate(state.order, state.now, state.session_open);
    const parityOk = verdict.action === local.action && verdict.reason === local.reason;
    document.getElementById('tif-summary').innerHTML = [
        card(t('view.time_in_force.card.verdict'),     badge.label + (pending ? t('common.suffix.local') : ''), badge.cls),
        card(t('view.time_in_force.card.reason'),      verdict.reason),
        card(t('view.time_in_force.card.tif'),         state.order.tif.toUpperCase()),
        card(t('view.time_in_force.card.original_qty'), String(state.order.original_qty)),
        card(t('view.time_in_force.card.filled_qty'),   String(state.order.filled_qty),
            state.order.filled_qty > 0 ? 'pos' : ''),
        card(t('view.time_in_force.card.remaining_qty'), String(state.order.original_qty - state.order.filled_qty),
            (state.order.original_qty - state.order.filled_qty) > 0 ? 'neg' : 'pos'),
        card(t('view.time_in_force.card.good_until'),   state.order.good_until || '—'),
        card(t('view.time_in_force.card.local_parity'), parityOk ? t('common.ok') : t('view.time_in_force.parity.diverged', { action: local.action, reason: local.reason }),
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
