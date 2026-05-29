// Elder's Triple Screen view — 3-timeframe entry filter cascade.
//
// Long-tide (weekly trend) → Intermediate-wave (daily oscillator
// pullback against tide) → Short-ripple (intraday breakout in tide
// direction). All three must align for a Buy/Sell verdict; otherwise
// Wait. Surfaces which screens passed/failed for transparency.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    validateInputs, buildBody, localEvaluate,
    stageResults, verdictBadge, makeDemoData, fmtN,
} from '../_triple_screen_inputs.js';

import { t } from '../i18n.js';
let state = { params: makeDemoData('buy') };

export async function renderTripleScreen(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.triple_screen.h1.triple_screen_elder" class="view-title">// TRIPLE SCREEN · ELDER</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.triple_screen.h2.screen_1_long_tide_weekly_trend">Screen 1 — Long-tide (weekly trend)</h2>
            <div class="inline-form">
                <label><span data-i18n="view.triple_screen.label.weekly_trend">Weekly trend</span>
                    <select id="ts-trend">
                        <option data-i18n="view.triple_screen.opt.up" value="up"      ${state.params.weekly_trend === 'up'      ? 'selected' : ''}>UP</option>
                        <option data-i18n="view.triple_screen.opt.down" value="down"    ${state.params.weekly_trend === 'down'    ? 'selected' : ''}>DOWN</option>
                        <option data-i18n="view.triple_screen.opt.neutral" value="neutral" ${state.params.weekly_trend === 'neutral' ? 'selected' : ''}>NEUTRAL</option>
                    </select></label>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.triple_screen.h2.screen_2_intermediate_wave_daily_oscillator">Screen 2 — Intermediate-wave (daily oscillator)</h2>
            <div class="inline-form">
                <label><span data-i18n="view.triple_screen.label.oscillator">Oscillator value (RSI / stoch)</span>
                    <input id="ts-osc" type="number" step="any" value="${state.params.daily_oscillator_value}"></label>
                <label><span data-i18n="view.triple_screen.label.oversold">Oversold threshold</span>
                    <input id="ts-os"  type="number" step="any" value="${state.params.oversold_threshold}"></label>
                <label><span data-i18n="view.triple_screen.label.overbought">Overbought threshold</span>
                    <input id="ts-ob"  type="number" step="any" value="${state.params.overbought_threshold}"></label>
            </div>
            <p data-i18n="view.triple_screen.hint.in_an_up_tide_pull_back_below_oversold_entry_zone_" class="muted">In an UP tide, pull-back below oversold = entry zone.
                In a DOWN tide, rally above overbought = entry zone.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.triple_screen.h2.screen_3_short_ripple_intraday_breakout">Screen 3 — Short-ripple (intraday breakout)</h2>
            <div class="inline-form">
                <label><input id="ts-buy"  type="checkbox" ${state.params.intraday_breakout_up   ? 'checked' : ''}> Intraday breakout UP</label>
                <label><input id="ts-sell" type="checkbox" ${state.params.intraday_breakout_down ? 'checked' : ''}> Intraday breakout DOWN</label>
                <button data-i18n="view.triple_screen.btn.evaluate" id="ts-run" class="primary" type="button">Evaluate</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.triple_screen.btn.demo_buy_all_aligned_long" id="ts-demo-buy"      class="secondary" type="button">Demo: BUY (all aligned long)</button>
                <button data-i18n="view.triple_screen.btn.demo_sell_all_aligned_short" id="ts-demo-sell"     class="secondary" type="button">Demo: SELL (all aligned short)</button>
                <button data-i18n="view.triple_screen.btn.demo_wait_no_pullback" id="ts-demo-no-pb"    class="secondary" type="button">Demo: WAIT — no pullback</button>
                <button data-i18n="view.triple_screen.btn.demo_wait_no_breakout" id="ts-demo-no-bo"    class="secondary" type="button">Demo: WAIT — no breakout</button>
                <button data-i18n="view.triple_screen.btn.demo_wait_neutral_tide" id="ts-demo-neutral"  class="secondary" type="button">Demo: WAIT — neutral tide</button>
            </div>
        </div>

        <div id="ts-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.triple_screen.h2.screen_cascade">Screen cascade</h2>
            <div id="ts-cascade"></div>
        </div>

        <div id="ts-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (kind) => {
        state.params = makeDemoData(kind);
        document.getElementById('ts-trend').value = state.params.weekly_trend;
        document.getElementById('ts-osc').value = state.params.daily_oscillator_value;
        document.getElementById('ts-os').value = state.params.oversold_threshold;
        document.getElementById('ts-ob').value = state.params.overbought_threshold;
        document.getElementById('ts-buy').checked = state.params.intraday_breakout_up;
        document.getElementById('ts-sell').checked = state.params.intraday_breakout_down;
    };
    document.getElementById('ts-demo-buy').addEventListener('click',     () => loadDemo('buy'));
    document.getElementById('ts-demo-sell').addEventListener('click',    () => loadDemo('sell'));
    document.getElementById('ts-demo-no-pb').addEventListener('click',   () => loadDemo('wait-no-pullback'));
    document.getElementById('ts-demo-no-bo').addEventListener('click',   () => loadDemo('wait-no-breakout'));
    document.getElementById('ts-demo-neutral').addEventListener('click', () => loadDemo('wait-neutral-tide'));
    document.getElementById('ts-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
    readInputs(); void compute(tok);
}

