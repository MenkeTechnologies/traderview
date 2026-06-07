// Finviz-style S&P 500 heatmap — color-coded grid by sector.
import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

export async function renderHeatmap(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.heatmap.h1.market_heatmap" class="view-title">// MARKET HEATMAP</h1>
        <p data-i18n="view.heatmap.hint.150_s_p_500_names_grouped_by_gics_sector_colored_b" class="muted small">~150 S&P 500 names grouped by GICS sector, colored by today's % change. Your watchlist symbols add to a "Watchlist" pseudo-sector.</p>
        <div id="hm"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
        <div class="chart-panel">
            <h2 data-i18n="view.heatmap.h2.sector_chart">Avg sector change %</h2>
            <div id="hm-chart" style="width:100%;height:240px"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.heatmap.h2.dispersion_chart">Intra-sector dispersion (stddev of constituent change %)</h2>
            <div id="hm-dispersion-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.heatmap.hint.dispersion" class="muted small">Low dispersion = sector moves in unison (macro/sector rotation). High dispersion = sector is being torn apart by single-name stories (idiosyncratic divergence).</p>
        </div>
    `;
    try {
        const r = await api.heatmap();
        if (!viewIsCurrent(tok)) return;
        renderTiles(r, mount);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#hm');
        if (el) el.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function renderTiles(r, mount) {
    const bySector = new Map();
    for (const t of r.tiles) {
        const arr = bySector.get(t.sector) || [];
        arr.push(t);
        bySector.set(t.sector, arr);
    }
    // Sort sectors by total weight (count for now).
    const sectors = Array.from(bySector.entries())
        .sort((a, b) => b[1].length - a[1].length);

    const html = sectors.map(([sector, tiles]) => {
        const avg = tiles.reduce((a, t) => a + Number(t.change_pct), 0) / tiles.length;
        return `<div class="hm-sector">
            <div class="hm-sector-head">
                <span class="hm-sector-name">${esc(sector)}</span>
                <span class="${avg >= 0 ? 'pos' : 'neg'} small">${avg >= 0 ? '+' : ''}${avg.toFixed(2)}%</span>
            </div>
            <div class="hm-grid">
                ${tiles.sort((a, b) => Number(b.change_pct) - Number(a.change_pct)).map(t => {
                    const pct = Number(t.change_pct);
                    const intensity = Math.min(1, Math.abs(pct) / 4);
                    const color = pct >= 0
                        ? `rgba(35, 209, 96, ${0.15 + intensity * 0.7})`
                        : `rgba(255, 56, 96, ${0.15 + intensity * 0.7})`;
                    return `<a class="hm-tile" href="#research/${encodeURIComponent(t.symbol)}"
                        data-context-scope="symbol-row" data-symbol="${esc(t.symbol)}"
                        style="background:${color}" title="${esc(t.symbol)} · ${fmt(t.price)} · ${pct >= 0 ? '+' : ''}${pct.toFixed(2)}%">
                        <span class="hm-sym">${esc(t.symbol)}</span>
                        <span class="hm-pct">${pct >= 0 ? '+' : ''}${pct.toFixed(2)}%</span>
                    </a>`;
                }).join('')}
            </div>
        </div>`;
    }).join('');

    const el = mount.querySelector('#hm');
    if (el) el.innerHTML = html || '<p data-i18n="view.heatmap.hint.no_quotes_cached_yet_refresh_in_a_minute" class="muted">No quotes cached yet — refresh in a minute.</p>';
    renderSectorChart(sectors);
    renderDispersionChart(sectors);
}

function renderDispersionChart(sectors) {
    const el = document.getElementById('hm-dispersion-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!sectors || !sectors.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.heatmap.empty_dispersion_chart">${esc(t('view.heatmap.empty_dispersion_chart'))}</div>`;
        return;
    }
    const stats = sectors.map(([sector, tiles]) => {
        const vals = tiles.map(ti => Number(ti.change_pct)).filter(Number.isFinite);
        if (vals.length < 2) return { sector, sd: 0 };
        const mean = vals.reduce((a, v) => a + v, 0) / vals.length;
        const variance = vals.reduce((a, v) => a + (v - mean) ** 2, 0) / vals.length;
        return { sector, sd: Math.sqrt(variance) };
    }).filter(s => Number.isFinite(s.sd))
       .sort((a, b) => b.sd - a.sd);
    const labels = stats.map(s => s.sector);
    const ys = stats.map(s => s.sd);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.heatmap.chart.sector_idx') },
            { label: t('view.heatmap.chart.stddev_pct'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 12, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderSectorChart(sectors) {
    const el = document.getElementById('hm-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!sectors || !sectors.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.heatmap.empty_chart">${esc(t('view.heatmap.empty_chart'))}</div>`;
        return;
    }
    const avgs = sectors.map(([sector, tiles]) => ({
        sector,
        avg: tiles.reduce((a, t) => a + Number(t.change_pct), 0) / tiles.length,
    })).filter(s => Number.isFinite(s.avg))
       .sort((a, b) => b.avg - a.avg);
    const labels = avgs.map(s => s.sector);
    const ys = avgs.map(s => s.avg);
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.heatmap.chart.sector_idx') },
            { label: t('view.heatmap.chart.avg_pct'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.heatmap.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ys, zero], el);
}
