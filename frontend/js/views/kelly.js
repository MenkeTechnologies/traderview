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

let state = {
    winRate: 0.60,
    payoffRatio: 2.0,
    pnls: makeDemoPnls('positive-edge'),
    window: 10,
};

export async function renderKelly(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// KELLY SIZER</h1>

        <div class="chart-panel">
            <h2>Static Kelly</h2>
            <div class="inline-form">
                <label>Win rate (0–1)
                    <input id="kl-wr" type="number" step="any" min="0" max="1" value="${state.winRate}"></label>
                <label>Payoff ratio (avg win / avg loss)
                    <input id="kl-payoff" type="number" step="any" min="0" value="${state.payoffRatio}"></label>
                <button id="kl-run-static" class="primary" type="button">Compute static</button>
            </div>
            <div class="inline-form">
                <button id="kl-demo-positive"  class="secondary" type="button">Demo: 60% wr × 2:1</button>
                <button id="kl-demo-tiny"      class="secondary" type="button">Demo: tiny edge (50.1%)</button>
                <button id="kl-demo-extreme"   class="secondary" type="button">Demo: extreme (90% × 5:1)</button>
                <button id="kl-demo-noedge"    class="secondary" type="button">Demo: no edge (50% × 1:1)</button>
                <button id="kl-demo-negative"  class="secondary" type="button">Demo: negative (40% × 1:1)</button>
            </div>
        </div>

        <div id="kl-static-cards" class="cards"></div>

        <div class="chart-panel">
            <h2>Dynamic Kelly <small class="muted">(rolling window over trade PnLs)</small></h2>
            <textarea id="kl-pnls" rows="5" placeholder="200&#10;-100&#10;200&#10;-100&#10;...">${esc(pnlsToBlob(state.pnls))}</textarea>
            <div class="inline-form">
                <label>Window (last N trades)
                    <input id="kl-win" type="number" step="1" min="1" value="${state.window}"></label>
                <button id="kl-run-dyn" class="primary" type="button">Compute dynamic</button>
                <button id="kl-import-static" class="secondary" type="button">Derive static (wr / payoff) from these PnLs</button>
            </div>
            <div class="inline-form">
                <button id="kl-demo-pos-pnl"  class="secondary" type="button">Demo PnLs: positive edge</button>
                <button id="kl-demo-neg-pnl"  class="secondary" type="button">Demo PnLs: negative edge</button>
                <button id="kl-demo-be-pnl"   class="secondary" type="button">Demo PnLs: break-even</button>
                <button id="kl-demo-ext-pnl"  class="secondary" type="button">Demo PnLs: extreme edge</button>
                <button id="kl-demo-switch-pnl" class="secondary" type="button">Demo PnLs: regime switch</button>
            </div>
        </div>

        <div id="kl-dyn-cards" class="cards"></div>

        <div class="chart-panel">
            <h2>Rolling Kelly fraction over trades</h2>
            <div id="kl-dyn-chart" style="height:340px"></div>
            <p class="muted">Cyan = raw Kelly fraction. Yellow = half-Kelly (clamped ≥ 0). Red dashed = zero line.</p>
        </div>

        <div class="chart-panel">
            <h2>Per-trade window stats</h2>
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
            showErr('PnLs have no wins or no losses — can\'t derive payoff ratio.');
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
        showErr(`Parse errors: ${parsed.errors.slice(0, 3).map(e => `[${e.line}] ${e.message}`).join('; ')}`);
        return;
    }
    state.pnls   = parsed.pnls;
    state.window = parseInt(document.getElementById('kl-win').value, 10);
    hideErr();
}

