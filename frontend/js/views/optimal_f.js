// Optimal-f Position Sizer view (Ralph Vince 1990).
//
// User pastes a trade-P/L series (one trade's $ or % return per token).
// Backend returns the geometric-growth-optimal bet fraction + the TWR
// at that f + practitioner-conservative fractions (half-Kelly,
// quarter-Kelly).
//
// View adds a local "TWR vs f" sweep chart so the user can see how
// quickly the geometric growth degrades as f deviates from optimal —
// often the right-side of the curve falls off a cliff (overbetting),
// which is exactly why half-Kelly is the practitioner default.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseReturns, validateInputs, buildBody,
    twrSweep, fmtPctF, fmtMoney, fmtMultiple,
} from '../_optimal_f_inputs.js';

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
const DEFAULT_TEXT = `# Paste trade P/Ls (one per token, signed: positive = win, negative = loss).
# Demo: 20 simulated trades with ~55% win rate, win:loss ratio ~ 1.5:1.
500   -300   450   -280   600   -310   520   -270   480   -290
-310  640   -300   570   -260   590   -280   510   -290   620
`;

let state = { text: DEFAULT_TEXT, lastReturns: null };

export async function renderOptimalF(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.optimal_f.h1.optimal_f_sizer" class="view-title">// OPTIMAL-F SIZER</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.optimal_f.h2.trade_p_l_series">Trade P/L series</h2>
            <textarea id="of-text" rows="8"
                data-tip="view.optimal_f.tip.pnls"
                style="width:100%;font-family:monospace;font-size:13px">${esc(state.text)}</textarea>
            <div class="inline-form" style="margin-top:8px">
                <button data-i18n="view.optimal_f.btn.compute" data-tip="view.optimal_f.tip.compute" data-shortcut="optimal_f_compute" id="of-run" class="primary" type="button">Compute</button>
            </div>
            <p data-i18n="view.optimal_f.hint.optimal_f_the_fraction_of_capital_to_risk_per_trad" class="muted">
                Optimal-f = the fraction of capital to risk per trade that maximizes
                long-run geometric growth (Vince 1990). Practitioners almost always use
                half- or quarter-Kelly because the TWR curve is asymmetric — overbetting
                by 50% loses more growth than underbetting by 50%.
            </p>
        </div>

        <div id="of-parse-errors" class="boot" style="display:none;color:var(--red)"></div>

        <div id="of-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.optimal_f.h2.twr_vs_bet_fraction">TWR vs bet fraction</h2>
            <div id="of-chart" style="width:100%;height:340px"></div>
            <p data-i18n="view.optimal_f.hint.geometric_growth_curve_from_f_0_to_1_cyan_twr_vert" class="muted">
                Geometric growth curve from f = 0 to 1. Cyan: TWR. Vertical markers:
                optimal-f (orange), half-Kelly (cyan), quarter-Kelly (green). Notice the
                cliff to the right of optimal-f — overbetting wipes out wealth faster than
                underbetting forgoes it.
            </p>
        </div>

        <div id="of-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('of-run').addEventListener('click', () => {
        state.text = document.getElementById('of-text').value;
        void compute(mount, tok);
    });
    void fmt;
}

