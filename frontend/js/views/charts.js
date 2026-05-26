import { api } from '../api.js';
import { ohlcChart } from '../charts.js';
import { esc } from '../util.js';

const COLORS = ['#00e5ff', '#ff7a1f', '#7af0a8', '#ff1f7a', '#ffd24a'];
const FIB_LEVELS = [0.0, 0.236, 0.382, 0.5, 0.618, 0.786, 1.0];

export async function renderCharts(mount, _state, symbol = '') {
    if (!symbol) symbol = 'SPY';
    mount.innerHTML = `
        <h1 class="view-title">// CHARTS</h1>
        <div class="chart-toolbar">
            <label>Symbol <input id="sym" value="${esc(symbol)}"></label>
            <label>Interval
                <select id="iv">
                    <option value="1m">1m</option>
                    <option value="5m">5m</option>
                    <option value="15m">15m</option>
                    <option value="1h">1h</option>
                    <option value="1d" selected>1d</option>
                    <option value="1w">1w</option>
                </select>
            </label>
            <label>From <input type="date" id="from"></label>
            <label>To <input type="date" id="to"></label>
            <button class="primary" id="load">Load</button>
        </div>

        <div class="chart-toolbar" id="drawToolbar">
            <span class="muted small">Tool:</span>
            <button class="btn tool-btn active" data-tool="select">Select</button>
            <button class="btn tool-btn" data-tool="trendline">Trendline</button>
            <button class="btn tool-btn" data-tool="hline">H-line</button>
            <button class="btn tool-btn" data-tool="fib">Fib</button>
            <button class="btn tool-btn" data-tool="text">Text</button>
            <span class="muted small">Color:</span>
            <span id="colorPicker"></span>
            <button class="btn" id="clearDrawings" style="margin-left:auto;">Clear all</button>
        </div>

        <div class="chart-panel">
            <div id="chartWrap" style="position:relative;">
                <div id="chart-mount"></div>
                <svg id="drawLayer"
                     style="position:absolute; inset:0; pointer-events:auto; cursor:crosshair;"
                     xmlns="http://www.w3.org/2000/svg"></svg>
            </div>
        </div>
        <p class="muted small" id="drawHint">
            Trendline/Fib: click two points. H-line/Text: click once. Drawings persist per symbol.
        </p>
    `;

    const to = new Date();
    const from = new Date(to.getTime() - 90 * 86400_000);
    document.getElementById('from').value = from.toISOString().slice(0, 10);
    document.getElementById('to').value = to.toISOString().slice(0, 10);

    // State for the drawing layer.
    const ds = {
        plot: null,
        symbol: symbol.toUpperCase(),
        tool: 'select',
        color: COLORS[0],
        pending: null,         // first click of a 2-click tool
        drawings: [],
    };

    renderColorPicker(ds);
    document.querySelectorAll('.tool-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            ds.tool = btn.dataset.tool;
            ds.pending = null;
            document.querySelectorAll('.tool-btn').forEach(b => b.classList.toggle('active', b === btn));
            const svg = document.getElementById('drawLayer');
            svg.style.cursor = ds.tool === 'select' ? 'default' : 'crosshair';
        });
    });
    document.getElementById('clearDrawings').addEventListener('click', async () => {
        if (!confirm(`Delete ALL drawings on ${ds.symbol}?`)) return;
        try {
            await api.deleteChartDrawings(ds.symbol);
            ds.drawings = [];
            drawAll(ds);
        } catch (e) { alert(e.message); }
    });

    document.getElementById('drawLayer').addEventListener('click', (e) => onDrawClick(e, ds));

    const load = async () => {
        const sym = document.getElementById('sym').value.trim().toUpperCase();
        ds.symbol = sym;
        const iv = document.getElementById('iv').value;
        const f = Math.floor(new Date(document.getElementById('from').value).getTime() / 1000);
        const t = Math.floor(new Date(document.getElementById('to').value).getTime() / 1000) + 86400;
        try {
            const resp = await api.bars(sym, iv, f, t);
            ds.plot = ohlcChart(document.getElementById('chart-mount'), resp.bars, [], { height: 480 });
            sizeOverlay(ds);
            ds.drawings = await api.listChartDrawings(sym);
            drawAll(ds);
        } catch (e) {
            document.getElementById('chart-mount').innerHTML =
                `<div class="boot">Error: ${e.message}</div>`;
        }
    };

    document.getElementById('load').addEventListener('click', load);
    window.addEventListener('resize', () => { sizeOverlay(ds); drawAll(ds); });
    load();
}

