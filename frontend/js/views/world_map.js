// World markets map — Natural Earth 110m country outlines + lat/lng-pinned
// index dots + commodity strip. Polls /api/markets/snapshot (server cache).
//
// The SVG path data lives in `frontend/lib/world-110m.svg`, generated from
// the Natural Earth public-domain dataset (admin-0 countries, 287 outlines).
// Equirectangular projection, viewport 960×480, matches the project() helper
// below so dots position correctly on top of the country layer.

import { api } from '../api.js';
import { fmt, esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

// Fetched and cached on first render — embedded inline so we can recolor the
// land via CSS `currentColor` and stack pins on top in the same SVG layer.
let WORLD_SVG_CACHE = null;
async function loadWorldSvg() {
    if (WORLD_SVG_CACHE !== null) return WORLD_SVG_CACHE;
    try {
        const res = await fetch('lib/world-110m.svg');
        if (!res.ok) throw new Error('fetch failed');
        const text = await res.text();
        // Extract just the inner path d= — we wrap it in our own <svg> so we
        // control gradients, pin layer, viewBox alignment.
        const m = text.match(/<path[^>]+d="([^"]+)"/);
        WORLD_SVG_CACHE = m ? m[1] : '';
    } catch (_) {
        WORLD_SVG_CACHE = '';  // gracefully degrade to no continents
    }
    return WORLD_SVG_CACHE;
}

