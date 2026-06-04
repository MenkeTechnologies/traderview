// Reusable trading chart component (TradingView Lightweight Charts wrapper).
//
// Features:
//   • Candlestick + volume rendering from the backend `/bars` endpoint.
//   • Selectable timeframe (1m / 5m / 15m / 1h / 1d / 1w) via a toolbar.
//   • Zoom in / out / reset buttons (mouse-wheel zoom + drag-pan are native).
//   • Right-click (or the "Indicators" button) opens a context menu to toggle
//     technical indicators, fetched live from the per-symbol indicator routes
//     (`/bars/:sym/sma`, `/ema`, `/rsi`, …) and overlaid on the chart.
//
// Usage:
//   import { createTradingChart } from '../components/trading_chart.js';
//   const chart = createTradingChart(el, { symbol: 'AAPL', interval: '1d' });
//   …later…  chart.setSymbol('NVDA');  chart.destroy();
//
// NOTE: the backend only buckets bars at 1m / 5m / 15m / 1h / 1d / 1w. Sub-minute
// intervals (e.g. 10s) are not available until the bar pipeline supports them.

import { api } from '../api.js';
import { t } from '../i18n.js';
import { esc } from '../util.js';

// Supported intervals → how far back to load (seconds) for a useful default view.
const TIMEFRAMES = ['1m', '5m', '15m', '1h', '1d', '1w'];
const DAY = 86400;
const WINDOW_SECONDS = {
    '1m': 2 * DAY,
    '5m': 5 * DAY,
    '15m': 10 * DAY,
    '1h': 60 * DAY,
    '1d': 730 * DAY,
    '1w': 8 * 365 * DAY,
};

// Overlay indicators share the scalar `{ t:[iso], v:[num|null] }` shape and draw
// on the main price scale. Oscillators draw on a dedicated bottom scale.
const OVERLAY_INDICATORS = [
    { id: 'sma20', apiFn: 'indSma',  q: { period: 20 }, color: '#f5a623', labelKey: 'component.chart.ind.sma', period: 20 },
    { id: 'sma50', apiFn: 'indSma',  q: { period: 50 }, color: '#f78fb3', labelKey: 'component.chart.ind.sma', period: 50 },
    { id: 'ema20', apiFn: 'indEma',  q: { period: 20 }, color: '#00e5ff', labelKey: 'component.chart.ind.ema', period: 20 },
    { id: 'ema50', apiFn: 'indEma',  q: { period: 50 }, color: '#bd10e0', labelKey: 'component.chart.ind.ema', period: 50 },
    { id: 'wma20', apiFn: 'indWma',  q: { period: 20 }, color: '#7ed321', labelKey: 'component.chart.ind.wma', period: 20 },
    { id: 'hma20', apiFn: 'indHull', q: { period: 20 }, color: '#ff6b6b', labelKey: 'component.chart.ind.hma', period: 20 },
];
const OSCILLATOR_INDICATORS = [
    { id: 'rsi14', apiFn: 'indRsi', q: { period: 14 }, color: '#facc15', labelKey: 'component.chart.ind.rsi', period: 14, scaleId: 'rsi' },
];

function toEpochSec(iso) { return Math.floor(new Date(iso).getTime() / 1000); }

// Bars → Lightweight Charts series data. Times must be ascending + unique.
function toCandleData(bars) {
    const out = [];
    let last = -Infinity;
    for (const b of bars) {
        const time = toEpochSec(b.bar_time);
        if (time <= last) continue;
        last = time;
        out.push({ time, open: +b.open, high: +b.high, low: +b.low, close: +b.close });
    }
    return out;
}

function toVolumeData(bars) {
    const out = [];
    let last = -Infinity;
    for (const b of bars) {
        const time = toEpochSec(b.bar_time);
        if (time <= last) continue;
        last = time;
        const up = +b.close >= +b.open;
        out.push({ time, value: +b.volume, color: up ? 'rgba(35,209,96,0.45)' : 'rgba(255,56,96,0.45)' });
    }
    return out;
}

