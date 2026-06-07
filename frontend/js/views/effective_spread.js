// Effective + realized spread view — Lee-Ready / Bessembinder TCA.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseObsBlob, obsToBlob, validateInputs, buildBody, localAnalyze,
    executionBadge, adverseBadge, enrich,
    makeDemoInput,
    fmtUSD, fmtUSDSigned, fmtBps, fmtRatio, fmtInt, dirLabelKey,
} from '../_effective_spread_inputs.js';

let state = { ...makeDemoInput('at-quote') };

export async function renderEffectiveSpread(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.eff_spread.h1.title" class="view-title">// EFFECTIVE SPREAD</h1>

        <div class="chart-panel" data-context-scope="eff-spread">
            <h2 data-i18n="view.eff_spread.h2.obs">Observations
                <small data-i18n="view.eff_spread.h2.obs_hint" class="muted">(per line: trade_price current_mid delayed_mid quoted_spread direction)</small></h2>
            <textarea id="es-blob" rows="8"
                      data-tip="view.eff_spread.tip.obs"
                      placeholder="100.05 100.00 100.00 0.10 buy&#10;99.95 100.00 100.00 0.10 sell">${esc(obsToBlob(state.observations))}</textarea>

            <div class="inline-form">
                <button data-i18n="view.eff_spread.btn.compute" id="es-run" class="primary"
                        data-tip="view.eff_spread.tip.compute" data-shortcut="effective_spread_run" type="button">Analyze</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.eff_spread.btn.demo_at_quote"   id="es-demo-atq"     class="secondary" type="button" data-tip="view.eff_spread.tip.demo_atq">Demo: at-quote trades</button>
                <button data-i18n="view.eff_spread.btn.demo_improve"    id="es-demo-imp"     class="secondary" type="button" data-tip="view.eff_spread.tip.demo_imp">Demo: price improvement</button>
                <button data-i18n="view.eff_spread.btn.demo_adverse"    id="es-demo-adv"     class="secondary" type="button" data-tip="view.eff_spread.tip.demo_adv">Demo: adverse selection (informed flow)</button>
                <button data-i18n="view.eff_spread.btn.demo_lp_wins"    id="es-demo-lpw"     class="secondary" type="button" data-tip="view.eff_spread.tip.demo_lpw">Demo: LP wins (uninformed)</button>
                <button data-i18n="view.eff_spread.btn.demo_trade_thru" id="es-demo-tt"      class="secondary" type="button" data-tip="view.eff_spread.tip.demo_tt">Demo: trade-through</button>
                <button data-i18n="view.eff_spread.btn.demo_mixed"      id="es-demo-mix"     class="secondary" type="button" data-tip="view.eff_spread.tip.demo_mix">Demo: mixed quality</button>
                <button data-i18n="view.eff_spread.btn.demo_tight"      id="es-demo-tight"   class="secondary" type="button" data-tip="view.eff_spread.tip.demo_tight">Demo: penny-spread market</button>
                <button data-i18n="view.eff_spread.btn.demo_wide"       id="es-demo-wide"    class="secondary" type="button" data-tip="view.eff_spread.tip.demo_wide">Demo: wide-spread (50¢)</button>
            </div>
            <p data-i18n="view.eff_spread.hint.about" class="muted">effective = 2·D·(trade − mid). realized = 2·D·(trade − mid_delayed). impact = effective − realized. eff/quoted ratio: &lt;1 = price improvement, &gt;1 = trade-through. D=+1 buy, −1 sell.</p>
        </div>

        <div id="es-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.eff_spread.h2.table">Per-observation breakdown</h2>
            <div id="es-table"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.eff_spread.h2.spread_chart">Effective vs realized spread per observation</h2>
            <div id="es-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.eff_spread.h2.ratio_chart">Effective/quoted ratio per observation — &lt;1 = price improvement, &gt;1 = trade-through</h2>
            <div id="es-ratio-chart" style="width:100%;height:220px"></div>
        </div>

        <div id="es-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('es-blob').value = obsToBlob(state.observations);
    };
    document.getElementById('es-demo-atq').addEventListener('click',   () => { loadDemo('at-quote');           void compute(tok); });
    document.getElementById('es-demo-imp').addEventListener('click',   () => { loadDemo('price-improvement'); void compute(tok); });
    document.getElementById('es-demo-adv').addEventListener('click',   () => { loadDemo('adverse-selection'); void compute(tok); });
    document.getElementById('es-demo-lpw').addEventListener('click',   () => { loadDemo('lp-wins');           void compute(tok); });
    document.getElementById('es-demo-tt').addEventListener('click',    () => { loadDemo('trade-through');     void compute(tok); });
    document.getElementById('es-demo-mix').addEventListener('click',   () => { loadDemo('mixed-quality');     void compute(tok); });
    document.getElementById('es-demo-tight').addEventListener('click', () => { loadDemo('tight-market');      void compute(tok); });
    document.getElementById('es-demo-wide').addEventListener('click',  () => { loadDemo('large-tick');        void compute(tok); });
    document.getElementById('es-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseObsBlob(document.getElementById('es-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.eff_spread.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        showToast(t('view.eff_spread.toast.parse_error', { n: p.errors.length }), { level: 'warning' });
        return;
    }
    hideErr();
    state.observations = p.observations;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.eff_spread.toast.invalid'), { level: 'warning' }); return; }
    const local = localAnalyze(state.observations);
    if (!local) {
        showErr(t('view.eff_spread.err.degenerate'));
        showToast(t('view.eff_spread.toast.degenerate'), { level: 'warning' });
        return;
    }
    renderSummary(local, true);
    renderTable();
    renderSpreadChart();
    renderRatioChart();
    let resp;
    try {
        resp = await api.microEffectiveSpread(buildBody(state));
    } catch (e) {
        showErr(`${t('view.eff_spread.err.api')}: ${e.message || e}`);
        showToast(t('view.eff_spread.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) {
        showErr(t('view.eff_spread.err.server_rejected'));
        showToast(t('view.eff_spread.toast.rejected'), { level: 'error' });
        return;
    }
    renderSummary(resp, false);
    renderTable();
    renderSpreadChart();
    renderRatioChart();
    const refPrice = state.observations.length > 0 ? state.observations[0].current_mid : NaN;
    const effBps = Number.isFinite(refPrice) && refPrice > 0
        ? (Number(resp.avg_effective_spread) / refPrice) * 10000
        : NaN;
    const effBpsStr = Number.isFinite(effBps) ? effBps.toFixed(2) : '—';
    const realized = Number(resp.avg_realized_spread) || 0;
    const level = realized < 0 ? 'warning' : 'success';
    showToast(t('view.eff_spread.toast.analyzed', {
        n: resp.n_observations | 0,
        eff: effBpsStr,
        realized: realized.toFixed(4),
    }), { level });
}

function renderSummary(report, pending) {
    const local = localAnalyze(state.observations);
    const parityOk = !!local
        && Math.abs(local.avg_effective_spread - report.avg_effective_spread) < 1e-9
        && Math.abs(local.avg_realized_spread - report.avg_realized_spread)   < 1e-9
        && local.n_observations === report.n_observations;
    const exBadge = executionBadge(report);
    const adBadge = adverseBadge(report);
    const localTag = pending ? ` (${t('view.eff_spread.tag.local')})` : '';
    const refPrice = state.observations.length > 0 ? state.observations[0].current_mid : NaN;
    document.getElementById('es-summary').innerHTML = [
        card(t('view.eff_spread.card.exec'),       t(exBadge.key) + localTag, exBadge.cls),
        card(t('view.eff_spread.card.adverse'),    t(adBadge.key), adBadge.cls),
        card(t('view.eff_spread.card.n'),          fmtInt(report.n_observations)),
        card(t('view.eff_spread.card.quoted'),     fmtUSD(report.avg_quoted_spread)),
        card(t('view.eff_spread.card.quoted_bps'), fmtBps(report.avg_quoted_spread, refPrice)),
        card(t('view.eff_spread.card.effective'),  fmtUSD(report.avg_effective_spread)),
        card(t('view.eff_spread.card.effective_bps'), fmtBps(report.avg_effective_spread, refPrice)),
        card(t('view.eff_spread.card.realized'),   fmtUSDSigned(report.avg_realized_spread),
             report.avg_realized_spread > 0 ? 'pos' : report.avg_realized_spread < 0 ? 'neg' : ''),
        card(t('view.eff_spread.card.impact'),     fmtUSDSigned(report.avg_price_impact),
             report.avg_price_impact > 0 ? 'neg' : report.avg_price_impact < 0 ? 'pos' : ''),
        card(t('view.eff_spread.card.ratio'),      fmtRatio(report.effective_to_quoted_ratio),
             report.effective_to_quoted_ratio < 1 ? 'pos' : report.effective_to_quoted_ratio > 1.05 ? 'neg' : ''),
        card(t('view.eff_spread.card.parity'),
             parityOk ? t('view.eff_spread.tag.ok') : t('view.eff_spread.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderTable() {
    const wrap = document.getElementById('es-table');
    if (!state.observations || state.observations.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.eff_spread.empty">${esc(t('view.eff_spread.empty'))}</div>`;
        return;
    }
    const enriched = state.observations.map(enrich);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.eff_spread.col.idx">#</th>
                <th data-i18n="view.eff_spread.col.dir">Dir</th>
                <th data-i18n="view.eff_spread.col.trade">Trade</th>
                <th data-i18n="view.eff_spread.col.mid">Mid</th>
                <th data-i18n="view.eff_spread.col.mid_delayed">Mid (delayed)</th>
                <th data-i18n="view.eff_spread.col.quoted">Quoted</th>
                <th data-i18n="view.eff_spread.col.effective">Effective</th>
                <th data-i18n="view.eff_spread.col.realized">Realized</th>
                <th data-i18n="view.eff_spread.col.impact">Impact</th>
            </tr></thead>
            <tbody>
                ${enriched.map((o, i) => {
                    const dirCls = o.direction === 'buy' ? 'pos' : 'neg';
                    const impactCls = o.price_impact > 0 ? 'neg' : o.price_impact < 0 ? 'pos' : '';
                    const realizedCls = o.realized_spread > 0 ? 'pos' : o.realized_spread < 0 ? 'neg' : '';
                    return `<tr>
                        <td>${i + 1}</td>
                        <td data-i18n="${esc(dirLabelKey(o.direction))}" class="${dirCls}">${esc(t(dirLabelKey(o.direction)))}</td>
                        <td>${esc(fmtUSD(o.trade_price, 4))}</td>
                        <td>${esc(fmtUSD(o.current_mid, 4))}</td>
                        <td>${esc(fmtUSD(o.delayed_mid, 4))}</td>
                        <td>${esc(fmtUSD(o.quoted_spread, 4))}</td>
                        <td>${esc(fmtUSD(o.effective_spread, 4))}</td>
                        <td class="${realizedCls}">${esc(fmtUSDSigned(o.realized_spread, 4))}</td>
                        <td class="${impactCls}">${esc(fmtUSDSigned(o.price_impact, 4))}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
}

function renderSpreadChart() {
    const el = document.getElementById('es-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const obs = Array.isArray(state.observations) ? state.observations : [];
    if (obs.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.eff_spread.empty_chart">${esc(t('view.eff_spread.empty_chart'))}</div>`;
        return;
    }
    const enriched = obs.map(enrich);
    const xs = enriched.map((_, i) => i + 1);
    const effective = enriched.map(o => Number.isFinite(o.effective_spread) ? o.effective_spread : null);
    const realized = enriched.map(o => Number.isFinite(o.realized_spread) ? o.realized_spread : null);
    const impact = enriched.map(o => Number.isFinite(o.price_impact) ? o.price_impact : null);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.eff_spread.chart.obs_idx') },
            { label: t('view.eff_spread.chart.effective'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 8, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.eff_spread.chart.realized'),
              stroke: '#ff9f1a', width: 0,
              points: { show: true, size: 8, fill: '#ff9f1a', stroke: '#ff9f1a' } },
            { label: t('view.eff_spread.chart.impact'),
              stroke: '#a06bff', width: 0,
              points: { show: true, size: 8, fill: '#a06bff', stroke: '#a06bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, effective, realized, impact], el);
}

function renderRatioChart() {
    const el = document.getElementById('es-ratio-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const obs = Array.isArray(state.observations) ? state.observations : [];
    if (obs.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.eff_spread.empty_ratio_chart">${esc(t('view.eff_spread.empty_ratio_chart'))}</div>`;
        return;
    }
    const enriched = obs.map(enrich);
    const xs = enriched.map((_, i) => i + 1);
    const ratios = enriched.map(o => {
        if (!Number.isFinite(o.quoted_spread) || o.quoted_spread <= 0) return null;
        const r = o.effective_spread / o.quoted_spread;
        return Number.isFinite(r) ? r : null;
    });
    const one = xs.map(() => 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.eff_spread.chart.obs_idx') },
            { label: t('view.eff_spread.chart.eff_quoted_ratio'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 10, fill: '#b86bff', stroke: '#b86bff' } },
            { label: t('view.eff_spread.chart.at_quote'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [ { stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 } ],
        legend: { show: true },
    }, [xs, ratios, one], el);
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('es-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('es-err').style.display = 'none'; }
