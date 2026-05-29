// ATR cone projection view. σ_N = ATR × √N projected ±1σ / ±2σ bands
// forward N days from entry.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_INPUTS, MAX_HORIZON_DAYS,
    validateInputs, buildBody, localProject,
    widthAtHorizon, widthPctAtHorizon, noiseBadge, daysToReachOffset,
    makeDemoInput, fmtUSD, fmtUSDSigned, fmtPct, fmtDays,
} from '../_atr_cone_inputs.js';

let state = { ...DEFAULT_INPUTS };

export async function renderAtrCone(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.atr_cone.h1.title" class="view-title">// ATR CONE</h1>

        <div class="chart-panel" data-context-scope="atr-cone">
            <h2 data-i18n="view.atr_cone.h2.inputs">Projection inputs</h2>
            <div class="inline-form">
                <label><span data-i18n="view.atr_cone.label.entry">Entry price ($)</span>
                    <input id="ac-entry" type="number" step="any" min="0" value="${state.entry}"></label>
                <label><span data-i18n="view.atr_cone.label.atr">Daily ATR ($)</span>
                    <input id="ac-atr" type="number" step="any" min="0" value="${state.daily_atr}"></label>
                <label><span data-i18n="view.atr_cone.label.horizon">Horizon (days)</span>
                    <input id="ac-h" type="number" step="1" min="0" max="${MAX_HORIZON_DAYS}" value="${state.horizon_days}"></label>
                <button data-i18n="view.atr_cone.btn.project" id="ac-run" class="primary"
                        data-tip="view.atr_cone.tip.project" type="button">Project</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.atr_cone.btn.demo_spy"     id="ac-demo-spy"   class="secondary" type="button">Demo: SPY normal</button>
                <button data-i18n="view.atr_cone.btn.demo_aapl"    id="ac-demo-aapl"  class="secondary" type="button">Demo: AAPL medium</button>
                <button data-i18n="view.atr_cone.btn.demo_tsla"    id="ac-demo-tsla"  class="secondary" type="button">Demo: TSLA loud</button>
                <button data-i18n="view.atr_cone.btn.demo_penny"   id="ac-demo-penny" class="secondary" type="button">Demo: penny stock extreme</button>
                <button data-i18n="view.atr_cone.btn.demo_long"    id="ac-demo-long"  class="secondary" type="button">Demo: long horizon (60d)</button>
                <button data-i18n="view.atr_cone.btn.demo_zero"    id="ac-demo-zero"  class="secondary" type="button">Demo: zero-ATR (flat)</button>
                <button data-i18n="view.atr_cone.btn.demo_es"      id="ac-demo-es"    class="secondary" type="button">Demo: ES futures</button>
                <button data-i18n="view.atr_cone.btn.demo_cap"     id="ac-demo-cap"   class="secondary" type="button">Demo: horizon > cap (2000)</button>
            </div>
            <p data-i18n="view.atr_cone.hint.about" class="muted">σ_N = ATR × √N (Brownian scaling). Bands at ±1σ and ±2σ. Stops inside ±1σ get knocked out by noise; targets outside ±2σ need an edge to hit. Horizon capped at 1000 days server-side.</p>
        </div>

        <div id="ac-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.atr_cone.h2.cone">Cone projection</h2>
            <div id="ac-chart" style="height:340px"></div>
            <p data-i18n="view.atr_cone.hint.chart" class="muted">Center cyan = entry. Yellow = ±1σ band (≈68% range). Red = ±2σ band (≈95% range).</p>
        </div>

        <div id="ac-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('ac-entry').value = state.entry;
        document.getElementById('ac-atr').value   = state.daily_atr;
        document.getElementById('ac-h').value     = state.horizon_days;
    };
    document.getElementById('ac-demo-spy').addEventListener('click',   () => loadDemo('spy-normal'));
    document.getElementById('ac-demo-aapl').addEventListener('click',  () => loadDemo('aapl-medium'));
    document.getElementById('ac-demo-tsla').addEventListener('click',  () => loadDemo('tsla-loud'));
    document.getElementById('ac-demo-penny').addEventListener('click', () => loadDemo('penny-extreme'));
    document.getElementById('ac-demo-long').addEventListener('click',  () => loadDemo('long-horizon'));
    document.getElementById('ac-demo-zero').addEventListener('click',  () => loadDemo('zero-atr'));
    document.getElementById('ac-demo-es').addEventListener('click',    () => loadDemo('es-futures'));
    document.getElementById('ac-demo-cap').addEventListener('click',   () => loadDemo('huge-horizon'));
    document.getElementById('ac-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function readInputs() {
    state = {
        entry:        Number(document.getElementById('ac-entry').value),
        daily_atr:    Number(document.getElementById('ac-atr').value),
        horizon_days: parseInt(document.getElementById('ac-h').value, 10),
    };
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localProject(state.entry, state.daily_atr, state.horizon_days);
    renderSummary(local, true);
    renderChart(local);
    let resp;
    try {
        resp = await api.chartsAtrCone(buildBody(state));
    } catch (e) {
        showErr(`${t('view.atr_cone.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(resp);
}

function renderSummary(points, pending) {
    const local = localProject(state.entry, state.daily_atr, state.horizon_days);
    const parityOk = points.length === local.length;
    const last = points[points.length - 1] || null;
    const badge = noiseBadge(state.entry, state.daily_atr, state.horizon_days);
    const width = widthAtHorizon(state.daily_atr, state.horizon_days);
    const widthPct = widthPctAtHorizon(state.entry, state.daily_atr, state.horizon_days);
    const cappedHorizon = Math.min(state.horizon_days, MAX_HORIZON_DAYS);
    const cappedTag = state.horizon_days > MAX_HORIZON_DAYS
        ? ` (${t('view.atr_cone.tag.capped')} ${MAX_HORIZON_DAYS})` : '';
    const localTag = pending ? ` (${t('view.atr_cone.tag.local')})` : '';
    const daysToOneAtr = daysToReachOffset(state.daily_atr, state.daily_atr);
    document.getElementById('ac-summary').innerHTML = [
        card(t('view.atr_cone.card.noise'),         t(badge.key) + localTag, badge.cls),
        card(t('view.atr_cone.card.entry'),         fmtUSD(state.entry)),
        card(t('view.atr_cone.card.daily_atr'),     fmtUSD(state.daily_atr)),
        card(t('view.atr_cone.card.horizon'),       String(cappedHorizon) + ' d' + cappedTag,
             state.horizon_days > MAX_HORIZON_DAYS ? 'neg' : ''),
        card(t('view.atr_cone.card.width_dollars'), fmtUSD(width), badge.cls),
        card(t('view.atr_cone.card.width_pct'),     fmtPct(widthPct), badge.cls),
        card(t('view.atr_cone.card.upper_2sd'),
             last ? fmtUSD(last.upper_2sd) : '—', 'pos'),
        card(t('view.atr_cone.card.lower_2sd'),
             last ? fmtUSD(last.lower_2sd) : '—', 'neg'),
        card(t('view.atr_cone.card.days_to_atr'),   fmtDays(daysToOneAtr)),
        card(t('view.atr_cone.card.parity'),
             parityOk ? t('view.atr_cone.tag.ok') : t('view.atr_cone.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
    void fmtUSDSigned;
}

function renderChart(points) {
    if (!window.uPlot) return;
    const el = document.getElementById('ac-chart');
    if (!el || !points || points.length === 0) return;
    el.innerHTML = '';
    const xs = points.map(p => p.days_forward);
    const u2 = points.map(p => p.upper_2sd);
    const u1 = points.map(p => p.upper_1sd);
    const c  = points.map(p => p.center);
    const l1 = points.map(p => p.lower_1sd);
    const l2 = points.map(p => p.lower_2sd);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 320,
        scales: { x: {}, y: {} },
        series: [
            { label: t('chart.series.days') },
            { label: '+2σ',    stroke: '#ff3860', width: 1.0, dash: [4, 4], points: { show: false } },
            { label: '+1σ',    stroke: '#ffd84a', width: 1.0,                 points: { show: false } },
            { label: 'entry',  stroke: '#00e5ff', width: 1.5,                 points: { show: false } },
            { label: '-1σ',    stroke: '#ffd84a', width: 1.0,                 points: { show: false } },
            { label: '-2σ',    stroke: '#ff3860', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => v + 'd') },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, u2, u1, c, l1, l2], el);
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('ac-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ac-err').style.display = 'none'; }
