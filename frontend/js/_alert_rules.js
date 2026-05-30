// Custom alert rules engine — multi-rule, multi-type alerts with
// per-rule sound + TTS configuration.
//

import { t } from './i18n.js';
//
// Schema (key `tv-alert-rules-v1`):
//   { version: 1, rules: [
//       { id, name, type, enabled, sound, tts_template,
//         cooldown_seconds, watchlist, params: {...} }
//   ] }
//
// Rule types:
//   * `squeeze`       — price_pct + volume_mult thresholds (same engine
//                       as Squeeze Alerts view).
//   * `price_above`   — fires when price crosses above `threshold`.
//   * `price_below`   — fires when price crosses below `threshold`.
//   * `pct_change`    — fires when |price_change_pct| within window
//                       exceeds `pct_threshold`.
//   * `volume_spike`  — fires when cumulative window volume × ADV
//                       exceeds `volume_mult`.
//
// Each rule fires independently. Cooldown is per (rule, symbol) so a
// SPY squeeze rule firing on AAPL doesn't suppress TSLA's match.

export const STORAGE_KEY = 'tv-alert-rules-v1';
export const SCHEMA_VERSION = 1;

export const VALID_TYPES = new Set([
    'squeeze', 'price_above', 'price_below', 'pct_change', 'volume_spike',
]);

export const VALID_SOUNDS = new Set([
    'none', 'bell', 'alarm', 'single_beep', 'double_beep',
]);

export const DEFAULT_RULES = [
    {
        id: 'rule-default-squeeze',
        name: 'Squeeze (5% / 2× vol / 5min)',
        type: 'squeeze',
        enabled: true,
        sound: 'bell',
        tts_template: '{symbol} squeezing, up {change_pct} percent on {volume_mult} times volume',
        cooldown_seconds: 60,
        watchlist: [],
        params: {
            price_threshold_pct: 0.05,
            volume_threshold: 2.0,
            window_seconds: 300,
        },
    },
];

export function defaultState() {
    return { version: SCHEMA_VERSION, rules: [...DEFAULT_RULES] };
}

export function migrate(raw) {
    if (!raw || typeof raw !== 'object' || raw.version !== SCHEMA_VERSION) return defaultState();
    const rules = Array.isArray(raw.rules)
        ? raw.rules
            .map(r => sanitizeRule(r))
            .filter(r => r !== null)
        : [];
    return { version: SCHEMA_VERSION, rules };
}

function sanitizeRule(r) {
    if (!r || typeof r !== 'object') return null;
    if (typeof r.id !== 'string' || !r.id.trim()) return null;
    if (typeof r.name !== 'string' || !r.name.trim()) return null;
    if (!VALID_TYPES.has(r.type)) return null;
    return {
        id: r.id,
        name: r.name,
        type: r.type,
        enabled: r.enabled !== false,
        sound: VALID_SOUNDS.has(r.sound) ? r.sound : 'bell',
        tts_template: typeof r.tts_template === 'string' ? r.tts_template : '',
        cooldown_seconds: Number.isFinite(r.cooldown_seconds) && r.cooldown_seconds >= 0
            ? r.cooldown_seconds : 60,
        watchlist: Array.isArray(r.watchlist)
            ? r.watchlist.filter(s => typeof s === 'string').map(s => s.toUpperCase())
            : [],
        params: (r.params && typeof r.params === 'object') ? { ...r.params } : {},
    };
}

export function loadState(storage = globalThis.localStorage) {
    if (!storage) return defaultState();
    try {
        const s = storage.getItem(STORAGE_KEY);
        if (!s) return defaultState();
        return migrate(JSON.parse(s));
    } catch { return defaultState(); }
}

export function saveState(state, storage = globalThis.localStorage) {
    if (!storage) return false;
    try {
        storage.setItem(STORAGE_KEY, JSON.stringify(state));
        return true;
    } catch { return false; }
}

// ── Rule CRUD ────────────────────────────────────────────────────

function newRuleId(existing = new Set()) {
    for (let i = 0; i < 50; i++) {
        const id = `rule-${Math.random().toString(36).slice(2, 10)}`;
        if (!existing.has(id)) return id;
    }
    return `rule-${Date.now()}`;
}

export function newRule(type = 'squeeze', name = '') {
    const r = {
        id: newRuleId(),
        name: name || `${type} rule`,
        type, enabled: true, sound: 'bell',
        tts_template: '', cooldown_seconds: 60, watchlist: [],
        params: defaultParamsFor(type),
    };
    return r;
}

