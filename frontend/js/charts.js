// Chart helpers — uPlot wrappers for equity curve, OHLC candles, bars.

import { t } from './i18n.js';
import { esc } from './util.js';

export function equityChart(el, points, opts = {}) {
    if (!window.uPlot) { el.textContent = t('chart.error.uplot_missing'); return; }
    if (!points || !points.length) {
        el.innerHTML = `<div class="boot">${esc(t('chart.empty.equity'))}</div>`;
        return;
    }
    el.innerHTML = '';
    const xs = points.map(p => new Date(p.day).getTime() / 1000);
    const ys = points.map(p => Number(p.cum_net_pnl));
    const dd = points.map(p => Number(p.drawdown ?? 0));
    const w = el.clientWidth || 800;
    const h = opts.height || 280;

    return new window.uPlot({
        title: '',
        width: w,
        height: h,
        scales: { x: { time: true } },
        series: [
            { label: t('chart.series.day') },
            { label: t('chart.series.cum_pnl'), stroke: '#00e5ff', width: 2, fill: 'rgba(0,229,255,0.08)' },
            { label: t('chart.series.drawdown'), stroke: '#ff3860', width: 1 },
        ],
        axes: [
            { stroke: '#aab' },
            { stroke: '#aab' },
        ],
    }, [xs, ys, dd], el);
}

export function ohlcChart(el, bars, marks = [], opts = {}) {
    el.innerHTML = '';
    if (!window.uPlot) { el.textContent = t('chart.error.uplot_missing_short'); return; }
    if (!bars || !bars.length) {
        el.innerHTML = `<div class="boot">${esc(t('chart.empty.bars'))}</div>`;
        return;
    }
    const xs = bars.map(b => new Date(b.bar_time).getTime() / 1000);
    const o = bars.map(b => Number(b.open));
    const hi = bars.map(b => Number(b.high));
    const lo = bars.map(b => Number(b.low));
    const c = bars.map(b => Number(b.close));
    const w = el.clientWidth || 900;
    const h = opts.height || 420;

    const candlePath = (u) => {
        const ctx = u.ctx;
        ctx.save();
        const colorUp = '#23d160';
        const colorDown = '#ff3860';
        const barWidth = Math.max(1, (u.bbox.width / xs.length) * 0.6);
        for (let i = 0; i < xs.length; i++) {
            const x = u.valToPos(xs[i], 'x', true);
            const yOpen = u.valToPos(o[i], 'y', true);
            const yClose = u.valToPos(c[i], 'y', true);
            const yHigh = u.valToPos(hi[i], 'y', true);
            const yLow = u.valToPos(lo[i], 'y', true);
            const up = c[i] >= o[i];
            ctx.strokeStyle = up ? colorUp : colorDown;
            ctx.fillStyle = up ? colorUp : colorDown;
            ctx.lineWidth = 1;
            ctx.beginPath();
            ctx.moveTo(x, yHigh);
            ctx.lineTo(x, yLow);
            ctx.stroke();
            const yTop = Math.min(yOpen, yClose);
            const yBot = Math.max(yOpen, yClose);
            ctx.fillRect(x - barWidth / 2, yTop, barWidth, Math.max(1, yBot - yTop));
        }
        if (marks && marks.length) {
            for (const m of marks) {
                const x = u.valToPos(m.x, 'x', true);
                const y = u.valToPos(m.y, 'y', true);
                ctx.fillStyle = m.color || (m.side === 'buy' ? '#23d160' : '#ff3860');
                ctx.beginPath();
                ctx.moveTo(x, y);
                ctx.lineTo(x - 5, y - 8);
                ctx.lineTo(x + 5, y - 8);
                ctx.closePath();
                ctx.fill();
            }
        }
        ctx.restore();
        return null;
    };

    // Restore-on-open + persist-on-change zoom range. Callers (e.g. the
    // #charts view) pass `initialZoom: [from_ts, to_ts]` to set the visible
    // x range after init, and `onZoomChange([from, to])` to receive
    // debounced range-change events for saving to the user preset.
    const initialZoom = Array.isArray(opts.initialZoom) ? opts.initialZoom : null;
    const onZoomChange = typeof opts.onZoomChange === 'function' ? opts.onZoomChange : null;
    let zoomTimer = null;
    const setScaleHook = onZoomChange ? (u, key) => {
        if (key !== 'x') return;
        const s = u.scales.x;
        if (!s || s.min == null || s.max == null) return;
        if (zoomTimer) clearTimeout(zoomTimer);
        const from = Number(s.min);
        const to = Number(s.max);
        zoomTimer = setTimeout(() => {
            zoomTimer = null;
            if (Number.isFinite(from) && Number.isFinite(to) && to > from) {
                onZoomChange([from, to]);
            }
        }, 350);
    } : null;

    const plot = new window.uPlot({
        title: '',
        width: w,
        height: h,
        scales: { x: { time: true } },
        series: [
            { label: t('chart.series.time') },
            { label: t('chart.series.price'), stroke: 'transparent', paths: candlePath },
        ],
        axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
        ...(setScaleHook ? { hooks: { setScale: [setScaleHook] } } : {}),
    }, [xs, c], el);

    if (initialZoom && initialZoom.length === 2
        && Number.isFinite(initialZoom[0]) && Number.isFinite(initialZoom[1])
        && initialZoom[1] > initialZoom[0]) {
        try { plot.setScale('x', { min: initialZoom[0], max: initialZoom[1] }); }
        catch (_) { /* ignore — initial fit-content kicks in */ }
    }
    return plot;
}