// Legacy hand-drawn polygon constant kept only so the equirectangular
// project() helper below still anchors at (W,H) = (960,480). The actual
// path data is no longer used — see loadWorldSvg() above.
const WORLD_SVG_PATHS = [
    // North America (Canada + USA + Mexico + Central America)
    'M120,90 L180,82 L215,78 L245,80 L270,86 L285,95 L298,105 L305,118 L308,130 L302,142 L295,148 L288,142 L282,135 L275,140 L272,150 L268,158 L265,166 L260,172 L252,176 L243,180 L233,185 L224,191 L215,198 L208,206 L202,213 L196,218 L188,224 L182,232 L175,238 L168,244 L162,250 L155,256 L150,255 L148,247 L150,238 L155,228 L160,218 L162,208 L160,198 L155,190 L150,184 L142,180 L132,178 L122,178 L114,180 L108,178 L104,172 L100,164 L96,156 L92,148 L88,140 L86,132 L84,124 L84,116 L86,108 L90,100 L100,94 L110,92 Z',
    // Greenland
    'M310,50 L335,46 L355,48 L370,54 L378,64 L380,76 L376,88 L368,98 L356,104 L344,108 L332,108 L322,104 L314,96 L308,86 L306,74 L308,62 Z',
    // South America (eastern bulge, narrowing south)
    'M225,228 L248,225 L268,228 L282,236 L290,248 L294,262 L296,276 L294,290 L290,304 L284,318 L278,332 L272,346 L264,358 L256,368 L248,376 L240,382 L232,386 L224,388 L218,386 L214,382 L212,376 L210,368 L208,358 L208,346 L210,332 L212,318 L212,304 L210,290 L208,276 L208,262 L212,248 L218,236 Z',
    // British Isles
    'M455,128 L468,124 L475,128 L478,138 L476,148 L470,154 L462,154 L456,148 L452,138 Z',
    // Europe mainland (Iberia → Scandinavia → European Russia)
    'M438,140 L450,135 L466,138 L478,142 L490,140 L502,138 L514,138 L526,140 L538,138 L548,134 L556,128 L562,122 L568,118 L572,112 L578,108 L584,108 L586,112 L584,118 L580,124 L576,128 L572,132 L572,138 L576,142 L582,144 L588,142 L592,140 L596,142 L596,148 L590,154 L582,158 L572,160 L562,158 L552,158 L542,160 L532,164 L522,164 L512,160 L500,156 L488,154 L476,156 L466,160 L458,164 L450,162 L444,156 L440,148 Z',
    // Africa (Mediterranean coast down to Cape, around to Horn)
    'M468,180 L482,176 L498,174 L514,176 L530,180 L546,184 L560,188 L572,194 L582,202 L590,212 L596,222 L600,234 L602,246 L602,258 L600,272 L596,286 L590,300 L582,312 L572,324 L562,332 L550,338 L538,342 L526,342 L514,340 L502,336 L492,330 L484,322 L478,312 L474,300 L472,288 L472,274 L472,260 L472,246 L472,232 L470,220 L468,208 L466,196 Z',
    // Madagascar
    'M610,278 L618,276 L624,280 L626,290 L624,302 L618,310 L612,308 L608,300 L608,290 Z',
    // Arabian peninsula
    'M580,190 L600,188 L614,194 L622,206 L624,222 L620,234 L610,242 L600,242 L592,236 L586,226 L582,214 L580,202 Z',
    // Asia main (Russia + central Asia + China + SE Asia)
    'M590,100 L620,94 L654,90 L688,88 L722,90 L756,94 L788,102 L818,114 L842,128 L860,144 L872,160 L876,176 L876,190 L870,200 L862,206 L850,210 L834,212 L818,210 L800,206 L780,200 L760,196 L740,200 L720,206 L702,214 L688,224 L676,234 L664,242 L654,248 L646,250 L640,246 L638,238 L640,228 L644,218 L646,208 L644,198 L640,188 L634,180 L624,172 L612,166 L598,162 L584,156 L572,150 L562,142 L556,132 L554,120 L558,110 L568,104 L580,102 Z',
    // India
    'M664,194 L680,192 L692,196 L700,206 L702,220 L700,234 L694,246 L686,254 L676,256 L668,250 L662,240 L660,228 L660,216 L662,204 Z',
    // SE Asia mainland (Indochina)
    'M736,210 L754,210 L766,216 L774,226 L774,238 L770,248 L762,254 L754,252 L748,246 L744,236 L740,226 Z',
    // Indonesia islands (Sumatra, Java, Borneo, Sulawesi)
    'M752,256 L770,254 L786,258 L800,262 L814,260 L826,260 L838,264 L848,272 L848,282 L840,288 L828,290 L814,288 L800,286 L786,284 L772,282 L760,278 L752,272 L750,264 Z',
    // Philippines
    'M816,232 L830,230 L838,238 L840,250 L834,256 L824,256 L818,250 L816,242 Z',
    // Japan
    'M870,150 L884,146 L892,154 L898,164 L902,176 L898,188 L890,194 L880,194 L872,188 L868,178 L866,168 L868,158 Z',
    // Australia
    'M788,294 L812,290 L836,290 L858,294 L876,302 L890,314 L898,328 L900,342 L898,354 L890,362 L878,366 L862,368 L844,368 L824,366 L806,362 L790,356 L776,348 L768,338 L766,326 L770,314 L778,304 Z',
    // New Zealand
    'M912,358 L922,356 L928,364 L930,374 L924,380 L916,378 L912,372 L910,366 Z',
    // Tasmania
    'M860,374 L870,372 L876,378 L876,386 L870,390 L862,388 L858,382 Z',
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
    // Kick off world-svg fetch in parallel with the markets snapshot so we
    // don't pay sequential latency on first render.
    const worldPromise = loadWorldSvg();
    mount.innerHTML = `
        <div class="markets-panel">
            <div class="markets-header">
                <h2 data-i18n="view.world_map.h2.world_markets">// WORLD MARKETS</h2>
                <span class="market-status" id="market-status"><span data-i18n="common.loading">loading…</span></span>
            </div>
            <div class="world-map-wrap" id="world-map-wrap">
                ${renderSvg([], '')}
            </div>
            <div class="commodities-strip" id="commodities-strip"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.world_map.h2.change_chart">Change % per index</h2>
            <div id="wm-chart" class="chart-h-220"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.world_map.h2.com_chart">Change % per commodity</h2>
            <div id="wm-com-chart" class="chart-h-200"></div>
        </div>
    `;
    try {
        const [snap, worldPath] = await Promise.all([api.marketsSnapshot(), worldPromise]);
        if (!viewIsCurrent(tok)) return;  // user navigated away mid-fetch
        renderSnapshot(snap, mount, worldPath);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const wrap = mount.querySelector('#world-map-wrap');
        if (wrap) {
            wrap.innerHTML = `<div class="boot">${esc(t('view.world_map.boot.markets_unavailable', { err: e.message }))}</div>`;
        }
    }
}