export function defaultParamsFor(type) {
    switch (type) {
        case 'squeeze':      return { price_threshold_pct: 0.05, volume_threshold: 2.0, window_seconds: 300 };
        case 'price_above':  return { threshold: 100 };
        case 'price_below':  return { threshold: 100 };
        case 'pct_change':   return { pct_threshold: 0.05, window_seconds: 300 };
        case 'volume_spike': return { volume_mult: 3.0, window_seconds: 300 };
        default:             return {};
    }
}

export function addRule(state, rule) {
    if (!rule) return state;
    const existing = new Set(state.rules.map(r => r.id));
    const safe = sanitizeRule({ ...rule, id: existing.has(rule.id) ? newRuleId(existing) : rule.id });
    if (!safe) return state;
    return { ...state, rules: [...state.rules, safe] };
}

export function updateRule(state, id, patch) {
    return {
        ...state,
        rules: state.rules.map(r => {
            if (r.id !== id) return r;
            const merged = { ...r, ...patch };
            const safe = sanitizeRule(merged);
            return safe || r;
        }),
    };
}

export function removeRule(state, id) {
    return { ...state, rules: state.rules.filter(r => r.id !== id) };
}

export function setEnabled(state, id, enabled) {
    return updateRule(state, id, { enabled: !!enabled });
}

// ── Detection engine ────────────────────────────────────────────

// Tick shape: { symbol, ts, price, volume }.
// Returns event objects: { rule_id, rule_name, type, symbol, ts,
//                          price, params_snapshot, message }
// where `message` is the rendered TTS template + a friendly fallback.
export function detectEvents(ticks, advMap, state, sinceTs = -Infinity) {
    const out = [];
    if (!Array.isArray(ticks) || ticks.length === 0) return out;
    if (!state || !Array.isArray(state.rules)) return out;
    const sorted = [...ticks].sort((a, b) => a.ts - b.ts);
    // Per-rule per-symbol state — window of (ts, price, volume) +
    // last-fired ts for cooldown enforcement.
    const ruleStates = new Map();
    const getRS = (ruleId, symbol) => {
        const key = `${ruleId}|${symbol}`;
        let s = ruleStates.get(key);
        if (!s) { s = { window: [], last_fired: -Infinity, crossed_above: false, crossed_below: false }; ruleStates.set(key, s); }
        return s;
    };
    for (const t of sorted) {
        for (const r of state.rules) {
            if (!r.enabled) continue;
            if (Array.isArray(r.watchlist) && r.watchlist.length > 0) {
                if (!r.watchlist.includes(t.symbol)) continue;
            }
            const rs = getRS(r.id, t.symbol);
            // Maintain rolling window for window-based rules.
            const winSec = Number(r.params.window_seconds) || 300;
            const cutoff = t.ts - winSec;
            while (rs.window.length && rs.window[0].ts < cutoff) rs.window.shift();
            rs.window.push(t);
            // Cooldown gate.
            const onCooldown = (t.ts - rs.last_fired) < (Number(r.cooldown_seconds) || 0);
            const fire = (extra) => {
                if (onCooldown) return;
                const message = renderTemplate(r.tts_template, {
                    symbol: t.symbol,
                    price: t.price,
                    ...extra,
                }) || fallbackMessage(r, t, extra);
                out.push({
                    rule_id: r.id, rule_name: r.name, type: r.type,
                    sound: r.sound,
                    symbol: t.symbol, ts: t.ts, price: t.price,
                    extra,
                    message,
                });
                rs.last_fired = t.ts;
            };
            switch (r.type) {
                case 'squeeze': {
                    if (rs.window.length < 2) break;
                    const oldest = rs.window[0];
                    const priceChange = (t.price - oldest.price) / oldest.price;
                    const adv = advMap?.[t.symbol];
                    if (!Number.isFinite(adv) || adv <= 0) break;
                    const cumVol = rs.window.reduce((a, x) => a + (x.volume || 0), 0);
                    const advForWindow = adv * (Math.max(1, t.ts - oldest.ts) / 23400);
                    const volMult = advForWindow > 0 ? cumVol / advForWindow : 0;
                    if (priceChange >= (Number(r.params.price_threshold_pct) || 0.05) &&
                        volMult     >= (Number(r.params.volume_threshold)    || 2.0)) {
                        fire({ price_change_pct: priceChange, volume_mult: volMult });
                    }
                    break;
                }
                case 'price_above': {
                    if (t.price > (Number(r.params.threshold) || 0) && !rs.crossed_above) {
                        fire({ threshold: r.params.threshold });
                        rs.crossed_above = true;
                    } else if (t.price <= (Number(r.params.threshold) || 0)) {
                        rs.crossed_above = false;   // reset on cross-back
                    }
                    break;
                }
                case 'price_below': {
                    if (t.price < (Number(r.params.threshold) || 0) && !rs.crossed_below) {
                        fire({ threshold: r.params.threshold });
                        rs.crossed_below = true;
                    } else if (t.price >= (Number(r.params.threshold) || 0)) {
                        rs.crossed_below = false;
                    }
                    break;
                }
                case 'pct_change': {
                    if (rs.window.length < 2) break;
                    const pct = (t.price - rs.window[0].price) / rs.window[0].price;
                    if (Math.abs(pct) >= (Number(r.params.pct_threshold) || 0.05)) {
                        fire({ price_change_pct: pct });
                    }
                    break;
                }
                case 'volume_spike': {
                    if (rs.window.length < 2) break;
                    const adv = advMap?.[t.symbol];
                    if (!Number.isFinite(adv) || adv <= 0) break;
                    const cumVol = rs.window.reduce((a, x) => a + (x.volume || 0), 0);
                    const advForWindow = adv * (Math.max(1, t.ts - rs.window[0].ts) / 23400);
                    const volMult = advForWindow > 0 ? cumVol / advForWindow : 0;
                    if (volMult >= (Number(r.params.volume_mult) || 3.0)) {
                        fire({ volume_mult: volMult });
                    }
                    break;
                }
            }
        }
    }
    return out.filter(e => e.ts >= sinceTs);
}

