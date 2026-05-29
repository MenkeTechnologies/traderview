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

    return new window.uPlot({
        title: '',
        width: w,
        height: h,
        scales: { x: { time: true } },
        series: [
            { label: t('chart.series.time') },
            { label: t('chart.series.price'), stroke: 'transparent', paths: candlePath },
        ],
        axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
    }, [xs, c], el);
}

export function barChart(el, labels, values, opts = {}) {
    el.innerHTML = '';
    if (!window.uPlot) { el.textContent = t('chart.error.uplot_missing_short'); return; }
    const xs = labels.map((_, i) => i);
    const vs = values.map(Number);
    const w = el.clientWidth || 800;
    const h = opts.height || 240;
    const barColor = opts.color || '#00e5ff';

    const barsPath = (u) => {
        const ctx = u.ctx;
        ctx.save();
        const bw = Math.max(2, (u.bbox.width / xs.length) * 0.7);
        const yZero = u.valToPos(0, 'y', true);
        for (let i = 0; i < xs.length; i++) {
            const x = u.valToPos(xs[i], 'x', true);
            const y = u.valToPos(vs[i], 'y', true);
            ctx.fillStyle = vs[i] >= 0 ? barColor : '#ff3860';
            ctx.fillRect(x - bw / 2, Math.min(yZero, y), bw, Math.abs(y - yZero));
        }
        ctx.restore();
        return null;
    };

    new window.uPlot({
        title: opts.title || '',
        width: w,
        height: h,
        scales: { x: {}, y: {} },
        series: [
            { label: t('chart.series.idx') },
            { label: opts.seriesLabel || t('chart.series.value'), stroke: 'transparent', paths: barsPath },
        ],
        axes: [{
            stroke: '#aab',
            values: (_, ticks) => ticks.map(t => labels[Math.round(t)] || ''),
            rotate: -45,
            size: 60,
        }, { stroke: '#aab' }],
    }, [xs, vs], el);
}
