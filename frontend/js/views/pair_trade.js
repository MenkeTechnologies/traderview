// Pair Trade Z-Score view.
//
// Backend gives a snapshot (β from OLS y~x, spread stats, current z,
// signal). The view ALSO computes the full historical z series locally
// using the backend's β so the user sees when the strategy would have
// triggered + how often it mean-reverted.
//
// Visualization:
//   * Top chart: y price and β·x price overlaid (the dollar-neutral
//     hedge view — if β is right, the two curves track).
//   * Bottom chart: z-score time series with horizontal threshold
//     bands (entry, exit, stop).

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseSeries, validateInputs, buildBody,
    spreadAndZSeries, countCrossings,
    fmtSignal, signalCssClass,
} from '../_pair_trade_inputs.js';

import { t } from '../i18n.js';
const DEFAULT_Y = `# Y leg prices (the dependent leg in the OLS β regression).
# Demo: cointegrated pair — y wanders around 2 × x + 50 with noise.
${synthY(150).join('\n')}
`;
const DEFAULT_X = `# X leg prices (the independent / "hedge" leg).
${synthX(150).join('\n')}
`;

function rng(tag) {
    let s = tag;
    return () => {
        s = (s * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(s >> 11n) / 2 ** 53;
    };
}

function synthX(n) {
    const r = rng(0xCAFEBEEF0n);
    const normal = () => {
        const u1 = Math.max(r(), 1e-10);
        const u2 = r();
        return Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
    };
    let p = 100;
    const out = [];
    for (let i = 0; i < n; i++) {
        p += normal() * 0.5;
        out.push(p.toFixed(3));
    }
    return out;
}

function synthY(n) {
    const rX = rng(0xCAFEBEEF0n);
    const rY = rng(0xDECAFBADn);
    const normal = (gen) => {
        const u1 = Math.max(gen(), 1e-10);
        const u2 = gen();
        return Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
    };
    let px = 100;
    const out = [];
    for (let i = 0; i < n; i++) {
        px += normal(rX) * 0.5;
        // y = 2·x + 50 + mean-reverting spread (OU-like)
        const noise = normal(rY) * 1.0;
        out.push((2 * px + 50 + noise).toFixed(3));
    }
    return out;
}

let state = {
    yText: DEFAULT_Y,
    xText: DEFAULT_X,
    config: { entry_z: 2.0, exit_z: 0.5, stop_z: 3.5 },
};

export async function renderPairTrade(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.pair_trade.h1.pair_trade" class="view-title">// PAIR TRADE</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.pair_trade.h2.price_series">Price series</h2>
            <div class="op-inputs-grid">
                <div>
                    <h3 data-i18n="view.pair_trade.h3.y_leg_dependent">y leg (dependent)</h3>
                    <textarea id="pt-y" rows="10"
                        style="width:100%;font-family:monospace;font-size:13px">${esc(state.yText)}</textarea>
                </div>
                <div>
                    <h3 data-i18n="view.pair_trade.h3.x_leg_independent_hedge">x leg (independent / hedge)</h3>
                    <textarea id="pt-x" rows="10"
                        style="width:100%;font-family:monospace;font-size:13px">${esc(state.xText)}</textarea>
                </div>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.pair_trade.h2.signal_thresholds_z_score_bands">Signal thresholds (z-score bands)</h2>
            <div class="inline-form">
                <label><span data-i18n="view.pair_trade.label.entry_z">Entry z (enter when |z| &gt;)</span>
                    <input id="pt-entry" type="number" step="any" min="0" value="${state.config.entry_z}"></label>
                <label><span data-i18n="view.pair_trade.label.exit_z">Exit z (exit when |z| &lt;)</span>
                    <input id="pt-exit"  type="number" step="any" min="0" value="${state.config.exit_z}"></label>
                <label><span data-i18n="view.pair_trade.label.stop_z">Stop z (bail when |z| &gt;)</span>
                    <input id="pt-stop"  type="number" step="any" min="0" value="${state.config.stop_z}"></label>
                <button data-i18n="view.pair_trade.btn.analyze" id="pt-run" class="primary" type="button">Analyze</button>
            </div>
            <p data-i18n="view.pair_trade.hint.is_fit_by_ols_regression_of_y_on_x_spread_y_x_z_sp" class="muted">
                β is fit by OLS regression of y on x. Spread = y − β·x. Z = (spread − mean) /
                stdev. Convention: positive z → spread expensive (sell y, buy x); negative
                z → spread cheap (buy y, sell x). exit_z must be &lt; entry_z; stop_z must be &gt; entry_z.
            </p>
        </div>

        <div id="pt-parse-errors" class="boot" style="display:none;color:var(--red)"></div>

        <div id="pt-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.pair_trade.h2.price_overlay_y_vs_x">Price overlay (y vs β·x)</h2>
            <div id="pt-price-chart" style="width:100%;height:280px"></div>
            <p data-i18n="view.pair_trade.hint.if_is_right_the_two_lines_track_each_other_and_the" class="muted">
                If β is right, the two lines track each other and their difference (the
                spread) oscillates around 0. Visible drift between them = the spread is
                widening — opportunity if mean-reverting, danger if cointegration broke.
            </p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.pair_trade.h2.spread_z_score_history">Spread z-score (history)</h2>
            <div id="pt-z-chart" style="width:100%;height:340px"></div>
            <p data-i18n="view.pair_trade.hint.cyan_z_score_dashed_orange_entry_band_dashed_red_s" class="muted">
                Cyan: z-score. Dashed orange: ±entry band. Dashed red: ±stop band. Bars
                outside the entry band would have triggered an entry signal; bars between
                ±exit band would have triggered an exit.
            </p>
        </div>

        <div id="pt-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('pt-run').addEventListener('click', () => {
        readInputs();
        void analyze(mount, tok);
    });
    void fmt;
}

