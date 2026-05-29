// Order Flow view — Lee-Ready-style tick aggressor classification +
// rolled-up imbalance scalars.
//
// Calls /microstructure/order-flow-classify (per-tick sides) and
// /microstructure/order-flow-aggregate (rolled-up totals) in parallel.
// Visualizes cumulative buy/sell/net flow + the imbalance gauge.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseTickBlob, validateInputs, buildBody,
    sideBadge, cumulativeFlow, makeDemoTicks,
    fmtN, fmtImbalance,
} from '../_order_flow_inputs.js';

import { t } from '../i18n.js';
let state = { tickText: '' };

export async function renderOrderFlow(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.order_flow.h1.order_flow_aggressor_classification" class="view-title">// ORDER FLOW · AGGRESSOR CLASSIFICATION</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.order_flow.h2.tick_stream">Tick stream</h2>
            <p class="muted">Paste <code>price volume bid ask</code> per line. Trades at
                ask = BUY, at bid = SELL, mid-spread falls back to tick rule (vs prior price).
                Demo loads 400 ticks with engineered net-buy pressure.</p>
            <textarea id="of-ticks" rows="8" placeholder="100.05 250 100.04 100.05&#10;100.04 1200 100.04 100.05&#10;..."></textarea>
            <div class="inline-form">
                <button data-i18n="view.order_flow.btn.load_demo_400_ticks_buy_pressure" id="of-demo" class="secondary" type="button">Load demo (400 ticks, buy pressure)</button>
                <button data-i18n="view.order_flow.btn.clear" id="of-clear" class="secondary" type="button">Clear</button>
                <button data-i18n="view.order_flow.btn.classify" id="of-run" class="primary" type="button">Classify</button>
            </div>
        </div>

        <div id="of-errors" class="boot" style="display:none"></div>
        <div id="of-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.order_flow.h2.imbalance_gauge">Imbalance gauge</h2>
            <div id="of-gauge"></div>
            <p data-i18n="view.order_flow.hint.buy_sell_buy_sell_cyan_aggressive_buying_red_insti" class="muted">(buy − sell) / (buy + sell). Cyan = aggressive buying,
                red = institutional dumping. Excludes uncertain volume.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.order_flow.h2.cumulative_flow">Cumulative flow</h2>
            <div id="of-chart" style="height:260px"></div>
            <p data-i18n="view.order_flow.hint.cyan_cumulative_aggressive_buy_volume_magenta_draw" class="muted">Cyan = cumulative aggressive-buy volume. Magenta (drawn
                negative for visual contrast) = cumulative sell. Yellow = net (buy − sell).
                Smooth-rising net = sustained accumulation. Steep drop = distribution.</p>
        </div>

        <div id="of-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('of-demo').addEventListener('click', () => {
        const t = makeDemoTicks(42);
        document.getElementById('of-ticks').value =
            t.map(x => `${x.price} ${x.volume} ${x.bid} ${x.ask}`).join('\n');
    });
    document.getElementById('of-clear').addEventListener('click', () => {
        document.getElementById('of-ticks').value = '';
    });
    document.getElementById('of-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.tickText = document.getElementById('of-ticks').value;
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('of-errors');
    errs.style.display = 'none';
    const { ticks, errors } = parseTickBlob(state.tickText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            `line ${e.line_no}: ${esc(e.message)} — ${esc(e.raw.slice(0, 80))}`).join('<br>');
        const more = errors.length > 8 ? `<br>… and ${errors.length - 8} more.` : '';
        errs.innerHTML = `<strong>${errors.length} parse error(s):</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (ticks.length < 5) return;
    }
    const err = validateInputs(ticks);
    if (err) { showErr(err); return; }

    let classified, aggregate;
    try {
        const body = buildBody(ticks);
        [classified, aggregate] = await Promise.all([
            api.microOrderFlowClassify(body),
            api.microOrderFlowAggregate(body),
        ]);
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(aggregate, classified);
    renderGauge(aggregate);
    renderFlowChart(classified);
}

function renderSummary(report, classified) {
    const buyTicks  = (classified || []).filter(c => c.side === 'buy').length;
    const sellTicks = (classified || []).filter(c => c.side === 'sell').length;
    const uncTicks  = (classified || []).filter(c => c.side === 'uncertain').length;
    document.getElementById('of-summary').innerHTML = [
        card(t('view.order_flow.card.total_ticks'),     String((classified || []).length)),
        card(t('view.order_flow.card.buy_ticks'),       String(buyTicks), buyTicks ? 'pos' : ''),
        card(t('view.order_flow.card.sell_ticks'),      String(sellTicks), sellTicks ? 'neg' : ''),
        card(t('view.order_flow.card.uncertain_ticks'), String(uncTicks)),
        card(t('view.order_flow.card.buy_volume'),      fmtN(report.buy_volume),       'pos'),
        card(t('view.order_flow.card.sell_volume'),     fmtN(report.sell_volume),      'neg'),
        card(t('view.order_flow.card.net_volume'),      fmtN(report.net_volume),       report.net_volume >= 0 ? 'pos' : 'neg'),
        card(t('view.order_flow.card.imbalance_ratio'), fmtImbalance(report.imbalance_ratio),
            report.imbalance_ratio > 0.1 ? 'pos'
                : report.imbalance_ratio < -0.1 ? 'neg' : ''),
    ].join('');
    void sideBadge;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderGauge(report) {
    const wrap = document.getElementById('of-gauge');
    const imb = Math.max(-1, Math.min(1, report.imbalance_ratio || 0));
    const halfPct = (Math.abs(imb) * 50).toFixed(2);
    const sideClass = imb >= 0 ? 'is-fill-pos obi-fill-bid' : 'is-fill-neg obi-fill-ask';
    wrap.innerHTML = `
        <div class="is-bar-row">
            <div class="is-bar-label">imbalance</div>
            <div class="is-bar-track">
                <div class="is-bar-midline"></div>
                <div class="is-bar-midline obi-q-neg-strong"></div>
                <div class="is-bar-midline obi-q-neg"></div>
                <div class="is-bar-midline obi-q-pos"></div>
                <div class="is-bar-midline obi-q-pos-strong"></div>
                <div class="is-bar-fill ${sideClass}" data-bar-pct="${halfPct}"></div>
            </div>
            <div class="is-bar-value">${esc(fmtImbalance(report.imbalance_ratio))}</div>
        </div>
    `;
    requestAnimationFrame(() => {
        wrap.querySelectorAll('.is-bar-fill').forEach(el => {
            const pct = Number(el.dataset.barPct);
            if (Number.isFinite(pct)) el.style.width = pct + '%';
        });
    });
}

function renderFlowChart(classified) {
    if (!window.uPlot) return;
    const { xs, buy, sell, net } = cumulativeFlow(classified);
    const el = document.getElementById('of-chart');
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 260,
        scales: { x: {}, y: {} },
        series: [
            { label: 'tick #' },
            { label: 'cum buy',  stroke: '#00e5ff', width: 1.2,
              fill: '#00e5ff14', points: { show: false } },
            { label: 'cum sell', stroke: '#ff3860', width: 1.2,
              fill: '#ff386014', points: { show: false } },
            { label: 'net',      stroke: '#ffd84a', width: 1.5,
              points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: true },
    }, [xs, buy, sell, net], el);
}

function showErr(msg) {
    const el = document.getElementById('of-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('of-err').style.display = 'none'; }
