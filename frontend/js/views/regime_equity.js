// Equity-curve Regime view — classifies the trader's equity curve into
// TrendingUp/Down, VolatileUp/Down, or Choppy via linear-fit slope +
// residual variance.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_CONFIG, parseEquityBlob, validateInputs, buildBody, localEvaluate,
    regimeBadge, fitLine, makeDemoEquity,
    fmtUSD, fmtUSDSigned, fmtPct, fmtNum,
} from '../_regime_equity_inputs.js';

import { t } from '../i18n.js';
let state = {
    equity: makeDemoEquity('realistic'),
    config: { ...DEFAULT_CONFIG },
};

export async function renderRegimeEquity(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.regime_equity.h1.equity_regime" class="view-title">// EQUITY REGIME</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.regime_equity.h2.paste_equity_curve_one_value_per_line_or_csv_white">Paste equity curve (one value per line, or CSV / whitespace-separated)</h2>
            <textarea id="re-blob" rows="6" placeholder="10000&#10;10100&#10;10250&#10;...">${esc(state.equity.join('\n'))}</textarea>
            <div class="inline-form">
                <label><span data-i18n="view.regime_equity.label.trend_slope">Trend slope %</span>
                    <small class="muted" data-i18n="view.regime_equity.hint.trend_slope">(min |slope/mean_eq| to count as trending; default 0.001 = 0.1%)</small>
                    <input id="re-slope" type="number" step="any" min="0" value="${state.config.trend_slope_pct}"></label>
                <label><span data-i18n="view.regime_equity.label.clean_rel_stdev">Clean rel stdev</span>
                    <small class="muted" data-i18n="view.regime_equity.hint.clean_rel_stdev">(max residual/mean_eq for "clean" trend; default 0.02 = 2%)</small>
                    <input id="re-rsd"   type="number" step="any" min="0" value="${state.config.clean_trend_rel_stdev}"></label>
                <button data-i18n="view.regime_equity.btn.analyze" id="re-run" class="primary" type="button">Analyze</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.regime_equity.btn.demo_trending_up" id="re-demo-trend-up"   class="secondary" type="button">Demo: TRENDING UP</button>
                <button data-i18n="view.regime_equity.btn.demo_trending_down" id="re-demo-trend-dn"   class="secondary" type="button">Demo: TRENDING DOWN</button>
                <button data-i18n="view.regime_equity.btn.demo_volatile_up" id="re-demo-vol-up"     class="secondary" type="button">Demo: VOLATILE UP</button>
                <button data-i18n="view.regime_equity.btn.demo_volatile_down" id="re-demo-vol-dn"     class="secondary" type="button">Demo: VOLATILE DOWN</button>
                <button data-i18n="view.regime_equity.btn.demo_choppy" id="re-demo-choppy"    class="secondary" type="button">Demo: CHOPPY</button>
                <button data-i18n="view.regime_equity.btn.demo_realistic_90_day" id="re-demo-realistic" class="secondary" type="button">Demo: realistic 90-day</button>
            </div>
        </div>

        <div id="re-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.regime_equity.h2.equity_vs_fitted_trend_line">Equity vs fitted trend line</h2>
            <div id="re-chart" style="height:320px"></div>
            <p data-i18n="view.regime_equity.hint.cyan_your_equity_yellow_ols_regression_fit_spread_" class="muted">Cyan = your equity. Yellow = OLS regression fit. Spread of equity around the fit ⇒ residual stdev ⇒ trending vs volatile classification.</p>
        </div>

        <div id="re-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (kind) => {
        state.equity = makeDemoEquity(kind);
        document.getElementById('re-blob').value = state.equity.join('\n');
    };
    document.getElementById('re-demo-trend-up').addEventListener('click',   () => loadDemo('trending-up'));
    document.getElementById('re-demo-trend-dn').addEventListener('click',   () => loadDemo('trending-down'));
    document.getElementById('re-demo-vol-up').addEventListener('click',     () => loadDemo('volatile-up'));
    document.getElementById('re-demo-vol-dn').addEventListener('click',     () => loadDemo('volatile-down'));
    document.getElementById('re-demo-choppy').addEventListener('click',     () => loadDemo('choppy'));
    document.getElementById('re-demo-realistic').addEventListener('click',  () => loadDemo('realistic'));
    document.getElementById('re-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
    readInputs(); void compute(tok);
}

function readInputs() {
    const parsed = parseEquityBlob(document.getElementById('re-blob').value);
    if (parsed.errors.length) {
        showErr(t("common.error.parse_errors", { summary: parsed.errors.slice(0, 3).map(e => `[] `).join("; ") }));
        return;
    }
    state.equity = parsed.equity;
    state.config = {
        trend_slope_pct: Number(document.getElementById('re-slope').value),
        clean_trend_rel_stdev: Number(document.getElementById('re-rsd').value),
    };
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.equity, state.config);
    if (err) { showErr(err); return; }
    const local = localEvaluate(state.equity, state.config);
    renderSummary(local, true);
    renderChart(state.equity, local);
    let resp;
    try {
        resp = await api.regimeEquity(buildBody(state.equity, state.config));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e })); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary({ ...resp, intercept: local.intercept, mean_equity: local.mean_equity }, false);
    renderChart(state.equity, { ...resp, intercept: local.intercept });
}

function renderSummary(r, pending) {
    const badge = regimeBadge(r.regime);
    const local = localEvaluate(state.equity, state.config);
    const parityOk = r.regime === local.regime;
    document.getElementById('re-summary').innerHTML = [
        card(t('view.regime_equity.card.regime'),           badge.label + (pending ? ' (local)' : ''), badge.cls),
        card(t('view.regime_equity.card.action'),           badge.hint),
        card(t('view.regime_equity.card.samples'),        String(r.n)),
        card(t('view.regime_equity.card.slope_period'),   fmtUSDSigned(r.slope_per_period, 2),
            r.slope_per_period >= 0 ? 'pos' : 'neg'),
        card(t('view.regime_equity.card.r'),               fmtNum(r.r_squared, 4),
            r.r_squared >= 0.5 ? 'pos' : ''),
        card(t('view.regime_equity.card.residual_stdev'),   fmtUSD(r.residual_stdev, 2)),
        card(t('view.regime_equity.card.mean_equity'),      fmtUSD(r.mean_equity ?? local.mean_equity, 2)),
        card(t('view.regime_equity.card.total'),          fmtUSDSigned(state.equity[state.equity.length - 1] - state.equity[0], 2),
            state.equity[state.equity.length - 1] - state.equity[0] >= 0 ? 'pos' : 'neg'),
        card(t('view.regime_equity.card.rel_slope'),        fmtPct((r.mean_equity ?? local.mean_equity) > 0
            ? r.slope_per_period / (r.mean_equity ?? local.mean_equity) : 0, 4)),
        card(t('view.regime_equity.card.local_parity'),     parityOk ? t('common.ok') : t('common.diverged'), parityOk ? 'pos' : 'neg'),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderChart(equity, report) {
    if (!window.uPlot) return;
    const el = document.getElementById('re-chart');
    if (!el) return;
    el.innerHTML = '';
    const xs = equity.map((_, i) => i);
    const fit = fitLine(equity, report);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 300,
        scales: { x: {}, y: {} },
        series: [
            { label: 'bar #' },
            { label: 'equity', stroke: '#00e5ff', width: 1.5,
              fill: '#00e5ff14', points: { show: false } },
            { label: 'fit', stroke: '#ffd84a', width: 1.0,
              dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, equity, fit], el);
}

function showErr(msg) {
    const el = document.getElementById('re-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('re-err').style.display = 'none'; }