async function compute(mount, tok) {
    hideErrs();
    const parsed = parseReturns(state.text);
    if (parsed.errors.length) renderParseErrors(parsed.errors);

    const err = validateInputs(parsed.value);
    if (err) { showErr(err); showToast(err, { level: 'warning' }); return; }

    state.lastReturns = parsed.value;

    let res;
    try {
        res = await api.calcOptimalF(buildBody(parsed.value));
        if (!res) throw new Error(t('view.optimal_f.error.null_result'));
    } catch (e) {
        const m = t("common.error.api", { msg: e.message || e });
        showErr(m); showToast(m, { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;

    renderSummary(parsed.value, res);
    renderChart(parsed.value, res);
    showToast(t('view.optimal_f.toast.done', {
        f: fmtPctF(res.optimal_f),
        twr: fmtMultiple(res.twr_at_optimal),
    }), { level: 'success' });
}

function renderSummary(returns, res) {
    const winners = returns.filter(r => r > 0);
    const losers  = returns.filter(r => r < 0);
    const winRate = returns.length > 0 ? (winners.length / returns.length) : NaN;
    const avgWin  = winners.length > 0
        ? winners.reduce((a, b) => a + b, 0) / winners.length : NaN;
    const avgLoss = losers.length > 0
        ? losers.reduce((a, b) => a + b, 0) / losers.length : NaN;
    const cards = [];
    cards.push(card(t('view.optimal_f.card.optimal_f'),  fmtPctF(res.optimal_f), 'pos',
        `<div class="vc-row"><span class="muted">${esc(t('view.optimal_f.sub.twr_at_optimal'))}</span>
            <strong>${fmtMultiple(res.twr_at_optimal)}</strong></div>`));
    cards.push(card(t('view.optimal_f.card.half_kelly_recommended'), fmtPctF(res.half_kelly), '',
        `<div class="vc-row"><span class="muted">${esc(t('view.optimal_f.sub.conservative_default'))}</span>
            <strong>optimal-f / 2</strong></div>`));
    cards.push(card(t('view.optimal_f.card.quarter_kelly'), fmtPctF(res.quarter_kelly), '',
        `<div class="vc-row"><span class="muted">${esc(t('view.optimal_f.sub.ultra_conservative'))}</span>
            <strong>optimal-f / 4</strong></div>`));
    cards.push(card(t('view.optimal_f.card.worst_single_trade_loss'), fmtMoney(-res.worst_loss), 'neg'));
    cards.push(card(t('view.optimal_f.card.trades'), String(returns.length), '',
        `<div class="vc-row"><span class="muted">${esc(t('view.optimal_f.sub.wins_losses'))}</span>
            <strong>${winners.length} / ${losers.length}</strong></div>`));
    cards.push(card(t('view.optimal_f.card.win_rate'), fmtPctF(winRate)));
    cards.push(card(t('view.optimal_f.card.avg_win'),  fmtMoney(avgWin),  winners.length > 0 ? 'pos' : ''));
    cards.push(card(t('view.optimal_f.card.avg_loss'), fmtMoney(avgLoss), losers.length > 0 ? 'neg' : ''));
    if (res.note) {
        cards.push(card(t('view.optimal_f.card.note'), res.note));
    }
    document.getElementById('of-summary').innerHTML = cards.join('');
}

function card(label, value, cls = '', body = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
        ${body ? `<div class="value of-summary-value">${body}</div>` : ''}
    </div>`;
}

function renderChart(returns, res) {
    const el = document.getElementById('of-chart');
    if (!window.uPlot) { el.textContent = t('common.error.uplot_not_loaded'); return; }
    el.innerHTML = '';
    const { xs, ys } = twrSweep(returns, 101);
    if (xs.length === 0) {
        el.innerHTML = `<div class="boot">${esc(t('view.optimal_f.empty.need_losing_trade'))}</div>`;
        return;
    }
    // Build single-point marker series for the 3 reference fractions
    // (optimal, half, quarter). Each marker is null everywhere except
    // at the x closest to the marker's f value.
    const halfWidth = (xs[1] - xs[0]) / 2;
    const markerAt = (target) => xs.map(x =>
        Math.abs(x - target) < halfWidth ? ys[xs.indexOf(x)] : null);
    const optimalMarker = markerAt(res.optimal_f);
    const halfMarker    = markerAt(res.half_kelly);
    const quarterMarker = markerAt(res.quarter_kelly);

    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 340,
        scales: { x: {}, y: {} },
        series: [
            { label: 'f' },
            { label: 'TWR', stroke: '#00e5ff', width: 2, points: { show: false } },
            { label: t('chart.series.optimalf'),   stroke: '#ff9f1a', width: 0,
              points: { show: true, size: 12, stroke: '#ff9f1a', fill: '#ff9f1a' } },
            { label: t('chart.series.halfkelly'),  stroke: '#00e5ff', width: 0,
              points: { show: true, size: 10, stroke: '#00e5ff', fill: '#00e5ff' } },
            { label: t('chart.series.quarterkelly'), stroke: '#39ff14', width: 0,
              points: { show: true, size: 8, stroke: '#39ff14', fill: '#39ff14' } },
        ],
        axes: [
            { stroke: '#aab',
              values: (_, ticks) => ticks.map(t => `${(t * 100).toFixed(0)}%`) },
            { stroke: '#aab' },
        ],
    }, [xs, ys, optimalMarker, halfMarker, quarterMarker], el);
}

function renderParseErrors(errors) {
    const el = document.getElementById('of-parse-errors');
    el.innerHTML = errors.slice(0, 20).map(e =>
        `<div>${esc(t('common.parse_error_line', { line: e.line_no, msg: e.message }))} <span class="muted">→ <code>${esc(e.raw || '')}</code></span></div>`
    ).join('');
    if (errors.length > 20) {
        el.innerHTML += `<div class="muted">${esc(t("common.plus_n_more", { n: errors.length - 20 }))}</div>`;
    }
    el.style.display = 'block';
}

function showErr(msg) {
    const el = document.getElementById('of-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErrs() {
    document.getElementById('of-parse-errors').style.display = 'none';
    document.getElementById('of-err').style.display = 'none';
}
