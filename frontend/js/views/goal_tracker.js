// Goal Tracker view — measurable return + DD goals + on-pace verdict.
//
// Tracks current equity vs (period_start, period_end, target return,
// max DD) and produces an on-pace classification. Pre-flight via local
// evaluator → backend confirmation.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseEquity, validateInputs, buildBody, localEvaluate, paceBadge,
    makeDemoData, todayIso, fmtUSD, fmtPct,
} from '../_goal_tracker_inputs.js';

let state = { params: makeDemoData('on-pace') };

export async function renderGoalTracker(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// GOAL TRACKER</h1>

        <div class="chart-panel">
            <h2>Period goals</h2>
            <div class="inline-form">
                <label>Period start ($)
                    <input id="gt-eq0" type="number" step="any" min="0" value="${state.params.period_start_equity}"></label>
                <label>Target return (decimal — 0.30 = 30%)
                    <input id="gt-tgt" type="number" step="any" value="${state.params.target_pct_return}"></label>
                <label>Max DD (decimal — 0.10 = 10%)
                    <input id="gt-dd" type="number" step="any" min="0" max="1" value="${state.params.max_dd_pct}"></label>
            </div>
            <div class="inline-form">
                <label>Period start (YYYY-MM-DD)
                    <input id="gt-ps" type="text" value="${esc(state.params.period_start)}"></label>
                <label>Period end (YYYY-MM-DD)
                    <input id="gt-pe" type="text" value="${esc(state.params.period_end)}"></label>
                <label>Today (YYYY-MM-DD)
                    <input id="gt-today" type="text" value="${esc(state.params.today)}"></label>
                <button id="gt-today-now" class="secondary" type="button">today = now</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2>Equity history</h2>
            <textarea id="gt-eq" rows="5" placeholder="100000&#10;105000&#10;110000">${state.params.equity.join('\n')}</textarea>
            <div class="inline-form">
                <button id="gt-demo-ahead"  class="secondary" type="button">Demo: AHEAD</button>
                <button id="gt-demo-onpace" class="secondary" type="button">Demo: ON PACE</button>
                <button id="gt-demo-behind" class="secondary" type="button">Demo: BEHIND</button>
                <button id="gt-demo-kill"   class="secondary" type="button">Demo: KILL SWITCH</button>
                <button id="gt-demo-out"    class="secondary" type="button">Demo: OUT-OF-PERIOD</button>
                <button id="gt-run" class="primary" type="button">Evaluate</button>
            </div>
        </div>

        <div id="gt-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>Target progress</h2>
            <div id="gt-progress"></div>
        </div>

        <div class="chart-panel">
            <h2>Equity curve</h2>
            <div id="gt-chart" style="height:260px"></div>
        </div>

        <div id="gt-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (kind) => {
        state.params = makeDemoData(kind);
        document.getElementById('gt-eq0').value = state.params.period_start_equity;
        document.getElementById('gt-tgt').value = state.params.target_pct_return;
        document.getElementById('gt-dd').value = state.params.max_dd_pct;
        document.getElementById('gt-ps').value = state.params.period_start;
        document.getElementById('gt-pe').value = state.params.period_end;
        document.getElementById('gt-today').value = state.params.today;
        document.getElementById('gt-eq').value = state.params.equity.join('\n');
    };
    document.getElementById('gt-demo-ahead').addEventListener('click',  () => loadDemo('ahead'));
    document.getElementById('gt-demo-onpace').addEventListener('click', () => loadDemo('on-pace'));
    document.getElementById('gt-demo-behind').addEventListener('click', () => loadDemo('behind'));
    document.getElementById('gt-demo-kill').addEventListener('click',   () => loadDemo('kill-switch'));
    document.getElementById('gt-demo-out').addEventListener('click',    () => loadDemo('out-of-period'));
    document.getElementById('gt-today-now').addEventListener('click', () => {
        document.getElementById('gt-today').value = todayIso();
    });
    document.getElementById('gt-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    const { value: equity, errors } = parseEquity(document.getElementById('gt-eq').value);
    state.params = {
        period_start_equity: Number(document.getElementById('gt-eq0').value),
        target_pct_return:   Number(document.getElementById('gt-tgt').value),
        max_dd_pct:          Number(document.getElementById('gt-dd').value),
        period_start:        String(document.getElementById('gt-ps').value).trim(),
        period_end:          String(document.getElementById('gt-pe').value).trim(),
        today:               String(document.getElementById('gt-today').value).trim(),
        equity,
    };
    state.parseErrors = errors;
}

