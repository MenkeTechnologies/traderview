import { api } from '../api.js';
import { ohlcChart } from '../charts.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const COLORS = ['#00e5ff', '#ff7a1f', '#7af0a8', '#ff1f7a', '#ffd24a'];
const FIB_LEVELS = [0.0, 0.236, 0.382, 0.5, 0.618, 0.786, 1.0];

export async function renderCharts(mount, _state, symbol = '') {
    const tok = currentViewToken();
    if (!symbol) symbol = 'SPY';
    mount.innerHTML = `
        <h1 data-i18n="view.charts.h1.charts" class="view-title">// CHARTS</h1>
        <div class="chart-toolbar">
            <label><span data-i18n="view.charts.label.symbol">Symbol</span>
                <input id="sym" value="${esc(symbol)}"></label>
            <label><span data-i18n="view.charts.label.interval">Interval</span>
                <select id="iv">
                    <option data-i18n="view.charts.opt.1m" value="1m">1m</option>
                    <option data-i18n="view.charts.opt.5m" value="5m">5m</option>
                    <option data-i18n="view.charts.opt.15m" value="15m">15m</option>
                    <option data-i18n="view.charts.opt.1h" value="1h">1h</option>
                    <option data-i18n="view.charts.opt.1d" value="1d" selected>1d</option>
                    <option data-i18n="view.charts.opt.1w" value="1w">1w</option>
                </select>
            </label>
            <label><span data-i18n="view.charts.label.from">From</span>
                <input type="date" id="from"></label>
            <label><span data-i18n="view.charts.label.to">To</span>
                <input type="date" id="to"></label>
            <button data-i18n="view.charts.btn.load" class="primary" id="load">Load</button>
        </div>

        <div class="chart-toolbar" id="drawToolbar">
            <span class="muted small" data-i18n="view.charts.label.tool">Tool:</span>
            <button data-i18n="view.charts.btn.select" class="btn tool-btn active" data-tool="select">Select</button>
            <button data-i18n="view.charts.btn.trendline" class="btn tool-btn" data-tool="trendline">Trendline</button>
            <button data-i18n="view.charts.btn.h_line" class="btn tool-btn" data-tool="hline">H-line</button>
            <button data-i18n="view.charts.btn.fib" class="btn tool-btn" data-tool="fib">Fib</button>
            <button data-i18n="view.charts.btn.text" class="btn tool-btn" data-tool="text">Text</button>
            <span class="muted small" data-i18n="view.charts.label.color">Color:</span>
            <span id="colorPicker"></span>
            <button data-i18n="view.charts.btn.clear_all" class="btn" id="clearDrawings" style="margin-left:auto;">Clear all</button>
        </div>

        <div class="chart-toolbar">
            <span class="muted small" data-i18n="view.charts.label.indicators">Indicators:</span>
            <select id="indicatorSel" multiple size="3" style="min-width:240px;"></select>
            <button data-i18n="view.charts.btn.apply" class="btn" id="indicatorReload">Apply</button>
            <a href="#custom-indicators" class="small muted">manage…</a>
        </div>

        <div class="chart-panel">
            <div id="chartWrap" style="position:relative;">
                <div id="chart-mount"></div>
                <svg id="drawLayer"
                     style="position:absolute; inset:0; pointer-events:auto; cursor:crosshair;"
                     xmlns="http://www.w3.org/2000/svg"></svg>
            </div>
        </div>
        <p data-i18n="view.charts.hint.trendline_fib_click_two_points_h_line_text_click_o" class="muted small" id="drawHint">
            Trendline/Fib: click two points. H-line/Text: click once. Drawings persist per symbol.
        </p>
    `;

    const to = new Date();
    const from = new Date(to.getTime() - 90 * 86400_000);
    mount.querySelector('#from').value = from.toISOString().slice(0, 10);
    mount.querySelector('#to').value = to.toISOString().slice(0, 10);

    // State for the drawing layer.
    const ds = {
        plot: null,
        symbol: symbol.toUpperCase(),
        tool: 'select',
        color: COLORS[0],
        pending: null,         // first click of a 2-click tool
        drawings: [],
        mount,
        tok,
    };

    renderColorPicker(ds);
    mount.querySelectorAll('.tool-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            ds.tool = btn.dataset.tool;
            ds.pending = null;
            mount.querySelectorAll('.tool-btn').forEach(b => b.classList.toggle('active', b === btn));
            const svg = mount.querySelector('#drawLayer');
            if (svg) svg.style.cursor = ds.tool === 'select' ? 'default' : 'crosshair';
        });
    });
    mount.querySelector('#clearDrawings').addEventListener('click', async () => {
        if (!confirm(t('view.charts.confirm.delete_all_drawings', { symbol: ds.symbol }))) return;
        try {
            await api.deleteChartDrawings(ds.symbol);
            if (!viewIsCurrent(tok)) return;
            ds.drawings = [];
            drawAll(ds);
        } catch (e) { alert(t('common.error', { err: e.message })); }
    });

    mount.querySelector('#drawLayer').addEventListener('click', (e) => onDrawClick(e, ds));

    // Populate indicator selector from registry.
    let allIndicators = [];
    try {
        allIndicators = await api.listCustomIndicators();
        if (!viewIsCurrent(tok)) return;
        const sel = mount.querySelector('#indicatorSel');
        if (sel) sel.innerHTML = allIndicators.map(i =>
            `<option value="${i.id}" ${i.is_default ? 'selected' : ''}>${esc(i.name)}</option>`
        ).join('');
    } catch (_) { /* ignore */ }
    ds.indicatorSeries = [];

    const load = async () => {
        const sym = mount.querySelector('#sym').value.trim().toUpperCase();
        ds.symbol = sym;
        const iv = mount.querySelector('#iv').value;
        const f = Math.floor(new Date(mount.querySelector('#from').value).getTime() / 1000);
        const t = Math.floor(new Date(mount.querySelector('#to').value).getTime() / 1000) + 86400;
        try {
            const resp = await api.bars(sym, iv, f, t);
            if (!viewIsCurrent(tok)) return;
            const cm = mount.querySelector('#chart-mount');
            if (!cm) return;
            ds.plot = ohlcChart(cm, resp.bars, [], { height: 480 });
            sizeOverlay(ds);
            ds.drawings = await api.listChartDrawings(sym);
            if (!viewIsCurrent(tok)) return;
            await loadIndicators(ds, sym, iv);
            if (!viewIsCurrent(tok)) return;
            drawAll(ds);
        } catch (e) {
            if (!viewIsCurrent(tok)) return;
            const cm = mount.querySelector('#chart-mount');
            if (cm) cm.innerHTML = `<div class="boot">Error: ${e.message}</div>`;
        }
    };

    mount.querySelector('#load').addEventListener('click', load);
    mount.querySelector('#indicatorReload').addEventListener('click', async () => {
        const sym = mount.querySelector('#sym').value.trim().toUpperCase();
        const iv = mount.querySelector('#iv').value;
        await loadIndicators(ds, sym, iv);
        if (!viewIsCurrent(tok)) return;
        drawAll(ds);
    });
    // Self-cleaning resize listener: once a later navigation bumps the
    // view token, the handler removes itself so multiple charts visits
    // don't accumulate parallel listeners on the window.
    const onResize = () => {
        if (!viewIsCurrent(tok)) {
            window.removeEventListener('resize', onResize);
            return;
        }
        sizeOverlay(ds); drawAll(ds);
    };
    window.addEventListener('resize', onResize);
    load();
}

