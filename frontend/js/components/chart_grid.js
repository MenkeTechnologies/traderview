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
import { api } from '../api.js';
import { t } from '../i18n.js';
import { esc } from '../util.js';

// Layout presets: how many columns + the default timeframe of each pane.
// Each pane's per-pane toolbar still exposes the full 10s / 1m / 5m / 15m
// / 1h / 1d / 1w set; these are just the starting selection. Defaults match
// Webull's "scalper's multichart" layout (5m macro, 1m entry, 1d context,
// 10s tape) — user changes are sticky via `chart_preset.multichart_intervals`.
const LAYOUTS = {
    '1': { cols: 1, intervals: ['1d'] },
    '2': { cols: 2, intervals: ['5m', '1d'] },
    '4': { cols: 2, intervals: ['5m', '1m', '1d', '10s'] },
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

    // Lazy-loaded chart preset (shared with `#charts` via user_settings).
    // We keep the full settings object so save calls can round-trip every
    // field without clobbering anything else the user changes elsewhere.
    let cachedSettings = null;
    let presetIndicatorIds = [];
    let zoomBySymbol = {};
    // Per-layout saved intervals keyed by layout key ('1' | '2' | '4'). Each
    // value is an array of intervals, one per pane position. Lets the 1-pane
    // and 4-pane layouts remember independent sticky timeframes.
    let intervalsByLayout = {};
    let saveTimer = null;
    function loadPresetAsync() {
        // Snapshot the layout the user is currently on. If they switch
        // layouts (or symbols, which also rebuilds via `broadcastSymbol`)
        // before the preset arrives, we must NOT rebuild — that would
        // stomp the user's newer selection and discard freshly-built
        // panes. The user-action path always re-renders with the same
        // preset state in scope, so we just no-op here.
        const startedAtLayout = layoutKey;
        api.settings().then((s) => {
            cachedSettings = s;
            const p = (s && s.chart_preset) || {};
            if (Array.isArray(p.multichart_indicators)) {
                presetIndicatorIds = p.multichart_indicators.map(String);
            }
            if (p.zoom_by_symbol && typeof p.zoom_by_symbol === 'object') {
                zoomBySymbol = p.zoom_by_symbol;
            }
            if (p.multichart_intervals && typeof p.multichart_intervals === 'object') {
                intervalsByLayout = p.multichart_intervals;
            }
            // Bail if the user changed layouts mid-fetch.
            if (layoutKey !== startedAtLayout) return;
            // If we landed with the layout's default intervals because the
            // preset hadn't arrived yet, rebuild now that it has.
            const saved = intervalsByLayout[layoutKey];
            const def = LAYOUTS[layoutKey] || LAYOUTS['4'];
            const wantRebuild = Array.isArray(saved)
                && saved.length === def.intervals.length
                && saved.some((iv, i) => iv !== def.intervals[i]);
            if (wantRebuild) buildPanes(layoutKey);
            else panes.forEach(p => p.setIndicators && p.setIndicators(presetIndicatorIds));
        }).catch(() => { /* fall back to whatever the pane defaults pick */ });
    }
    function savePresetSoon() {
        if (saveTimer) clearTimeout(saveTimer);
        saveTimer = setTimeout(async () => {
            saveTimer = null;
            try {
                if (!cachedSettings) cachedSettings = await api.settings();
                const next = {
                    ...cachedSettings,
                    chart_preset: {
                        ...((cachedSettings && cachedSettings.chart_preset) || {}),
                        multichart_indicators: presetIndicatorIds,
                        zoom_by_symbol: zoomBySymbol,
                        multichart_intervals: intervalsByLayout,
                    },
                };
                await api.updateSettings(next);
                cachedSettings = next;
            } catch (e) { console.warn('chart preset save failed:', e?.message || e); }
        }, 400);
    }
    loadPresetAsync();

    container.innerHTML = `
        <div class="chart-grid-bar">
            <label class="chart-grid-sym">${esc(t('component.grid.symbol'))}
                <input id="cg-sym" name="symbol" data-symbol-input value="${esc(initialSymbol)}"
                       autocomplete="off"
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
        // Saved per-pane intervals for this layout override the layout's
        // default intervals when present and the right length. Lets a user
        // pin "top-left = 5m, top-right = 1m, bot-left = 1d, bot-right = 1m"
        // and have it survive page reloads.
        const saved = intervalsByLayout[key];
        const useIntervals = (Array.isArray(saved) && saved.length === def.intervals.length)
            ? saved.slice()
            : def.intervals.slice();
        grid.style.gridTemplateColumns = `repeat(${def.cols}, minmax(0, 1fr))`;
        grid.innerHTML = '';
        useIntervals.forEach((iv, idx) => {
            const cell = document.createElement('div');
            cell.className = 'chart-grid-cell';
            grid.appendChild(cell);
            panes.push(createTradingChart(cell, {
                symbol: sym,
                interval: iv,
                height: cellHeight,
                indicatorIds: presetIndicatorIds,
                savedZoomBySymbol: zoomBySymbol,
                // When the user toggles an indicator on ANY pane, mirror
                // the change to the global preset and push the same
                // selection back to the other panes so the 4 panes stay
                // visually in sync.
                onIndicatorsChange: (ids) => {
                    presetIndicatorIds = Array.from(new Set(ids));
                    panes.forEach(p => p.setIndicators && p.setIndicators(presetIndicatorIds));
                    savePresetSoon();
                },
                onZoomChange: (symbol, range) => {
                    if (!symbol || !Array.isArray(range)) return;
                    zoomBySymbol[symbol] = range;
                    savePresetSoon();
                },
                // Each pane reports its own interval change back. We track
                // by position within the current layout, so layout '1', '2',
                // '4' each keep an independent sticky-interval array.
                onIntervalChange: (newIv) => {
                    const cur = (intervalsByLayout[key] || def.intervals).slice();
                    if (cur.length !== def.intervals.length) {
                        cur.length = def.intervals.length;
                        for (let i = 0; i < cur.length; i++) {
                            if (!cur[i]) cur[i] = def.intervals[i];
                        }
                    }
                    cur[idx] = newIv;
                    intervalsByLayout[key] = cur;
                    savePresetSoon();
                },
            }));
        });
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
