// Risk-Parity allocator view. Naive inverse-volatility weighting: each
// asset contributes the SAME amount of portfolio variance. Backend
// /calc/risk-parity.
//
// All user-facing strings via i18n.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseAssetBlob, validateInputs, buildBody, localAllocate,
    equalWeightAllocation, riskContribDispersion, maxConcentration,
    makeDemoAssets, fmtPct, fmtVol, fmtNum, symbolColor,
} from '../_risk_parity_inputs.js';

let state = { assets: makeDemoAssets('classic-60-40') };

export async function renderRiskParity(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.risk_parity.h1.title" class="view-title">// RISK PARITY</h1>

        <div class="chart-panel" data-context-scope="risk-parity">
            <h2 data-i18n="view.risk_parity.h2.assets">Assets
                <small data-i18n="view.risk_parity.h2.assets_hint" class="muted">(per line: SYMBOL vol; %-suffix OK)</small></h2>
            <textarea id="rp-blob" rows="6"
                      data-tip="view.risk_parity.tip.blob"
                      placeholder="SPY 15%&#10;AGG 5%&#10;GLD 16%">${esc(assetsToBlob(state.assets))}</textarea>
            <div class="inline-form">
                <button data-i18n="view.risk_parity.btn.allocate" id="rp-run" class="primary"
                        data-tip="view.risk_parity.tip.allocate" type="button">Allocate</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.risk_parity.btn.demo_6040"    id="rp-demo-6040"  class="secondary" type="button">Demo: classic 60/40</button>
                <button data-i18n="view.risk_parity.btn.demo_five"    id="rp-demo-five"  class="secondary" type="button">Demo: 5-asset diversified</button>
                <button data-i18n="view.risk_parity.btn.demo_equal"   id="rp-demo-equal" class="secondary" type="button">Demo: equal vol</button>
                <button data-i18n="view.risk_parity.btn.demo_extreme" id="rp-demo-extr"  class="secondary" type="button">Demo: extreme vol skew</button>
                <button data-i18n="view.risk_parity.btn.demo_single"  id="rp-demo-one"   class="secondary" type="button">Demo: single asset</button>
                <button data-i18n="view.risk_parity.btn.demo_zero"    id="rp-demo-zero"  class="secondary" type="button">Demo: cash + risk assets</button>
            </div>
            <p data-i18n="view.risk_parity.hint.about" class="muted">Naive risk-parity: weight ∝ 1/σ, normalized to 1. Each asset contributes the same dollar variance — low-vol assets get a larger weight. Assumes uncorrelated returns; for real correlation matrices use HRP (coming soon).</p>
        </div>

        <div id="rp-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.risk_parity.h2.weights">Risk-parity vs equal-weight</h2>
            <div id="rp-bars"></div>
            <p data-i18n="view.risk_parity.hint.weights" class="muted">Solid bars = risk-parity weight. Hollow = naive 1/N equal weight. The wider the gap, the more vol dispersion in the basket.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.risk_parity.h2.table">Per-asset detail</h2>
            <div id="rp-table"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.risk_parity.h2.weights_chart">RP vs EW weight per asset</h2>
            <div id="rp-chart" style="width:100%;height:220px"></div>
        </div>

        <div id="rp-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state.assets = makeDemoAssets(k);
        document.getElementById('rp-blob').value = assetsToBlob(state.assets);
    };
    document.getElementById('rp-demo-6040').addEventListener('click',  () => loadDemo('classic-60-40'));
    document.getElementById('rp-demo-five').addEventListener('click',  () => loadDemo('five-asset'));
    document.getElementById('rp-demo-equal').addEventListener('click', () => loadDemo('equal-vol'));
    document.getElementById('rp-demo-extr').addEventListener('click',  () => loadDemo('extreme-vol'));
    document.getElementById('rp-demo-one').addEventListener('click',   () => loadDemo('single'));
    document.getElementById('rp-demo-zero').addEventListener('click',  () => loadDemo('zero-vol-mixed'));
    document.getElementById('rp-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function assetsToBlob(assets) {
    return assets.map(a => `${a.symbol} ${(a.vol * 100).toFixed(2)}%`).join('\n');
}

