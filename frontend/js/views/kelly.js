// Kelly view — two panels: static (win-rate + payoff) and dynamic
// (rolling Kelly from trade-PnL history). Both POST endpoints share a
// view so the user can see how their actual track record translates
// into Kelly sizing.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    validateStaticInputs, buildStaticBody, localComputeStatic,
    validateDynamicInputs, buildDynamicBody, localComputeDynamic,
    parsePnlBlob, pnlsToStaticInput, makeDemoPnls,
    sizeBadge, fmtPct, fmtNum, fmtUSD, fmtUSDSigned,
} from '../_kelly_inputs.js';

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
let state = {
    winRate: 0.60,
    payoffRatio: 2.0,
    pnls: makeDemoPnls('positive-edge'),
    window: 10,
};

export async function renderKelly(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.kelly.h1.kelly_sizer" class="view-title">// KELLY SIZER</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.kelly.h2.static_kelly">Static Kelly</h2>
            <div class="inline-form">
                <label><span data-i18n="view.kelly.label.win_rate">Win rate (0–1)</span>
                    <input id="kl-wr" type="number" step="0.01" min="0" max="1" value="${state.winRate}" data-tip="view.kelly.tip.win_rate"></label>
                <label><span data-i18n="view.kelly.label.payoff_ratio">Payoff ratio (avg win / avg loss)</span>
                    <input id="kl-payoff" type="number" step="0.01" min="0" value="${state.payoffRatio}" data-tip="view.kelly.tip.payoff"></label>
                <button data-i18n="view.kelly.btn.compute_static" data-tip="view.kelly.tip.run_static" data-shortcut="kelly_compute_static" id="kl-run-static" class="primary" type="button">Compute static</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.kelly.btn.demo_60_wr_2_1" id="kl-demo-positive"  class="secondary" type="button">Demo: 60% wr × 2:1</button>
                <button data-i18n="view.kelly.btn.demo_tiny_edge_50_1" id="kl-demo-tiny"      class="secondary" type="button">Demo: tiny edge (50.1%)</button>
                <button data-i18n="view.kelly.btn.demo_extreme_90_5_1" id="kl-demo-extreme"   class="secondary" type="button">Demo: extreme (90% × 5:1)</button>
                <button data-i18n="view.kelly.btn.demo_no_edge_50_1_1" id="kl-demo-noedge"    class="secondary" type="button">Demo: no edge (50% × 1:1)</button>
                <button data-i18n="view.kelly.btn.demo_negative_40_1_1" id="kl-demo-negative"  class="secondary" type="button">Demo: negative (40% × 1:1)</button>
            </div>
        </div>

        <div id="kl-static-cards" class="cards"></div>

        <div class="chart-panel">
            <h2><span data-i18n="view.kelly.h2.dynamic_kelly">Dynamic Kelly</span> <small class="muted" data-i18n="view.kelly.h2.dynamic_kelly_hint">(rolling window over trade PnLs)</small></h2>
            <textarea id="kl-pnls" rows="5" placeholder="200&#10;-100&#10;200&#10;-100&#10;...">${esc(pnlsToBlob(state.pnls))}</textarea>
            <div class="inline-form">
                <label><span data-i18n="view.kelly.label.window">Window (last N trades)</span>
                    <input id="kl-win" type="number" step="1" min="1" value="${state.window}" data-tip="view.kelly.tip.window"></label>
                <button data-i18n="view.kelly.btn.compute_dynamic" data-tip="view.kelly.tip.run_dyn" data-shortcut="kelly_compute_dynamic" id="kl-run-dyn" class="primary" type="button">Compute dynamic</button>
                <button data-i18n="view.kelly.btn.derive_static_wr_payoff_from_these_pnls" data-tip="view.kelly.tip.derive_static" id="kl-import-static" class="secondary" type="button">Derive static (wr / payoff) from these PnLs</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.kelly.btn.demo_pnls_positive_edge" id="kl-demo-pos-pnl"  class="secondary" type="button">Demo PnLs: positive edge</button>
                <button data-i18n="view.kelly.btn.demo_pnls_negative_edge" id="kl-demo-neg-pnl"  class="secondary" type="button">Demo PnLs: negative edge</button>
                <button data-i18n="view.kelly.btn.demo_pnls_break_even" id="kl-demo-be-pnl"   class="secondary" type="button">Demo PnLs: break-even</button>
                <button data-i18n="view.kelly.btn.demo_pnls_extreme_edge" id="kl-demo-ext-pnl"  class="secondary" type="button">Demo PnLs: extreme edge</button>
                <button data-i18n="view.kelly.btn.demo_pnls_regime_switch" id="kl-demo-switch-pnl" class="secondary" type="button">Demo PnLs: regime switch</button>
            </div>
        </div>

        <div id="kl-dyn-cards" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.kelly.h2.rolling_kelly_fraction_over_trades">Rolling Kelly fraction over trades</h2>
            <div id="kl-dyn-chart" style="height:340px"></div>
            <p data-i18n="view.kelly.hint.cyan_raw_kelly_fraction_yellow_half_kelly_clamped_" class="muted">Cyan = raw Kelly fraction. Yellow = half-Kelly (clamped ≥ 0). Red dashed = zero line.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.kelly.h2.cum_pnl_chart">Cumulative P&L over trades</h2>
            <div id="kl-cum-chart" style="width:100%;height:240px"></div>
            <p data-i18n="view.kelly.hint.cum_pnl" class="muted small">Running sum of trade PnLs. Overlay against the rolling-Kelly chart above to see whether your equity grew during the trades that Kelly said to bet aggressively.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.kelly.h2.per_trade_window_stats">Per-trade window stats</h2>
            <div id="kl-dyn-table"></div>
        </div>

        <div id="kl-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('kl-demo-positive').addEventListener('click', () => loadStatic(0.60, 2.0));
    document.getElementById('kl-demo-tiny').addEventListener('click',     () => loadStatic(0.501, 1.0));
    document.getElementById('kl-demo-extreme').addEventListener('click',  () => loadStatic(0.90, 5.0));
    document.getElementById('kl-demo-noedge').addEventListener('click',   () => loadStatic(0.50, 1.0));
    document.getElementById('kl-demo-negative').addEventListener('click', () => loadStatic(0.40, 1.0));
    document.getElementById('kl-run-static').addEventListener('click', () => { readStatic(); void computeStatic(tok); });

    document.getElementById('kl-demo-pos-pnl').addEventListener('click',    () => loadPnls('positive-edge'));
    document.getElementById('kl-demo-neg-pnl').addEventListener('click',    () => loadPnls('negative-edge'));
    document.getElementById('kl-demo-be-pnl').addEventListener('click',     () => loadPnls('break-even'));
    document.getElementById('kl-demo-ext-pnl').addEventListener('click',    () => loadPnls('extreme-edge'));
    document.getElementById('kl-demo-switch-pnl').addEventListener('click', () => loadPnls('regime-switch'));
    document.getElementById('kl-run-dyn').addEventListener('click', () => { readDynamic(); void computeDynamic(tok); });
    document.getElementById('kl-import-static').addEventListener('click', () => {
        readDynamic();
        const d = pnlsToStaticInput(state.pnls);
        if (d.payoff_ratio <= 0) {
            showErr(t('view.kelly.err.no_wins_or_losses'));
            return;
        }
        state.winRate = d.win_rate;
        state.payoffRatio = d.payoff_ratio;
        document.getElementById('kl-wr').value = d.win_rate;
        document.getElementById('kl-payoff').value = d.payoff_ratio;
        void computeStatic(tok);
    });

    readStatic(); readDynamic();
    void computeStatic(tok);
    void computeDynamic(tok);
}

