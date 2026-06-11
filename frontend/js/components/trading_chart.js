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
// NOTE: the backend buckets bars at 10s / 1m / 5m / 15m / 1h / 1d / 1w. 10s rows
// only land if the live-tick aggregator (Finnhub WS) or a broker CSV import has
// populated them — Yahoo's free chart endpoint floors at 1m. Cold-start 10s
// charts will render empty until ticks accumulate for the symbol.

import { api } from '../api.js';
import { t } from '../i18n.js';
import { esc } from '../util.js';

// Supported intervals → how far back to load (seconds) for a useful default view.
//
// Sized to the data provider's documented intraday history caps so the
// chart pulls the maximum bars the backend can return in one shot,
// instead of a sliver at the right edge.
//   * Yahoo Finance: 1m → 7d max, 5m / 15m → 60d max, 1h → 730d max.
//   * Finnhub WS feeds the 10s aggregation in real time; one trading
//     day (~24h) is the useful window before the canvas chokes.
// 1d / 1w stay at "since 1970" so a full ticker history (IPO → now)
// loads — Yahoo silently caps to the earliest bar, so asking for more
// is free.
const TIMEFRAMES = ['10s', '1m', '5m', '15m', '1h', '1d', '1w'];
const DAY = 86400;
const WINDOW_SECONDS = {
    '10s': 1 * DAY,
    '1m':  7 * DAY,
    '5m':  60 * DAY,
    '15m': 60 * DAY,
    '1h':  730 * DAY,
    '1d':  56 * 365 * DAY,
    '1w':  56 * 365 * DAY,
};