function readInputs() {
    const parsed = parseAssetBlob(document.getElementById('rp-blob').value);
    if (parsed.errors.length) {
        showErr(`${t('view.risk_parity.err.parse_prefix')}: `
            + parsed.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.assets = parsed.assets;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.assets);
    if (err) { showErr(err); return; }
    const local = localAllocate(state.assets);
    renderSummary(local, true);
    renderBars(local);
    renderTable(local);
    renderWeightsChart(local);
    let resp;
    try {
        resp = await api.calcRiskParity(buildBody(state.assets));
    } catch (e) {
        showErr(`${t('view.risk_parity.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderBars(resp);
    renderTable(resp);
    renderWeightsChart(resp);
}

function renderWeightsChart(report) {
    const el = document.getElementById('rp-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (report?.allocations || []).filter(a => Number.isFinite(Number(a.weight)));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.risk_parity.empty_chart">${esc(t('view.risk_parity.empty_chart'))}</div>`;
        return;
    }
    const eq = equalWeightAllocation(state.assets);
    const eqByVid = new Map(eq.map(a => [a.symbol, a.weight]));
    rows.sort((a, b) => Number(b.weight) - Number(a.weight));
    const labels = rows.map(a => a.symbol);
    const xs = labels.map((_, i) => i + 1);
    const rpY = rows.map(a => Number(a.weight) * 100);
    const ewY = rows.map(a => Number(eqByVid.get(a.symbol) || 0) * 100);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.risk_parity.chart.symbol') },
            { label: t('view.risk_parity.chart.rp_pct'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 14, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.risk_parity.chart.ew_pct'),
              stroke: '#ffd84a', width: 0,
              points: { show: true, size: 14, fill: '#ffd84a', stroke: '#ffd84a' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, rpY, ewY], el);
}

function renderSummary(report, pending) {
    const local = localAllocate(state.assets);
    const parity = reportEq(report, local);
    const dispersion = riskContribDispersion(report.allocations);
    const conc = maxConcentration(report.allocations);
    const isParity = dispersion < 1e-6;
    const localTag = pending ? ` (${t('view.risk_parity.tag.local')})` : '';
    document.getElementById('rp-summary').innerHTML = [
        card(t('view.risk_parity.card.verdict'),
             isParity ? t('view.risk_parity.tag.equal_risk') + localTag
                      : t('view.risk_parity.tag.uneven') + localTag,
             isParity ? 'pos' : 'neg'),
        card(t('view.risk_parity.card.assets'),       String(report.allocations.length)),
        card(t('view.risk_parity.card.total_weight'), fmtPct(report.total_weight),
             Math.abs(report.total_weight - 1) < 1e-9 || report.total_weight === 0 ? 'pos' : 'neg'),
        card(t('view.risk_parity.card.dispersion'),   fmtNum(dispersion, 9),
             isParity ? 'pos' : 'neg'),
        card(t('view.risk_parity.card.max_concentration'), fmtPct(conc),
             conc > 0.6 ? 'neg' : ''),
        card(t('view.risk_parity.card.equal_risk_contrib'),
             report.allocations.length > 0
               ? fmtNum(report.allocations[0].risk_contribution, 6)
               : '—'),
        card(t('view.risk_parity.card.parity'),
             parity ? t('view.risk_parity.tag.ok') : t('view.risk_parity.tag.diverged'),
             parity ? 'pos' : 'neg'),
    ].join('');
}

function reportEq(a, b) {
    if (!a || !b) return false;
    if (a.allocations.length !== b.allocations.length) return false;
    for (let i = 0; i < a.allocations.length; i++) {
        if (a.allocations[i].symbol !== b.allocations[i].symbol) return false;
        if (Math.abs(a.allocations[i].weight - b.allocations[i].weight) > 1e-9) return false;
    }
    return true;
}

function renderBars(report) {
    const wrap = document.getElementById('rp-bars');
    if (!report.allocations || report.allocations.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.risk_parity.empty">${esc(t('view.risk_parity.empty'))}</div>`;
        return;
    }
    const eq = equalWeightAllocation(state.assets);
    const eqByVid = new Map(eq.map(a => [a.symbol, a.weight]));
    const max = Math.max(...report.allocations.map(a => Math.max(a.weight, eqByVid.get(a.symbol) || 0)), 0.01);
    wrap.innerHTML = report.allocations.map((a, i) => {
        const rpPct = (a.weight / max) * 100;
        const ewPct = ((eqByVid.get(a.symbol) || 0) / max) * 100;
        const color = symbolColor(i);
        return `
            <div class="rp-bar-row" style="margin-bottom:10px">
                <div style="display:flex;justify-content:space-between;font-size:12px;margin-bottom:4px">
                    <span style="color:${esc(color)};font-weight:bold">●</span>
                    <strong>${esc(a.symbol)}</strong>
                    <span class="muted">${esc(fmtPct(a.weight))} RP / ${esc(fmtPct(eqByVid.get(a.symbol) || 0))} EW</span>
                </div>
                <div class="rp-bar-track" style="position:relative;height:14px;background:#1a1d22;border-radius:2px">
                    <div class="rp-bar-rp" data-pct="${rpPct.toFixed(2)}"
                         style="position:absolute;height:100%;border-radius:2px;background:${esc(color)};width:0;transition:width .2s"></div>
                    <div class="rp-bar-ew" data-pct="${ewPct.toFixed(2)}"
                         style="position:absolute;height:100%;border-radius:2px;background:transparent;border:1px dashed ${esc(color)};width:0;transition:width .2s"></div>
                </div>
            </div>
        `;
    }).join('');
    requestAnimationFrame(() => {
        wrap.querySelectorAll('.rp-bar-rp, .rp-bar-ew').forEach(el => {
            el.style.width = el.dataset.pct + '%';
        });
    });
}

function renderTable(report) {
    const wrap = document.getElementById('rp-table');
    if (!report.allocations || report.allocations.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.risk_parity.empty">${esc(t('view.risk_parity.empty'))}</div>`;
        return;
    }
    const eq = equalWeightAllocation(state.assets);
    const eqByVid = new Map(eq.map(a => [a.symbol, a]));
    const inputByVid = new Map(state.assets.map(a => [a.symbol, a]));
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.risk_parity.col.symbol">Symbol</th>
                <th data-i18n="view.risk_parity.col.vol">Volatility</th>
                <th data-i18n="view.risk_parity.col.rp_weight">RP weight</th>
                <th data-i18n="view.risk_parity.col.ew_weight">EW weight</th>
                <th data-i18n="view.risk_parity.col.rp_risk">RP risk contrib</th>
                <th data-i18n="view.risk_parity.col.ew_risk">EW risk contrib</th>
            </tr></thead>
            <tbody>
                ${report.allocations.map((a, i) => {
                    const e = eqByVid.get(a.symbol) || { weight: 0, risk_contribution: 0 };
                    const v = (inputByVid.get(a.symbol) || { vol: 0 }).vol;
                    return `<tr>
                        <td><span style="color:${esc(symbolColor(i))};font-weight:bold">●</span> <strong>${esc(a.symbol)}</strong></td>
                        <td>${esc(fmtVol(v))}</td>
                        <td><strong>${esc(fmtPct(a.weight))}</strong></td>
                        <td class="muted">${esc(fmtPct(e.weight))}</td>
                        <td>${esc(fmtNum(a.risk_contribution, 6))}</td>
                        <td class="muted">${esc(fmtNum(e.risk_contribution, 6))}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('rp-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('rp-err').style.display = 'none'; }
