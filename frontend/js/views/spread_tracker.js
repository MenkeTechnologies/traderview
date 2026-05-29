// Spread Tracker view — bid/ask spread cost-of-immediacy analyzer.
//
// Complements Implementation Shortfall + Market Impact + Liquidity:
// every market order pays half this spread. Wide / pathological
// spreads = use limit orders or skip the trade.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseQuoteBlob, validateInputs, buildBody,
    computeSpreadSeries, makeDemoQuotes,
    REGIME_THRESHOLDS, REGIME_LABELS, REGIME_CSS,
    fmtBps, fmtN, fmtPct,
} from '../_spread_tracker_inputs.js';

import { t } from '../i18n.js';
let state = { quotesText: '' };

export async function renderSpreadTracker(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.spread_tracker.h1.spread_tracker_cost_of_immediacy" class="view-title">// SPREAD TRACKER · COST OF IMMEDIACY</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.spread_tracker.h2.quote_samples">Quote samples</h2>
            <p class="muted" data-i18n-html="view.spread_tracker.help">Paste <code>bid ask</code> per line. Demo loads 300
                samples that include a 20-sample pathological burst near the end
                (a feed glitch / circuit breaker analog).</p>
            <textarea id="st-quotes" rows="8" placeholder="100.04 100.05&#10;100.05 100.06&#10;..."></textarea>
            <div class="inline-form">
                <button data-i18n="view.spread_tracker.btn.load_demo_300_quotes" id="st-demo" class="secondary" type="button">Load demo (300 quotes)</button>
                <button data-i18n="view.spread_tracker.btn.clear" id="st-clear" class="secondary" type="button">Clear</button>
                <button data-i18n="view.spread_tracker.btn.analyze" id="st-run" class="primary" type="button">Analyze</button>
            </div>
            <p data-i18n="view.spread_tracker.hint.regime_cuts_at_5_25_100_bps_wide_use_limit_orders_" class="muted">Regime cuts at 5 / 25 / 100 bps. Wide → use limit orders.
                Pathological → feed broken or illiquid name; sit out.</p>
        </div>

        <div id="st-errors" class="boot" style="display:none"></div>
        <div id="st-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.spread_tracker.h2.backend_note">Backend note</h2>
            <div id="st-note" class="muted">—</div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.spread_tracker.h2.spread_bps_over_time">Spread bps over time</h2>
            <div id="st-chart" style="height:280px"></div>
            <p data-i18n="view.spread_tracker.hint.cyan_spread_bps_dashed_reference_lines_green_5_bps" class="muted">Cyan = spread bps. Dashed reference lines: green = 5 bps (tight),
                yellow = 25 bps (normal cap), red = 100 bps (pathological cliff).</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.spread_tracker.h2.mid_price_track">Mid price track</h2>
            <div id="st-mid-chart" style="height:200px"></div>
            <p data-i18n="view.spread_tracker.hint.just_so_you_see_if_the_spread_blew_out_alongside_a" class="muted">Just so you see if the spread blew out alongside a real
                price move (newsy event) vs. a pure feed glitch (mid steady).</p>
        </div>

        <div id="st-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('st-demo').addEventListener('click', () => {
        const q = makeDemoQuotes(42);
        document.getElementById('st-quotes').value =
            q.map(s => `${s.bid} ${s.ask}`).join('\n');
    });
    document.getElementById('st-clear').addEventListener('click', () => {
        document.getElementById('st-quotes').value = '';
    });
    document.getElementById('st-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.quotesText = document.getElementById('st-quotes').value;
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('st-errors');
    errs.style.display = 'none';
    const { samples, errors } = parseQuoteBlob(state.quotesText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            `line ${e.line_no}: ${esc(e.message)} — ${esc(e.raw.slice(0, 80))}`).join('<br>');
        const more = errors.length > 8 ? `<br>… and ${errors.length - 8} more.` : '';
        errs.innerHTML = `<strong>${errors.length} parse error(s):</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (samples.length < 5) return;
    }
    const err = validateInputs(samples);
    if (err) { showErr(err); return; }
    let res;
    try {
        res = await api.microSpreadTracker(buildBody(samples));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(res);
    document.getElementById('st-note').textContent = res.note || '—';
    renderCharts(samples, res);
}

function renderSummary(r) {
    const regime = r.regime || 'normal';
    const cls = REGIME_CSS[regime] || '';
    document.getElementById('st-summary').innerHTML = [
        card(t('view.spread_tracker.card.regime'),          REGIME_LABELS[regime] || regime, cls),
        card(t('view.spread_tracker.card.valid_samples'),   String(r.samples)),
        card(t('view.spread_tracker.card.avg_spread'),      fmtBps(r.avg_spread_bps), cls),
        card(t('view.spread_tracker.card.min_spread'),      fmtBps(r.min_spread_bps), 'pos'),
        card(t('view.spread_tracker.card.max_spread'),      fmtBps(r.max_spread_bps), r.max_spread_bps > REGIME_THRESHOLDS.wide ? 'neg' : ''),
        card(t('view.spread_tracker.card.avg_mid'),         fmtN(r.avg_mid, 2)),
        card(t('view.spread_tracker.card.pathological'),  fmtPct(r.pathological_pct), r.pathological_pct > 0 ? 'neg' : 'pos'),
        card(t('view.spread_tracker.card.half_spread_cost'), fmtBps(r.avg_spread_bps / 2)),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderCharts(samples, _report) {
    if (!window.uPlot) return;
    const { bps, mids } = computeSpreadSeries(samples);
    const xs = samples.map((_, i) => i);
    const tightYs = xs.map(() => REGIME_THRESHOLDS.tight);
    const normYs  = xs.map(() => REGIME_THRESHOLDS.normal);
    const wideYs  = xs.map(() => REGIME_THRESHOLDS.wide);

    const elBps = document.getElementById('st-chart');
    new window.uPlot({
        title: '', width: elBps.clientWidth || 600, height: 280,
        scales: { x: {}, y: {} },
        series: [
            { label: 'sample #' },
            { label: 'spread bps', stroke: '#00e5ff', width: 1.2,
              fill: '#00e5ff1A', points: { show: false } },
            { label: 'tight ≤ 5',   stroke: '#39ff14', width: 1.0,
              dash: [4, 4], points: { show: false } },
            { label: 'normal cap',  stroke: '#ffd84a', width: 1.0,
              dash: [4, 4], points: { show: false } },
            { label: 'patho ≥ 100', stroke: '#ff3860', width: 1.0,
              dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: true },
    }, [xs, bps, tightYs, normYs, wideYs], elBps);

    const elMid = document.getElementById('st-mid-chart');
    new window.uPlot({
        title: '', width: elMid.clientWidth || 600, height: 200,
        scales: { x: {}, y: {} },
        series: [
            { label: 'sample #' },
            { label: 'mid', stroke: '#a06bff', width: 1.2,
              fill: '#a06bff14', points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: false },
    }, [xs, mids], elMid);
}

function showErr(msg) {
    const el = document.getElementById('st-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('st-err').style.display = 'none'; }