async function computeStatic(tok) {
    const err = validateStaticInputs(state.winRate, state.payoffRatio);
    if (err) { showErr(err); return; }
    hideErr();
    const local = localComputeStatic(state.winRate, state.payoffRatio);
    renderStatic(local, true);
    let resp;
    try {
        resp = await api.calcKelly(buildStaticBody(state.winRate, state.payoffRatio));
    } catch (e) {
        showErr(`API error (static): ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderStatic(resp, false);
}

async function computeDynamic(tok) {
    const err = validateDynamicInputs(state.pnls, state.window);
    if (err) { showErr(err); return; }
    hideErr();
    const local = localComputeDynamic(state.pnls, state.window);
    renderDynamic(local, true);
    renderDynamicChart(local);
    renderDynamicTable(local);
    let resp;
    try {
        resp = await api.calcDynamicKelly(buildDynamicBody(state.pnls, state.window));
    } catch (e) {
        showErr(`API error (dynamic): ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderDynamic(resp, false);
    renderDynamicChart(resp);
    renderDynamicTable(resp);
}

function renderStatic(report, pending) {
    const badge = sizeBadge(report.full_kelly);
    const local = localComputeStatic(state.winRate, state.payoffRatio);
    const parityOk = Math.abs(report.full_kelly - local.full_kelly) < 1e-9
                  && report.note === local.note;
    document.getElementById('kl-static-cards').innerHTML = [
        card('Verdict',         badge.label + (pending ? ' (local)' : ''), badge.cls),
        card('Action',          badge.hint),
        card('Full Kelly',      fmtPct(report.full_kelly, 2),
            report.full_kelly >= 0 ? 'pos' : 'neg'),
        card('Half Kelly',      fmtPct(report.half_kelly, 2)),
        card('Quarter Kelly',   fmtPct(report.quarter_kelly, 2)),
        card('Recommended f',   fmtPct(report.recommended_f, 2),
            report.recommended_f > 0 ? 'pos' : ''),
        card('Note',            report.note || '—'),
        card('p × b vs q',      `${(state.winRate * state.payoffRatio).toFixed(3)} vs ${(1 - state.winRate).toFixed(3)}`,
            (state.winRate * state.payoffRatio) > (1 - state.winRate) ? 'pos' : 'neg'),
        card('Local parity',    parityOk ? 'OK' : 'DIVERGED', parityOk ? 'pos' : 'neg'),
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
        card('Trades (n)',       String(state.pnls.length) + (pending ? ' (local)' : '')),
        card('Window',           String(state.window)),
        card('Wins / Losses',    `${stats.wins} / ${stats.losses}`,
            stats.wins > stats.losses ? 'pos' : 'neg'),
        card('Overall win rate', fmtPct(stats.win_rate, 1)),
        card('Overall payoff',   fmtNum(stats.payoff_ratio, 3),
            stats.payoff_ratio >= 1 ? 'pos' : 'neg'),
        card('Latest Kelly',     latest && latest.kelly_fraction != null
            ? fmtPct(latest.kelly_fraction, 2) : '—',
            latest && latest.kelly_fraction > 0 ? 'pos' : 'neg'),
        card('Latest half-Kelly', latest && latest.half_kelly_fraction != null
            ? fmtPct(latest.half_kelly_fraction, 2) : '—',
            latest && latest.half_kelly_fraction > 0 ? 'pos' : ''),
        card('Latest payoff',    latest && latest.window_payoff_ratio != null
            ? fmtNum(latest.window_payoff_ratio, 3) : '—'),
        card('Local parity',     parityOk ? 'OK' : 'DIVERGED', parityOk ? 'pos' : 'neg'),
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
            { label: 'trade #' },
            { label: 'Kelly',      stroke: '#00e5ff', width: 1.5, points: { show: false } },
            { label: 'Half-Kelly', stroke: '#ffd84a', width: 1.5, points: { show: false } },
            { label: 'zero',       stroke: '#ff3860', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60,
              values: (_u, splits) => splits.map(v => fmtPct(v, 0)) },
        ],
        legend: { show: true },
    }, [xs, ks, hk, zero], el);
}

function renderDynamicTable(points) {
    const wrap = document.getElementById('kl-dyn-table');
    if (!points.length) { wrap.innerHTML = '<div class="muted">No data.</div>'; return; }
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
                <th>#</th><th>PnL</th><th>Window WR</th><th>Window Payoff</th>
                <th>Kelly</th><th>Half-Kelly</th>
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