function renderColorPicker(ds) {
    const el = document.getElementById('colorPicker');
    el.innerHTML = COLORS.map(c =>
        `<button type="button" class="scheme-swatch" data-color="${c}"
                 style="background:${c};width:18px;height:18px;border:2px solid ${ds.color === c ? '#fff' : 'transparent'};border-radius:50%;margin-right:4px;cursor:pointer;"></button>`
    ).join('');
    el.querySelectorAll('button[data-color]').forEach(b => {
        b.addEventListener('click', () => {
            ds.color = b.dataset.color;
            renderColorPicker(ds);
        });
    });
}

function sizeOverlay(ds) {
    const wrap = document.getElementById('chartWrap');
    const svg = document.getElementById('drawLayer');
    if (!wrap || !svg) return;
    const w = wrap.clientWidth;
    const h = wrap.clientHeight;
    svg.setAttribute('viewBox', `0 0 ${w} ${h}`);
    svg.setAttribute('width', w);
    svg.setAttribute('height', h);
}

function uPlotToCanvas(ds, px, py) {
    // (px, py) are relative to the SVG which sits on top of the wrap.
    // uPlot's valToPos returns canvas coords inside its own root element,
    // which is positioned at the chart-mount inside the wrap. We approximate
    // by treating wrap = uPlot root (both pin to 0,0).
    return { x: px, y: py };
}

function pxToVal(ds, px, py) {
    if (!ds.plot) return null;
    const u = ds.plot;
    const t = u.posToVal(px, 'x');
    const p = u.posToVal(py, 'y');
    return { t, p };
}

function valToPx(ds, t, p) {
    if (!ds.plot) return null;
    const u = ds.plot;
    return { x: u.valToPos(t, 'x'), y: u.valToPos(p, 'y') };
}

async function onDrawClick(e, ds) {
    if (!ds.plot) return;
    const svg = e.currentTarget;
    const rect = svg.getBoundingClientRect();
    const px = e.clientX - rect.left;
    const py = e.clientY - rect.top;
    const val = pxToVal(ds, px, py);
    if (!val) return;
    if (ds.tool === 'select') return;

    if (ds.tool === 'hline') {
        await persistAndAdd(ds, {
            kind: 'hline',
            points: [{ price: val.p }],
            color: ds.color,
            label: val.p.toFixed(2),
        });
        return;
    }
    if (ds.tool === 'text') {
        const text = prompt('Text:', '');
        if (!text) return;
        await persistAndAdd(ds, {
            kind: 'text',
            points: [{ time: val.t, price: val.p }],
            color: ds.color,
            label: text,
        });
        return;
    }
    // Two-click tools (trendline, fib).
    if (!ds.pending) {
        ds.pending = val;
        drawAll(ds, { preview: { from: val, kind: ds.tool } });
        return;
    }
    const first = ds.pending;
    ds.pending = null;
    await persistAndAdd(ds, {
        kind: ds.tool,
        points: [
            { time: first.t, price: first.p },
            { time: val.t,   price: val.p },
        ],
        color: ds.color,
        label: null,
    });
}

async function persistAndAdd(ds, draft) {
    try {
        const saved = await api.createChartDrawing(ds.symbol, draft);
        ds.drawings.push(saved);
        drawAll(ds);
    } catch (e) { alert(e.message); }
}

function drawAll(ds, opts = {}) {
    const svg = document.getElementById('drawLayer');
    if (!svg) return;
    svg.innerHTML = '';
    for (const d of ds.drawings) renderOne(svg, ds, d);
    // Live preview of the in-flight 2-click drawing.
    if (opts.preview) renderPreview(svg, ds, opts.preview);
}