function readInputs() {
    const get = id => document.getElementById(id).value;
    state.yText = get('pt-y');
    state.xText = get('pt-x');
    state.config = {
        entry_z: Number(get('pt-entry')),
        exit_z:  Number(get('pt-exit')),
        stop_z:  Number(get('pt-stop')),
    };
}

async function analyze(mount, tok) {
    hideErrs();
    const parsedY = parseSeries(state.yText);
    const parsedX = parseSeries(state.xText);
    const errors = parsedY.errors.concat(parsedX.errors);
    if (errors.length) renderParseErrors(errors);

    const err = validateInputs(parsedY.value, parsedX.value, state.config);
    if (err) { showErr(err); return; }

    let res;
    try {
        res = await api.anlyPairTradeSignal(buildBody(parsedY.value, parsedX.value, state.config));
        if (!res) throw new Error(t('view.pair_trade.error.null'));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        return;
    }
    if (!viewIsCurrent(tok)) return;

    const local = spreadAndZSeries(parsedY.value, parsedX.value, res.hedge_ratio);
    renderSummary(res, local);
    renderPriceChart(parsedY.value, parsedX.value, res.hedge_ratio);
    renderZChart(local.zs);
}

function renderSummary(res, local) {
    const cards = [];
    cards.push(card(t('view.pair_trade.card.signal'), fmtSignal(res.signal), signalCssClass(res.signal)));
    cards.push(card(t('view.pair_trade.card.hedge_ratio'), res.hedge_ratio.toFixed(4)));
    cards.push(card(t('view.pair_trade.card.current_z'), res.current_z.toFixed(3),
        Math.abs(res.current_z) > state.config.entry_z ? 'pos' : ''));
    cards.push(card(t('view.pair_trade.card.current_spread'), res.current_spread.toFixed(4), '',
        `<div class="vc-row"><span class="muted" data-i18n="view.pair_trade.row.mean_sigma">mean / σ</span>
            <strong>${res.spread_mean.toFixed(4)} / ${res.spread_stdev.toFixed(4)}</strong></div>`));
    cards.push(card(t('view.pair_trade.card.entry_crossings_history'),
        String(countCrossings(local.zs, state.config.entry_z))));
    cards.push(card(t('view.pair_trade.card.stop_crossings_history'),
        String(countCrossings(local.zs, state.config.stop_z)),
        countCrossings(local.zs, state.config.stop_z) > 0 ? 'neg' : 'pos'));
    document.getElementById('pt-summary').innerHTML = cards.join('');
}

function card(label, value, cls = '', body = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
        ${body ? `<div class="value pt-summary-value">${body}</div>` : ''}
    </div>`;
}

function renderPriceChart(y, x, beta) {
    const el = document.getElementById('pt-price-chart');
    if (!window.uPlot) { el.textContent = t('common.error.uplot_not_loaded'); return; }
    el.innerHTML = '';
    const xs = Array.from({ length: y.length }, (_, i) => i);
    const betaX = x.map(v => beta * v);
    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 280,
        scales: { x: {}, y: {} },
        series: [
            { label: 'bar' },
            { label: 'y',   stroke: '#00e5ff', width: 1.5, points: { show: false } },
            { label: 'β·x', stroke: '#ff9f1a', width: 1.5, points: { show: false } },
        ],
        axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
    }, [xs, y, betaX], el);
}

function renderZChart(zs) {
    const el = document.getElementById('pt-z-chart');
    if (!window.uPlot) { el.textContent = t('common.error.uplot_not_loaded'); return; }
    el.innerHTML = '';
    const xs = Array.from({ length: zs.length }, (_, i) => i);
    const entryUp   = xs.map(() =>  state.config.entry_z);
    const entryDown = xs.map(() => -state.config.entry_z);
    const stopUp    = xs.map(() =>  state.config.stop_z);
    const stopDown  = xs.map(() => -state.config.stop_z);
    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 340,
        scales: { x: {}, y: {} },
        series: [
            { label: 'bar' },
            { label: 'z',        stroke: '#00e5ff', width: 2,
              fill: 'rgba(0,229,255,0.06)', points: { show: false } },
            { label: '+entry',   stroke: '#ff9f1a', width: 1, dash: [4, 4], points: { show: false } },
            { label: '−entry',   stroke: '#ff9f1a', width: 1, dash: [4, 4], points: { show: false } },
            { label: '+stop',    stroke: '#ff3860', width: 1, dash: [2, 4], points: { show: false } },
            { label: '−stop',    stroke: '#ff3860', width: 1, dash: [2, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
    }, [xs, zs, entryUp, entryDown, stopUp, stopDown], el);
}

function renderParseErrors(errors) {
    const el = document.getElementById('pt-parse-errors');
    el.innerHTML = errors.slice(0, 20).map(e =>
        `<div>line ${e.line_no}: ${esc(e.message)} <span class="muted">→ <code>${esc(e.raw || '')}</code></span></div>`
    ).join('');
    if (errors.length > 20) {
        el.innerHTML += `<div class="muted">… (+${errors.length - 20} more)</div>`;
    }
    el.style.display = 'block';
}

function showErr(msg) {
    const el = document.getElementById('pt-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErrs() {
    document.getElementById('pt-parse-errors').style.display = 'none';
    document.getElementById('pt-err').style.display = 'none';
}