async function compute(tok) {
    hideErr();
    if (state.parseErrors && state.parseErrors.length && state.params.equity.length === 0) {
        showErr(`${state.parseErrors.length} parse error(s) in equity history — first: ${state.parseErrors[0].message}`);
        return;
    }
    const err = validateInputs(state.params);
    if (err) { showErr(err); return; }

    const local = localEvaluate(state.params);
    renderSummary(local, true);
    renderProgress(local);
    renderChart(state.params.equity);

    let resp;
    try {
        resp = await api.discGoalTracker(buildBody(state.params));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderProgress(resp);
}

function renderSummary(r, pending) {
    const badge = paceBadge(r.on_pace);
    const local = localEvaluate(state.params);
    const parityOk = r.on_pace === local.on_pace;
    document.getElementById('gt-summary').innerHTML = [
        card('On pace',           badge.label + (pending ? ' (local)' : ''), badge.cls),
        card('Current equity',    fmtUSD(r.current_equity)),
        card('Period return',     fmtPct(r.current_pct_return),
            r.current_pct_return >= 0 ? 'pos' : 'neg'),
        card('% of target',       fmtPct(r.pct_of_target, 1),
            r.pct_of_target >= 1 ? 'pos' : r.pct_of_target >= 0.5 ? '' : 'neg'),
        card('Drawdown',          fmtPct(r.current_dd_pct),
            r.kill_switch_breached ? 'neg' : ''),
        card('Kill switch?',      r.kill_switch_breached ? 'BREACHED' : 'OK',
            r.kill_switch_breached ? 'neg' : 'pos'),
        card('Days elapsed',      `${r.days_elapsed} / ${r.days_total}`),
        card('Annualized pace',   fmtPct(r.annualized_pace, 2),
            r.annualized_pace >= 0 ? 'pos' : 'neg'),
        card('Action',            badge.hint),
        card('Local parity',      parityOk ? 'OK' : 'DIVERGED', parityOk ? 'pos' : 'neg'),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderProgress(r) {
    const wrap = document.getElementById('gt-progress');
    const pctTarget = Math.max(0, Math.min(1.5, r.pct_of_target));
    const fillPct = (Math.min(pctTarget, 1.0) * 100).toFixed(2);
    const fillCls = r.pct_of_target >= 1 ? 'dl-fill-ok' :
                    r.pct_of_target >= 0.5 ? 'dl-fill-warn' : 'dl-fill-cut';
    const ddFillPct = (Math.min(r.current_dd_pct / state.params.max_dd_pct, 1.0) * 100).toFixed(2);
    const ddCls = r.kill_switch_breached ? 'dl-fill-kill' :
                  r.current_dd_pct > state.params.max_dd_pct * 0.75 ? 'dl-fill-cut' : 'dl-fill-warn';
    wrap.innerHTML = `
        <div class="gt-bar-row">
            <div class="gt-bar-label">Return vs target</div>
            <div class="dl-bar-track">
                <div class="dl-bar-fill ${fillCls}" data-pct="${fillPct}"></div>
                <div class="dl-bar-label">${esc(fmtPct(r.current_pct_return))} of ${esc(fmtPct(r.target_pct_return))} target · ${esc(fmtPct(r.pct_of_target, 0))} of target</div>
            </div>
        </div>
        <div class="gt-bar-row" style="margin-top:8px">
            <div class="gt-bar-label">DD vs max-DD limit</div>
            <div class="dl-bar-track">
                <div class="dl-bar-fill ${ddCls}" data-pct="${ddFillPct}"></div>
                <div class="dl-bar-label">${esc(fmtPct(r.current_dd_pct))} DD · cap ${esc(fmtPct(state.params.max_dd_pct))}${r.kill_switch_breached ? ' · BREACHED' : ''}</div>
            </div>
        </div>
        <div class="gt-bar-row" style="margin-top:8px">
            <div class="gt-bar-label">Period elapsed</div>
            <div class="dl-bar-track">
                <div class="dl-bar-fill dl-fill-warn" data-pct="${r.days_total > 0 ? Math.max(0, Math.min(100, r.days_elapsed / r.days_total * 100)).toFixed(2) : 0}"></div>
                <div class="dl-bar-label">${r.days_elapsed} of ${r.days_total} days</div>
            </div>
        </div>
    `;
    requestAnimationFrame(() => {
        wrap.querySelectorAll('.dl-bar-fill').forEach(el => {
            el.style.width = el.dataset.pct + '%';
        });
    });
}

function renderChart(equity) {
    if (!window.uPlot) return;
    const el = document.getElementById('gt-chart');
    if (!equity.length) { el.innerHTML = '<div class="muted">No equity history.</div>'; return; }
    const xs = equity.map((_, i) => i);
    const targetLine = xs.map((_, i) => {
        const frac = equity.length > 1 ? i / (equity.length - 1) : 1;
        return state.params.period_start_equity * (1 + state.params.target_pct_return * frac);
    });
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 260,
        scales: { x: {}, y: {} },
        series: [
            { label: 'bar #' },
            { label: 'equity', stroke: '#00e5ff', width: 1.5,
              fill: '#00e5ff14', points: { show: false } },
            { label: 'target line', stroke: '#ffd84a', width: 1.0,
              dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 60 }],
        legend: { show: true },
    }, [xs, equity, targetLine], el);
}

function showErr(msg) {
    const el = document.getElementById('gt-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('gt-err').style.display = 'none'; }