// Indicator catalog. Two response shapes are supported:
//   * Single-line: response is `{ t:[iso], v:[num|null] }`, def has
//     `valueKey: 'v'` (default).
//   * Multi-line: response is `{ t:[iso], <keyA>:[num], <keyB>:[num], ... }`,
//     def has `lines: [{ key, color }, ...]` and one chart series is
//     created per line. Examples: bollinger (upper/middle/lower),
//     macd (line/signal/histogram), stochastic (k/d).
// Oscillators carry `scaleId` so they draw on a separate bottom pane.
const OVERLAY_INDICATORS = [
    // ── Moving averages (main price scale) ─────────────────────────
    { id: 'sma20',  apiFn: 'indSma',  q: { period: 20 },  color: '#f5a623', labelKey: 'component.chart.ind.sma',  period: 20 },
    { id: 'sma50',  apiFn: 'indSma',  q: { period: 50 },  color: '#f78fb3', labelKey: 'component.chart.ind.sma',  period: 50 },
    { id: 'sma200', apiFn: 'indSma',  q: { period: 200 }, color: '#ffe066', labelKey: 'component.chart.ind.sma',  period: 200 },
    { id: 'ema9',   apiFn: 'indEma',  q: { period: 9 },   color: '#9be7ff', labelKey: 'component.chart.ind.ema',  period: 9 },
    { id: 'ema20',  apiFn: 'indEma',  q: { period: 20 },  color: '#00e5ff', labelKey: 'component.chart.ind.ema',  period: 20 },
    { id: 'ema50',  apiFn: 'indEma',  q: { period: 50 },  color: '#bd10e0', labelKey: 'component.chart.ind.ema',  period: 50 },
    { id: 'ema200', apiFn: 'indEma',  q: { period: 200 }, color: '#ff9f43', labelKey: 'component.chart.ind.ema',  period: 200 },
    { id: 'wma20',  apiFn: 'indWma',  q: { period: 20 },  color: '#7ed321', labelKey: 'component.chart.ind.wma',  period: 20 },
    { id: 'hma20',  apiFn: 'indHull', q: { period: 20 },  color: '#ff6b6b', labelKey: 'component.chart.ind.hma',  period: 20 },
    { id: 'dema20', apiFn: 'indDema', q: { period: 20 },  color: '#36c8d4', labelKey: 'component.chart.ind.dema', period: 20 },
    { id: 'tema20', apiFn: 'indTema', q: { period: 20 },  color: '#a4ffd1', labelKey: 'component.chart.ind.tema', period: 20 },
    { id: 'kama20', apiFn: 'indKama', q: { period: 10 },  color: '#e8d595', labelKey: 'component.chart.ind.kama', period: 10 },
    { id: 'frama20', apiFn: 'indFrama', q: { period: 16 }, color: '#ffb86c', labelKey: 'component.chart.ind.frama', period: 16 },
    { id: 'vidya14', apiFn: 'indVidya', q: { period: 14 }, color: '#dc89e0', labelKey: 'component.chart.ind.vidya', period: 14 },
    { id: 'zlema20', apiFn: 'indZlema', q: { period: 20 }, color: '#82e0d4', labelKey: 'component.chart.ind.zlema', period: 20 },
    { id: 'mcginley', apiFn: 'indMcGinleyDynamic', q: { period: 14 }, color: '#c0c5ce', labelKey: 'component.chart.ind.mcginley', period: 14 },
    { id: 'ssmoother', apiFn: 'indSuperSmoother', q: { period: 10 }, color: '#7e8ce0', labelKey: 'component.chart.ind.super_smoother', period: 10 },
    { id: 'decycler', apiFn: 'indEhlersDecycler', q: { period: 60 }, color: '#f88c70', labelKey: 'component.chart.ind.decycler', period: 60 },
    // ── Band / channel overlays. Backend uses Vec<Out> for some of
    //    these — toLineDataKey auto-detects shape. ────────────────────
    { id: 'bb20', apiFn: 'indBollinger', q: { period: 20, k: 2 },
      labelKey: 'component.chart.ind.bbands', period: 20,
      lines: [
          { key: 'upper',  color: '#39ff14' },
          { key: 'middle', color: '#ffffff' },
          { key: 'lower',  color: '#39ff14' },
      ] },
    { id: 'kc20', apiFn: 'indKeltner', q: { period: 20, mult: 2 },
      labelKey: 'component.chart.ind.keltner', period: 20,
      lines: [
          { key: 'upper',  color: '#ff66c4' },
          { key: 'middle', color: '#ffffff' },
          { key: 'lower',  color: '#ff66c4' },
      ] },
    { id: 'donch20', apiFn: 'indDonchian', q: { period: 20 },
      labelKey: 'component.chart.ind.donchian', period: 20,
      lines: [
          { key: 'upper',  color: '#ffd84a' },
          { key: 'middle', color: '#aaa' },
          { key: 'lower',  color: '#ffd84a' },
      ] },
    { id: 'vwap_bands', apiFn: 'indVwapBands', q: { stddev: 2 },
      labelKey: 'component.chart.ind.vwap_bands',
      lines: [
          { key: 'upper',  color: '#36c8d4' },
          { key: 'middle', color: '#ffffff' },
          { key: 'lower',  color: '#36c8d4' },
      ] },
    { id: 'psar', apiFn: 'indParabolicSar', q: { af_start: 0.02, af_increment: 0.02, af_max: 0.2 },
      labelKey: 'component.chart.ind.psar',
      lines: [{ key: 'sar', color: '#ffd84a' }] },
    { id: 'avwap', apiFn: 'indAnchoredVwap', q: {},
      labelKey: 'component.chart.ind.avwap',
      lines: [{ key: 'vwap', color: '#bd10e0' }] },
    // Ichimoku Kinko Hyo — 4-line system, cloud (senkou A/B) shows
    // future support/resistance. Tenkan-sen + Kijun-sen on the price.
    { id: 'ichimoku', apiFn: 'indIchimoku', q: {},
      labelKey: 'component.chart.ind.ichimoku',
      lines: [
          { key: 'tenkan',   color: '#00e5ff' },
          { key: 'kijun',    color: '#ff66c4' },
          { key: 'senkou_a', color: '#39ff14' },
          { key: 'senkou_b', color: '#ff3366' },
      ] },
    // Supertrend — ATR-based trend filter. Single line + colored
    // upper/lower channels.
    { id: 'supertrend', apiFn: 'indSupertrend', q: { period: 10, multiplier: 3 },
      labelKey: 'component.chart.ind.supertrend',
      lines: [
          { key: 'super_trend', color: '#facc15' },
          { key: 'upper_band',  color: '#39ff14' },
          { key: 'lower_band',  color: '#ff3366' },
      ] },
    // BB Squeeze — overlays the Bollinger Bands AND Keltner Channel
    // simultaneously so the squeeze (BB inside KC) is visually obvious.
    { id: 'bb_squeeze', apiFn: 'indBbSqueeze',
      q: { sma_period: 20, ema_period: 20, atr_period: 14, bb_mult: 2, kc_mult: 1.5 },
      labelKey: 'component.chart.ind.bb_squeeze',
      lines: [
          { key: 'bb_upper', color: '#39ff14' },
          { key: 'bb_lower', color: '#39ff14' },
          { key: 'kc_upper', color: '#ff66c4' },
          { key: 'kc_lower', color: '#ff66c4' },
      ] },
];
const OSCILLATOR_INDICATORS = [
    // Each oscillator gets its own `scaleId` so it draws on a
    // dedicated bottom pane (uPlot stacks them).
    { id: 'rsi14',  apiFn: 'indRsi',  q: { period: 14 }, color: '#facc15', labelKey: 'component.chart.ind.rsi',  period: 14, scaleId: 'rsi' },
    { id: 'macd',   apiFn: 'indMacd', q: { fast: 12, slow: 26, signal: 9 }, labelKey: 'component.chart.ind.macd', scaleId: 'macd',
      lines: [
          { key: 'line',      color: '#00e5ff' },
          { key: 'signal',    color: '#ff6b6b' },
          { key: 'histogram', color: '#888' },
      ] },
    // MACD Crossover — same MACD endpoint, but adds ▲/▼ markers at
    // every bullish/bearish line-vs-signal crossing. Independent toggle
    // so users get either the raw lines, the crossover overlay, or
    // both at once.
    { id: 'macd_cross', apiFn: 'indMacd', q: { fast: 12, slow: 26, signal: 9 },
      labelKey: 'component.chart.ind.macd_cross', scaleId: 'macdcross',
      lines: [
          { key: 'line',   color: '#36c8d4' },
          { key: 'signal', color: '#ff9f43' },
      ],
      crossover: {
          fastKey: 'line', slowKey: 'signal',
          markerSeriesIdx: 0,
          upColor: '#39ff14', downColor: '#ff3366',
      } },
    { id: 'ppo', apiFn: 'indPpo', q: { fast: 12, slow: 26, signal: 9 }, labelKey: 'component.chart.ind.ppo', scaleId: 'ppo',
      lines: [
          { key: 'line',      color: '#00e5ff' },
          { key: 'signal',    color: '#ff6b6b' },
          { key: 'histogram', color: '#888' },
      ] },
    { id: 'atr14',  apiFn: 'indAtr',  q: { period: 14 }, color: '#ff9f43', labelKey: 'component.chart.ind.atr',  period: 14, scaleId: 'atr' },
    { id: 'roc14',  apiFn: 'indRoc',  q: { period: 14 }, color: '#7af0a8', labelKey: 'component.chart.ind.roc',  period: 14, scaleId: 'roc' },
    { id: 'trix15', apiFn: 'indTrix', q: { period: 15 }, color: '#bd10e0', labelKey: 'component.chart.ind.trix', period: 15, scaleId: 'trix' },
    { id: 'dpo20',  apiFn: 'indDpo',  q: { period: 20 }, color: '#39ff14', labelKey: 'component.chart.ind.dpo',  period: 20, scaleId: 'dpo' },
    { id: 'coppock', apiFn: 'indCoppock', q: { roc1: 14, roc2: 11, wma: 10 }, color: '#ff66c4', labelKey: 'component.chart.ind.coppock', scaleId: 'coppock' },
    { id: 'vixfix22', apiFn: 'indVixFix', q: { period: 22 }, color: '#ff3366', labelKey: 'component.chart.ind.vix_fix', period: 22, scaleId: 'vixfix' },
    { id: 'laguerre', apiFn: 'indLaguerreRsi', q: { gamma: 0.5 }, color: '#36c8d4', labelKey: 'component.chart.ind.laguerre_rsi', scaleId: 'laguerre' },
    { id: 'adx14',  apiFn: 'indAdx',  q: { period: 14 }, labelKey: 'component.chart.ind.adx', period: 14, scaleId: 'adx',
      lines: [
          { key: 'adx',      color: '#facc15' },
          { key: 'plus_di',  color: '#39ff14' },
          { key: 'minus_di', color: '#ff3366' },
      ] },
    { id: 'stoch',  apiFn: 'indStochastic', q: { k_period: 14, d_period: 3 }, labelKey: 'component.chart.ind.stoch', scaleId: 'stoch',
      lines: [
          { key: 'k', color: '#00e5ff' },
          { key: 'd', color: '#ff9f43' },
      ] },
    { id: 'stochrsi', apiFn: 'indStochRsi', q: { period: 14, k_period: 14, d_period: 3 }, labelKey: 'component.chart.ind.stochrsi', scaleId: 'stochrsi',
      lines: [
          { key: 'raw', color: '#888' },
          { key: 'k',   color: '#00e5ff' },
          { key: 'd',   color: '#ff9f43' },
      ] },
    { id: 'cci20',     apiFn: 'indCci',                  q: { period: 20 },  color: '#bd10e0', labelKey: 'component.chart.ind.cci',  period: 20,  scaleId: 'cci' },
    { id: 'cmo14',     apiFn: 'indCmo',                  q: { period: 14 },  color: '#e83e8c', labelKey: 'component.chart.ind.cmo',  period: 14,  scaleId: 'cmo' },
    { id: 'mfi14',     apiFn: 'indMfi',                  q: { period: 14 },  color: '#7af0a8', labelKey: 'component.chart.ind.mfi',  period: 14,  scaleId: 'mfi' },
    { id: 'williams14', apiFn: 'indWilliamsR',           q: { period: 14 },  color: '#ffb86c', labelKey: 'component.chart.ind.williams_r', period: 14, scaleId: 'williams' },
    { id: 'demarker14', apiFn: 'indDemarker',            q: { period: 14 },  color: '#dc89e0', labelKey: 'component.chart.ind.demarker', period: 14, scaleId: 'demarker' },
    { id: 'tsi',        apiFn: 'indTsi',                 q: { long: 25, short: 13 }, color: '#36c8d4', labelKey: 'component.chart.ind.tsi', scaleId: 'tsi' },
    { id: 'connors_rsi', apiFn: 'indConnorsRsi',          q: { rsi_period: 3, streak_period: 2, lookback: 100 }, color: '#facc15', labelKey: 'component.chart.ind.connors_rsi', scaleId: 'connorsrsi' },
    { id: 'ultimate', apiFn: 'indUltimateOscillator',     q: { p1: 7, p2: 14, p3: 28 }, color: '#39ff14', labelKey: 'component.chart.ind.ultimate', scaleId: 'ultimate' },
    { id: 'awesome', apiFn: 'indAwesomeOscillator',       q: {}, color: '#ff6b6b', labelKey: 'component.chart.ind.awesome', scaleId: 'awesome' },
    { id: 'aroon14',   apiFn: 'indAroon',                 q: { period: 14 }, labelKey: 'component.chart.ind.aroon', period: 14, scaleId: 'aroon',
      lines: [
          { key: 'up',   color: '#39ff14' },
          { key: 'down', color: '#ff3366' },
      ] },
    { id: 'aroon_osc14', apiFn: 'indAroonOscillator',     q: { period: 14 }, color: '#bd10e0', labelKey: 'component.chart.ind.aroon_osc', period: 14, scaleId: 'aroonosc' },
    { id: 'vortex14',  apiFn: 'indVortex',                q: { period: 14 }, labelKey: 'component.chart.ind.vortex', period: 14, scaleId: 'vortex',
      lines: [
          { key: 'vi_plus',  color: '#39ff14' },
          { key: 'vi_minus', color: '#ff3366' },
      ] },
    { id: 'schaff',    apiFn: 'indSchaffTrend',           q: { fast: 23, slow: 50, cycle: 10 }, color: '#7af0a8', labelKey: 'component.chart.ind.schaff', scaleId: 'schaff' },
    { id: 'qqe14',     apiFn: 'indQqe',                   q: { rsi_period: 14, smooth: 5, factor: 4.236 }, labelKey: 'component.chart.ind.qqe', scaleId: 'qqe',
      lines: [
          { key: 'rsi_ma',   color: '#00e5ff' },
          { key: 'fast_atr', color: '#ff9f43' },
      ] },
    { id: 'rvi',       apiFn: 'indRvi',                   q: { period: 10 }, labelKey: 'component.chart.ind.rvi', scaleId: 'rvi',
      lines: [
          { key: 'line',   color: '#00e5ff' },
          { key: 'signal', color: '#ff6b6b' },
      ] },
    { id: 'fisher',    apiFn: 'indFisherTransform',       q: { period: 10 }, labelKey: 'component.chart.ind.fisher', scaleId: 'fisher',
      lines: [
          { key: 'fisher',  color: '#36c8d4' },
          { key: 'trigger', color: '#ff66c4' },
      ] },
    { id: 'mass',      apiFn: 'indMassIndex',             q: { ema_period: 9, sum_period: 25 }, color: '#facc15', labelKey: 'component.chart.ind.mass', scaleId: 'mass' },
    { id: 'vhf',       apiFn: 'indVhf',                   q: { period: 28 }, color: '#bd10e0', labelKey: 'component.chart.ind.vhf', period: 28, scaleId: 'vhf' },
    { id: 'cog',       apiFn: 'indCenterOfGravity',       q: { period: 10 }, color: '#ffb86c', labelKey: 'component.chart.ind.cog', period: 10, scaleId: 'cog' },
    { id: 'eom14',     apiFn: 'indEaseOfMovement',        q: { period: 14 }, color: '#7af0a8', labelKey: 'component.chart.ind.eom', period: 14, scaleId: 'eom' },
    { id: 'chaikinvol', apiFn: 'indChaikinVolatility',    q: { period: 10 }, color: '#dc89e0', labelKey: 'component.chart.ind.chaikinvol', period: 10, scaleId: 'chaikinvol' },
    { id: 'chaikinmf', apiFn: 'indChaikinMoneyFlow',      q: { period: 20 }, color: '#36c8d4', labelKey: 'component.chart.ind.chaikinmf', period: 20, scaleId: 'chaikinmf' },
    { id: 'klinger',   apiFn: 'indKlingerOscillator',     q: { fast: 34, slow: 55, signal: 13 }, color: '#facc15', labelKey: 'component.chart.ind.klinger', scaleId: 'klinger' },
    { id: 'elderforce14', apiFn: 'indElderForce',         q: { period: 13 }, color: '#7af0a8', labelKey: 'component.chart.ind.elder_force', period: 13, scaleId: 'elderforce' },
    { id: 'elderray',  apiFn: 'indElderRay',              q: { period: 13 }, labelKey: 'component.chart.ind.elder_ray', period: 13, scaleId: 'elderray',
      lines: [
          { key: 'bull_power', color: '#39ff14' },
          { key: 'bear_power', color: '#ff3366' },
      ] },
    { id: 'forceindex13', apiFn: 'indForceIndex',         q: { period: 13 }, color: '#ff9f43', labelKey: 'component.chart.ind.force_index', period: 13, scaleId: 'forceindex' },
    { id: 'obv',       apiFn: 'indObv',                   q: {}, color: '#ffd84a', labelKey: 'component.chart.ind.obv',                scaleId: 'obv' },
    { id: 'ad',        apiFn: 'indAccumulationDistribution', q: {}, color: '#bd10e0', labelKey: 'component.chart.ind.ad',                scaleId: 'ad' },
    { id: 'pvt',       apiFn: 'indPvt',                   q: {}, color: '#7af0a8', labelKey: 'component.chart.ind.pvt',                scaleId: 'pvt' },
    { id: 'pvi',       apiFn: 'indPvi',                   q: {}, color: '#36c8d4', labelKey: 'component.chart.ind.pvi',                scaleId: 'pvi' },
    { id: 'nvi',       apiFn: 'indNvi',                   q: {}, color: '#ff6b6b', labelKey: 'component.chart.ind.nvi',                scaleId: 'nvi' },
    { id: 'rvol20',    apiFn: 'indRelativeVolume',        q: { period: 20 }, color: '#facc15', labelKey: 'component.chart.ind.relative_volume', period: 20, scaleId: 'rvol' },
    { id: 'swingidx',  apiFn: 'indSwingIndex',            q: { limit_move: 0.5 }, labelKey: 'component.chart.ind.swing_index', scaleId: 'swingidx',
      lines: [
          { key: 'si',  color: '#00e5ff' },
          { key: 'asi', color: '#ff66c4' },
      ] },
    { id: 'squeezemom', apiFn: 'indBbSqueezeMomentum',    q: { bb_period: 20, kc_period: 20 }, color: '#facc15', labelKey: 'component.chart.ind.squeeze_momentum', valueKey: 'momentum', scaleId: 'squeezemom' },
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
    return toLineDataKey(series, 'v');
}

