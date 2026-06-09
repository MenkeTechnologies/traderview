// Global indicator catalog — the single source of truth for every indicator
// kind the backend evaluator (crates/traderview-db/src/custom_indicators.rs)
// can compute. The Custom Indicators view builds its create-form + presets
// from this list, and the Charts overlay consumes the saved presets.
//
// Each entry:
//   id      — wire value sent to the backend `definition.kind`.
//   label   — human label shown in the picker.
//   params  — default parameter set (also defines which inputs to render).
//   outputs — number of series the backend emits (informational).
//
// Keep this in lockstep with the backend `compute_one` / `validate` match
// arms. Adding a kind here without a backend arm yields an empty overlay.

export const INDICATOR_CATALOG = [
    { id: 'sma',        label: 'SMA',         params: { period: 20 },                    outputs: 1 },
    { id: 'ema',        label: 'EMA',         params: { period: 20 },                    outputs: 1 },
    { id: 'rsi',        label: 'RSI',         params: { period: 14 },                    outputs: 1 },
    { id: 'atr',        label: 'ATR',         params: { period: 14 },                    outputs: 1 },
    { id: 'adx',        label: 'ADX / DI',    params: { period: 14 },                    outputs: 3 },
    { id: 'stochastic', label: 'Stochastic',  params: { k: 14, d: 3 },                   outputs: 2 },
    { id: 'bollinger',  label: 'Bollinger',   params: { period: 20, k: 2 },              outputs: 3 },
    { id: 'macd',       label: 'MACD',        params: { fast: 12, slow: 26, signal: 9 }, outputs: 3 },
];

export function indicatorKind(id) {
    return INDICATOR_CATALOG.find(k => k.id === id) || null;
}
