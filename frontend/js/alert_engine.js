// Audio + voice alert engine. Polls /api/alerts every 60s, fetches current
// quote per symbol, evaluates triggers, fires sound or SpeechSynthesis voice.

import { api } from './api.js';

let timer = null;
const FIRE_COOLDOWN_MS = 5 * 60_000;
const lastFired = new Map();
const lastDayHigh = new Map();
const lastDayLow  = new Map();

export function startAlertEngine() {
    if (timer) return;
    timer = setInterval(tick, 60_000);
    tick();
}

export function stopAlertEngine() {
    if (timer) { clearInterval(timer); timer = null; }
}

async function tick() {
    let rules;
    try { rules = await api.alerts(); } catch (_) { return; }
    rules = rules.filter(r => r.enabled);
    const symbols = new Set(rules.map(r => r.symbol));
    if (!symbols.size) return;

    for (const sym of symbols) {
        let q;
        try { q = await api.quote(sym); } catch (_) { continue; }
        const dayHi = Number(q.day_high ?? lastDayHigh.get(sym) ?? q.price);
        const dayLo = Number(q.day_low  ?? lastDayLow.get(sym)  ?? q.price);
        lastDayHigh.set(sym, dayHi);
        lastDayLow.set(sym, dayLo);
        for (const r of rules) {
            if (r.symbol !== sym) continue;
            const cooldown = lastFired.get(r.id);
            if (cooldown && Date.now() - cooldown < FIRE_COOLDOWN_MS) continue;
            if (matches(r, q)) {
                fire(r);
                lastFired.set(r.id, Date.now());
                api.markAlertFired(r.id).catch(() => {});
            }
        }
    }
}

function matches(r, q) {
    const price = Number(q.price);
    const ch    = Number(q.change_pct ?? 0);
    const vol   = Number(q.volume ?? 0);
    const thr   = Number(r.threshold ?? 0);
    switch (r.trigger) {
        case 'price_above':       return price >= thr;
        case 'price_below':       return price <= thr;
        case 'pct_up':            return ch >=  thr;
        case 'pct_down':          return ch <= -thr;
        case 'new_high_of_day':   return q.day_high != null && price >= Number(q.day_high);
        case 'new_low_of_day':    return q.day_low  != null && price <= Number(q.day_low);
        case 'volume_surge':      return thr > 0 && vol >= thr;
        // RSI / SMA crossovers need indicator data — fetch /symbols/:sym/signals.
        // Skipped here; the screener and signals page surface them statically.
        default: return false;
    }
}

function fire(r) {
    if (r.sound === 'voice' && r.voice_text) {
        speak(r.voice_text);
    } else {
        playSound(r.sound || 'bell');
    }
    if ('Notification' in window && Notification.permission === 'granted') {
        new Notification(`TraderView · ${r.symbol}`, {
            body: `${r.trigger}${r.threshold != null ? ' @ ' + r.threshold : ''}`,
        });
    }
}

// ---- exports for manual test from UI -----------------------------

let audioCtx;
export function playSound(kind = 'bell') {
    audioCtx = audioCtx || new (window.AudioContext || window.webkitAudioContext)();
    const o = audioCtx.createOscillator();
    const g = audioCtx.createGain();
    o.connect(g); g.connect(audioCtx.destination);
    if (kind === 'chime') {
        o.type = 'sine'; o.frequency.value = 880;
        g.gain.value = 0.0001;
        g.gain.exponentialRampToValueAtTime(0.25, audioCtx.currentTime + 0.02);
        g.gain.exponentialRampToValueAtTime(0.0001, audioCtx.currentTime + 0.8);
        o.start(); o.stop(audioCtx.currentTime + 0.85);
    } else {
        // bell — quick triple beep at 1200/1500/1800Hz
        const freqs = [1200, 1500, 1800];
        freqs.forEach((f, i) => {
            const o2 = audioCtx.createOscillator();
            const g2 = audioCtx.createGain();
            o2.connect(g2); g2.connect(audioCtx.destination);
            o2.type = 'square'; o2.frequency.value = f;
            const start = audioCtx.currentTime + i * 0.12;
            g2.gain.value = 0.0001;
            g2.gain.setValueAtTime(0.0001, start);
            g2.gain.exponentialRampToValueAtTime(0.18, start + 0.01);
            g2.gain.exponentialRampToValueAtTime(0.0001, start + 0.1);
            o2.start(start); o2.stop(start + 0.12);
        });
        o.stop();
    }
}

export function speak(text) {
    if (!('speechSynthesis' in window)) return;
    const u = new SpeechSynthesisUtterance(text);
    u.rate = 1.0; u.pitch = 1.0; u.volume = 1.0;
    window.speechSynthesis.speak(u);
}

export function requestNotifPermission() {
    if ('Notification' in window && Notification.permission === 'default') {
        Notification.requestPermission();
    }
}
