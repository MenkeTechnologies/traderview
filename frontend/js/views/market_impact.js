// Market Impact view — participation-rate slippage analysis.
//
// "At what % of ADV does my slippage cliff?" Bucketed by participation
// rate, returns avg / median / max slippage per band + flags the first
// band where avg slippage exceeds the spike threshold.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseTradeBlob, validateInputs, buildBody,
    bucketParticipations, makeDemoTrades,
    BUCKET_LABELS, fmtBps, fmtN,
} from '../_market_impact_inputs.js';

let state = { trades: '', spikeBps: 30 };

export async function renderMarketImpact(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// MARKET IMPACT · PARTICIPATION CLIFF</h1>

        <div class="chart-panel">
            <h2>Trade ledger</h2>
            <p class="muted">Paste <code>qty adv slippage_bps</code> per line. Negative
                slippage is a favorable fill. Demo loads 400 trades that visibly cliff
                past 1% ADV.</p>
            <textarea id="mi-trades" rows="8" placeholder="2500 5000000 2.1&#10;120000 5000000 12.5&#10;..."></textarea>
            <div class="inline-form">
                <button id="mi-demo" class="secondary" type="button">Load demo (400 trades)</button>
                <button id="mi-clear" class="secondary" type="button">Clear</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2>Threshold</h2>
            <div class="inline-form">
                <label>Spike threshold (bps)
                    <input id="mi-spike" type="number" step="any" min="0" value="${state.spikeBps}"></label>
                <button id="mi-run" class="primary" type="button">Analyze</button>
            </div>
            <p class="muted">The first ADV bucket whose avg slippage clears the threshold
                is flagged as your "impact cliff" — size below it.</p>
        </div>

        <div id="mi-errors" class="boot" style="display:none"></div>
        <div id="mi-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>Slippage by participation bucket</h2>
            <div id="mi-bars"></div>
            <p class="muted">Per bucket: trade count · avg · median · max slippage.
                Threshold-busting buckets glow red.</p>
        </div>

        <div class="chart-panel">
            <h2>Trade distribution</h2>
            <div id="mi-dist-chart" style="height:200px"></div>
            <p class="muted">Where your trades actually land. If you have heavy mass past
                1% ADV you're paying the impact tax even if the average looks fine.</p>
        </div>

        <div id="mi-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('mi-demo').addEventListener('click', () => {
        const trades = makeDemoTrades(42);
        document.getElementById('mi-trades').value =
            trades.map(t => `${t.qty} ${t.adv} ${t.slippage_bps}`).join('\n');
    });
    document.getElementById('mi-clear').addEventListener('click', () => {
        document.getElementById('mi-trades').value = '';
    });
    document.getElementById('mi-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.trades = document.getElementById('mi-trades').value;
    state.spikeBps = Number(document.getElementById('mi-spike').value);
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('mi-errors');
    errs.style.display = 'none';
    const { trades, errors } = parseTradeBlob(state.trades);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            `line ${e.line_no}: ${esc(e.message)} — ${esc(e.raw.slice(0, 80))}`).join('<br>');
        const more = errors.length > 8 ? `<br>… and ${errors.length - 8} more.` : '';
        errs.innerHTML = `<strong>${errors.length} parse error(s):</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (trades.length < 5) return;
    }
    const err = validateInputs(trades, state.spikeBps);
    if (err) { showErr(err); return; }
    let res;
    try {
        res = await api.microMarketImpact(buildBody(trades, state.spikeBps));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(res, trades);
    renderBars(res, state.spikeBps);
    renderDistribution(trades);
}

function renderSummary(report, trades) {
    const total = trades.length;
    const allSlips = trades.map(t => t.slippage_bps).filter(Number.isFinite);
    const avgAll = allSlips.length ? allSlips.reduce((a, b) => a + b, 0) / allSlips.length : NaN;
    const cliffLabel = report.impact_threshold_label || '—';
    const cliffCls = report.impact_threshold_label ? 'neg' : 'pos';
    document.getElementById('mi-summary').innerHTML = [
        card('Trades',          fmtN(total)),
        card('Avg slip (all)',  fmtBps(avgAll), avgAll > state.spikeBps ? 'neg' : 'pos'),
        card('Impact cliff',    cliffLabel,     cliffCls),
        card('Spike thresh',    fmtBps(state.spikeBps)),
        card('Active buckets',  String(report.buckets.filter(b => b.trade_count > 0).length)),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderBars(report, spikeBps) {
    const wrap = document.getElementById('mi-bars');
    const maxAbs = Math.max(...report.buckets.map(b => Math.abs(b.max_slippage_bps)), 1);
    wrap.innerHTML = report.buckets.map(b => {
        if (b.trade_count === 0) {
            return `
                <div class="is-bar-row">
                    <div class="is-bar-label">${esc(b.label)}</div>
                    <div class="is-bar-track"><div class="is-bar-midline mi-axis-zero"></div></div>
                    <div class="is-bar-value">no trades</div>
                </div>`;
        }
        const widthAvg = Math.max(0, Math.min(100, (Math.abs(b.avg_slippage_bps) / maxAbs) * 100)).toFixed(2);
        const widthMax = Math.max(0, Math.min(100, (Math.abs(b.max_slippage_bps) / maxAbs) * 100)).toFixed(2);
        const colorCls = Math.abs(b.avg_slippage_bps) > spikeBps ? 'mi-fill-bust' : 'mi-fill-ok';
        return `
            <div class="is-bar-row">
                <div class="is-bar-label">${esc(b.label)}</div>
                <div class="is-bar-track">
                    <div class="is-bar-midline mi-axis-zero"></div>
                    <div class="mi-bar-fill mi-fill-max" data-bar-pct="${widthMax}"></div>
                    <div class="mi-bar-fill ${colorCls}" data-bar-pct="${widthAvg}"></div>
                </div>
                <div class="is-bar-value">
                    n=${b.trade_count} · avg ${esc(fmtBps(b.avg_slippage_bps))} · med ${esc(fmtBps(b.median_slippage_bps))} · max ${esc(fmtBps(b.max_slippage_bps))}
                </div>
            </div>
        `;
    }).join('');
    requestAnimationFrame(() => {
        wrap.querySelectorAll('.mi-bar-fill').forEach(el => {
            const pct = Number(el.dataset.barPct);
            if (Number.isFinite(pct)) el.style.width = pct + '%';
        });
    });
}

function renderDistribution(trades) {
    if (!window.uPlot) return;
    const counts = bucketParticipations(trades);
    const xs = BUCKET_LABELS.map((_, i) => i);
    const el = document.getElementById('mi-dist-chart');
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: {} },
        series: [
            { label: 'bucket' },
            { label: 'trade count', stroke: '#00e5ff', width: 1.5,
              fill: '#00e5ff33', points: { show: true, size: 5 } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(s => BUCKET_LABELS[s] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: false },
    }, [xs, counts], el);
}

function showErr(msg) {
    const el = document.getElementById('mi-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('mi-err').style.display = 'none'; }
