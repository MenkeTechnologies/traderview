// Forward Vol Curve — paste a term structure of implied vols (1M, 3M,
// 6M, 1Y, 2Y …), bootstrap the forward vol between each adjacent pair,
// and flag any no-arbitrage violations.
//
// Math: total variance σ²·t is additive across time. Forward vol
// between expiries T₁ < T₂:
//   σ_fwd² = (σ²(T₂)·T₂ − σ²(T₁)·T₁) / (T₂ − T₁)
// If that quantity is negative, the back-end vol is "too low" relative
// to the front-end vol (a calendar-arbitrage opportunity — the longer-
// dated vol cannot be lower than what's already implied by the
// shorter-dated one).
//
// Companion to the Vol Smile view (skew dimension); together they
// give the full vol-surface picture for a single underlying.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseTermStructure, sortRowsByTenor, checkUniqueTenors,
    validateTermStructure, buildBody, forwardVolStepSeries,
} from '../_forward_vol_inputs.js';

import { t } from '../i18n.js';
const DEFAULT_TEXT = `# Term structure of implied volatility.
#   tenor  iv     (whitespace or comma separated)
#   tenor accepts "1D" "1W" "1M" "3M" "1Y" "2Y" or bare years (e.g. 0.25).
#   iv accepts decimal (0.20) or percent (20 / 20%).
# Demo: typical SPX vol term structure (contango — back-end > front-end).
1W   18%
1M   20%
3M   22%
6M   23%
1Y   24%
2Y   24.5%
`;

let state = { text: DEFAULT_TEXT };

export async function renderForwardVolCurve(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.forward_vol_curve.h1.forward_vol_curve" class="view-title">// FORWARD VOL CURVE</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.forward_vol_curve.h2.term_structure">Term structure</h2>
            <textarea id="fv-text" rows="10"
                style="width:100%;font-family:monospace;font-size:13px">${esc(state.text)}</textarea>
            <button data-i18n="view.forward_vol_curve.btn.bootstrap" id="fv-run" class="primary" type="button" style="margin-top:10px">Bootstrap</button>
            <p data-i18n="view.forward_vol_curve.hint.variance_additivity_t_accumulates_linearly_across_" class="muted">
                Variance additivity: σ²·t accumulates linearly across non-overlapping windows.
                A negative back-end forward variance flags a calendar-arb violation.
            </p>
        </div>

        <div id="fv-parse-errors" class="boot" style="display:none;color:var(--red)"></div>

        <div id="fv-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.forward_vol_curve.h2.spot_iv_vs_forward_iv">Spot IV vs forward IV</h2>
            <div id="fv-chart" style="width:100%;height:340px"></div>
            <p data-i18n="view.forward_vol_curve.hint.cyan_line_spot_iv_at_each_tenor_interpolation_pure" class="muted">
                Cyan line = spot IV at each tenor (interpolation purely cosmetic).
                Orange step = bootstrapped forward vol active over each tenor window.
                Red markers flag violation intervals.
            </p>
        </div>

        <div id="fv-violations" class="chart-panel" style="display:none">
            <h2 data-i18n="view.forward_vol_curve.h2.arbitrage_violations">Arbitrage violations</h2>
            <div id="fv-violations-body"></div>
        </div>

        <div id="fv-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('fv-run').addEventListener('click', () => {
        state.text = document.getElementById('fv-text').value;
        void bootstrap(mount, tok);
    });
    void fmt;
}

async function bootstrap(mount, tok) {
    hideErrs();
    const parsed = parseTermStructure(state.text);
    if (parsed.errors.length) renderParseErrors(parsed.errors);

    const sorted = sortRowsByTenor(parsed.value);
    const dupErr = checkUniqueTenors(sorted);
    if (dupErr) { showErr(dupErr); return; }
    const valErr = validateTermStructure(sorted);
    if (valErr) { showErr(valErr); return; }

    let res;
    try {
        res = await api.anlyForwardVolatilityBootstrap(buildBody(sorted));
        if (!res) throw new Error(t('view.forward_vol_curve.error.null'));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        return;
    }
    if (!viewIsCurrent(tok)) return;

    renderSummary(sorted, res);
    renderChart(sorted, res);
    renderViolations(sorted, res);
}