// ScalarSeries { t, v } → line data, dropping null warmup points.
function toLineData(series) {
    const out = [];
    const ts = series.t || [];
    const vs = series.v || [];
    let last = -Infinity;
    for (let i = 0; i < ts.length; i++) {
        const val = vs[i];
        if (val == null) continue;
        const time = toEpochSec(ts[i]);
        if (time <= last) continue;
        last = time;
        out.push({ time, value: Number(val) });
    }
    return out;
}

export function createTradingChart(container, opts = {}) {
    if (!container) return { destroy() {}, setSymbol() {}, setInterval() {} };

    const state = {
        symbol: (opts.symbol || '').toUpperCase(),
        interval: TIMEFRAMES.includes(opts.interval) ? opts.interval : '1d',
        height: opts.height || 440,
        bars: [],
        active: new Map(),   // indicator id → { series, def }
        showVolume: opts.volume !== false,
        destroyed: false,
        loadSeq: 0,
        // Caller-supplied callbacks for chart-grid → preset persistence.
        // `onIndicatorsChange(ids)` fires whenever the active indicator
        // set mutates; `onZoomChange(symbol, [from, to])` fires when the
        // visible time-range moves (pan or zoom). Both default to no-op.
        onIndicatorsChange: typeof opts.onIndicatorsChange === 'function'
            ? opts.onIndicatorsChange : () => {},
        onZoomChange: typeof opts.onZoomChange === 'function'
            ? opts.onZoomChange : () => {},
        onIntervalChange: typeof opts.onIntervalChange === 'function'
            ? opts.onIntervalChange : () => {},
        // Per-symbol saved zoom range (seconds since epoch). Optional;
        // when present, the chart restores it after `loadBars()` completes.
        savedZoomBySymbol: opts.savedZoomBySymbol || {},
    };

    const timeframes = (opts.timeframes || TIMEFRAMES).filter(tf => TIMEFRAMES.includes(tf));

    // ---- DOM scaffold (built with t(); not data-i18n so it survives view re-render) ----
    container.innerHTML = `
        <div class="tv-chart">
            <div class="tv-chart-toolbar">
                <span class="tv-chart-symbol">${esc(state.symbol)}</span>
                <div class="tv-chart-tfs" role="group">
                    ${timeframes.map(tf => `<button type="button" class="tv-tf-btn${tf === state.interval ? ' active' : ''}" data-tf="${tf}">${tf}</button>`).join('')}
                </div>
                <div class="tv-chart-actions">
                    <button type="button" class="tv-chart-icon" data-act="zoom-in"  title="${esc(t('component.chart.zoom_in'))}">+</button>
                    <button type="button" class="tv-chart-icon" data-act="zoom-out" title="${esc(t('component.chart.zoom_out'))}">−</button>
                    <button type="button" class="tv-chart-icon" data-act="reset"    title="${esc(t('component.chart.reset'))}">⟳</button>
                    <button type="button" class="tv-chart-ind"  data-act="indicators">${esc(t('component.chart.indicators'))} ▾</button>
                </div>
            </div>
            <div class="tv-chart-canvas"></div>
            <div class="tv-chart-status"></div>
        </div>`;

    const canvasEl = container.querySelector('.tv-chart-canvas');
    const statusEl = container.querySelector('.tv-chart-status');
    const symbolEl = container.querySelector('.tv-chart-symbol');
    const setStatus = (txt) => { if (statusEl) statusEl.textContent = txt || ''; };

    if (!window.LightweightCharts) {
        canvasEl.innerHTML = `<div class="boot">${esc(t('component.chart.error.lib_missing'))}</div>`;
        return { destroy() {}, setSymbol() {}, setInterval() {} };
    }

    const LWC = window.LightweightCharts;
    const chart = LWC.createChart(canvasEl, {
        height: state.height,
        layout: { background: { type: 'solid', color: 'transparent' }, textColor: '#aab4c2' },
        grid: { vertLines: { color: 'rgba(120,140,160,0.08)' }, horzLines: { color: 'rgba(120,140,160,0.08)' } },
        rightPriceScale: { borderColor: 'rgba(120,140,160,0.25)' },
        timeScale: { borderColor: 'rgba(120,140,160,0.25)', timeVisible: true, secondsVisible: false },
        crosshair: { mode: LWC.CrosshairMode ? LWC.CrosshairMode.Normal : 0 },
        autoSize: false,
    });

    const candleSeries = chart.addCandlestickSeries({
        upColor: '#23d160', downColor: '#ff3860',
        borderVisible: false, wickUpColor: '#23d160', wickDownColor: '#ff3860',
    });

    let volumeSeries = null;
    function ensureVolumeSeries() {
        if (volumeSeries) return volumeSeries;
        volumeSeries = chart.addHistogramSeries({
            priceFormat: { type: 'volume' },
            priceScaleId: 'vol',
            lastValueVisible: false,
            priceLineVisible: false,
        });
        chart.priceScale('vol').applyOptions({ scaleMargins: { top: 0.82, bottom: 0 } });
        return volumeSeries;
    }

    // ---- responsive width ----
    const resize = () => {
        if (state.destroyed) return;
        const w = canvasEl.clientWidth || container.clientWidth || 800;
        chart.applyOptions({ width: w });
    };
    const ro = (typeof ResizeObserver !== 'undefined') ? new ResizeObserver(resize) : null;
    if (ro) ro.observe(canvasEl);
    resize();

    // ---- data loading ----
    async function loadBars() {
        if (!state.symbol) { setStatus(''); return; }
        const seq = ++state.loadSeq;
        setStatus(t('component.chart.status.loading'));
        const to = Math.floor(Date.now() / 1000);
        const from = to - (WINDOW_SECONDS[state.interval] || 730 * DAY);
        let resp;
        try {
            resp = await api.bars(state.symbol, state.interval, from, to);
        } catch (e) {
            if (seq !== state.loadSeq || state.destroyed) return;
            setStatus(t('component.chart.status.load_err', { err: e.message }));
            return;
        }
        if (seq !== state.loadSeq || state.destroyed) return;
        state.bars = resp.bars || [];
        if (!state.bars.length) {
            candleSeries.setData([]);
            if (volumeSeries) volumeSeries.setData([]);
            setStatus(t('component.chart.empty.no_bars'));
            return;
        }
        candleSeries.setData(toCandleData(state.bars));
        if (state.showVolume) ensureVolumeSeries().setData(toVolumeData(state.bars));
        else if (volumeSeries) volumeSeries.setData([]);
        // Restore saved zoom for this symbol if we have one, else fit.
        const saved = state.savedZoomBySymbol[state.symbol];
        if (Array.isArray(saved) && saved.length === 2
            && Number.isFinite(saved[0]) && Number.isFinite(saved[1])
            && saved[1] > saved[0]) {
            try {
                chart.timeScale().setVisibleRange({ from: saved[0], to: saved[1] });
            } catch (_) { chart.timeScale().fitContent(); }
        } else {
            chart.timeScale().fitContent();
        }
        setStatus('');
        // Re-fetch any active indicators for the new window.
        for (const { def } of [...state.active.values()]) addIndicator(def, true);
    }

    // ---- indicators ----
    function indicatorWindow() {
        const to = Math.floor(Date.now() / 1000);
        return { interval: state.interval, from: to - (WINDOW_SECONDS[state.interval] || 730 * DAY), to };
    }

    async function addIndicator(def, reload = false) {
        if (!state.symbol) return;
        if (state.active.has(def.id) && !reload) return;
        let series = state.active.get(def.id)?.series;
        if (!series) {
            if (def.scaleId) {
                series = chart.addLineSeries({ color: def.color, lineWidth: 1, priceScaleId: def.scaleId, lastValueVisible: false, priceLineVisible: false });
                chart.priceScale(def.scaleId).applyOptions({ scaleMargins: { top: 0.7, bottom: 0 } });
            } else {
                series = chart.addLineSeries({ color: def.color, lineWidth: 2, lastValueVisible: false, priceLineVisible: false });
            }
            state.active.set(def.id, { series, def });
        }
        const q = { ...indicatorWindow(), ...def.q };
        try {
            const data = await api[def.apiFn](state.symbol, q);
            if (state.destroyed) return;
            series.setData(toLineData(data));
        } catch (e) {
            if (state.destroyed) return;
            setStatus(t('component.chart.status.load_err', { err: e.message }));
        }
    }

    function removeIndicator(id) {
        const entry = state.active.get(id);
        if (!entry) return;
        chart.removeSeries(entry.series);
        state.active.delete(id);
    }

    function toggleIndicator(def) {
        if (state.active.has(def.id)) removeIndicator(def.id);
        else addIndicator(def);
        // Notify the parent (chart-grid → preset persistence) that the
        // active indicator set changed. Sends the resulting list of ids.
        state.onIndicatorsChange([...state.active.keys()]);
    }

    // Public method: apply a list of indicator IDs (idempotent). Used by
    // chart_grid to push the user's saved preset selection down on first
    // render and after pane rebuilds. IDs not in our known catalog are
    // ignored.
    function setIndicators(ids) {
        if (!Array.isArray(ids)) return;
        const want = new Set(ids);
        const have = new Set(state.active.keys());
        const all = [...OVERLAY_INDICATORS, ...OSCILLATOR_INDICATORS];
        // Remove any no-longer-wanted.
        for (const id of have) if (!want.has(id)) removeIndicator(id);
        // Add any newly-wanted.
        for (const id of want) {
            if (have.has(id)) continue;
            const def = all.find(d => d.id === id);
            if (def) addIndicator(def);
        }
    }

    function toggleVolume() {
        state.showVolume = !state.showVolume;
        if (state.showVolume) ensureVolumeSeries().setData(toVolumeData(state.bars));
        else if (volumeSeries) volumeSeries.setData([]);
    }

    // ---- context menu ----
    let menuEl = null;
    function closeMenu() { if (menuEl) { menuEl.remove(); menuEl = null; document.removeEventListener('mousedown', onDocDown, true); } }
    function onDocDown(e) { if (menuEl && !menuEl.contains(e.target)) closeMenu(); }

    function openMenu(x, y) {
        closeMenu();
        const row = (id, label, on) =>
            `<button type="button" class="tv-menu-item${on ? ' on' : ''}" data-ind="${esc(id)}">
                <span class="tv-menu-check">${on ? '✓' : ''}</span>${esc(label)}</button>`;
        menuEl = document.createElement('div');
        menuEl.className = 'tv-chart-menu';
        menuEl.innerHTML = `
            <div class="tv-menu-head">${esc(t('component.chart.menu.title'))}</div>
            <div class="tv-menu-group">${esc(t('component.chart.group.overlays'))}</div>
            ${OVERLAY_INDICATORS.map(d => row(d.id, t(d.labelKey, { period: d.period }), state.active.has(d.id))).join('')}
            <div class="tv-menu-group">${esc(t('component.chart.group.oscillators'))}</div>
            ${OSCILLATOR_INDICATORS.map(d => row(d.id, t(d.labelKey, { period: d.period }), state.active.has(d.id))).join('')}
            <div class="tv-menu-sep"></div>
            ${row('__vol__', t('component.chart.ind.volume'), state.showVolume)}
            <button type="button" class="tv-menu-item tv-menu-clear" data-ind="__clear__">${esc(t('component.chart.menu.clear'))}</button>`;
        document.body.appendChild(menuEl);
        const vw = window.innerWidth, vh = window.innerHeight;
        const rect = menuEl.getBoundingClientRect();
        menuEl.style.left = Math.min(x, vw - rect.width - 8) + 'px';
        menuEl.style.top = Math.min(y, vh - rect.height - 8) + 'px';
        menuEl.addEventListener('click', (e) => {
            const btn = e.target.closest('.tv-menu-item');
            if (!btn) return;
            const id = btn.dataset.ind;
            if (id === '__clear__') { for (const k of [...state.active.keys()]) removeIndicator(k); closeMenu(); return; }
            if (id === '__vol__') { toggleVolume(); }
            else {
                const def = [...OVERLAY_INDICATORS, ...OSCILLATOR_INDICATORS].find(d => d.id === id);
                if (def) toggleIndicator(def);
            }
            closeMenu();
        });
        document.addEventListener('mousedown', onDocDown, true);
    }

    // ---- toolbar + interaction wiring ----
    container.querySelector('.tv-chart-tfs').addEventListener('click', (e) => {
        const btn = e.target.closest('.tv-tf-btn');
        if (!btn) return;
        setInterval_(btn.dataset.tf);
    });
    container.querySelector('.tv-chart-actions').addEventListener('click', (e) => {
        const btn = e.target.closest('[data-act]');
        if (!btn) return;
        const act = btn.dataset.act;
        if (act === 'zoom-in') zoom(0.6);
        else if (act === 'zoom-out') zoom(1.6);
        else if (act === 'reset') chart.timeScale().fitContent();
        else if (act === 'indicators') {
            const r = btn.getBoundingClientRect();
            openMenu(r.left, r.bottom + 4);
        }
    });
    canvasEl.addEventListener('contextmenu', (e) => {
        e.preventDefault();
        openMenu(e.clientX, e.clientY);
    });

    function zoom(factor) {
        const ts = chart.timeScale();
        const r = ts.getVisibleLogicalRange();
        if (!r) return;
        const center = (r.from + r.to) / 2;
        const half = ((r.to - r.from) * factor) / 2;
        ts.setVisibleLogicalRange({ from: center - half, to: center + half });
    }

    function setInterval_(iv) {
        if (!TIMEFRAMES.includes(iv) || iv === state.interval) return;
        state.interval = iv;
        container.querySelectorAll('.tv-tf-btn').forEach(b => b.classList.toggle('active', b.dataset.tf === iv));
        loadBars();
        state.onIntervalChange(iv);
    }

    function setSymbol(sym) {
        const s = (sym || '').toUpperCase();
        if (s === state.symbol) return;
        state.symbol = s;
        if (symbolEl) symbolEl.textContent = s;
        // Keep the selected indicators + timeframe; loadBars() re-fetches the
        // active indicators for the new symbol so nothing else changes.
        loadBars();
    }

    function destroy() {
        state.destroyed = true;
        closeMenu();
        if (ro) ro.disconnect();
        try { chart.remove(); } catch { /* already removed */ }
        container.innerHTML = '';
    }

    // Save the visible range to the parent whenever the user pans/zooms.
    // LightweightCharts fires `subscribeVisibleTimeRangeChange` on every
    // frame during a drag, so debounce to once per 350ms — well under
    // any plausible interaction cadence but still feels responsive.
    let zoomSaveTimer = null;
    try {
        chart.timeScale().subscribeVisibleTimeRangeChange((range) => {
            if (state.destroyed || !range) return;
            const from = Number(range.from);
            const to = Number(range.to);
            if (!Number.isFinite(from) || !Number.isFinite(to) || to <= from) return;
            if (zoomSaveTimer) clearTimeout(zoomSaveTimer);
            zoomSaveTimer = setTimeout(() => {
                zoomSaveTimer = null;
                if (state.destroyed) return;
                state.onZoomChange(state.symbol, [from, to]);
            }, 350);
        });
    } catch (_) { /* zoom persistence not available on this build of LWC */ }

    // Apply preset indicator selection (from chart_grid → user_settings)
    // BEFORE the first bar load so it shows up on the initial render.
    if (Array.isArray(opts.indicatorIds) && opts.indicatorIds.length) {
        setIndicators(opts.indicatorIds);
    }

    loadBars();

    return { setSymbol, setInterval: setInterval_, setIndicators, destroy, chart };
}
