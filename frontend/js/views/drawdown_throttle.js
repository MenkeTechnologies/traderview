// Drawdown Throttle view — conditional position-size scale based on
// current drawdown. Visualizes the equity curve + rolling peak +
// underwater drawdown alongside the tier ladder.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseEquity, DEFAULT_TIERS, validateInputs, buildBody,
    localEvaluate, activeTier, rollingDrawdown, multiplierCls,
    makeDemoEquity, fmtUSD, fmtPct, fmtMult,
} from '../_drawdown_throttle_inputs.js';

import { t } from '../i18n.js';
let state = { equityText: '', tiers: DEFAULT_TIERS.map(t => ({ ...t })) };

export async function renderDrawdownThrottle(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.drawdown_throttle.h1.drawdown_throttle" class="view-title">// DRAWDOWN THROTTLE</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.drawdown_throttle.h2.equity_history">Equity history</h2>
            <p data-i18n="view.drawdown_throttle.hint.one_value_per_line_most_recent_last_demo_presets_l" class="muted">One value per line — most-recent last. Demo presets land in
                each tier so the trader sees the full throttle ladder in action.</p>
            <textarea id="dt-eq" rows="6" placeholder="10000&#10;10500&#10;11000&#10;..."></textarea>
            <div class="inline-form">
                <button data-i18n="view.drawdown_throttle.btn.demo_shallow_3_ok" id="dt-demo-shallow" class="secondary" type="button">Demo: shallow 3% (OK)</button>
                <button data-i18n="view.drawdown_throttle.btn.demo_7_0_75" id="dt-demo-mild"    class="secondary" type="button">Demo: 7% (0.75×)</button>
                <button data-i18n="view.drawdown_throttle.btn.demo_12_0_50" id="dt-demo-mid"     class="secondary" type="button">Demo: 12% (0.50×)</button>
                <button data-i18n="view.drawdown_throttle.btn.demo_17_0_25" id="dt-demo-deep"    class="secondary" type="button">Demo: 17% (0.25×)</button>
                <button data-i18n="view.drawdown_throttle.btn.demo_25_0_10" id="dt-demo-crisis"  class="secondary" type="button">Demo: 25% (0.10×)</button>
                <button data-i18n="view.drawdown_throttle.btn.clear" id="dt-clear" class="secondary" type="button">Clear</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.drawdown_throttle.h2.throttle_tiers_min_dd_ascending">Throttle tiers (min_dd ascending)</h2>
            <div id="dt-tiers"></div>
            <div class="inline-form">
                <button data-i18n="view.drawdown_throttle.btn.add_tier" id="dt-tier-add" class="secondary" type="button">+ Add tier</button>
                <button data-i18n="view.drawdown_throttle.btn.reset_to_defaults" id="dt-tier-reset" class="secondary" type="button">Reset to defaults</button>
                <button data-i18n="view.drawdown_throttle.btn.evaluate" id="dt-run" class="primary" type="button">Evaluate</button>
            </div>
            <p class="muted" data-i18n-html="view.drawdown_throttle.tier_hint">Tiers must be ascending by <code>min_dd</code>. The active tier
                is the LARGEST <code>min_dd</code> that current DD ≥ it.</p>
        </div>

        <div id="dt-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.drawdown_throttle.h2.equity_curve_rolling_peak_underwater_drawdown">Equity curve + rolling peak + underwater drawdown</h2>
            <div id="dt-chart" style="height:300px"></div>
            <p data-i18n="view.drawdown_throttle.hint.cyan_equity_yellow_rolling_peak_red_underwater_pea" class="muted">Cyan = equity. Yellow = rolling peak. Red = underwater
                (peak − current as a negative %). Active throttle tier highlighted in legend.</p>
        </div>

        <div id="dt-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (kind) => {
        const eq = makeDemoEquity(kind);
        document.getElementById('dt-eq').value = eq.join('\n');
    };
    document.getElementById('dt-demo-shallow').addEventListener('click', () => loadDemo('shallow'));
    document.getElementById('dt-demo-mild').addEventListener('click',    () => loadDemo('mild'));
    document.getElementById('dt-demo-mid').addEventListener('click',     () => loadDemo('mid'));
    document.getElementById('dt-demo-deep').addEventListener('click',    () => loadDemo('deep'));
    document.getElementById('dt-demo-crisis').addEventListener('click',  () => loadDemo('crisis'));
    document.getElementById('dt-clear').addEventListener('click', () => {
        document.getElementById('dt-eq').value = '';
    });
    document.getElementById('dt-tier-add').addEventListener('click', () => {
        state.tiers = [...state.tiers, { min_dd: 0.30, multiplier: 0.05 }];
        renderTiers();
    });
    document.getElementById('dt-tier-reset').addEventListener('click', () => {
        state.tiers = DEFAULT_TIERS.map(t => ({ ...t }));
        renderTiers();
    });
    document.getElementById('dt-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
    renderTiers();
}