/**
 * Walk two parallel time-aligned series and emit one Lightweight-Charts
 * marker per crossover. Up-crossover (fast crosses ABOVE slow) → green
 * ▲ below the bar; down-crossover → red ▼ above the bar. Used by the
 * MACD-crossover indicator to surface bull/bear signals visually.
 */
function computeCrossoverMarkers(fast, slow, opts = {}) {
    const out = [];
    const upColor = opts.upColor || '#39ff14';
    const downColor = opts.downColor || '#ff3366';
    // Build a time → slow.value map for O(1) lookup. `fast` and `slow`
    // share the same x-axis (both originate from the same indicator
    // response) but may have different warmup spans.
    const slowByT = new Map();
    for (const p of slow) slowByT.set(p.time, p.value);
    let prevDiff = null;
    for (const p of fast) {
        const s = slowByT.get(p.time);
        if (s == null) continue;
        const diff = p.value - s;
        if (prevDiff != null && Number.isFinite(prevDiff) && Number.isFinite(diff)) {
            if (prevDiff < 0 && diff >= 0) {
                out.push({
                    time: p.time, position: 'belowBar',
                    color: upColor, shape: 'arrowUp',
                });
            } else if (prevDiff > 0 && diff <= 0) {
                out.push({
                    time: p.time, position: 'aboveBar',
                    color: downColor, shape: 'arrowDown',
                });
            }
        }
        prevDiff = diff;
    }
    return out;
}

