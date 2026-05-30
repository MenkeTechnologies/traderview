// Margin-call distance view. Dollar cushion before broker call.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_INPUTS, validateInputs, buildBody, localEvaluate, dec,
    triggerLmv, cushionBadge, makeDemoInput,
    fmtUSD, fmtUSDSigned, fmtPct, fmtMaintPct,
} from '../_margin_call_inputs.js';

let state = { ...DEFAULT_INPUTS };

export async function renderMarginCall(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.margin_call.h1.title" class="view-title">// MARGIN CALL DISTANCE</h1>

        <div class="chart-panel" data-context-scope="margin-call">
            <h2 data-i18n="view.margin_call.h2.account">Account snapshot</h2>
            <div class="inline-form">
                <label><span data-i18n="view.margin_call.label.lmv">Long market value ($)</span>
                    <input id="mc-lmv" type="number" step="any" min="0" value="${state.long_market_value}" data-tip="view.margin_call.tip.lmv"></label>
                <label><span data-i18n="view.margin_call.label.debt">Margin debt ($)</span>
                    <input id="mc-debt" type="number" step="any" min="0" value="${state.margin_debt}" data-tip="view.margin_call.tip.debt"></label>
                <label><span data-i18n="view.margin_call.label.maint">Maintenance %  (decimal — 0.25 = 25%)</span>
                    <input id="mc-mp" type="number" step="any" min="0" max="1" value="${state.maintenance_pct}" data-tip="view.margin_call.tip.maint"></label>
                <button data-i18n="view.margin_call.btn.evaluate" id="mc-run" class="primary"
                        data-tip="view.margin_call.tip.evaluate" data-shortcut="margin_call_run" type="button">Evaluate</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.margin_call.btn.demo_cash"    id="mc-demo-cash"   class="secondary" type="button" data-tip="view.margin_call.tip.demo_cash">Demo: fully cash (no debt)</button>
                <button data-i18n="view.margin_call.btn.demo_std"     id="mc-demo-std"    class="secondary" type="button" data-tip="view.margin_call.tip.demo_std">Demo: standard ($20k cushion)</button>
                <button data-i18n="view.margin_call.btn.demo_call"    id="mc-demo-call"   class="secondary" type="button" data-tip="view.margin_call.tip.demo_call">Demo: already in call</button>
                <button data-i18n="view.margin_call.btn.demo_at"      id="mc-demo-at"     class="secondary" type="button" data-tip="view.margin_call.tip.demo_at">Demo: exactly at line ($0 cushion)</button>
                <button data-i18n="view.margin_call.btn.demo_high"    id="mc-demo-high"   class="secondary" type="button" data-tip="view.margin_call.tip.demo_high">Demo: high maint (40% small-cap)</button>
                <button data-i18n="view.margin_call.btn.demo_cashreq" id="mc-demo-cashreq" class="secondary" type="button" data-tip="view.margin_call.tip.demo_cashreq">Demo: 100% maint + $1 debt</button>
                <button data-i18n="view.margin_call.btn.demo_empty"   id="mc-demo-empty"  class="secondary" type="button" data-tip="view.margin_call.tip.demo_empty">Demo: no positions</button>
                <button data-i18n="view.margin_call.btn.demo_lever"   id="mc-demo-lever"  class="secondary" type="button" data-tip="view.margin_call.tip.demo_lever">Demo: leveraged bull (500k/300k)</button>
            </div>
            <p data-i18n="view.margin_call.hint.about" class="muted">Trigger LMV = debt / (1 − maintenance%). Cushion = LMV − trigger. Cushion = 0 is NOT yet in call (Rust uses strict &lt;). Reg-T retail standard is 25% maintenance; brokers raise for small-cap / volatile names.</p>
        </div>

        <div id="mc-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.margin_call.h2.bar">Equity vs maintenance wall</h2>
            <div id="mc-bar"></div>
            <p data-i18n="view.margin_call.hint.bar" class="muted">Cyan = current equity (LMV − debt). Yellow = trigger LMV. Bar shows current LMV; mark is where the call hits.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.margin_call.h2.dropdown_chart">Equity %-of-LMV across a market drop sweep</h2>
            <div id="mc-chart" style="width:100%;height:240px"></div>
        </div>

        <div id="mc-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('mc-lmv').value  = state.long_market_value;
        document.getElementById('mc-debt').value = state.margin_debt;
        document.getElementById('mc-mp').value   = state.maintenance_pct;
    };
    document.getElementById('mc-demo-cash').addEventListener('click',    () => loadDemo('fully-cash'));
    document.getElementById('mc-demo-std').addEventListener('click',     () => loadDemo('standard'));
    document.getElementById('mc-demo-call').addEventListener('click',    () => loadDemo('in-call'));
    document.getElementById('mc-demo-at').addEventListener('click',      () => loadDemo('at-line'));
    document.getElementById('mc-demo-high').addEventListener('click',    () => loadDemo('high-maint'));
    document.getElementById('mc-demo-cashreq').addEventListener('click', () => loadDemo('cash-only-with-debt'));
    document.getElementById('mc-demo-empty').addEventListener('click',   () => loadDemo('no-positions'));
    document.getElementById('mc-demo-lever').addEventListener('click',   () => loadDemo('leveraged-bull'));
    document.getElementById('mc-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function readInputs() {
    state = {
        long_market_value: Number(document.getElementById('mc-lmv').value),
        margin_debt:       Number(document.getElementById('mc-debt').value),
        maintenance_pct:   Number(document.getElementById('mc-mp').value),
    };
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.margin_call.toast.invalid'), { level: 'warning' }); return; }
    const local = localEvaluate(state);
    renderSummary(local, true);
    renderBar(local);
    let resp;
    try {
        resp = await api.calcMarginCall(buildBody(state));
    } catch (e) {
        showErr(`${t('view.margin_call.err.api')}: ${e.message || e}`);
        showToast(t('view.margin_call.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    const normalized = {
        ...resp,
        current_equity: dec(resp.current_equity),
        dollar_cushion: dec(resp.dollar_cushion),
    };
    renderSummary(normalized, false);
    renderBar(normalized);
    renderDropChart();
    const cushion = Number(normalized.dollar_cushion) || 0;
    if (cushion < 0) {
        showToast(t('view.margin_call.toast.in_call', { gap: Math.round(-cushion).toLocaleString() }), { level: 'error' });
    } else {
        const level = cushion < 5000 ? 'warning' : 'success';
        showToast(t('view.margin_call.toast.computed', { cushion: Math.round(cushion).toLocaleString() }), { level });
    }
}

function renderDropChart() {
    const el = document.getElementById('mc-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const lmv = Number(state.long_market_value);
    const debt = Number(state.margin_debt);
    const maint = Number(state.maintenance_pct);
    if (!Number.isFinite(lmv) || lmv <= 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.margin_call.empty_chart">${esc(t('view.margin_call.empty_chart'))}</div>`;
        return;
    }
    const xs = [];
    const equityPct = [];
    const maintLine = [];
    const zero = [];
    for (let drop = 0; drop >= -50; drop -= 1) {
        const newLmv = lmv * (1 + drop / 100);
        const newEquity = newLmv - debt;
        const pct = newLmv > 0 ? newEquity / newLmv : 0;
        xs.push(drop);
        equityPct.push(pct * 100);
        maintLine.push(maint * 100);
        zero.push(0);
    }
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { auto: true }, y: { auto: true } },
        series: [
            { label: t('view.margin_call.chart.drop_pct') },
            { label: t('view.margin_call.chart.equity_pct'),
              stroke: '#00e5ff', width: 1.6, points: { show: false } },
            { label: t('view.margin_call.chart.maint_pct'),
              stroke: '#ff3860', width: 1.0, dash: [4, 4], points: { show: false } },
            { label: t('view.margin_call.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [ { stroke: '#aab', size: 28 }, { stroke: '#aab', size: 40 } ],
        legend: { show: true },
    }, [xs, equityPct, maintLine, zero], el);
}

function renderSummary(report, pending) {
    const badge = cushionBadge(report);
    const local = localEvaluate(state);
    const parityOk = Math.abs(report.dollar_cushion - local.dollar_cushion) < 1e-6
                  && report.in_call === local.in_call;
    const localTag = pending ? ` (${t('view.margin_call.tag.local')})` : '';
    const trigger = triggerLmv(state);
    document.getElementById('mc-summary').innerHTML = [
        card(t('view.margin_call.card.verdict'),
             t(badge.key) + localTag, badge.cls),
        card(t('view.margin_call.card.equity'),
             fmtUSD(report.current_equity),
             report.current_equity >= 0 ? 'pos' : 'neg'),
        card(t('view.margin_call.card.equity_pct'),
             fmtPct(report.current_equity_pct),
             report.current_equity_pct >= state.maintenance_pct + 0.05 ? 'pos'
              : report.current_equity_pct < state.maintenance_pct ? 'neg' : ''),
        card(t('view.margin_call.card.dollar_cushion'),
             fmtUSDSigned(report.dollar_cushion),
             report.dollar_cushion >= 0 ? 'pos' : 'neg'),
        card(t('view.margin_call.card.pct_cushion'),
             fmtPct(report.pct_cushion),
             badge.cls),
        card(t('view.margin_call.card.trigger_lmv'),
             Number.isFinite(trigger) ? fmtUSD(trigger) : t('view.margin_call.tag.none')),
        card(t('view.margin_call.card.maint_pct'),
             fmtMaintPct(state.maintenance_pct)),
        card(t('view.margin_call.card.lmv'),
             fmtUSD(state.long_market_value)),
        card(t('view.margin_call.card.debt'),
             fmtUSD(state.margin_debt), state.margin_debt > 0 ? 'neg' : ''),
        card(t('view.margin_call.card.parity'),
             parityOk ? t('view.margin_call.tag.ok') : t('view.margin_call.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderBar(report) {
    const wrap = document.getElementById('mc-bar');
    if (!wrap) return;
    const trigger = triggerLmv(state);
    const lmv = state.long_market_value;
    if (lmv <= 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.margin_call.empty">${esc(t('view.margin_call.empty'))}</div>`;
        return;
    }
    // Bar scaled so LMV occupies 100%. If trigger > LMV (in call), the
    // trigger marker would fall off-screen; cap the visualization at LMV.
    const triggerPct = Number.isFinite(trigger) && lmv > 0
        ? Math.min(100, Math.max(0, (trigger / lmv) * 100))
        : 0;
    const equityPct = lmv > 0 ? Math.min(100, Math.max(0, (report.current_equity / lmv) * 100)) : 0;
    wrap.innerHTML = `
        <div class="dl-bar-row" style="margin-bottom:6px">
            <div style="display:flex;justify-content:space-between;font-size:12px;margin-bottom:4px">
                <span><strong>${esc(t('view.margin_call.bar.equity'))}</strong></span>
                <span>${esc(fmtUSD(report.current_equity))} / ${esc(fmtUSD(lmv))}</span>
            </div>
            <div class="dl-bar-track" style="position:relative;height:18px;background:#1a1d22;border-radius:2px">
                <div class="dl-bar-fill ${report.in_call ? 'dl-fill-kill' : 'dl-fill-ok'}"
                     data-pct="${equityPct.toFixed(2)}"
                     style="position:absolute;height:100%;border-radius:2px;width:0;transition:width .25s"></div>
                <div class="dl-bar-mark dl-mark-kill"
                     data-pct="${triggerPct.toFixed(2)}"
                     style="position:absolute;top:0;bottom:0;width:2px;background:#ffd84a;left:0"></div>
            </div>
            <div style="display:flex;justify-content:space-between;font-size:11px;color:#aab;margin-top:4px">
                <span>${esc(t('view.margin_call.bar.zero'))}</span>
                <span style="position:absolute;left:${triggerPct.toFixed(2)}%;transform:translateX(-50%)">${esc(t('view.margin_call.bar.trigger'))} (${esc(fmtUSD(trigger))})</span>
                <span>${esc(t('view.margin_call.bar.full_lmv'))}</span>
            </div>
        </div>
    `;
    requestAnimationFrame(() => {
        wrap.querySelectorAll('.dl-bar-fill').forEach(el => { el.style.width = el.dataset.pct + '%'; });
        wrap.querySelectorAll('.dl-bar-mark').forEach(el => { el.style.left  = el.dataset.pct + '%'; });
    });
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('mc-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('mc-err').style.display = 'none'; }