function renderOne(svg, ds, d) {
    const stroke = d.color || '#00e5ff';
    if (d.kind === 'hline') {
        const price = d.points[0].price;
        const pos = ds.plot ? ds.plot.valToPos(price, 'y') : null;
        if (pos == null) return;
        const w = svg.clientWidth || svg.getBoundingClientRect().width;
        appendLine(svg, 0, pos, w, pos, stroke, 1.5, d.id);
        appendText(svg, 4, pos - 4, d.label || price.toFixed(2), stroke, d.id);
        appendDeleteBtn(svg, ds, d, w - 14, pos);
        return;
    }
    if (d.kind === 'text') {
        const p = d.points[0];
        const pos = valToPx(ds, p.time, p.price);
        if (!pos) return;
        appendText(svg, pos.x + 4, pos.y - 4, d.label || '', stroke, d.id);
        appendCircle(svg, pos.x, pos.y, 4, stroke, d.id);
        appendDeleteBtn(svg, ds, d, pos.x + textWidth(d.label || '') + 10, pos.y - 6);
        return;
    }
    if (d.kind === 'trendline' && d.points.length >= 2) {
        const a = valToPx(ds, d.points[0].time, d.points[0].price);
        const b = valToPx(ds, d.points[1].time, d.points[1].price);
        if (!a || !b) return;
        appendLine(svg, a.x, a.y, b.x, b.y, stroke, 2, d.id);
        appendCircle(svg, a.x, a.y, 3, stroke, d.id);
        appendCircle(svg, b.x, b.y, 3, stroke, d.id);
        appendDeleteBtn(svg, ds, d, (a.x + b.x) / 2 + 4, (a.y + b.y) / 2 - 6);
        return;
    }
    if (d.kind === 'fib' && d.points.length >= 2) {
        const hiP = Math.max(d.points[0].price, d.points[1].price);
        const loP = Math.min(d.points[0].price, d.points[1].price);
        const tMin = Math.min(d.points[0].time, d.points[1].time);
        const tMax = Math.max(d.points[0].time, d.points[1].time);
        const xA = ds.plot.valToPos(tMin, 'x');
        const xB = ds.plot.valToPos(tMax, 'x');
        const w  = svg.clientWidth || svg.getBoundingClientRect().width;
        for (const lvl of FIB_LEVELS) {
            const price = hiP - (hiP - loP) * lvl;
            const y = ds.plot.valToPos(price, 'y');
            appendLine(svg, Math.min(xA, 0), y, w, y, stroke, 1, d.id, 0.55);
            appendText(svg, 4, y - 3, `${(lvl * 100).toFixed(1)}% ${price.toFixed(2)}`, stroke, d.id);
        }
        // Bounding box edges.
        appendLine(svg, xA, ds.plot.valToPos(hiP, 'y'), xA, ds.plot.valToPos(loP, 'y'), stroke, 1, d.id, 0.7);
        appendLine(svg, xB, ds.plot.valToPos(hiP, 'y'), xB, ds.plot.valToPos(loP, 'y'), stroke, 1, d.id, 0.7);
        appendDeleteBtn(svg, ds, d, xB - 14, ds.plot.valToPos(hiP, 'y') - 6);
    }
}

function renderPreview(svg, ds, prev) {
    const a = valToPx(ds, prev.from.t, prev.from.p);
    if (!a) return;
    appendCircle(svg, a.x, a.y, 4, '#ffffff');
    appendText(svg, a.x + 6, a.y - 6, `${prev.kind}: click 2nd point`, '#ffffff');
}

// ---- SVG helpers ----------------------------------------------------------

function appendLine(svg, x1, y1, x2, y2, stroke, width = 1, id, opacity = 1) {
    const el = document.createElementNS('http://www.w3.org/2000/svg', 'line');
    el.setAttribute('x1', x1); el.setAttribute('y1', y1);
    el.setAttribute('x2', x2); el.setAttribute('y2', y2);
    el.setAttribute('stroke', stroke);
    el.setAttribute('stroke-width', width);
    el.setAttribute('opacity', opacity);
    if (id) el.dataset.id = id;
    svg.appendChild(el);
}
function appendText(svg, x, y, text, fill, id) {
    const el = document.createElementNS('http://www.w3.org/2000/svg', 'text');
    el.setAttribute('x', x); el.setAttribute('y', y);
    el.setAttribute('fill', fill);
    el.setAttribute('font-size', '11');
    el.setAttribute('font-family', 'Share Tech Mono, monospace');
    el.textContent = text;
    if (id) el.dataset.id = id;
    svg.appendChild(el);
}
function appendCircle(svg, cx, cy, r, fill, id) {
    const el = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
    el.setAttribute('cx', cx); el.setAttribute('cy', cy); el.setAttribute('r', r);
    el.setAttribute('fill', fill);
    if (id) el.dataset.id = id;
    svg.appendChild(el);
}
function appendDeleteBtn(svg, ds, d, x, y) {
    const g = document.createElementNS('http://www.w3.org/2000/svg', 'g');
    g.style.cursor = 'pointer';
    const c = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
    c.setAttribute('cx', x); c.setAttribute('cy', y); c.setAttribute('r', 7);
    c.setAttribute('fill', '#1a1d2e'); c.setAttribute('stroke', '#ff1f7a');
    const t = document.createElementNS('http://www.w3.org/2000/svg', 'text');
    t.setAttribute('x', x); t.setAttribute('y', y + 3);
    t.setAttribute('text-anchor', 'middle');
    t.setAttribute('fill', '#ff1f7a'); t.setAttribute('font-size', '10');
    t.textContent = '×';
    g.appendChild(c); g.appendChild(t);
    g.addEventListener('click', async (e) => {
        e.stopPropagation();
        try {
            await api.deleteChartDrawing(d.id);
            ds.drawings = ds.drawings.filter(x => x.id !== d.id);
            drawAll(ds);
        } catch (err) { alert(err.message); }
    });
    svg.appendChild(g);
}
function textWidth(s) { return s.length * 6.5; }