// Template renderer — replaces {var} placeholders.
//   {symbol}, {price}, {change_pct}, {volume_mult}, {threshold}
// Empty template → null so the caller falls back to a friendly default.
export function renderTemplate(template, vars) {
    if (typeof template !== 'string' || !template.trim()) return null;
    return template.replace(/\{(\w+)\}/g, (_, key) => {
        const v = vars[key];
        if (!Number.isFinite(v)) return String(vars[key] ?? '');
        if (key === 'change_pct' || key === 'price_change_pct') return (v * 100).toFixed(1);
        if (key === 'volume_mult') return v.toFixed(1);
        if (key === 'price' || key === 'threshold') return v.toFixed(2);
        return String(v);
    });
}

export function fallbackMessage(rule, tick, extra) {
    switch (rule.type) {
        case 'squeeze':
            return t('view.alert_rules.msg.squeeze', { symbol: tick.symbol, pct: ((extra.price_change_pct || 0) * 100).toFixed(1), mult: (extra.volume_mult || 0).toFixed(1) });
        case 'price_above':
            return t('view.alert_rules.msg.price_above', { symbol: tick.symbol, threshold: (extra.threshold || 0).toFixed(2) });
        case 'price_below':
            return t('view.alert_rules.msg.price_below', { symbol: tick.symbol, threshold: (extra.threshold || 0).toFixed(2) });
        case 'pct_change': {
            const dir = t((extra.price_change_pct || 0) >= 0 ? 'view.alert_rules.msg.pct_dir_up' : 'view.alert_rules.msg.pct_dir_down');
            return t('view.alert_rules.msg.pct_change', { symbol: tick.symbol, dir, pct: Math.abs((extra.price_change_pct || 0) * 100).toFixed(1) });
        }
        case 'volume_spike':
            return t('view.alert_rules.msg.volume_spike', { symbol: tick.symbol, mult: (extra.volume_mult || 0).toFixed(1) });
        default:
            return t('view.alert_rules.msg.default', { symbol: tick.symbol });
    }
}

// Deterministic demo data — same tick stream as squeeze-alerts so the
// user can compare detector behavior side-by-side.
export function makeDemoData() {
    const ticks = [];
    const baseTs = 1_700_000_000;
    for (let s = 0; s < 1800; s += 30) {
        const inSpike = s >= 1500;
        const price = inSpike
            ? 150 + (s - 1500) / 300 * 12
            : 150 + Math.sin(s / 120) * 0.4;
        const volume = inSpike ? 200_000 : 5000;
        ticks.push({ symbol: 'AAPL', ts: baseTs + s, price: Number(price.toFixed(2)), volume });
    }
    for (let s = 0; s < 1800; s += 30) {
        const inSpike = s >= 1200;
        const price = inSpike ? 8 + (s - 1200) / 600 * 1.0 : 8 + Math.sin(s / 90) * 0.05;
        const volume = inSpike ? 20000 : 200;
        ticks.push({ symbol: 'SMID', ts: baseTs + s, price: Number(price.toFixed(2)), volume });
    }
    const adv = { AAPL: 50_000_000, SMID: 250_000 };
    return { ticks, adv };
}
