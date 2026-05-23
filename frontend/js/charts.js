// uPlot wrappers. uPlot is loaded as IIFE in index.html (window.uPlot).

export function equityChart(container, points) {
    if (!window.uPlot) {
        container.textContent = 'uPlot not loaded — run scripts/vendor-uplot.sh';
        return null;
    }
    if (!points.length) {
        container.textContent = 'No closed trades yet.';
        return null;
    }
    const xs = points.map(p => new Date(p.day).getTime() / 1000);
    const ys = points.map(p => Number(p.cum_net_pnl));

    const opts = {
        width: container.clientWidth || 800,
        height: 320,
        scales: { x: { time: true } },
        series: [
            {},
            {
                label: 'Cumulative P&L',
                stroke: '#00e5ff',
                width: 2,
                fill: 'rgba(0, 229, 255, 0.1)',
            },
        ],
        axes: [
            { stroke: '#778' },
            { stroke: '#778' },
        ],
    };
    return new window.uPlot(opts, [xs, ys], container);
}
