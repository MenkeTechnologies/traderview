// Currency-exposure aggregator view. Aggregates per-position notional by
// underlying CURRENCY, converts to home currency via supplied FX rates,
// flags overweight currencies (> 25% non-home gross).
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parsePositionBlob, parseFxBlob, validateInputs, buildBody, localAnalyze,
    concentrationBadge, makeDemoPositions, makeDemoFx, ccyColor,
    fmtUSD, fmtUSDSigned, fmtPct, fmtNum, fmtRate,
} from '../_currency_exposure_inputs.js';

let state = {
    positions: makeDemoPositions('multi-region'),
    home_currency: 'USD',
    fx: makeDemoFx('multi-region'),
};

export async function renderCurrencyExposure(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.currency_exposure.h1.title" class="view-title">// CURRENCY EXPOSURE</h1>

        <div class="chart-panel" data-context-scope="currency-exposure">
            <h2 data-i18n="view.currency_exposure.h2.positions">Foreign positions
                <small data-i18n="view.currency_exposure.h2.positions_hint" class="muted">(per line: SYMBOL CCY notional_in_native; negative = short)</small></h2>
            <textarea id="ce-pos" rows="6"
                      data-tip="view.currency_exposure.tip.positions"
                      placeholder="AAPL USD 30000&#10;SAP EUR 20000&#10;SONY JPY 1500000">${esc(positionsToBlob(state.positions))}</textarea>

            <h2 data-i18n="view.currency_exposure.h2.fx">FX rates → home
                <small data-i18n="view.currency_exposure.h2.fx_hint" class="muted">(per line: CCY rate; rate = how many home-units = 1 foreign-unit)</small></h2>
            <textarea id="ce-fx" rows="6"
                      data-tip="view.currency_exposure.tip.fx"
                      placeholder="EUR 1.10&#10;GBP 1.27&#10;JPY 0.0064">${esc(fxToBlob(state.fx))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.currency_exposure.label.home">Home currency</span>
                    <input id="ce-home" type="text" maxlength="5" value="${esc(state.home_currency)}"
                           style="text-transform:uppercase;width:80px" data-tip="view.currency_exposure.tip.home"></label>
                <button data-i18n="view.currency_exposure.btn.analyze" id="ce-run" class="primary"
                        data-tip="view.currency_exposure.tip.analyze" data-shortcut="currency_exposure_run" type="button">Analyze</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.currency_exposure.btn.demo_multi" id="ce-demo-multi"  class="secondary" type="button" data-tip="view.currency_exposure.tip.demo_multi">Demo: multi-region (4 ccys)</button>
                <button data-i18n="view.currency_exposure.btn.demo_eur"   id="ce-demo-eur"    class="secondary" type="button" data-tip="view.currency_exposure.tip.demo_eur">Demo: EUR concentrated</button>
                <button data-i18n="view.currency_exposure.btn.demo_short" id="ce-demo-short"  class="secondary" type="button" data-tip="view.currency_exposure.tip.demo_short">Demo: short hedged EUR</button>
                <button data-i18n="view.currency_exposure.btn.demo_home"  id="ce-demo-home"   class="secondary" type="button" data-tip="view.currency_exposure.tip.demo_home">Demo: home only (no FX)</button>
                <button data-i18n="view.currency_exposure.btn.demo_miss"  id="ce-demo-miss"   class="secondary" type="button" data-tip="view.currency_exposure.tip.demo_miss">Demo: missing FX rate (CAD)</button>
            </div>
            <p data-i18n="view.currency_exposure.hint.about" class="muted">Home currency gets rate 1.0. Missing FX → 0 exposure (defensive; warn in chart). Overweight = currency > 25% of total home-gross AND not the home currency. Buckets sorted by gross_home DESC.</p>
        </div>

        <div id="ce-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.currency_exposure.h2.buckets">Per-currency buckets</h2>
            <div id="ce-table"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.currency_exposure.h2.exposure_chart">Gross exposure per currency (home currency)</h2>
            <div id="ce-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.currency_exposure.h2.net_chart">Net exposure per currency — directional FX bet (long vs short)</h2>
            <div id="ce-net-chart" style="width:100%;height:220px"></div>
        </div>

        <div id="ce-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state.positions     = makeDemoPositions(k);
        state.fx            = makeDemoFx(k);
        state.home_currency = 'USD';
        document.getElementById('ce-pos').value  = positionsToBlob(state.positions);
        document.getElementById('ce-fx').value   = fxToBlob(state.fx);
        document.getElementById('ce-home').value = state.home_currency;
    };
    document.getElementById('ce-demo-multi').addEventListener('click', () => loadDemo('multi-region'));
    document.getElementById('ce-demo-eur').addEventListener('click',   () => loadDemo('eur-concentrated'));
    document.getElementById('ce-demo-short').addEventListener('click', () => loadDemo('short-hedged'));
    document.getElementById('ce-demo-home').addEventListener('click',  () => loadDemo('home-only'));
    document.getElementById('ce-demo-miss').addEventListener('click',  () => loadDemo('missing-fx'));
    document.getElementById('ce-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function positionsToBlob(positions) {
    return positions.map(p => `${p.symbol} ${p.currency} ${p.notional_native}`).join('\n');
}

function fxToBlob(fx) {
    return Object.entries(fx).map(([k, v]) => `${k} ${v}`).join('\n');
}

function readInputs() {
    const pp = parsePositionBlob(document.getElementById('ce-pos').value);
    const pf = parseFxBlob(document.getElementById('ce-fx').value);
    const errs = [
        ...pp.errors.map(e => `pos[${e.line_no}] ${e.message}`),
        ...pf.errors.map(e => `fx[${e.line_no}] ${e.message}`),
    ];
    if (errs.length) {
        showErr(errs.slice(0, 4).join('; '));
        showToast(t('view.currency_exposure.toast.parse_error', { n: errs.length }), { level: 'warning' });
        return;
    }
    hideErr();
    state.positions     = pp.positions;
    state.fx            = pf.fx;
    state.home_currency = (document.getElementById('ce-home').value || 'USD').toUpperCase();
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.positions, state.home_currency, state.fx);
    if (err) { showErr(err); showToast(t('view.currency_exposure.toast.invalid'), { level: 'warning' }); return; }
    const local = localAnalyze(state.positions, state.home_currency, state.fx);
    renderSummary(local, true);
    renderTable(local);
    renderExposureChart(local);
    renderNetChart(local);
    let resp;
    try {
        resp = await api.calcCurrencyExposure(buildBody(
            state.positions, state.home_currency, state.fx));
    } catch (e) {
        showErr(`${t('view.currency_exposure.err.api')}: ${e.message || e}`);
        showToast(t('view.currency_exposure.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderTable(resp);
    renderExposureChart(resp);
    renderNetChart(resp);
    const buckets = (resp.buckets || []).length;
    const overweight = (resp.buckets || []).filter(b => b.overweight).length;
    const level = overweight > 0 ? 'warning' : 'success';
    showToast(t('view.currency_exposure.toast.analyzed', { n: buckets, ow: overweight }), { level });
}

function renderSummary(report, pending) {
    const badge = concentrationBadge(report, state.home_currency);
    const local = localAnalyze(state.positions, state.home_currency, state.fx);
    const parityOk = Math.abs(report.total_gross_home - local.total_gross_home) < 1e-6
                  && report.buckets.length === local.buckets.length
                  && report.overweight_currencies.length === local.overweight_currencies.length;
    const localTag = pending ? ` (${t('view.currency_exposure.tag.local')})` : '';
    const missingCount = report.buckets.filter(b => b.currency !== state.home_currency
        && state.fx[b.currency] == null).length;
    document.getElementById('ce-summary').innerHTML = [
        card(t('view.currency_exposure.card.concentration'),
             t(badge.key) + localTag, badge.cls),
        card(t('view.currency_exposure.card.home'),
             state.home_currency),
        card(t('view.currency_exposure.card.gross_home'),
             fmtUSD(report.total_gross_home)),
        card(t('view.currency_exposure.card.net_home'),
             fmtUSDSigned(report.total_net_home),
             report.total_net_home >= 0 ? 'pos' : 'neg'),
        card(t('view.currency_exposure.card.currencies'),
             String(report.buckets.length)),
        card(t('view.currency_exposure.card.overweight'),
             report.overweight_currencies.length === 0
                ? t('view.currency_exposure.tag.none')
                : report.overweight_currencies.join(', '),
             report.overweight_currencies.length > 0 ? 'neg' : 'pos'),
        card(t('view.currency_exposure.card.missing_fx'),
             String(missingCount),
             missingCount > 0 ? 'neg' : 'pos'),
        card(t('view.currency_exposure.card.parity'),
             parityOk ? t('view.currency_exposure.tag.ok') : t('view.currency_exposure.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderTable(report) {
    const wrap = document.getElementById('ce-table');
    if (!report.buckets || report.buckets.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.currency_exposure.empty">${esc(t('view.currency_exposure.empty'))}</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.currency_exposure.col.currency">Currency</th>
                <th data-i18n="view.currency_exposure.col.positions">Positions</th>
                <th data-i18n="view.currency_exposure.col.gross_native">Gross (native)</th>
                <th data-i18n="view.currency_exposure.col.net_native">Net (native)</th>
                <th data-i18n="view.currency_exposure.col.fx">FX → home</th>
                <th data-i18n="view.currency_exposure.col.gross_home">Gross (home)</th>
                <th data-i18n="view.currency_exposure.col.net_home">Net (home)</th>
                <th data-i18n="view.currency_exposure.col.pct">% of gross</th>
                <th data-i18n="view.currency_exposure.col.flag">Flag</th>
            </tr></thead>
            <tbody>
                ${report.buckets.map((b, i) => {
                    const isHome = b.currency === state.home_currency;
                    const rate = isHome ? 1.0 : (state.fx[b.currency] ?? 0);
                    const missing = !isHome && state.fx[b.currency] == null;
                    const overweight = report.overweight_currencies.includes(b.currency);
                    return `<tr>
                        <td><span style="color:${esc(ccyColor(i))};font-weight:bold">●</span> <strong>${esc(b.currency)}</strong>${isHome ? ' <span class="muted">(home)</span>' : ''}</td>
                        <td>${b.position_count}</td>
                        <td>${esc(fmtNum(b.gross_native, 2))}</td>
                        <td class="${b.net_native >= 0 ? 'pos' : 'neg'}">${esc(fmtUSDSigned(b.net_native).replace('$', ''))}</td>
                        <td class="${missing ? 'neg' : ''}">${esc(fmtRate(rate))}${missing ? ' ⚠' : ''}</td>
                        <td>${esc(fmtUSD(b.gross_home))}</td>
                        <td class="${b.net_home >= 0 ? 'pos' : 'neg'}">${esc(fmtUSDSigned(b.net_home))}</td>
                        <td class="${b.pct_of_total > 0.25 ? 'neg' : ''}">${esc(fmtPct(b.pct_of_total))}</td>
                        <td class="${overweight ? 'neg' : ''}">${overweight ? '⚠ OVERWEIGHT' : isHome ? t('view.currency_exposure.tag.home') : '·'}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
}

function renderExposureChart(report) {
    const el = document.getElementById('ce-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!report || !Array.isArray(report.buckets) || report.buckets.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.currency_exposure.empty_chart">${esc(t('view.currency_exposure.empty_chart'))}</div>`;
        return;
    }
    const labels = report.buckets.map(b => b.currency);
    const gross = report.buckets.map(b => Number.isFinite(b.gross_home) ? b.gross_home : null);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.currency_exposure.chart.ccy_idx') },
            { label: t('view.currency_exposure.chart.gross_home'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, gross], el);
}

function renderNetChart(report) {
    const el = document.getElementById('ce-net-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!report || !Array.isArray(report.buckets) || report.buckets.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.currency_exposure.empty_net_chart">${esc(t('view.currency_exposure.empty_net_chart'))}</div>`;
        return;
    }
    const labels = report.buckets.map(b => b.currency);
    const net = report.buckets.map(b => Number.isFinite(b.net_home) ? b.net_home : null);
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.currency_exposure.chart.ccy_idx') },
            { label: t('view.currency_exposure.chart.net_home'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 12, fill: '#b86bff', stroke: '#b86bff' } },
            { label: t('view.currency_exposure.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, net, zero], el);
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('ce-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ce-err').style.display = 'none'; }
