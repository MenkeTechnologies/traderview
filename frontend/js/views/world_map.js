// World markets map — SVG silhouette + lat/lng-pinned index tiles + commodity
// strip. Polls /api/markets/snapshot (in-process cache on server).

import { api } from '../api.js';
import { fmt, esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

// Simplified low-poly world continents in equirectangular projection.
// Viewport is the rectangle: lng [-180,180], lat [85,-85].
// Path data is hand-built to silhouette North/South America, Eurasia, Africa,
// Oceania at low resolution — enough to anchor pins visually.
const WORLD_SVG_PATHS = [
    // North America (USA + Canada + Mexico, rough outline)
    'M70,80 L150,75 L200,85 L240,95 L260,120 L275,150 L260,175 L240,185 L215,200 L185,215 L155,210 L130,200 L105,185 L85,165 L70,135 Z',
    // Greenland
    'M280,55 L320,55 L335,80 L320,105 L290,100 L280,80 Z',
    // South America
    'M210,225 L255,225 L275,255 L280,300 L260,360 L235,395 L210,400 L195,360 L195,300 L200,250 Z',
    // Europe
    'M460,90 L520,85 L545,95 L560,115 L555,135 L530,150 L500,155 L475,145 L460,125 Z',
    // Africa
    'M475,165 L545,160 L575,180 L590,220 L585,275 L555,320 L515,335 L490,320 L475,285 L470,235 L470,195 Z',
    // Middle East + Arabia
    'M560,150 L595,150 L610,175 L605,205 L585,215 L560,200 L555,170 Z',
    // Asia (large blob — Russia + China + India + SE Asia)
    'M560,90 L700,85 L795,90 L850,110 L880,140 L865,175 L820,195 L770,200 L735,210 L705,235 L685,250 L650,245 L620,225 L595,200 L575,165 L560,135 Z',
    // Japan
    'M870,140 L885,138 L895,160 L890,175 L878,175 L870,160 Z',
    // SE Asia islands (Philippines / Indonesia)
    'M810,230 L865,225 L880,245 L855,265 L820,270 L800,255 Z',
    // Australia
    'M830,290 L890,285 L920,305 L915,340 L885,355 L845,355 L820,340 L820,310 Z',
];

const WIDTH = 960;
const HEIGHT = 480;

// Equirectangular projection (lng -180..180 → 0..WIDTH, lat 85..-85 → 0..HEIGHT)
const project = (lat, lng) => {
    const x = (lng + 180) / 360 * WIDTH;
    const y = (85 - lat) / 170 * HEIGHT;
    return [x, y];
};

const fmtPct = (n) => {
    const v = Number(n);
    if (!Number.isFinite(v)) return '—';
    const sign = v >= 0 ? '+' : '';
    return `${sign}${v.toFixed(2)}%`;
};

export async function renderWorldMarkets(mount) {
    if (!mount) return;
    const tok = currentViewToken();
    mount.innerHTML = `
        <div class="markets-panel">
            <div class="markets-header">
                <h2 data-i18n="view.world_map.h2.world_markets">// WORLD MARKETS</h2>
                <span class="market-status" id="market-status"><span data-i18n="common.loading">loading…</span></span>
            </div>
            <div class="world-map-wrap" id="world-map-wrap">
                ${renderSvg([])}
            </div>
            <div class="commodities-strip" id="commodities-strip"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.world_map.h2.change_chart">Change % per index</h2>
            <div id="wm-chart" style="width:100%;height:220px"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.world_map.h2.com_chart">Change % per commodity</h2>
            <div id="wm-com-chart" style="width:100%;height:200px"></div>
        </div>
    `;
    try {
        const snap = await api.marketsSnapshot();
        if (!viewIsCurrent(tok)) return;  // user navigated away mid-fetch
        renderSnapshot(snap, mount);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const wrap = mount.querySelector('#world-map-wrap');
        if (wrap) {
            wrap.innerHTML = `<div class="boot">${esc(t('view.world_map.boot.markets_unavailable', { err: e.message }))}</div>`;
        }
    }
}

function renderSnapshot(snap, mount) {
    if (!mount) return;
    const wrap = mount.querySelector('#world-map-wrap');
    if (!wrap) return;
    wrap.innerHTML = renderSvg(snap.indices);
    renderChangeChart(snap.indices);
    renderCommodityChart(snap.commodities);

    const status = mount.querySelector('#market-status');
    if (status) {
        status.className = 'market-status ' + (snap.us_market_open ? 'open' : 'closed');
        status.innerHTML = snap.us_market_open
            ? t('view.world_map.status.open_html')
            : t('view.world_map.status.closed_html');
    }

    const strip = mount.querySelector('#commodities-strip');
    if (strip) {
        strip.innerHTML = snap.commodities.map(c => `
            <div class="commodity" data-context-scope="symbol-row" data-symbol="${esc(c.symbol)}">
                <div class="commodity-label">
                    <span class="flag">${c.flag}</span>
                    <span>${esc(c.label)}</span>
                </div>
                <div class="commodity-price">${fmt(c.price, c.symbol.includes('USD') ? 2 : 4)}</div>
                <div class="commodity-pct ${c.change_pct >= 0 ? 'pos' : 'neg'}">${fmtPct(c.change_pct)}</div>
            </div>
        `).join('');
    }
}

function renderCommodityChart(commodities) {
    const el = document.getElementById('wm-com-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (commodities || []).filter(c => Number.isFinite(Number(c.change_pct)));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.world_map.empty_com_chart">${esc(t('view.world_map.empty_com_chart'))}</div>`;
        return;
    }
    rows.sort((a, b) => Number(b.change_pct) - Number(a.change_pct));
    const labels = rows.map(c => c.label);
    const xs = labels.map((_, i) => i + 1);
    const upY   = rows.map(c => Number(c.change_pct) >= 0 ? Number(c.change_pct) : null);
    const downY = rows.map(c => Number(c.change_pct) <  0 ? Number(c.change_pct) : null);
    const zero  = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.world_map.chart.commodity') },
            { label: t('view.world_map.chart.up'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.world_map.chart.down'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.world_map.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
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

function renderChangeChart(indices) {
    const el = document.getElementById('wm-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (indices || []).filter(p => Number.isFinite(Number(p.change_pct)));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.world_map.empty_chart">${esc(t('view.world_map.empty_chart'))}</div>`;
        return;
    }
    rows.sort((a, b) => Number(b.change_pct) - Number(a.change_pct));
    const labels = rows.map(p => p.label);
    const xs = labels.map((_, i) => i + 1);
    const upY   = rows.map(p => Number(p.change_pct) >= 0 ? Number(p.change_pct) : null);
    const downY = rows.map(p => Number(p.change_pct) <  0 ? Number(p.change_pct) : null);
    const zero  = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.world_map.chart.index') },
            { label: t('view.world_map.chart.up'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.world_map.chart.down'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.world_map.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
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

function renderSvg(pins) {
    const continents = WORLD_SVG_PATHS
        .map(d => `<path d="${d}" />`).join('');
    const pinMarkers = pins.map(p => {
        const [x, y] = project(p.lat, p.lng);
        return `<circle cx="${x.toFixed(1)}" cy="${y.toFixed(1)}" r="4"
                  fill="${p.change_pct >= 0 ? '#23d160' : '#ff3860'}"
                  stroke="#0a0a12" stroke-width="2"/>`;
    }).join('');

    return `
        <svg viewBox="0 0 ${WIDTH} ${HEIGHT}" preserveAspectRatio="xMidYMid meet"
             class="world-map" xmlns="http://www.w3.org/2000/svg">
            <defs>
                <radialGradient id="map-glow" cx="50%" cy="50%" r="60%">
                    <stop offset="0%"  stop-color="rgba(255, 84, 0, 0.18)"/>
                    <stop offset="60%" stop-color="rgba(255, 42, 109, 0.10)"/>
                    <stop offset="100%" stop-color="rgba(5, 217, 232, 0.0)"/>
                </radialGradient>
                <linearGradient id="landmass" x1="0%" y1="0%" x2="0%" y2="100%">
                    <stop offset="0%"  stop-color="#ff7a18"/>
                    <stop offset="100%" stop-color="#ff2a6d"/>
                </linearGradient>
            </defs>
            <rect width="${WIDTH}" height="${HEIGHT}" fill="url(#map-glow)"/>
            <g fill="url(#landmass)" stroke="rgba(255,255,255,0.08)" stroke-width="0.5">
                ${continents}
            </g>
            ${pinMarkers}
        </svg>
        <div class="pin-overlay" style="position:relative">
            ${pins.map(p => pinLabel(p)).join('')}
        </div>
    `;
}

function pinLabel(p) {
    // Convert projected [x,y] to % of SVG viewport so absolute positioning works
    // regardless of how the SVG is scaled.
    const [x, y] = project(p.lat, p.lng);
    const left = (x / WIDTH) * 100;
    const top  = (y / HEIGHT) * 100;
    const cls = p.change_pct >= 0 ? 'pos' : 'neg';
    return `
        <div class="pin" style="left:${left.toFixed(2)}%;top:${top.toFixed(2)}%">
            <span class="pin-flag">${p.flag}</span>
            <span class="pin-label">${esc(p.label)}</span>
            <span class="pin-pct ${cls}">${fmtPct(p.change_pct)}</span>
        </div>
    `;
}