function pnlsToBlob(pnls) {
    return pnls.map(p => p.toString()).join('\n');
}

function loadStatic(wr, payoff) {
    state.winRate = wr;
    state.payoffRatio = payoff;
    document.getElementById('kl-wr').value = wr;
    document.getElementById('kl-payoff').value = payoff;
}

function loadPnls(kind) {
    state.pnls = makeDemoPnls(kind);
    document.getElementById('kl-pnls').value = pnlsToBlob(state.pnls);
}

function readStatic() {
    state.winRate     = Number(document.getElementById('kl-wr').value);
    state.payoffRatio = Number(document.getElementById('kl-payoff').value);
}

function readDynamic() {
    const parsed = parsePnlBlob(document.getElementById('kl-pnls').value);
    if (parsed.errors.length) {
        showErr(t("common.error.parse_errors", { summary: parsed.errors.slice(0, 3).map(e => `[] `).join("; ") }));
        return;
    }
    state.pnls   = parsed.pnls;
    state.window = parseInt(document.getElementById('kl-win').value, 10);
    hideErr();
}

async function computeStatic(tok) {
    const err = validateStaticInputs(state.winRate, state.payoffRatio);
    if (err) { showErr(err); showToast(err, { level: 'warning' }); return; }
    hideErr();
    const local = localComputeStatic(state.winRate, state.payoffRatio);
    renderStatic(local, true);
    let resp;
    try {
        resp = await api.calcKelly(buildStaticBody(state.winRate, state.payoffRatio));
    } catch (e) {
        const msg = t('view.kelly.error.api_static', { msg: e.message || e });
        showErr(msg); showToast(msg, { level: 'error' }); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderStatic(resp, false);
}

async function computeDynamic(tok) {
    const err = validateDynamicInputs(state.pnls, state.window);
    if (err) { showErr(err); showToast(err, { level: 'warning' }); return; }
    hideErr();
    const local = localComputeDynamic(state.pnls, state.window);
    renderDynamic(local, true);
    renderDynamicChart(local);
    renderCumPnlChart();
    renderDynamicTable(local);
    let resp;
    try {
        resp = await api.calcDynamicKelly(buildDynamicBody(state.pnls, state.window));
    } catch (e) {
        const msg = t('view.kelly.error.api_dynamic', { msg: e.message || e });
        showErr(msg); showToast(msg, { level: 'error' }); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderDynamic(resp, false);
    renderDynamicChart(resp);
    renderCumPnlChart();
    renderDynamicTable(resp);
}

function renderStatic(report, pending) {
    const badge = sizeBadge(report.full_kelly);
    const local = localComputeStatic(state.winRate, state.payoffRatio);
    const parityOk = Math.abs(report.full_kelly - local.full_kelly) < 1e-9
                  && report.note === local.note;
    document.getElementById('kl-static-cards').innerHTML = [
        card(t('view.kelly.card.verdict'),         badge.label + (pending ? t('common.suffix.local') : ''), badge.cls),
        card(t('view.kelly.card.action'),          badge.hint),
        card(t('view.kelly.card.full_kelly'),      fmtPct(report.full_kelly, 2),
            report.full_kelly >= 0 ? 'pos' : 'neg'),
        card(t('view.kelly.card.half_kelly'),      fmtPct(report.half_kelly, 2)),
        card(t('view.kelly.card.quarter_kelly'),   fmtPct(report.quarter_kelly, 2)),
        card(t('view.kelly.card.recommended_f'),   fmtPct(report.recommended_f, 2),
            report.recommended_f > 0 ? 'pos' : ''),
        card(t('view.kelly.card.note'),            report.note || '—'),
        card(t('view.kelly.card.p_b_vs_q'),      t('view.kelly.row.p_b_vs_q_value', { pb: (state.winRate * state.payoffRatio).toFixed(3), q: (1 - state.winRate).toFixed(3) }),
            (state.winRate * state.payoffRatio) > (1 - state.winRate) ? 'pos' : 'neg'),
        card(t('view.kelly.card.local_parity'),    parityOk ? t('common.ok') : t('common.diverged'), parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderDynamic(points, pending) {
    const local = localComputeDynamic(state.pnls, state.window);
    const lengthsMatch = points.length === local.length;
    let parityOk = lengthsMatch;
    for (let i = 0; parityOk && i < points.length; i++) {
        const a = points[i].kelly_fraction, b = local[i].kelly_fraction;
        if (a == null && b == null) continue;
        if (a == null || b == null) { parityOk = false; break; }
        if (Math.abs(a - b) > 1e-9)  { parityOk = false; break; }
    }
    const positioned = points.filter(p => p.kelly_fraction != null);
    const latest = positioned.length > 0 ? positioned[positioned.length - 1] : null;
    const stats = pnlsToStaticInput(state.pnls);
    document.getElementById('kl-dyn-cards').innerHTML = [
        card(t('view.kelly.card.trades_n'),       String(state.pnls.length) + (pending ? t('common.suffix.local') : '')),
        card(t('view.kelly.card.window'),           String(state.window)),
        card(t('view.kelly.card.wins_losses'),    `${stats.wins} / ${stats.losses}`,
            stats.wins > stats.losses ? 'pos' : 'neg'),
        card(t('view.kelly.card.overall_win_rate'), fmtPct(stats.win_rate, 1)),
        card(t('view.kelly.card.overall_payoff'),   fmtNum(stats.payoff_ratio, 3),
            stats.payoff_ratio >= 1 ? 'pos' : 'neg'),
        card(t('view.kelly.card.latest_kelly'),     latest && latest.kelly_fraction != null
            ? fmtPct(latest.kelly_fraction, 2) : '—',
            latest && latest.kelly_fraction > 0 ? 'pos' : 'neg'),
        card(t('view.kelly.card.latest_half_kelly'), latest && latest.half_kelly_fraction != null
            ? fmtPct(latest.half_kelly_fraction, 2) : '—',
            latest && latest.half_kelly_fraction > 0 ? 'pos' : ''),
        card(t('view.kelly.card.latest_payoff'),    latest && latest.window_payoff_ratio != null
            ? fmtNum(latest.window_payoff_ratio, 3) : '—'),
        card(t('view.kelly.card.local_parity_2'),     parityOk ? t('common.ok') : t('common.diverged'), parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderDynamicChart(points) {
    if (!window.uPlot) return;
    const el = document.getElementById('kl-dyn-chart');
    if (!el) return;
    el.innerHTML = '';
    const xs = points.map((_, i) => i);
    const ks = points.map(p => p.kelly_fraction);
    const hk = points.map(p => p.half_kelly_fraction);
    const zero = points.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 320,
        scales: { x: {}, y: {} },
        series: [
            { label: t('view.kelly.series.trade') },
            { label: t('view.kelly.series.kelly'),       stroke: '#00e5ff', width: 1.5, points: { show: false } },
            { label: t('view.kelly.series.half_kelly'),  stroke: '#ffd84a', width: 1.5, points: { show: false } },
            { label: t('view.kelly.series.zero'),        stroke: '#ff3860', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60,
              values: (_u, splits) => splits.map(v => fmtPct(v, 0)) },
        ],
        legend: { show: true },
    }, [xs, ks, hk, zero], el);
}

function renderCumPnlChart() {
    if (!window.uPlot) return;
    const el = document.getElementById('kl-cum-chart');
    if (!el) return;
    el.innerHTML = '';
    const pnls = state.pnls || [];
    if (pnls.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.kelly.empty_cum_chart">${esc(t('view.kelly.empty_cum_chart'))}</div>`;
        return;
    }
    const xs = pnls.map((_, i) => i + 1);
    const cum = [];
    let acc = 0;
    for (const p of pnls) { acc += Number(p) || 0; cum.push(acc); }
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.kelly.series.trade') },
            { label: t('view.kelly.chart.cum_pnl'),
              stroke: '#b86bff', width: 1.5, points: { show: false } },
            { label: t('view.kelly.series.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 60 }],
        legend: { show: true },
    }, [xs, cum, zero], el);
}

function renderDynamicTable(points) {
    const wrap = document.getElementById('kl-dyn-table');
    if (!points.length) { wrap.innerHTML = `<div class="muted" data-i18n="view.kelly.empty.data">No data.</div>`; return; }
    // Show every Nth row to keep the table small for long PnL series.
    const stride = Math.max(1, Math.floor(points.length / 20));
    const sampled = [];
    for (let i = 0; i < points.length; i++) {
        if (i === 0 || i === points.length - 1 || i % stride === 0) {
            sampled.push({ i, p: points[i] });
        }
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th><th data-i18n="view.kelly.th.pnl">PnL</th><th data-i18n="view.kelly.th.window_wr">Window WR</th><th data-i18n="view.kelly.th.window_payoff">Window Payoff</th>
                <th data-i18n="view.kelly.th.kelly">Kelly</th><th data-i18n="view.kelly.th.half_kelly">Half-Kelly</th>
            </tr></thead>
            <tbody>
                ${sampled.map(({ i, p }) => `<tr>
                    <td>${i + 1}</td>
                    <td class="${state.pnls[i] >= 0 ? 'pos' : 'neg'}">${esc(fmtUSDSigned(state.pnls[i]))}</td>
                    <td>${esc(fmtPct(p.window_win_rate, 1))}</td>
                    <td>${esc(p.window_payoff_ratio == null ? '—' : fmtNum(p.window_payoff_ratio, 2))}</td>
                    <td class="${p.kelly_fraction == null ? '' : (p.kelly_fraction >= 0 ? 'pos' : 'neg')}">${esc(p.kelly_fraction == null ? '—' : fmtPct(p.kelly_fraction, 2))}</td>
                    <td class="${p.half_kelly_fraction != null && p.half_kelly_fraction > 0 ? 'pos' : ''}">${esc(p.half_kelly_fraction == null ? '—' : fmtPct(p.half_kelly_fraction, 2))}</td>
                </tr>`).join('')}
            </tbody>
        </table>
    `;
    void fmtUSD;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('kl-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('kl-err').style.display = 'none'; }
