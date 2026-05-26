// Volatility regime + Treasury curve + USD dashboard.
import { api } from '../api.js';
import { barChart } from '../charts.js';
import { esc, fmt } from '../util.js';

const sign = (n) => n == null ? '—' : (n >= 0 ? '+' : '') + n.toFixed(2);

export async function renderVol(mount) {
    mount.innerHTML = `
        <h1 class="view-title">// VIX TERM STRUCTURE · YIELDS · DXY</h1>
        <div id="vix" class="cards">loading…</div>
        <div class="chart-panel">
            <h2>VIX term curve (tenor → vol)</h2>
            <div id="vix-curve"></div>
        </div>

        <div id="yields-cards" class="cards" style="margin-top:14px">loading yields…</div>
        <div class="chart-panel">
            <h2>Treasury yield curve</h2>
            <div id="yc-chart"></div>
        </div>

        <div class="chart-panel">
            <h2>U.S. Dollar / major FX</h2>
            <div id="dxy" class="cards">loading…</div>
        </div>
    `;
    try {
        const [v, y, d] = await Promise.all([
            api.volVix(),
            api.volYields(),
            api.volDollar(),
        ]);
        renderVix(v);
        renderYields(y);
        renderDxy(d);
    } catch (e) {
        document.getElementById('vix').innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function renderVix(v) {
    const regimeCls = v.regime === 'backwardation' ? 'neg'
                    : v.regime === 'contango'      ? 'pos' : '';
    const cards = `
        <div class="card"><div class="label">VIX spot</div><div class="value">${fmt(v.spot)}</div></div>
        <div class="card"><div class="label">VIX3M</div><div class="value">${fmt(v.three_month)}</div></div>
        <div class="card"><div class="label">VVIX</div><div class="value">${fmt(v.vvix)}</div></div>
        <div class="card"><div class="label">Spot ÷ 3M (%)</div>
            <div class="value ${(v.contango_pct ?? 0) >= 0 ? 'neg' : 'pos'}">${v.contango_pct != null ? sign(v.contango_pct) + '%' : '—'}</div></div>
        <div class="card"><div class="label">Regime</div>
            <div class="value ${regimeCls}">${v.regime.toUpperCase()}</div></div>
    `;
    document.getElementById('vix').innerHTML = cards;
    if (v.points.length) {
        barChart(
            document.getElementById('vix-curve'),
            v.points.map(p => p.label),
            v.points.map(p => p.value),
            { color: '#ff2a6d' },
        );
    }
}

function renderYields(y) {
    document.getElementById('yields-cards').innerHTML = `
        ${y.points.map(p => `
            <div class="card"><div class="label">${esc(p.label)} (${esc(p.symbol)})</div>
                <div class="value">${fmt(p.yield_pct, 3)}%</div>
                <div class="small ${p.change_bp >= 0 ? 'pos' : 'neg'}">${sign(p.change_bp)} bp</div></div>
        `).join('')}
        <div class="card"><div class="label">10Y − 5Y</div>
            <div class="value ${(y.spread_10y_2y ?? 0) >= 0 ? 'pos' : 'neg'}">${y.spread_10y_2y != null ? sign(y.spread_10y_2y) + ' bp' : '—'}</div></div>
        <div class="card"><div class="label">10Y − 3M</div>
            <div class="value ${(y.spread_10y_3m ?? 0) >= 0 ? 'pos' : 'neg'}">${y.spread_10y_3m != null ? sign(y.spread_10y_3m) + ' bp' : '—'}</div></div>
        <div class="card"><div class="label">Inverted</div>
            <div class="value ${y.inverted ? 'neg' : 'pos'}">${y.inverted ? 'YES' : 'NO'}</div></div>
    `;
    if (y.points.length) {
        barChart(
            document.getElementById('yc-chart'),
            y.points.map(p => p.label),
            y.points.map(p => p.yield_pct),
            { color: '#00e5ff' },
        );
    }
}

function renderDxy(d) {
    document.getElementById('dxy').innerHTML = `
        <div class="card"><div class="label">DXY</div>
            <div class="value">${fmt(d.dxy, 3)}</div>
            <div class="small ${(d.dxy_change_pct ?? 0) >= 0 ? 'pos' : 'neg'}">${d.dxy_change_pct != null ? sign(d.dxy_change_pct) + '%' : '—'}</div></div>
        <div class="card"><div class="label">EUR/USD</div><div class="value">${fmt(d.eur_usd, 4)}</div></div>
        <div class="card"><div class="label">USD/JPY</div><div class="value">${fmt(d.usd_jpy, 3)}</div></div>
        <div class="card"><div class="label">GBP/USD</div><div class="value">${fmt(d.gbp_usd, 4)}</div></div>
    `;
}