function readInputs() {
    state.params = {
        weekly_trend: document.getElementById('ts-trend').value,
        daily_oscillator_value: Number(document.getElementById('ts-osc').value),
        oversold_threshold: Number(document.getElementById('ts-os').value),
        overbought_threshold: Number(document.getElementById('ts-ob').value),
        intraday_breakout_up: document.getElementById('ts-buy').checked,
        intraday_breakout_down: document.getElementById('ts-sell').checked,
    };
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.params);
    if (err) { showErr(err); return; }
    // Pre-flight: render local verdict immediately so the UI is responsive,
    // then refresh once the backend confirms.
    renderSummary({ verdict: localEvaluate(state.params) }, true);
    renderCascade();
    let resp;
    try {
        resp = await api.discTripleScreen(buildBody(state.params));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e })); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
}

function renderSummary(resp, pending) {
    const verdict = (resp && resp.verdict) || 'wait';
    const badge = verdictBadge(verdict);
    const localV = localEvaluate(state.params);
    const parity = verdict === localV ? 'pos' : 'neg';
    document.getElementById('ts-summary').innerHTML = [
        card(t('view.triple_screen.card.verdict'),       badge.label + (pending ? ' (local)' : ''), badge.cls),
        card(t('view.triple_screen.card.action'),        badge.hint),
        card(t('view.triple_screen.card.local_check'),   localV.toUpperCase(), parity),
        card(t('view.triple_screen.card.weekly_trend'),  state.params.weekly_trend.toUpperCase(),
            state.params.weekly_trend === 'up' ? 'pos' :
            state.params.weekly_trend === 'down' ? 'neg' : ''),
        card(t('view.triple_screen.card.daily_osc'),     fmtN(state.params.daily_oscillator_value)),
        card(t('view.triple_screen.card.bands'),         `${fmtN(state.params.oversold_threshold)} / ${fmtN(state.params.overbought_threshold)}`),
        card(t('view.triple_screen.card.intraday_up'),  state.params.intraday_breakout_up ? 'YES' : 'NO',
            state.params.intraday_breakout_up ? 'pos' : ''),
        card(t('view.triple_screen.card.intraday_down'), state.params.intraday_breakout_down ? 'YES' : 'NO',
            state.params.intraday_breakout_down ? 'neg' : ''),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderCascade() {
    const wrap = document.getElementById('ts-cascade');
    const r = stageResults(state.params);
    const stages = [r.longTide, r.intermediate, r.shortRipple];
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th><th data-i18n="view.triple_screen.th.screen">Screen</th><th data-i18n="view.triple_screen.th.pass">Pass?</th><th data-i18n="view.triple_screen.th.detail">Detail</th>
            </tr></thead>
            <tbody>
                ${stages.map((s, i) => `<tr>
                    <td>${i + 1}</td>
                    <td><strong>${esc(s.label)}</strong></td>
                    <td class="${s.pass ? 'pos' : 'neg'}">${s.pass ? '✓ PASS' : '× FAIL'}</td>
                    <td class="muted">${esc(s.detail)}</td>
                </tr>`).join('')}
            </tbody>
        </table>
    `;
}

function showErr(msg) {
    const el = document.getElementById('ts-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ts-err').style.display = 'none'; }