function renderSummary(rows, res) {
    const cards = [];
    const fwdAvg = res.forward_vols.length
        ? res.forward_vols.reduce((a, b) => a + b, 0) / res.forward_vols.length
        : NaN;
    const fwdMin = res.forward_vols.length ? Math.min(...res.forward_vols) : NaN;
    const fwdMax = res.forward_vols.length ? Math.max(...res.forward_vols) : NaN;
    cards.push(card(t('view.forward_vol_curve.card.tenors'), String(rows.length)));
    cards.push(card(t('view.forward_vol_curve.card.forward_intervals'), String(res.forward_vols.length)));
    cards.push(card(t('view.forward_vol_curve.card.mean_forward_vol'), pctStr(fwdAvg)));
    cards.push(card(t('view.forward_vol_curve.card.range'), `${pctStr(fwdMin)} – ${pctStr(fwdMax)}`));
    const slopeClass = res.arbitrage_violations.length ? 'neg' : 'pos';
    const arbLabel = res.arbitrage_violations.length
        ? t(res.arbitrage_violations.length === 1 ? 'view.forward_vol_curve.arb.one' : 'view.forward_vol_curve.arb.many', { n: res.arbitrage_violations.length })
        : t('view.forward_vol_curve.arb.none');
    cards.push(card(t('view.forward_vol_curve.card.no_arb'), arbLabel, slopeClass));
    document.getElementById('fv-summary').innerHTML = cards.join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function pctStr(x) {
    return Number.isFinite(x) ? `${(x * 100).toFixed(2)}%` : '—';
}

function renderChart(rows, res) {
    const el = document.getElementById('fv-chart');
    if (!window.uPlot) {
        el.textContent = t('common.error.uplot_not_loaded');
        return;
    }
    el.innerHTML = '';

    // Spot IV series: one point per tenor (rendered as a connected line
    // even though IV between tenors is not strictly defined — purely
    // cosmetic).
    const tenorXs = rows.map(r => r.tenor_years);
    const spotYs = rows.map(r => r.iv);

    // Forward vol step series: 2 points per interval.
    const { xs: fwdXs, ys: fwdYs } = forwardVolStepSeries(rows, res.forward_vols);

    // Violation markers: for each violation, plot a red point at the
    // interval midpoint.
    const violations = new Set(res.arbitrage_violations || []);
    const violXs = [];
    const violYs = [];
    for (let i = 0; i < res.forward_vols.length && i + 1 < rows.length; i++) {
        if (!violations.has(i)) continue;
        const midT = (rows[i].tenor_years + rows[i + 1].tenor_years) / 2;
        violXs.push(midT);
        violYs.push(res.forward_vols[i]);
    }

    // Merge all xs (so uPlot has one shared x scale). For series that
    // don't have a point at a given x we use null.
    const allXs = Array.from(new Set([...tenorXs, ...fwdXs, ...violXs])).sort((a, b) => a - b);
    const spotAligned = allXs.map(x => {
        const i = tenorXs.indexOf(x);
        return i >= 0 ? spotYs[i] : null;
    });
    const fwdAligned = allXs.map(x => {
        // Walk fwdXs for an exact match.
        for (let i = 0; i < fwdXs.length; i++) {
            if (fwdXs[i] === x) return fwdYs[i];
        }
        return null;
    });
    const violAligned = allXs.map(x => {
        const i = violXs.indexOf(x);
        return i >= 0 ? violYs[i] : null;
    });

    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 340,
        scales: { x: {}, y: {} },
        series: [
            { label: t('chart.series.tenor_years') },
            { label: t('chart.series.spot_iv'), stroke: '#00e5ff', width: 2,
              points: { show: true, size: 6, stroke: '#00e5ff', fill: '#00e5ff' } },
            { label: t('chart.series.forward_vol'), stroke: '#ff9f1a', width: 2,
              spanGaps: false, points: { show: false } },
            { label: t('chart.series.arb_violation'), stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, stroke: '#ff3860', fill: '#ff3860' } },
        ],
        axes: [
            { stroke: '#aab' },
            { stroke: '#aab',
              values: (_, ticks) => ticks.map(t => `${(t * 100).toFixed(1)}%`) },
        ],
    }, [allXs, spotAligned, fwdAligned, violAligned], el);
}

function renderViolations(rows, res) {
    const panel = document.getElementById('fv-violations');
    const body = document.getElementById('fv-violations-body');
    if (!res.arbitrage_violations || res.arbitrage_violations.length === 0) {
        panel.style.display = 'none';
        return;
    }
    panel.style.display = '';
    body.innerHTML = res.arbitrage_violations.map(i => {
        const t0 = rows[i].tenor_years;
        const t1 = rows[i + 1].tenor_years;
        const iv0 = rows[i].iv;
        const iv1 = rows[i + 1].iv;
        return `<div class="vc-row">
            <span class="muted">${pctStr(iv0)} @ ${t0.toFixed(3)}y → ${pctStr(iv1)} @ ${t1.toFixed(3)}y</span>
            <strong class="neg">σ²·t went backward by ${((iv0 * iv0 * t0) - (iv1 * iv1 * t1)).toFixed(6)}</strong>
        </div>`;
    }).join('');
}

function renderParseErrors(errors) {
    const el = document.getElementById('fv-parse-errors');
    el.innerHTML = errors.slice(0, 20).map(e =>
        `<div>${esc(t('common.parse_error_line', { line: e.line_no, msg: e.message }))} <span class="muted">→ <code>${esc(e.raw || '')}</code></span></div>`
    ).join('');
    if (errors.length > 20) {
        el.innerHTML += `<div class="muted">${esc(t("common.plus_n_more", { n: errors.length - 20 }))}</div>`;
    }
    el.style.display = 'block';
}

function showErr(msg) {
    const el = document.getElementById('fv-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErrs() {
    document.getElementById('fv-parse-errors').style.display = 'none';
    document.getElementById('fv-err').style.display = 'none';
}
