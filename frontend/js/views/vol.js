// Volatility regime + Treasury curve + USD dashboard.
import { api } from '../api.js';
import { barChart } from '../charts.js';
import { esc, fmt } from '../util.js';
import { applyUiI18n } from '../i18n.js';
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