export function barChart(el, labels, values, opts = {}) {
    el.innerHTML = '';
    if (!window.uPlot) { el.textContent = t('chart.error.uplot_missing_short'); return; }
    const xs = labels.map((_, i) => i);
    const vs = values.map(Number);
    const n = xs.length;
    const h = opts.height || 240;
    const barColor = opts.color || '#00e5ff';

    const barsPath = (u) => {
        const ctx = u.ctx;
        ctx.save();
        const bw = Math.max(
            2,
            Math.min((u.bbox.width / Math.max(1, n)) * 0.6, u.bbox.width * 0.18),
        );
        const yZero = u.valToPos(0, 'y', true);
        for (let i = 0; i < n; i++) {
            const x = u.valToPos(xs[i], 'x', true);
            const y = u.valToPos(vs[i], 'y', true);
            ctx.fillStyle = vs[i] >= 0 ? barColor : '#ff3860';
            ctx.fillRect(x - bw / 2, Math.min(yZero, y), bw, Math.abs(y - yZero));
        }
        ctx.restore();
        return null;
    };

    // yKind: 'plain' (default) for VIX/yield/pct — small decimals as-is,
    // big numbers abbreviated to K/M, no currency prefix. 'money' adds the
    // $ for P&L charts. 'count' is the same as 'plain' but always integer.
    // The previous 'money' default was prefixing $ onto VIX index values
    // and treasury yield percentages — wrong for every current caller.
    const yKind = opts.yKind || 'plain';
    const fmtY = (v) => {
        if (v == null || !Number.isFinite(v)) return '';
        const a = Math.abs(v);
        const sign = v < 0 ? '-' : '';
        const prefix = yKind === 'money' ? '$' : '';
        if (a >= 1e6) return `${sign}${prefix}${(a / 1e6).toFixed(a >= 1e7 ? 0 : 1)}M`;
        if (a >= 1e3) return `${sign}${prefix}${(a / 1e3).toFixed(a >= 1e4 ? 0 : 1)}K`;
        if (yKind === 'plain') {
            if (a >= 100) return `${sign}${prefix}${a.toFixed(0)}`;
            if (a >= 10)  return `${sign}${prefix}${a.toFixed(1)}`;
            return `${sign}${prefix}${a.toFixed(2)}`;
        }
        return `${sign}${prefix}${a.toFixed(0)}`;
    };

    let plot = null;
    const measure = () => el.clientWidth || el.getBoundingClientRect().width || 800;
    const buildPlot = (w) => new window.uPlot({
        title: opts.title || '',
        width: Math.max(120, Math.floor(w)),
        height: h,
        // Inset the first/last bars by half a slot on each side so they
        // don't hug the panel edges. Without an explicit range, uPlot's
        // auto-fit puts bar 0 at the left edge and bar n-1 at the right
        // edge — on the 4-point VIX/Treasury curves this read as bars
        // jammed against the panel walls.
        scales: {
            x: { time: false, range: () => [-0.5, Math.max(0.5, n - 0.5)] },
            y: {},
        },
        series: [
            {
                label: t('chart.series.idx'),
                value: (_u, raw) => labels[Math.round(Number(raw))] || '—',
            },
            {
                label: opts.seriesLabel || t('chart.series.value'),
                stroke: 'transparent',
                paths: barsPath,
                points: { show: false },
            },
        ],
        axes: [{
            stroke: '#aab',
            size: 60,
            rotate: -45,
            // Pin ticks at every 0-indexed integer position. With sparse
            // series (4-point VIX/Treasury) uPlot's auto-tick picks
            // 0.5-step increments that all round to duplicate labels.
            splits: () => xs,
            incrs: [1],
            values: (_u, ticks) => ticks.map(v => labels[Math.round(v)] || ''),
        }, {
            stroke: '#aab',
            size: 64,
            values: (_u, ticks) => ticks.map(fmtY),
        }],
        legend: { show: false },
    }, [xs, vs], el);

    // Defer one rAF tick so the panel has its final width before measuring.
    // Reading clientWidth on a freshly-injected child of a flex/grid panel
    // returned 0 in release WebKit, dropping the chart to the 800 fallback
    // and pushing rotated x-axis labels off-canvas — the bug visible in
    // the VIX/Treasury screenshot.
    requestAnimationFrame(() => { plot = buildPlot(measure()); });

    if (typeof ResizeObserver !== 'undefined') {
        const ro = new ResizeObserver(() => {
            if (!plot) return;
            const w = measure();
            if (Math.abs(plot.width - w) < 2) return;
            plot.destroy();
            plot = buildPlot(w);
        });
        ro.observe(el);
    }
}
