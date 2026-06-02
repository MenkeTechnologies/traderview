// Multi-Chart view — a 2×2 grid of the same symbol at different timeframes.
// Each pane keeps its own timeframe + indicators; one shared symbol box drives
// all panes (changing the symbol leaves timeframes + indicators untouched).

import { createChartGrid } from '../components/chart_grid.js';

export async function renderMultichart(mount, _state, symbol = '') {
    const sym = (symbol || 'SPY').toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.multichart.h1.title">// MULTI-CHART</span></h1>
        <p class="muted small" data-i18n="view.multichart.hint.intro">Four synchronized views of one symbol at different timeframes. Right-click any pane to add indicators — they persist when you change the symbol.</p>
        <div id="mc-grid"></div>`;
    const el = mount.querySelector('#mc-grid');
    if (el) createChartGrid(el, { symbol: sym, layout: '4' });
}