function renderTiers() {
    const wrap = document.getElementById('dt-tiers');
    if (!wrap) return;
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr><th>#</th><th data-i18n="view.drawdown_throttle.th.min_dd_decimal">Min DD (decimal)</th><th data-i18n="view.drawdown_throttle.th.size_multiplier">Size multiplier</th><th data-i18n="view.drawdown_throttle.th.remove">Remove</th></tr></thead>
            <tbody>
                ${state.tiers.map((t, i) => `
                    <tr>
                        <td>${i + 1}</td>
                        <td><input type="number" step="any" min="0" max="1" data-tier-idx="${i}" data-field="min_dd" value="${t.min_dd}"></td>
                        <td><input type="number" step="any" min="0" max="5" data-tier-idx="${i}" data-field="multiplier" value="${t.multiplier}"></td>
                        <td><button class="db-tile-btn db-tile-remove" data-tier-remove="${i}">×</button></td>
                    </tr>
                `).join('')}
            </tbody>
        </table>
    `;
    wrap.querySelectorAll('input[data-tier-idx]').forEach(input => {
        input.addEventListener('change', () => {
            const idx = parseInt(input.dataset.tierIdx, 10);
            const field = input.dataset.field;
            const v = Number(input.value);
            if (!Number.isFinite(v)) return;
            state.tiers = state.tiers.map((t, i) =>
                i === idx ? { ...t, [field]: v } : t);
        });
    });
    wrap.querySelectorAll('button[data-tier-remove]').forEach(btn => {
        btn.addEventListener('click', () => {
            const idx = parseInt(btn.dataset.tierRemove, 10);
            state.tiers = state.tiers.filter((_, i) => i !== idx);
            renderTiers();
        });
    });
}

function readInputs() {
    state.equityText = document.getElementById('dt-eq').value;
}

async function compute(tok) {
    hideErr();
    const { value: equity, errors } = parseEquity(state.equityText);
    if (errors.length && equity.length === 0) {
        showErr(t('view.drawdown_throttle.err.parse_errors', { n: errors.length }));
        return;
    }
    const err = validateInputs(equity, state.tiers);
    if (err) { showErr(err); return; }

    // Pre-flight render with local eval.
    const local = localEvaluate(equity, state.tiers);
    renderSummary(local, true);
    renderChart(equity, state.tiers, local);

    let resp;
    try {
        resp = await api.discDrawdownThrottle(buildBody(equity, state.tiers));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e })); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(equity, state.tiers, resp);
}

function renderSummary(r, pending) {
    const mult = r.active_multiplier;
    const cls = multiplierCls(mult);
    document.getElementById('dt-summary').innerHTML = [
        card(t('view.drawdown_throttle.card.current_equity'), fmtUSD(r.current_equity)),
        card(t('view.drawdown_throttle.card.peak_equity'),    fmtUSD(r.peak_equity)),
        card(t('view.drawdown_throttle.card.drawdown'),       fmtPct(r.drawdown_pct), r.drawdown_pct > 0.10 ? 'neg' : ''),
        card(t('view.drawdown_throttle.card.active_tier'),    `≥ ${fmtPct(r.tier_min_dd, 0)}`, cls),
        card(t('view.drawdown_throttle.card.size_multiplier'), fmtMult(mult) + (pending ? ' (local)' : ''), cls),
        card(t('view.drawdown_throttle.card.reduce_by'),      fmtPct(1 - mult, 0), cls),
        card(t('view.drawdown_throttle.card.note'),           r.note || `DD ${fmtPct(r.drawdown_pct)} — sizing at ${fmtMult(mult)}`),
        card(t('view.drawdown_throttle.card.tiers'),          String(state.tiers.length)),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderChart(equity, tiers, report) {
    if (!window.uPlot) return;
    const el = document.getElementById('dt-chart');
    const xs = equity.map((_, i) => i);
    const { peaks, dds } = rollingDrawdown(equity);
    // Two y-scales: equity (left) + drawdown-pct (right, range -50% to 0).
    const activeTier_ = activeTier(tiers, report.drawdown_pct);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 300,
        scales: { x: {}, y: {}, y_dd: { range: [-0.5, 0] } },
        series: [
            { label: t('chart.series.bar_num') },
            { label: t('chart.series.equity'), stroke: '#00e5ff', width: 1.5,
              fill: '#00e5ff14', points: { show: false } },
            { label: 'rolling peak', stroke: '#ffd84a', width: 1.0,
              dash: [4, 4], points: { show: false } },
            { label: 'drawdown',  stroke: '#ff3860', width: 1.0,
              fill: '#ff386033', points: { show: false }, scale: 'y_dd' },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60 },
            { stroke: '#ff3860', size: 50, scale: 'y_dd', side: 1,
              values: (_u, splits) => splits.map(v => (v * 100).toFixed(0) + '%') },
        ],
        legend: { show: true },
    }, [xs, equity, peaks, dds], el);
    void activeTier_;
}

function showErr(msg) {
    const el = document.getElementById('dt-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('dt-err').style.display = 'none'; }