async function loadIndicators(ds, sym, iv) {
    const sel = ds.mount.querySelector('#indicatorSel');
    if (!sel) { ds.indicatorSeries = []; return; }
    const ids = Array.from(sel.selectedOptions).map(o => o.value);
    if (!ids.length) { ds.indicatorSeries = []; return; }
    try {
        const days = Math.max(30, Math.ceil(
            (Number(ds.mount.querySelector('#to').valueAsNumber || Date.now()) -
             Number(ds.mount.querySelector('#from').valueAsNumber || Date.now())) / 86400_000));
        const r = await api.evalCustomIndicators(sym, iv, days, ids);
        if (!viewIsCurrent(ds.tok)) return;
        ds.indicatorSeries = (r.series || []).map(s => ({
            name: s.name, color: s.color, values: s.values, times: r.times,
        }));
    } catch (e) {
        console.warn('indicator eval failed', e);
        ds.indicatorSeries = [];
    }
}

function renderColorPicker(ds) {
    const el = ds.mount.querySelector('#colorPicker');
    if (!el) return;
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
    const wrap = ds.mount.querySelector('#chartWrap');
    const svg = ds.mount.querySelector('#drawLayer');
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
        const text = prompt(t('view.charts.prompt.text'), '');
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
        if (!viewIsCurrent(ds.tok)) return;
        ds.drawings.push(saved);
        drawAll(ds);
    } catch (e) { alert(t('common.error', { err: e.message })); }
}

function drawAll(ds, opts = {}) {
    const svg = ds.mount.querySelector('#drawLayer');
    if (!svg) return;
    svg.innerHTML = '';
    renderIndicators(svg, ds);
    for (const d of ds.drawings) renderOne(svg, ds, d);
    // Live preview of the in-flight 2-click drawing.
    if (opts.preview) renderPreview(svg, ds, opts.preview);
}

function renderIndicators(svg, ds) {
    if (!ds.plot || !ds.indicatorSeries?.length) return;
    let legendY = 14;
    for (const s of ds.indicatorSeries) {
        const pts = [];
        const n = Math.min(s.times.length, s.values.length);
        for (let i = 0; i < n; i++) {
            const v = s.values[i];
            if (v == null) continue;
            const t = new Date(s.times[i]).getTime() / 1000;
            const x = ds.plot.valToPos(t, 'x');
            const y = ds.plot.valToPos(v, 'y');
            if (!Number.isFinite(x) || !Number.isFinite(y)) continue;
            pts.push(pts.length === 0 ? `M${x.toFixed(1)},${y.toFixed(1)}` : `L${x.toFixed(1)},${y.toFixed(1)}`);
        }
        if (!pts.length) continue;
        const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
        path.setAttribute('d', pts.join(' '));
        path.setAttribute('stroke', s.color || '#00e5ff');
        path.setAttribute('stroke-width', '1.5');
        path.setAttribute('fill', 'none');
        path.setAttribute('opacity', '0.85');
        svg.appendChild(path);
        // Legend chip.
        const t = document.createElementNS('http://www.w3.org/2000/svg', 'text');
        t.setAttribute('x', '8'); t.setAttribute('y', String(legendY));
        t.setAttribute('fill', s.color); t.setAttribute('font-size', '11');
        t.textContent = s.name;
        svg.appendChild(t);
        legendY += 14;
    }
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
            if (!viewIsCurrent(ds.tok)) return;
            ds.drawings = ds.drawings.filter(x => x.id !== d.id);
            drawAll(ds);
        } catch (err) { alert(t('common.error', { err: err.message })); }
    });
    svg.appendChild(g);
}
function textWidth(s) { return s.length * 6.5; }
