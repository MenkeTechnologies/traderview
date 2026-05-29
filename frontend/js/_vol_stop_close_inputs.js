// Vol-Stop (close-based) helpers shared by view + vitest.
//
// Backend body shape: identical to chandelier — { bars, atr, side, config }.
// Returns Vec<StopPoint> with {stop_price, triggered}. The difference vs
// chandelier is that vol_stop_close references HIGHEST CLOSE (long) /
// LOWEST CLOSE (short) over the lookback window instead of high / low —
// it ignores intrabar wicks. Less whipsaws on volatile single bars.
//
// We reuse the bar parser + ATR helpers from _chandelier_stop_inputs.

export { parseBarBlob, trueRange, computeAtr, validateInputs, buildBody,
         splitStops, triggerMarkers, summarize, fmtN, fmtPct }
    from './_chandelier_stop_inputs.js';

// Local mirror of crates/traderview-core/src/volatility_stop.rs::vol_stop_close.
// Returns Vec<StopPoint>.
export function localVolStopClose(bars, atr, side, cfg) {
    const n = bars.length;
    const out = new Array(n);
    for (let i = 0; i < n; i++) out[i] = { stop_price: 0, triggered: false };
    if (n < cfg.lookback || cfg.lookback === 0 || atr.length !== n) return out;
    for (let i = cfg.lookback - 1; i < n; i++) {
        const lo = i + 1 - cfg.lookback;
        let stop;
        if (side === 'long') {
            let highestClose = -Infinity;
            for (let j = lo; j <= i; j++) if (bars[j].close > highestClose) highestClose = bars[j].close;
            stop = highestClose - cfg.atr_multiplier * atr[i];
        } else {
            let lowestClose = Infinity;
            for (let j = lo; j <= i; j++) if (bars[j].close < lowestClose) lowestClose = bars[j].close;
            stop = lowestClose + cfg.atr_multiplier * atr[i];
        }
        const triggered = side === 'long'
            ? bars[i].close <= stop
            : bars[i].close >= stop;
        out[i] = { stop_price: stop, triggered };
    }
    return out;
}

// Local mirror of chandelier (highest_HIGH / lowest_LOW) — used for the
// side-by-side comparison panel.
export function localChandelier(bars, atr, side, cfg) {
    const n = bars.length;
    const out = new Array(n);
    for (let i = 0; i < n; i++) out[i] = { stop_price: 0, triggered: false };
    if (n < cfg.lookback || cfg.lookback === 0 || atr.length !== n) return out;
    for (let i = cfg.lookback - 1; i < n; i++) {
        const lo = i + 1 - cfg.lookback;
        let stop;
        if (side === 'long') {
            let highestHigh = -Infinity;
            for (let j = lo; j <= i; j++) if (bars[j].high > highestHigh) highestHigh = bars[j].high;
            stop = highestHigh - cfg.atr_multiplier * atr[i];
        } else {
            let lowestLow = Infinity;
            for (let j = lo; j <= i; j++) if (bars[j].low < lowestLow) lowestLow = bars[j].low;
            stop = lowestLow + cfg.atr_multiplier * atr[i];
        }
        const triggered = side === 'long'
            ? bars[i].low <= stop
            : bars[i].high >= stop;
        out[i] = { stop_price: stop, triggered };
    }
    return out;
}

// Compare last non-warmup stop prices to quantify how much "wick
// protection" the close-based variant gives up to chandelier.
export function compareStops(chand, close) {
    const out = { chandLatest: NaN, closeLatest: NaN, diff: NaN, diffPct: NaN,
                  chandTriggers: 0, closeTriggers: 0, agreement: 0, disagreement: 0 };
    if (!Array.isArray(chand) || !Array.isArray(close)) return out;
    const n = Math.min(chand.length, close.length);
    for (let i = 0; i < n; i++) {
        const c = chand[i], k = close[i];
        if (!c || !k) continue;
        const cWarmup = c.stop_price === 0 && !c.triggered;
        const kWarmup = k.stop_price === 0 && !k.triggered;
        if (cWarmup || kWarmup) continue;
        if (c.triggered) out.chandTriggers++;
        if (k.triggered) out.closeTriggers++;
        if (c.triggered === k.triggered) out.agreement++; else out.disagreement++;
    }
    for (let i = n - 1; i >= 0; i--) {
        const c = chand[i], k = close[i];
        if (!c || !k) continue;
        if ((c.stop_price === 0 && !c.triggered) || (k.stop_price === 0 && !k.triggered)) continue;
        out.chandLatest = c.stop_price;
        out.closeLatest = k.stop_price;
        out.diff = c.stop_price - k.stop_price;
        if (k.stop_price > 0) out.diffPct = (c.stop_price - k.stop_price) / k.stop_price;
        break;
    }
    return out;
}

// 60-bar demos that highlight the methodologies' differences.
export function makeDemoBars(kind = 'wicks') {
    const out = [];
    let price = 100;
    switch (kind) {
        case 'wicks': {
            // Steady uptrend with one violent wick-up bar mid-rally.
            // Chandelier's stop will jump up & follow the wick; close-based ignores it.
            for (let i = 0; i < 60; i++) {
                price += 0.4;
                const wick = (i === 35) ? 8.0 : 0.4;  // dramatic upper wick on bar 35
                out.push({ high: price + wick, low: price - 0.4, close: price });
            }
            return out;
        }
        case 'uptrend-reverse': {
            // 40 bars rally → 20 bars reversal; both methods should trigger near the same place.
            for (let i = 0; i < 40; i++) { price += 0.5; out.push({ high: price + 0.3, low: price - 0.3, close: price }); }
            for (let i = 0; i < 20; i++) { price -= 0.6; out.push({ high: price + 0.3, low: price - 0.3, close: price }); }
            return out;
        }
        case 'downtrend': {
            // 40 bars selloff → 20 bars bounce.
            price = 150;
            for (let i = 0; i < 40; i++) { price -= 0.5; out.push({ high: price + 0.3, low: price - 0.3, close: price }); }
            for (let i = 0; i < 20; i++) { price += 0.6; out.push({ high: price + 0.3, low: price - 0.3, close: price }); }
            return out;
        }
        case 'chop': {
            // Flat with random wicks → close-based stop is far stabler.
            for (let i = 0; i < 60; i++) {
                const wHi = ((i * 17) % 7) * 0.5;
                const wLo = ((i * 11) % 5) * 0.5;
                out.push({ high: price + 0.4 + wHi, low: price - 0.4 - wLo, close: price + ((i * 13) % 3 - 1) * 0.1 });
            }
            return out;
        }
        default:
            return makeDemoBars('wicks');
    }
}
