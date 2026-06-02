// Multi-chart grid — N synchronized trading charts of the SAME symbol, each
// locked to its own timeframe and keeping its own indicator selection.
//
// One shared symbol input drives every pane: changing the symbol reloads each
// pane's data but leaves its timeframe and selected indicators untouched
// (see createTradingChart.setSymbol, which preserves indicators).
//
// Usage:
//   import { createChartGrid } from '../components/chart_grid.js';
//   const grid = createChartGrid(el, { symbol: 'SPY', layout: '4' });
//   …later…  grid.destroy();

import { createTradingChart } from './trading_chart.js';
import { t } from '../i18n.js';
import { esc } from '../util.js';

// Layout presets: how many columns + the default timeframe of each pane.
// Sub-minute intervals (e.g. 10s) are not offered because the backend only
// buckets bars down to 1m; each pane's own selector still exposes 1m–1w.
const LAYOUTS = {
    '1': { cols: 1, intervals: ['1d'] },
    '2': { cols: 2, intervals: ['5m', '1d'] },
    '4': { cols: 2, intervals: ['1m', '5m', '1h', '1d'] },
};

const LAYOUT_LABEL_KEYS = {
    '1': 'component.grid.layout_1',
    '2': 'component.grid.layout_2',
    '4': 'component.grid.layout_4',
};

export function createChartGrid(container, opts = {}) {
    if (!container) return { destroy() {}, broadcastSymbol() {} };

    const initialSymbol = (opts.symbol || 'SPY').toUpperCase();
    let layoutKey = LAYOUTS[opts.layout] ? String(opts.layout) : '4';
    const cellHeight = opts.cellHeight || 320;
    let panes = [];

    container.innerHTML = `
        <div class="chart-grid-bar">
            <label class="chart-grid-sym">${esc(t('component.grid.symbol'))}
                <input id="cg-sym" value="${esc(initialSymbol)}" autocomplete="off"
                       style="text-transform:uppercase;min-width:140px"></label>
            <button type="button" class="primary" id="cg-load">${esc(t('component.grid.load'))}</button>
            <span class="chart-grid-spacer"></span>
            <div class="chart-grid-layouts" role="group">
                ${['1', '2', '4'].map(k =>
                    `<button type="button" class="cg-layout-btn${k === layoutKey ? ' active' : ''}" data-layout="${k}">${esc(t(LAYOUT_LABEL_KEYS[k]))}</button>`
                ).join('')}
            </div>
        </div>
        <div class="chart-grid" id="cg-grid"></div>
        <p class="muted small">${esc(t('component.grid.hint'))}</p>`;

    const symInput = container.querySelector('#cg-sym');
    const grid = container.querySelector('#cg-grid');

    function currentSymbol() {
        return (symInput.value || '').trim().toUpperCase() || initialSymbol;
    }

    function buildPanes(key) {
        panes.forEach(p => p.destroy());
        panes = [];
        layoutKey = key;
        const def = LAYOUTS[key] || LAYOUTS['4'];
        const sym = currentSymbol();
        grid.style.gridTemplateColumns = `repeat(${def.cols}, minmax(0, 1fr))`;
        grid.innerHTML = '';
        for (const iv of def.intervals) {
            const cell = document.createElement('div');
            cell.className = 'chart-grid-cell';
            grid.appendChild(cell);
            panes.push(createTradingChart(cell, { symbol: sym, interval: iv, height: cellHeight }));
        }
    }

    // Broadcast the symbol to every pane. Each pane keeps its own timeframe +
    // indicators and only reloads its data for the new symbol.
    function broadcastSymbol() {
        const sym = currentSymbol();
        symInput.value = sym;
        panes.forEach(p => p.setSymbol(sym));
    }

    container.querySelector('#cg-load').addEventListener('click', broadcastSymbol);
    symInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') { e.preventDefault(); broadcastSymbol(); }
    });
    container.querySelector('.chart-grid-layouts').addEventListener('click', (e) => {
        const btn = e.target.closest('.cg-layout-btn');
        if (!btn) return;
        container.querySelectorAll('.cg-layout-btn').forEach(b => b.classList.toggle('active', b === btn));
        buildPanes(btn.dataset.layout);
    });

    buildPanes(layoutKey);

    return {
        broadcastSymbol,
        destroy() {
            panes.forEach(p => p.destroy());
            panes = [];
            container.innerHTML = '';
        },
    };
}
