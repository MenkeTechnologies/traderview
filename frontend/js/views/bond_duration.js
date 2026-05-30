// Bond duration view — Macaulay + Modified duration calculator with
// price-sensitivity grid.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseCashFlowBlob, validateInputs, buildBody, localCompute,
    priceChangePct, buildCouponBond, durationBadge, SENSITIVITY_BPS,
    makeDemoConfig, fmtUSD, fmtPctSigned, fmtPct, fmtYears, fmtBpsSigned,
} from '../_bond_duration_inputs.js';

let state = {
    ...makeDemoConfig('treasury-5yr-coupon'),
    builder: { par: 100, coupon_rate: 0.05, maturity_years: 5 },
};

export async function renderBondDuration(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.bond_duration.h1.title" class="view-title">// BOND DURATION</h1>

        <div class="chart-panel" data-context-scope="bond-duration">
            <h2 data-i18n="view.bond_duration.h2.builder">Quick bond builder</h2>
            <div class="inline-form">
                <label><span data-i18n="view.bond_duration.label.par">Par ($)</span>
                    <input id="bd-par" type="number" step="any" min="0" value="${state.builder.par}" data-tip="view.bond_duration.tip.par"></label>
                <label><span data-i18n="view.bond_duration.label.coupon">Coupon rate (decimal — 0.05 = 5%)</span>
                    <input id="bd-coupon" type="number" step="any" min="0" max="1" value="${state.builder.coupon_rate}" data-tip="view.bond_duration.tip.coupon"></label>
                <label><span data-i18n="view.bond_duration.label.maturity">Maturity (years)</span>
                    <input id="bd-maturity" type="number" step="any" min="0" value="${state.builder.maturity_years}" data-tip="view.bond_duration.tip.maturity"></label>
                <button data-i18n="view.bond_duration.btn.build_bond" id="bd-build" class="secondary"
                        data-tip="view.bond_duration.tip.build_bond" data-shortcut="bond_duration_build" type="button">Build coupon bond</button>
            </div>
            <p data-i18n="view.bond_duration.hint.builder" class="muted">Generates a standard coupon-bond cash-flow schedule and replaces the textarea below.</p>

            <h2 data-i18n="view.bond_duration.h2.cash_flows">Cash flows <small data-i18n="view.bond_duration.h2.cash_flows_hint" class="muted">(per line: time_years amount)</small></h2>
            <textarea id="bd-cf" rows="8"
                      data-tip="view.bond_duration.tip.cash_flows"
                      placeholder="1 5&#10;2 5&#10;3 5&#10;4 5&#10;5 105">${esc(cfsToBlob(state.cash_flows))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.bond_duration.label.ytm">YTM (decimal)</span>
                    <input id="bd-ytm" type="number" step="any" value="${state.ytm}" data-tip="view.bond_duration.tip.ytm"></label>
                <label><span data-i18n="view.bond_duration.label.compounding">Compounding / year</span>
                    <input id="bd-m" type="number" step="1" min="1" value="${state.compounding_per_year}" data-tip="view.bond_duration.tip.compounding"></label>
                <button data-i18n="view.bond_duration.btn.compute" id="bd-run" class="primary"
                        data-tip="view.bond_duration.tip.compute" data-shortcut="bond_duration_run" type="button">Compute duration</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.bond_duration.btn.demo_zero5"   id="bd-demo-zero"  class="secondary" type="button" data-tip="view.bond_duration.tip.demo_zero">Demo: zero-coupon 5yr</button>
                <button data-i18n="view.bond_duration.btn.demo_5yr"     id="bd-demo-5yr"   class="secondary" type="button" data-tip="view.bond_duration.tip.demo_5yr">Demo: 5% coupon 5yr (par)</button>
                <button data-i18n="view.bond_duration.btn.demo_10yr"    id="bd-demo-10yr"  class="secondary" type="button" data-tip="view.bond_duration.tip.demo_10yr">Demo: 4% × 10yr (semi)</button>
                <button data-i18n="view.bond_duration.btn.demo_30yr"    id="bd-demo-30yr"  class="secondary" type="button" data-tip="view.bond_duration.tip.demo_30yr">Demo: 4.5% × 30yr (semi)</button>
                <button data-i18n="view.bond_duration.btn.demo_premium" id="bd-demo-prem"  class="secondary" type="button" data-tip="view.bond_duration.tip.demo_prem">Demo: 8% × 7yr premium</button>
                <button data-i18n="view.bond_duration.btn.demo_tips"    id="bd-demo-tips"  class="secondary" type="button" data-tip="view.bond_duration.tip.demo_tips">Demo: low-rate 2yr zero</button>
            </div>
            <p data-i18n="view.bond_duration.hint.about" class="muted">Macaulay = weighted-avg time-to-CF. Modified = Macaulay / (1 + ytm/m). Rule of thumb: ΔP/P ≈ -ModDur × Δy. Compounding 2 = semi-annual (US Treasury convention); 1 = annual.</p>
        </div>

        <div id="bd-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.bond_duration.h2.sensitivity">Price sensitivity grid (modified-duration estimate)</h2>
            <div id="bd-sens"></div>
            <p data-i18n="view.bond_duration.hint.sensitivity" class="muted">Linear approximation only — for large yield moves convexity dominates. Good enough for ±100 bps shocks on intermediate paper.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.bond_duration.h2.cf_chart">Cash flows over time</h2>
            <div id="bd-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.bond_duration.h2.pv_chart">Discounted PV of each cash flow</h2>
            <div id="bd-pv-chart" style="width:100%;height:220px"></div>
        </div>

        <div id="bd-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        const cfg = makeDemoConfig(k);
        state.cash_flows = cfg.cash_flows;
        state.ytm = cfg.ytm;
        state.compounding_per_year = cfg.compounding_per_year;
        document.getElementById('bd-cf').value  = cfsToBlob(state.cash_flows);
        document.getElementById('bd-ytm').value = state.ytm;
        document.getElementById('bd-m').value   = state.compounding_per_year;
    };
    document.getElementById('bd-demo-zero').addEventListener('click',  () => loadDemo('zero-5yr'));
    document.getElementById('bd-demo-5yr').addEventListener('click',   () => loadDemo('treasury-5yr-coupon'));
    document.getElementById('bd-demo-10yr').addEventListener('click',  () => loadDemo('treasury-10yr-semi'));
    document.getElementById('bd-demo-30yr').addEventListener('click',  () => loadDemo('treasury-30yr-semi'));
    document.getElementById('bd-demo-prem').addEventListener('click',  () => loadDemo('corporate-7yr-high-coupon'));
    document.getElementById('bd-demo-tips').addEventListener('click',  () => loadDemo('tips-zero-2yr'));
    document.getElementById('bd-build').addEventListener('click', () => {
        state.builder.par           = Number(document.getElementById('bd-par').value);
        state.builder.coupon_rate   = Number(document.getElementById('bd-coupon').value);
        state.builder.maturity_years = Number(document.getElementById('bd-maturity').value);
        const m = Math.max(1, parseInt(document.getElementById('bd-m').value, 10));
        state.cash_flows = buildCouponBond(
            state.builder.par, state.builder.coupon_rate,
            state.builder.maturity_years, m);
        document.getElementById('bd-cf').value = cfsToBlob(state.cash_flows);
        showToast(t('view.bond_duration.toast.built', { n: state.cash_flows.length }), { level: 'info' });
    });
    document.getElementById('bd-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function cfsToBlob(cash_flows) {
    return cash_flows.map(c => `${c.time_years} ${c.amount}`).join('\n');
}

function readInputs() {
    const p = parseCashFlowBlob(document.getElementById('bd-cf').value);
    if (p.errors.length) {
        showErr(`${t('view.bond_duration.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        showToast(t('view.bond_duration.toast.parse_error', { n: p.errors.length }), { level: 'warning' });
        return;
    }
    hideErr();
    state.cash_flows           = p.cash_flows;
    state.ytm                  = Number(document.getElementById('bd-ytm').value);
    state.compounding_per_year = parseInt(document.getElementById('bd-m').value, 10);
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.cash_flows, state.ytm, state.compounding_per_year);
    if (err) { showErr(err); showToast(t('view.bond_duration.toast.invalid'), { level: 'warning' }); return; }
    const local = localCompute(state.cash_flows, state.ytm, state.compounding_per_year);
    renderSummary(local, true);
    renderSensitivity(local);
    renderCfChart();
    renderPvChart();
    let resp;
    try {
        resp = await api.calcBondDuration(buildBody(
            state.cash_flows, state.ytm, state.compounding_per_year));
    } catch (e) {
        showErr(`${t('view.bond_duration.err.api')}: ${e.message || e}`);
        showToast(t('view.bond_duration.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderSensitivity(resp);
    renderCfChart();
<<<<<<< Updated upstream
    const price = Number(resp.price || 0).toFixed(2);
    const mac = Number(resp.macaulay_duration || 0).toFixed(2);
    const mod = Number(resp.modified_duration || 0).toFixed(2);
    showToast(t('view.bond_duration.toast.computed', { price, mac, mod }), { level: 'success' });
=======
    renderPvChart();
>>>>>>> Stashed changes
}

function renderSummary(report, pending) {
    const badge = durationBadge(report.macaulay_duration);
    const local = localCompute(state.cash_flows, state.ytm, state.compounding_per_year);
    const parityOk = Math.abs(report.price - local.price) < 1e-6
                  && Math.abs(report.macaulay_duration - local.macaulay_duration) < 1e-9;
    const localTag = pending ? ` (${t('view.bond_duration.tag.local')})` : '';
    const totalCfAmount = state.cash_flows.reduce((s, c) => s + c.amount, 0);
    document.getElementById('bd-summary').innerHTML = [
        card(t('view.bond_duration.card.verdict'),       t(badge.key) + localTag, badge.cls),
        card(t('view.bond_duration.card.price'),         fmtUSD(report.price)),
        card(t('view.bond_duration.card.macaulay'),      fmtYears(report.macaulay_duration), badge.cls),
        card(t('view.bond_duration.card.modified'),      fmtYears(report.modified_duration), badge.cls),
        card(t('view.bond_duration.card.ytm'),           fmtPct(report.yield_to_maturity, 3)),
        card(t('view.bond_duration.card.compounding'),   String(state.compounding_per_year) + '/yr'),
        card(t('view.bond_duration.card.cash_flows'),    String(state.cash_flows.length)),
        card(t('view.bond_duration.card.total_cf'),      fmtUSD(totalCfAmount)),
        card(t('view.bond_duration.card.parity'),
             parityOk ? t('view.bond_duration.tag.ok') : t('view.bond_duration.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderSensitivity(report) {
    const wrap = document.getElementById('bd-sens');
    const mod = report.modified_duration;
    if (!Number.isFinite(mod) || mod === 0 || !Number.isFinite(report.price) || report.price === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.bond_duration.empty">${esc(t('view.bond_duration.empty'))}</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.bond_duration.col.dy">Δy</th>
                <th data-i18n="view.bond_duration.col.dp_pct">ΔP / P</th>
                <th data-i18n="view.bond_duration.col.dp_dollar">ΔP ($)</th>
                <th data-i18n="view.bond_duration.col.new_price">New price ($)</th>
            </tr></thead>
            <tbody>
                ${SENSITIVITY_BPS.map(bps => {
                    const pct = priceChangePct(mod, bps);
                    const dpUsd = pct * report.price;
                    const newPrice = report.price + dpUsd;
                    return `<tr>
                        <td>${esc(fmtBpsSigned(bps))}</td>
                        <td class="${pct >= 0 ? 'pos' : 'neg'}">${esc(fmtPctSigned(pct))}</td>
                        <td class="${dpUsd >= 0 ? 'pos' : 'neg'}">${esc(fmtUSD(dpUsd))}</td>
                        <td>${esc(fmtUSD(newPrice))}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
}

function renderCfChart() {
    const el = document.getElementById('bd-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const cfs = Array.isArray(state.cash_flows) ? state.cash_flows : [];
    const finite = cfs.filter(cf => Number.isFinite(cf.time_years) && Number.isFinite(cf.amount));
    if (finite.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.bond_duration.empty_chart">${esc(t('view.bond_duration.empty_chart'))}</div>`;
        return;
    }
    // Sort by time_years for monotonic x-axis.
    const sorted = [...finite].sort((a, b) => a.time_years - b.time_years);
    const xs = sorted.map(cf => cf.time_years);
    const ys = sorted.map(cf => cf.amount);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.bond_duration.chart.years') },
            { label: t('view.bond_duration.chart.cf_amount'),
              stroke: '#00e5ff', width: 1.0,
              fill: 'rgba(0,229,255,0.10)',
              points: { show: true, size: 8, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderPvChart() {
    const el = document.getElementById('bd-pv-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const cfs = Array.isArray(state.cash_flows) ? state.cash_flows : [];
    const finite = cfs.filter(cf => Number.isFinite(cf.time_years) && Number.isFinite(cf.amount));
    if (finite.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.bond_duration.empty_pv">${esc(t('view.bond_duration.empty_pv'))}</div>`;
        return;
    }
    const m = Math.max(1, state.compounding_per_year || 1);
    const y = state.ytm;
    const sorted = [...finite].sort((a, b) => a.time_years - b.time_years);
    const xs = sorted.map(cf => cf.time_years);
    const pv = sorted.map(cf => cf.amount / Math.pow(1 + y / m, m * cf.time_years));
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.bond_duration.chart.years') },
            { label: t('view.bond_duration.chart.pv'),
              stroke: '#7af0a8', width: 1.0,
              fill: 'rgba(122,240,168,0.10)',
              points: { show: true, size: 8, fill: '#7af0a8', stroke: '#7af0a8' } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, pv], el);
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('bd-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('bd-err').style.display = 'none'; }
