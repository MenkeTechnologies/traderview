// Stop-Loss Best-Of view — competes N stop strategies against your
// historical trade list and ranks them by total realized P&L.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseTradeBlob, validateInputs, buildBody,
    methodBadge, describeCandidate, bestByTotal, bestByAvg,
    defaultCandidates, makeDemoTrades, fmtSigned,
} from '../_stop_loss_best_of_inputs.js';

import { t } from '../i18n.js';
let state = { tradeText: '', sideLong: true, atr: 1.0 };

export async function renderStopLossBestOf(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.stop_loss_best_of.h1.stop_loss_best_of" class="view-title">// STOP-LOSS BEST-OF</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.stop_loss_best_of.h2.trade_outcomes">Trade outcomes</h2>
            <p class="muted" data-i18n="view.stop_loss_best_of.hint.format">One trade per line: entry mae mfe actual_exit. MAE/MFE are POSITIVE excursion magnitudes (max-adverse / max-favorable in dollars). Demo loads 20 mixed-outcome trades with realistic excursion ranges.</p>
            <textarea id="sl-trades" rows="8" placeholder="100 0.8 2.5 101.7&#10;100.5 2.1 0.5 99.2&#10;..."></textarea>
            <div class="inline-form">
                <label><span data-i18n="view.stop_loss_best_of.label.side">Side</span>
                    <select id="sl-side">
                        <option data-i18n="view.stop_loss_best_of.opt.long" value="long"  ${state.sideLong ? 'selected' : ''}>Long</option>
                        <option data-i18n="view.stop_loss_best_of.opt.short" value="short" ${!state.sideLong ? 'selected' : ''}>Short</option>
                    </select></label>
                <label><span data-i18n="view.stop_loss_best_of.label.atr">ATR (for ATR-multiple candidates)</span>
                    <input id="sl-atr" type="number" step="any" min="0" value="${state.atr}"></label>
                <button data-i18n="view.stop_loss_best_of.btn.load_demo_20_trades_9_stop_candidates" id="sl-demo" class="secondary" type="button">Load demo (20 trades, 9 stop candidates)</button>
                <button data-i18n="view.stop_loss_best_of.btn.clear" id="sl-clear" class="secondary" type="button">Clear</button>
                <button data-i18n="view.stop_loss_best_of.btn.compete" id="sl-run" class="primary" type="button">Compete</button>
            </div>
            <p data-i18n="view.stop_loss_best_of.hint.9_candidates_by_default_none_1_2_0_5_1_2_1_atr_2_a" class="muted">9 candidates by default: None / $1 / $2 / 0.5% / 1% / 2% / 1×ATR / 2×ATR / 3×ATR.
                Each is simulated against all trades; results ranked by total realized P&amp;L.</p>
        </div>

        <div id="sl-errors" class="boot" style="display:none"></div>
        <div id="sl-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.stop_loss_best_of.h2.candidate_results_sorted_best_total_first">Candidate results (sorted best-total-first)</h2>
            <div id="sl-results"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.stop_loss_best_of.h2.total_chart">Total realized per candidate</h2>
            <div id="sl-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.stop_loss_best_of.h2.stop_chart">Stop-out count per candidate</h2>
            <div id="sl-stop-chart" style="width:100%;height:200px"></div>
        </div>

        <div id="sl-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('sl-demo').addEventListener('click', () => {
        const t = makeDemoTrades(42);
        document.getElementById('sl-trades').value =
            t.map(x => `${x.entry} ${x.mae} ${x.mfe} ${x.actual_exit}`).join('\n');
        document.getElementById('sl-atr').value = 1.0;
    });
    document.getElementById('sl-clear').addEventListener('click', () => {
        document.getElementById('sl-trades').value = '';
    });
    document.getElementById('sl-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.tradeText = document.getElementById('sl-trades').value;
    state.sideLong = document.getElementById('sl-side').value === 'long';
    state.atr = Number(document.getElementById('sl-atr').value);
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('sl-errors');
    errs.style.display = 'none';
    const { trades, errors } = parseTradeBlob(state.tradeText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            t('common.parse_error_inline', { line: e.line_no, msg: e.message, raw: e.raw.slice(0, 80) })).join('<br>');
        const more = errors.length > 8 ? `<br>${esc(t('common.and_n_more', { n: errors.length - 8 }))}` : '';
        errs.innerHTML = `<strong>${esc(t('common.parse_errors_lead', { n: errors.length }))}</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (trades.length === 0) return;
    }
    const candidates = defaultCandidates(state.atr);
    const err = validateInputs(trades, candidates, state.sideLong);
    if (err) { showErr(err); return; }
    let results;
    try {
        results = await api.discStopLossBestOf(buildBody(trades, candidates, state.sideLong));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e })); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(results || [], trades, candidates);
    renderResults(results || [], candidates);
    renderTotalChart(results || [], candidates);
    renderStopChart(results || [], candidates);
}

function renderStopChart(results, candidates) {
    const el = document.getElementById('sl-stop-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (results || []).filter(r => Number.isFinite(Number(r.stopped_out_count)));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.stop_loss_best_of.empty_stop_chart">${esc(t('view.stop_loss_best_of.empty_stop_chart'))}</div>`;
        return;
    }
    rows.sort((a, b) => Number(b.stopped_out_count) - Number(a.stopped_out_count));
    const labels = rows.map(r => {
        const c = candidates.find(x => x.method === r.method && x.value === r.value);
        return c ? describeCandidate(c) : r.method;
    });
    const xs = labels.map((_, i) => i + 1);
    const ys = rows.map(r => Number(r.stopped_out_count));
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.stop_loss_best_of.chart.candidate') },
            { label: t('view.stop_loss_best_of.chart.stopped'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 14, fill: '#ff3860', stroke: '#ff3860' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderSummary(results, trades, candidates) {
    const best = bestByTotal(results);
    const bestAvg = bestByAvg(results);
    const matched = results.length;
    document.getElementById('sl-summary').innerHTML = [
        card(t('view.stop_loss_best_of.card.trades'),     String(trades.length)),
        card(t('view.stop_loss_best_of.card.candidates'), String(candidates.length)),
        card(t('view.stop_loss_best_of.card.results'),    String(matched)),
        card(t('view.stop_loss_best_of.card.best_total_d'), best ? fmtSigned(best.total_realized) : '—', best && best.total_realized >= 0 ? 'pos' : 'neg'),
        card(t('view.stop_loss_best_of.card.best_method'), best ? describeCandidate(candidatesByMethod(candidates, best)) : '—',
            best ? methodBadge(best.method).cls : ''),
        card(t('view.stop_loss_best_of.card.best_avg_per_trade_d'), bestAvg ? fmtSigned(bestAvg.avg_realized) : '—', bestAvg && bestAvg.avg_realized >= 0 ? 'pos' : 'neg'),
    ].join('');
}

function candidatesByMethod(candidates, result) {
    return candidates.find(c => c.method === result.method && c.value === result.value);
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderTotalChart(results, candidates) {
    const el = document.getElementById('sl-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (results || []).filter(r => Number.isFinite(Number(r.total_realized)));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.stop_loss_best_of.empty_chart">${esc(t('view.stop_loss_best_of.empty_chart'))}</div>`;
        return;
    }
    rows.sort((a, b) => Number(b.total_realized) - Number(a.total_realized));
    const labels = rows.map(r => {
        const c = candidates.find(x => x.method === r.method && x.value === r.value);
        return c ? describeCandidate(c) : r.method;
    });
    const xs = labels.map((_, i) => i + 1);
    const posY = rows.map(r => Number(r.total_realized) >= 0 ? Number(r.total_realized) : null);
    const negY = rows.map(r => Number(r.total_realized) <  0 ? Number(r.total_realized) : null);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.stop_loss_best_of.chart.candidate') },
            { label: t('view.stop_loss_best_of.chart.win'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.stop_loss_best_of.chart.lose'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.stop_loss_best_of.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 56 },
        ],
        legend: { show: true },
    }, [xs, posY, negY, zero], el);
}

function renderResults(results, candidates) {
    const wrap = document.getElementById('sl-results');
    if (!results.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.stop_loss_best_of.empty.results">No results.</div>`;
        return;
    }
    // Sort by total realized (desc) for the table; keep raw indices for
    // matching back to the candidate row (method + value tuple).
    const sorted = [...results].sort((a, b) => (b.total_realized || 0) - (a.total_realized || 0));
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.stop_loss_best_of.th.rank">Rank</th><th data-i18n="view.stop_loss_best_of.th.method">Method</th><th data-i18n="view.stop_loss_best_of.th.description">Description</th>
                <th data-i18n="view.stop_loss_best_of.th.total">Total $</th><th data-i18n="view.stop_loss_best_of.th.avg_trade">Avg $/trade</th><th data-i18n="view.stop_loss_best_of.th.wins">Wins</th><th data-i18n="view.stop_loss_best_of.th.stopped_out">Stopped out</th>
            </tr></thead>
            <tbody>
                ${sorted.map((r, i) => {
                    const c = candidates.find(x => x.method === r.method && x.value === r.value);
                    const badge = methodBadge(r.method);
                    return `<tr>
                        <td><strong>${i + 1}</strong></td>
                        <td class="${badge.cls}">${esc(badge.label)}</td>
                        <td>${esc(describeCandidate(c))}</td>
                        <td class="${r.total_realized >= 0 ? 'pos' : 'neg'}">${esc(fmtSigned(r.total_realized))}</td>
                        <td class="${r.avg_realized >= 0 ? 'pos' : 'neg'}">${esc(fmtSigned(r.avg_realized))}</td>
                        <td>${r.winning_trades}</td>
                        <td>${r.stopped_out_count}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
}

function showErr(msg) {
    const el = document.getElementById('sl-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('sl-err').style.display = 'none'; }
