// Volatility regime + Treasury curve + USD dashboard.
import { api } from '../api.js';
import { barChart } from '../charts.js';
import { esc, fmt } from '../util.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const sign = (n) => n == null ? '—' : (n >= 0 ? '+' : '') + n.toFixed(2);

export async function renderVol(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.vol.h1.vix_term_structure_yields_dxy" class="view-title">// VIX TERM STRUCTURE · YIELDS · DXY</h1>
        <div id="vix" class="cards" data-i18n="common.loading">loading…</div>
        <div class="chart-panel">
            <h2 data-i18n="view.vol.h2.vix_term_curve_tenor_vol">VIX term curve (tenor → vol)</h2>
            <div id="vix-curve"></div>
        </div>

        <div id="yields-cards" class="cards" style="margin-top:14px" data-i18n="view.vol.boot.loading_yields">loading yields…</div>
        <div class="chart-panel">
            <h2 data-i18n="view.vol.h2.treasury_yield_curve">Treasury yield curve</h2>
            <div id="yc-chart"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.vol.h2.u_s_dollar_major_fx">U.S. Dollar / major FX</h2>
            <div id="dxy" class="cards" data-i18n="common.loading">loading…</div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.vol.h2.yields_change_chart">Yield change per tenor (bp)</h2>
            <div id="vol-bp-chart" style="width:100%;height:220px"></div>
        </div>
    `;
    try {
        const [v, y, d] = await Promise.all([
            api.volVix(),
            api.volYields(),
            api.volDollar(),
        ]);
        if (!viewIsCurrent(tok)) return;
        renderVix(v, mount);
        renderYields(y, mount);
        renderDxy(d, mount);
        renderYieldsBpChart(y);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const vixEl = mount.querySelector('#vix');
        if (vixEl) vixEl.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function renderVix(v, mount) {
    const regimeCls = v.regime === 'backwardation' ? 'neg'
                    : v.regime === 'contango'      ? 'pos' : '';
    const cards = `
        <div class="card"><div class="label" data-i18n="view.vol.card.vix_spot">VIX spot</div><div class="value">${fmt(v.spot)}</div></div>
        <div class="card"><div class="label" data-i18n="view.vol.card.vix3m">VIX3M</div><div class="value">${fmt(v.three_month)}</div></div>
        <div class="card"><div class="label" data-i18n="view.vol.card.vvix">VVIX</div><div class="value">${fmt(v.vvix)}</div></div>
        <div class="card"><div class="label" data-i18n="view.vol.card.spot_3m">Spot ÷ 3M (%)</div>
            <div class="value ${(v.contango_pct ?? 0) >= 0 ? 'neg' : 'pos'}">${v.contango_pct != null ? sign(v.contango_pct) + '%' : '—'}</div></div>
        <div class="card"><div class="label" data-i18n="view.vol.card.regime">Regime</div>
            <div class="value ${regimeCls}">${v.regime.toUpperCase()}</div></div>
    `;
    const vixEl = mount.querySelector('#vix');
    if (vixEl) { vixEl.innerHTML = cards; try { applyUiI18n(vixEl); } catch (_) {} }
    const curveEl = mount.querySelector('#vix-curve');
    if (v.points.length && curveEl) {
        barChart(
            curveEl,
            v.points.map(p => p.label),
            v.points.map(p => p.value),
            { color: '#ff2a6d' },
        );
    }
}

function renderYields(y, mount) {
    const ycEl = mount.querySelector('#yields-cards');
    if (!ycEl) return;
    ycEl.innerHTML = `
        ${y.points.map(p => `
            <div class="card"><div class="label">${esc(p.label)} (${esc(p.symbol)})</div>
                <div class="value">${fmt(p.yield_pct, 3)}%</div>
                <div class="small ${p.change_bp >= 0 ? 'pos' : 'neg'}">${sign(p.change_bp)} bp</div></div>
        `).join('')}
        <div class="card"><div class="label" data-i18n="view.vol.card.spread_10y_2y">10Y − 5Y</div>
            <div class="value ${(y.spread_10y_2y ?? 0) >= 0 ? 'pos' : 'neg'}">${y.spread_10y_2y != null ? sign(y.spread_10y_2y) + ' bp' : '—'}</div></div>
        <div class="card"><div class="label" data-i18n="view.vol.card.spread_10y_3m">10Y − 3M</div>
            <div class="value ${(y.spread_10y_3m ?? 0) >= 0 ? 'pos' : 'neg'}">${y.spread_10y_3m != null ? sign(y.spread_10y_3m) + ' bp' : '—'}</div></div>
        <div class="card"><div class="label" data-i18n="view.vol.card.inverted">Inverted</div>
            <div class="value ${y.inverted ? 'neg' : 'pos'}">${y.inverted ? t('common.yes') : t('common.no')}</div></div>
    `;
    try { applyUiI18n(ycEl); } catch (_) {}
    const ycChart = mount.querySelector('#yc-chart');
    if (y.points.length && ycChart) {
        barChart(
            ycChart,
            y.points.map(p => p.label),
            y.points.map(p => p.yield_pct),
            { color: '#00e5ff' },
        );
    }
}

function renderYieldsBpChart(y) {
    const el = document.getElementById('vol-bp-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (y?.points || []).filter(p => Number.isFinite(Number(p.change_bp)));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.vol.empty_chart">${esc(t('view.vol.empty_chart'))}</div>`;
        return;
    }
    const labels = rows.map(p => p.label);
    const xs = labels.map((_, i) => i + 1);
    const upY   = rows.map(p => Number(p.change_bp) >= 0 ? Number(p.change_bp) : null);
    const downY = rows.map(p => Number(p.change_bp) <  0 ? Number(p.change_bp) : null);
    const zero  = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.vol.chart.tenor') },
            { label: t('view.vol.chart.up_bp'),  stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.vol.chart.down_bp'), stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.vol.chart.zero'),    stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, upY, downY, zero], el);
}

function renderDxy(d, mount) {
    const dxyEl = mount.querySelector('#dxy');
    if (!dxyEl) return;
    dxyEl.innerHTML = `
        <div class="card"><div class="label" data-i18n="view.vol.card.dxy">DXY</div>
            <div class="value">${fmt(d.dxy, 3)}</div>
            <div class="small ${(d.dxy_change_pct ?? 0) >= 0 ? 'pos' : 'neg'}">${d.dxy_change_pct != null ? sign(d.dxy_change_pct) + '%' : '—'}</div></div>
        <div class="card"><div class="label" data-i18n="view.vol.card.eur_usd">EUR/USD</div><div class="value">${fmt(d.eur_usd, 4)}</div></div>
        <div class="card"><div class="label" data-i18n="view.vol.card.usd_jpy">USD/JPY</div><div class="value">${fmt(d.usd_jpy, 3)}</div></div>
        <div class="card"><div class="label" data-i18n="view.vol.card.gbp_usd">GBP/USD</div><div class="value">${fmt(d.gbp_usd, 4)}</div></div>
    `;
    try { applyUiI18n(dxyEl); } catch (_) {}
}
