// Carhart 4-Factor Model view — Mkt + SMB + HML + WML regression.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseSeriesBlob, seriesToBlob, validateInputs, buildBody, localCompute,
    alphaBadge, styleBadge, fitBadge, marketBetaBadge, summarizeSeries,
    makeDemoInput,
    fmtBeta, fmtBetaSigned, fmtPct, fmtInt, fmtTStat,
} from '../_carhart4_inputs.js';

let state = { ...makeDemoInput('market-only') };

export async function renderCarhart4(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.car4.h1.title" class="view-title">// CARHART 4-FACTOR MODEL</h1>

        <div class="chart-panel" data-context-scope="carhart-4">
            <h2 data-i18n="view.car4.h2.series">Series
                <small data-i18n="view.car4.h2.series_hint" class="muted">(6 tokens per line: PORT MKT SMB HML WML RF — ≥ 10 valid rows)</small></h2>
            <textarea id="c4-blob" rows="8"
                      data-tip="view.car4.tip.series"
                      placeholder="0.01 0.012 0.003 -0.001 0.005 0.00005\n...">${esc(seriesToBlob(state))}</textarea>

            <div class="inline-form">
                <button data-i18n="view.car4.btn.compute" id="c4-run" class="primary"
                        data-tip="view.car4.tip.compute" type="button">Regress</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.car4.btn.demo_mkt"   id="c4-d1" class="secondary" type="button">Demo: market-only β=1</button>
                <button data-i18n="view.car4.btn.demo_small" id="c4-d2" class="secondary" type="button">Demo: small-cap tilt</button>
                <button data-i18n="view.car4.btn.demo_value" id="c4-d3" class="secondary" type="button">Demo: value tilt</button>
                <button data-i18n="view.car4.btn.demo_mom"   id="c4-d4" class="secondary" type="button">Demo: momentum tilt</button>
                <button data-i18n="view.car4.btn.demo_grow"  id="c4-d5" class="secondary" type="button">Demo: growth tilt</button>
                <button data-i18n="view.car4.btn.demo_alpha" id="c4-d6" class="secondary" type="button">Demo: positive α</button>
                <button data-i18n="view.car4.btn.demo_neg"   id="c4-d7" class="secondary" type="button">Demo: negative α</button>
                <button data-i18n="view.car4.btn.demo_small_n" id="c4-d8" class="secondary" type="button">Demo: small sample (n=15)</button>
            </div>
            <p data-i18n="view.car4.hint.about" class="muted">Carhart (1997) extension of Fama-French 3-factor: Mkt (market excess), SMB (size), HML (value), WML (momentum) regress portfolio excess returns. β coefficients = factor loadings; α (intercept) = unexplained excess return. |t-stat| > 1.96 ≈ 5% significant. Non-finite rows skipped silently.</p>
        </div>

        <div id="c4-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.car4.h2.factors">Factor loadings</h2>
            <div id="c4-factors"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.car4.h2.stats">Series summary</h2>
            <div id="c4-stats"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.car4.h2.factor_chart">Factor coefficients</h2>
            <div id="c4-chart" style="width:100%;height:240px"></div>
        </div>

        <div id="c4-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('c4-blob').value = seriesToBlob(state);
    };
    document.getElementById('c4-d1').addEventListener('click', () => { loadDemo('market-only');     void compute(tok); });
    document.getElementById('c4-d2').addEventListener('click', () => { loadDemo('small-cap-tilt');  void compute(tok); });
    document.getElementById('c4-d3').addEventListener('click', () => { loadDemo('value-tilt');      void compute(tok); });
    document.getElementById('c4-d4').addEventListener('click', () => { loadDemo('momentum-tilt');  void compute(tok); });
    document.getElementById('c4-d5').addEventListener('click', () => { loadDemo('growth-tilt');     void compute(tok); });
    document.getElementById('c4-d6').addEventListener('click', () => { loadDemo('positive-alpha'); void compute(tok); });
    document.getElementById('c4-d7').addEventListener('click', () => { loadDemo('negative-alpha'); void compute(tok); });
    document.getElementById('c4-d8').addEventListener('click', () => { loadDemo('small-sample');    void compute(tok); });
    document.getElementById('c4-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseSeriesBlob(document.getElementById('c4-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.car4.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.portfolio_returns = p.portfolio_returns;
    state.market_excess = p.market_excess;
    state.smb = p.smb;
    state.hml = p.hml;
    state.wml = p.wml;
    state.risk_free = p.risk_free;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state);
    if (!local) { showErr(t('view.car4.err.degenerate')); return; }
    renderSummary(local, true);
    renderFactors(local);
    renderStats();
    renderFactorChart(local);
    let resp;
    try {
        resp = await api.anlyCarhart4(buildBody(state));
    } catch (e) {
        showErr(`${t('view.car4.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) { showErr(t('view.car4.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderFactors(resp);
    renderStats();
    renderFactorChart(resp);
}

function renderSummary(report, pending) {
    const local = localCompute(state);
    const parityOk = !!local
        && near(local.alpha, report.alpha) && near(local.beta_mkt, report.beta_mkt)
        && near(local.beta_smb, report.beta_smb) && near(local.beta_hml, report.beta_hml)
        && near(local.beta_wml, report.beta_wml) && near(local.r_squared, report.r_squared)
        && local.n_observations === report.n_observations;
    const aBadge = alphaBadge(report);
    const sBadge = styleBadge(report);
    const fBadge = fitBadge(report.r_squared);
    const mBadge = marketBetaBadge(report.beta_mkt);
    const localTag = pending ? ` (${t('view.car4.tag.local')})` : '';
    document.getElementById('c4-summary').innerHTML = [
        card(t('view.car4.card.alpha_v'),  t(aBadge.key) + localTag, aBadge.cls),
        card(t('view.car4.card.style'),    t(sBadge.key), sBadge.cls),
        card(t('view.car4.card.fit'),      t(fBadge.key), fBadge.cls),
        card(t('view.car4.card.mkt_class'), t(mBadge.key), mBadge.cls),
        card(t('view.car4.card.alpha'),    fmtPct(report.alpha, 4),
             report.alpha > 0 ? 'pos' : report.alpha < 0 ? 'neg' : ''),
        card(t('view.car4.card.alpha_t'),  fmtTStat(report.alpha_tstat),
             Math.abs(report.alpha_tstat) >= 1.96 ? (report.alpha > 0 ? 'pos' : 'neg') : ''),
        card(t('view.car4.card.r2'),       fmtPct(report.r_squared, 2)),
        card(t('view.car4.card.n'),        fmtInt(report.n_observations)),
        card(t('view.car4.card.parity'),
             parityOk ? t('view.car4.tag.ok') : t('view.car4.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderFactors(report) {
    const rows = [
        ['view.car4.row.alpha', report.alpha,    report.alpha_se,    report.alpha_tstat],
        ['view.car4.row.mkt',   report.beta_mkt, report.beta_mkt_se, tstat(report.beta_mkt, report.beta_mkt_se)],
        ['view.car4.row.smb',   report.beta_smb, report.beta_smb_se, tstat(report.beta_smb, report.beta_smb_se)],
        ['view.car4.row.hml',   report.beta_hml, report.beta_hml_se, tstat(report.beta_hml, report.beta_hml_se)],
        ['view.car4.row.wml',   report.beta_wml, report.beta_wml_se, tstat(report.beta_wml, report.beta_wml_se)],
    ];
    const html = rows.map(([key, b, se, ts]) => {
        const sig = Math.abs(ts) >= 1.96;
        const cls = sig ? (b > 0 ? 'pos' : 'neg') : '';
        return `<tr>
            <td data-i18n="${key}">${esc(key)}</td>
            <td class="${cls}">${esc(fmtBetaSigned(b))}</td>
            <td>${esc(fmtBeta(se))}</td>
            <td class="${cls}">${esc(fmtTStat(ts))}</td>
            <td>${sig ? esc(t('view.car4.tag.sig')) : esc(t('view.car4.tag.ns'))}</td>
        </tr>`;
    }).join('');
    document.getElementById('c4-factors').innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.car4.col.factor">Factor</th>
                <th data-i18n="view.car4.col.coef">Coefficient</th>
                <th data-i18n="view.car4.col.se">SE</th>
                <th data-i18n="view.car4.col.tstat">t-stat</th>
                <th data-i18n="view.car4.col.sig">Significance</th>
            </tr></thead>
            <tbody>${html}</tbody>
        </table>
    `;
}

function renderStats() {
    const wrap = document.getElementById('c4-stats');
    if (!state.portfolio_returns.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.car4.empty">${esc(t('view.car4.empty'))}</div>`;
        return;
    }
    const s = summarizeSeries(state);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.car4.col.metric">Metric</th>
                <th data-i18n="view.car4.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.car4.row.n_in">Observations (input)</td><td>${fmtInt(s.n)}</td></tr>
                <tr><td data-i18n="view.car4.row.mean_p">Mean portfolio return</td><td>${esc(fmtPct(s.mean_p))}</td></tr>
                <tr><td data-i18n="view.car4.row.mean_m">Mean market excess</td><td>${esc(fmtPct(s.mean_m))}</td></tr>
                <tr><td data-i18n="view.car4.row.mean_rf">Mean risk-free</td>    <td>${esc(fmtPct(s.mean_rf))}</td></tr>
            </tbody>
        </table>
    `;
}

function tstat(b, se) {
    if (!Number.isFinite(b) || !Number.isFinite(se) || se <= 0) return 0;
    return b / se;
}

function near(a, b, tol = 1e-6) {
    if (!Number.isFinite(a) || !Number.isFinite(b)) return false;
    return Math.abs(a - b) < tol;
}

function renderFactorChart(report) {
    const el = document.getElementById('c4-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!report) {
        el.innerHTML = `<div class="muted" data-i18n="view.car4.empty_chart">${esc(t('view.car4.empty_chart'))}</div>`;
        return;
    }
    const labels = [
        t('view.car4.row.alpha'),
        t('view.car4.row.mkt'),
        t('view.car4.row.smb'),
        t('view.car4.row.hml'),
        t('view.car4.row.wml'),
    ];
    const coefs = [
        Number.isFinite(report.alpha) ? report.alpha : null,
        Number.isFinite(report.beta_mkt) ? report.beta_mkt : null,
        Number.isFinite(report.beta_smb) ? report.beta_smb : null,
        Number.isFinite(report.beta_hml) ? report.beta_hml : null,
        Number.isFinite(report.beta_wml) ? report.beta_wml : null,
    ];
    const xs = labels.map((_, i) => i + 1);
    const zeroLine = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.car4.chart.factor_idx') },
            { label: t('view.car4.chart.coef'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.car4.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, coefs, zeroLine], el);
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('c4-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('c4-err').style.display = 'none'; }