function renderSnapshot(snap, mount, worldPath) {
    if (!mount) return;
    const wrap = mount.querySelector('#world-map-wrap');
    if (!wrap) return;
    wrap.innerHTML = renderSvg(snap.indices, worldPath || '');
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
        scales: { x: { time: false,}, y: { auto: true } },
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
              splits: () => xs,
              incrs: [1],
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
        scales: { x: { time: false,}, y: { auto: true } },
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
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, upY, downY, zero], el);
}

function renderSvg(pins, worldPath) {
    // Per-pin label boxes — start at default position above-right of pin,
    // then run collision resolution + leader-line layout. Everything renders
    // inside the SVG so positions are in the same coordinate system as the
    // map, no HTML overlay math, and labels scale with the viewport.
    const CHAR_PX  = 6.0;   // approx mono char width @ 12px font in SVG
    const PAD_X    = 12;
    const BOX_H    = 22;
    const DOT_R    = 4;
    const LEAD_GAP = 6;     // gap between dot and leader line start

    const boxes = pins.map(p => {
        const [px, py] = project(p.lat, p.lng);
        // Text content drives label width — flag + label + pct.
        const text  = `${p.flag} ${p.label} ${fmtPct(p.change_pct)}`;
        // Flag emoji counts as ~2 chars visually; approximate.
        const w = Math.ceil(text.length * CHAR_PX) + PAD_X + 6;
        const h = BOX_H;
        // Default anchor: just above and right of pin.
        let lx = px + 12;
        let ly = py - h - 6;
        // Clamp to viewport so labels don't get pushed off the map.
        lx = Math.max(2, Math.min(WIDTH - w - 2, lx));
        ly = Math.max(2, Math.min(HEIGHT - h - 2, ly));
        return { p, px, py, lx, ly, w, h, text };
    });

    resolveLabelCollisions(boxes);

    const dots = boxes.map(b =>
        `<circle cx="${b.px.toFixed(1)}" cy="${b.py.toFixed(1)}" r="${DOT_R}"
                 fill="${b.p.change_pct >= 0 ? '#23d160' : '#ff3860'}"
                 stroke="#0a0a12" stroke-width="2"/>`
    ).join('');

    // Leader line: from dot edge to nearest corner/edge of label box.
    // Computed per box so the arrow always lands on the right side of the
    // label closest to the pin.
    const leaders = boxes.map(b => leaderPath(b, LEAD_GAP)).join('');

    const labelMarkup = boxes.map((b) => {
        const cls = b.p.change_pct >= 0 ? 'wm-pct-pos' : 'wm-pct-neg';
        const cxText = (b.lx + 6).toFixed(1);
        const cyText = (b.ly + b.h / 2 + 4).toFixed(1);  // +4 for baseline
        return `
            <g class="wm-label" pointer-events="auto">
                <rect x="${b.lx.toFixed(1)}" y="${b.ly.toFixed(1)}"
                      width="${b.w}" height="${b.h}" rx="3"
                      fill="rgba(8,12,20,0.92)"
                      stroke="rgba(0,229,255,0.35)" stroke-width="0.6"/>
                <text x="${cxText}" y="${cyText}"
                      class="wm-text"
                      font-family="'JetBrains Mono', monospace"
                      font-size="11">${esc(b.p.flag)} ${esc(b.p.label)}<tspan class="${cls}"> ${esc(fmtPct(b.p.change_pct))}</tspan></text>
            </g>`;
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
                <marker id="wm-arrow" viewBox="0 0 10 10" refX="8" refY="5"
                        markerWidth="6" markerHeight="6" orient="auto-start-reverse">
                    <path d="M0,0 L10,5 L0,10 Z" fill="rgba(0,229,255,0.7)"/>
                </marker>
            </defs>
            <rect width="${WIDTH}" height="${HEIGHT}" fill="url(#map-glow)"/>
            ${worldPath ? `<path d="${worldPath}" fill="url(#landmass)"
                                  stroke="rgba(255,255,255,0.12)"
                                  stroke-width="0.3"
                                  fill-rule="evenodd"/>` : ''}
            <g class="wm-leaders">${leaders}</g>
            <g class="wm-dots">${dots}</g>
            <g class="wm-labels">${labelMarkup}</g>
        </svg>
    `;
}

/**
 * Iterative box-push collision resolution. Each pass: detect overlapping
 * label pairs and shove them apart in the axis of largest overlap. Repeats
 * until no overlaps or the iteration cap is hit. Cheap O(N² × iters) for
 * the ~12 index pins we render — well under a millisecond.
 */
function resolveLabelCollisions(boxes) {
    const PAD = 2;
    const MAX_ITERS = 80;
    for (let iter = 0; iter < MAX_ITERS; iter++) {
        let moved = false;
        for (let i = 0; i < boxes.length; i++) {
            for (let j = i + 1; j < boxes.length; j++) {
                const a = boxes[i], b = boxes[j];
                const overlapX = Math.min(a.lx + a.w, b.lx + b.w) - Math.max(a.lx, b.lx);
                const overlapY = Math.min(a.ly + a.h, b.ly + b.h) - Math.max(a.ly, b.ly);
                if (overlapX > 0 && overlapY > 0) {
                    // Push apart along axis of smallest overlap so labels
                    // separate by the minimum distance needed.
                    if (overlapX < overlapY) {
                        const push = (overlapX + PAD) / 2;
                        if (a.lx < b.lx) { a.lx -= push; b.lx += push; }
                        else             { a.lx += push; b.lx -= push; }
                    } else {
                        const push = (overlapY + PAD) / 2;
                        if (a.ly < b.ly) { a.ly -= push; b.ly += push; }
                        else             { a.ly += push; b.ly -= push; }
                    }
                    // Re-clamp to viewport.
                    a.lx = Math.max(2, Math.min(WIDTH - a.w - 2, a.lx));
                    a.ly = Math.max(2, Math.min(HEIGHT - a.h - 2, a.ly));
                    b.lx = Math.max(2, Math.min(WIDTH - b.w - 2, b.lx));
                    b.ly = Math.max(2, Math.min(HEIGHT - b.h - 2, b.ly));
                    moved = true;
                }
            }
        }
        if (!moved) return;
    }
}

/**
 * Leader-line path from the pin dot to the label edge nearest to it.
 * Anchors on the label's closest side (left/right/top/bottom) so the arrow
 * lands cleanly instead of cutting through the box.
 */
function leaderPath(b, gap) {
    // Label center
    const cx = b.lx + b.w / 2;
    const cy = b.ly + b.h / 2;
    // Vector from label center to pin
    const dx = b.px - cx;
    const dy = b.py - cy;
    // Which edge of the label is closest to the pin?
    let ax, ay;
    if (Math.abs(dx) * b.h > Math.abs(dy) * b.w) {
        // Horizontal edge intersection
        ax = dx > 0 ? b.lx + b.w : b.lx;
        ay = cy + dy * ((ax - cx) / dx);
    } else {
        // Vertical edge intersection
        ay = dy > 0 ? b.ly + b.h : b.ly;
        ax = cx + dx * ((ay - cy) / dy);
    }
    // Shorten line on the pin side so the arrow tip lands just outside the dot.
    const len = Math.hypot(b.px - ax, b.py - ay) || 1;
    const ux = (b.px - ax) / len;
    const uy = (b.py - ay) / len;
    const tipX = (b.px - ux * gap).toFixed(1);
    const tipY = (b.py - uy * gap).toFixed(1);
    return `<line x1="${ax.toFixed(1)}" y1="${ay.toFixed(1)}"
                  x2="${tipX}" y2="${tipY}"
                  stroke="rgba(0,229,255,0.6)" stroke-width="0.8"
                  marker-end="url(#wm-arrow)"/>`;
}