// Backend ships two shapes for indicator series:
//   * struct-of-arrays: { t:[iso], v:[num|null] } or
//                       { t:[iso], upper:[num], lower:[num], ... }
//   * array-of-objects: [{ t:iso, v:num }, ...] or
//                       [{ t:iso, upper:num, lower:num, ... }, ...]
// toLineDataKey auto-detects which shape it got and extracts the
// `key` column. Lets every multi-line indicator reuse the warmup-skip
// + ascending-time-dedup pass without per-indicator boilerplate.
function toLineDataKey(series, key) {
    const out = [];
    let last = -Infinity;
    if (Array.isArray(series)) {
        for (const row of series) {
            if (!row) continue;
            const val = row[key];
            if (val == null) continue;
            // Backend uses either `t` or `bar_time` for the timestamp
            // column depending on which `#[derive(Serialize)]` struct
            // wraps the indicator's per-bar output. Accept both so we
            // don't fork the catalog per indicator.
            const stamp = row.t ?? row.bar_time;
            const time = toEpochSec(stamp);
            if (!Number.isFinite(time) || time <= last) continue;
            last = time;
            out.push({ time, value: Number(val) });
        }
        return out;
    }
    const ts = series?.t || [];
    const vs = series?.[key] || [];
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
        timeScale: { borderColor: 'rgba(120,140,160,0.25)', timeVisible: true, secondsVisible: state.interval === '10s' },
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
        // Restore saved zoom — keyed BY interval too, so switching from
        // 1m to 15m doesn't replay a tiny 1m window over 60 days of
        // 15m bars (the bug that made the chart show data only at the
        // right edge after we expanded WINDOW_SECONDS). Falls back to
        // fitContent() on first load of a new (symbol, interval) pair.
        const zoomKey = `${state.symbol}:${state.interval}`;
        const saved = state.savedZoomBySymbol[zoomKey] || state.savedZoomBySymbol[state.symbol];
        const firstBar = state.bars[0]?.bar_time ? toEpochSec(state.bars[0].bar_time) : null;
        const lastBar  = state.bars[state.bars.length - 1]?.bar_time ? toEpochSec(state.bars[state.bars.length - 1].bar_time) : null;
        const savedIntersectsData =
            Array.isArray(saved) && saved.length === 2 &&
            Number.isFinite(saved[0]) && Number.isFinite(saved[1]) &&
            saved[1] > saved[0] &&
            firstBar != null && lastBar != null &&
            saved[1] >= firstBar && saved[0] <= lastBar;
        if (savedIntersectsData) {
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

    /**
     * Create chart series for an indicator definition. Single-line
     * defs get one `series`; multi-line defs (Bollinger, MACD, etc.)
     * get one series per `def.lines[i]`. Series live under
     * `entry.allSeries` so removeIndicator can drop them all.
     */
    function mkSeries(def) {
        const baseOpts = (color, width) => def.scaleId
            ? { color, lineWidth: width, priceScaleId: def.scaleId, lastValueVisible: false, priceLineVisible: false }
            : { color, lineWidth: width, lastValueVisible: false, priceLineVisible: false };
        if (def.scaleId) {
            chart.priceScale(def.scaleId).applyOptions({ scaleMargins: { top: 0.7, bottom: 0 } });
        }
        if (Array.isArray(def.lines) && def.lines.length) {
            return def.lines.map(line => chart.addLineSeries(baseOpts(line.color, 1)));
        }
        return [chart.addLineSeries(baseOpts(def.color, 2))];
    }

    async function addIndicator(def, reload = false) {
        if (!state.symbol) return;
        if (state.active.has(def.id) && !reload) return;
        let entry = state.active.get(def.id);
        if (!entry) {
            entry = { allSeries: mkSeries(def), def };
            // Backwards-compat — old call sites may read `entry.series`.
            entry.series = entry.allSeries[0];
            state.active.set(def.id, entry);
        }
        const q = { ...indicatorWindow(), ...def.q };
        try {
            const data = await api[def.apiFn](state.symbol, q);
            if (state.destroyed) return;
            if (Array.isArray(def.lines) && def.lines.length) {
                def.lines.forEach((line, i) => {
                    entry.allSeries[i].setData(toLineDataKey(data, line.key));
                });
            } else {
                // Single-line: default the response key to `v` but let
                // a def override it (e.g. `squeeze_momentum` returns
                // `momentum` instead of `v`).
                entry.allSeries[0].setData(toLineDataKey(data, def.valueKey || 'v'));
            }
            // MACD-style crossover markers — scan two parallel value
            // streams, place a green ▲ where the fast line crosses
            // above slow, red ▼ where it crosses below. Markers attach
            // to whichever series the def names.
            if (def.crossover) {
                const fast = toLineDataKey(data, def.crossover.fastKey);
                const slow = toLineDataKey(data, def.crossover.slowKey);
                const target = entry.allSeries[def.crossover.markerSeriesIdx ?? 0];
                if (target && typeof target.setMarkers === 'function') {
                    target.setMarkers(computeCrossoverMarkers(fast, slow, def.crossover));
                }
            }
        } catch (e) {
            if (state.destroyed) return;
            setStatus(t('component.chart.status.load_err', { err: e.message }));
        }
    }

    function removeIndicator(id) {
        const entry = state.active.get(id);
        if (!entry) return;
        const series = entry.allSeries || (entry.series ? [entry.series] : []);
        for (const s of series) chart.removeSeries(s);
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
        const row = (d, on) => {
            const label = t(d.labelKey, { period: d.period });
            // `data-search` holds a lowercased haystack — id + label + a
            // few common aliases — so the filter input matches typos and
            // acronyms (e.g. "bb" finds bollinger + bb-squeeze).
            const hay = `${d.id} ${label}`.toLowerCase();
            return `<button type="button" class="tv-menu-item${on ? ' on' : ''}"
                            data-ind="${esc(d.id)}" data-search="${esc(hay)}">
                        <span class="tv-menu-check">${on ? '✓' : ''}</span>${esc(label)}
                    </button>`;
        };
        menuEl = document.createElement('div');
        menuEl.className = 'tv-chart-menu';
        menuEl.innerHTML = `
            <div class="tv-menu-head">${esc(t('component.chart.menu.title'))}</div>
            <input type="text" class="tv-menu-filter" autocomplete="off" spellcheck="false"
                   placeholder="${esc(t('component.chart.menu.filter_placeholder'))}"
                   data-no-symbol-list>
            <div class="tv-menu-scroll">
                <div class="tv-menu-group" data-group>${esc(t('component.chart.group.overlays'))}</div>
                ${OVERLAY_INDICATORS.map(d => row(d, state.active.has(d.id))).join('')}
                <div class="tv-menu-group" data-group>${esc(t('component.chart.group.oscillators'))}</div>
                ${OSCILLATOR_INDICATORS.map(d => row(d, state.active.has(d.id))).join('')}
                <div class="tv-menu-sep" data-group></div>
                <button type="button" class="tv-menu-item" data-ind="__vol__"
                        data-search="volume">
                    <span class="tv-menu-check">${state.showVolume ? '✓' : ''}</span>${esc(t('component.chart.ind.volume'))}
                </button>
            </div>
            <button type="button" class="tv-menu-item tv-menu-clear" data-ind="__clear__">${esc(t('component.chart.menu.clear'))}</button>
            <div class="tv-menu-empty" hidden>${esc(t('component.chart.menu.no_matches'))}</div>`;
        document.body.appendChild(menuEl);
        const vw = window.innerWidth, vh = window.innerHeight;
        const rect = menuEl.getBoundingClientRect();
        // Position. With the new `max-height: 80vh` on the menu, `rect.height`
        // can reach 80% of the viewport — `Math.min(y, vh - rect.height - 8)`
        // would go negative and slide the menu off-screen above the visible
        // area, which read as "filter doesn't work" because the input was
        // out of frame. Clamp both axes to `>= 8px` so the menu always
        // lands inside the viewport.
        const left = Math.max(8, Math.min(x, vw - rect.width - 8));
        const top  = Math.max(8, Math.min(y, vh - rect.height - 8));
        menuEl.style.left = left + 'px';
        menuEl.style.top  = top + 'px';

        const filterInput = menuEl.querySelector('.tv-menu-filter');
        const emptyMsg = menuEl.querySelector('.tv-menu-empty');
        // Apply the filter to every `.tv-menu-item` + its preceding
        // group header. Hides the group label whenever every item under
        // it is filtered out.
        const applyFilter = () => {
            const q = filterInput.value.trim().toLowerCase();
            const items = menuEl.querySelectorAll('.tv-menu-item[data-ind]');
            let shown = 0;
            for (const it of items) {
                if (it.classList.contains('tv-menu-clear')) continue;
                const match = !q || (it.dataset.search || '').includes(q);
                it.hidden = !match;
                if (match) shown += 1;
            }
            // Group labels visible only if at least one item below them
            // (until the next group label) is visible.
            const groups = menuEl.querySelectorAll('[data-group]');
            for (const g of groups) {
                let next = g.nextElementSibling;
                let visible = false;
                while (next && !next.hasAttribute('data-group')) {
                    if (next.classList.contains('tv-menu-item') && !next.hidden) {
                        visible = true;
                        break;
                    }
                    next = next.nextElementSibling;
                }
                g.hidden = !visible;
            }
            if (emptyMsg) emptyMsg.hidden = shown !== 0;
        };
        filterInput.addEventListener('input', applyFilter);
        // Esc clears the filter first, then closes if already empty.
        filterInput.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                if (filterInput.value) {
                    filterInput.value = '';
                    applyFilter();
                    e.stopPropagation();
                } else {
                    closeMenu();
                }
            } else if (e.key === 'Enter') {
                // Enter on a filtered list toggles the first visible
                // item — single-keystroke add when the filter narrows
                // to one match.
                const first = menuEl.querySelector('.tv-menu-item[data-ind]:not([hidden])');
                if (first) first.click();
            }
        });

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
        // Focus the filter on open so typing-to-narrow works without
        // a click. rAF so the focus survives the layout flush.
        requestAnimationFrame(() => filterInput.focus());
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
        // Show seconds on the x-axis only for sub-minute resolutions; the
        // 10s bars otherwise collapse into identical "HH:MM" labels.
        chart.timeScale().applyOptions({ secondsVisible: iv === '10s' });
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
                // Persist per `${symbol}:${interval}` so each timeframe
                // has its own zoom memory — switching from 1m to 15m
                // doesn't drag the 1m window across.
                state.onZoomChange(`${state.symbol}:${state.interval}`, [from, to]);
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
