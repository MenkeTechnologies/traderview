// Reusable audio-alerts module.
//
// Two output channels, both browser-native (no asset loading):
//   * playBell()      — Web Audio API synth: short triangle-wave chime
//                       that decays naturally. Works in any modern browser.
//   * speakAlert(t)   — speechSynthesis.speak() — TTS announcement.
//
// Both are no-ops when browser APIs aren't available (Node test env,
// browsers with audio disabled, etc.) so calling code stays simple.
//
// Browser autoplay policy: an AudioContext cannot be created until the
// user interacts with the page (clicks any button). The first call to
// `playBell()` creates the context lazily; subsequent calls reuse it.

let audioCtx = null;

function getCtx() {
    if (typeof window === 'undefined') return null;
    const AC = window.AudioContext || window.webkitAudioContext;
    if (!AC) return null;
    if (!audioCtx) {
        try { audioCtx = new AC(); }
        catch { return null; }
    }
    // Some browsers leave context in "suspended" state until interaction.
    if (audioCtx.state === 'suspended') {
        audioCtx.resume().catch(() => {});
    }
    return audioCtx;
}

// Synthesizes a 3-tone bell chime: 880 → 1175 → 1568 Hz (E5/D6/G6,
// roughly a major-chord arpeggio). 0.6s total. Decays exponentially.
export function playBell({ volume = 0.25 } = {}) {
    const ctx = getCtx();
    if (!ctx) return false;
    const now = ctx.currentTime;
    const freqs = [880, 1175, 1568];
    const noteLen = 0.18;
    const gap = 0.05;
    for (let i = 0; i < freqs.length; i++) {
        const osc = ctx.createOscillator();
        const gain = ctx.createGain();
        osc.type = 'triangle';
        osc.frequency.value = freqs[i];
        const t0 = now + i * (noteLen + gap);
        gain.gain.setValueAtTime(0, t0);
        gain.gain.linearRampToValueAtTime(volume, t0 + 0.01);
        gain.gain.exponentialRampToValueAtTime(0.001, t0 + noteLen);
        osc.connect(gain).connect(ctx.destination);
        osc.start(t0);
        osc.stop(t0 + noteLen + 0.02);
    }
    return true;
}

// Alarm-style alert for high-priority events (e.g., squeeze on a watched
// symbol). 6-tone descending claxon, louder than the bell.
export function playAlarm({ volume = 0.35 } = {}) {
    const ctx = getCtx();
    if (!ctx) return false;
    const now = ctx.currentTime;
    const freqs = [880, 660, 880, 660, 880, 660];
    const noteLen = 0.12;
    for (let i = 0; i < freqs.length; i++) {
        const osc = ctx.createOscillator();
        const gain = ctx.createGain();
        osc.type = 'square';
        osc.frequency.value = freqs[i];
        const t0 = now + i * noteLen;
        gain.gain.setValueAtTime(0, t0);
        gain.gain.linearRampToValueAtTime(volume, t0 + 0.005);
        gain.gain.setValueAtTime(volume, t0 + noteLen - 0.02);
        gain.gain.exponentialRampToValueAtTime(0.001, t0 + noteLen);
        osc.connect(gain).connect(ctx.destination);
        osc.start(t0);
        osc.stop(t0 + noteLen + 0.01);
    }
    return true;
}

// Single short beep at configurable frequency. Used as the "minimal"
// alert sound for high-frequency rules where the full bell chime would
// be too disruptive.
export function playBeep({ freq = 1200, duration = 0.12, volume = 0.25 } = {}) {
    const ctx = getCtx();
    if (!ctx) return false;
    const now = ctx.currentTime;
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.type = 'sine';
    osc.frequency.value = freq;
    gain.gain.setValueAtTime(0, now);
    gain.gain.linearRampToValueAtTime(volume, now + 0.005);
    gain.gain.setValueAtTime(volume, now + duration - 0.02);
    gain.gain.exponentialRampToValueAtTime(0.001, now + duration);
    osc.connect(gain).connect(ctx.destination);
    osc.start(now);
    osc.stop(now + duration + 0.01);
    return true;
}

// Two short beeps spaced 180ms apart — distinguishes from a single beep
// without escalating to the bell or alarm.
export function playDoubleBeep({ freq = 1200, volume = 0.25 } = {}) {
    const ctx = getCtx();
    if (!ctx) return false;
    const now = ctx.currentTime;
    for (let i = 0; i < 2; i++) {
        const osc = ctx.createOscillator();
        const gain = ctx.createGain();
        osc.type = 'sine';
        osc.frequency.value = freq;
        const t0 = now + i * 0.18;
        gain.gain.setValueAtTime(0, t0);
        gain.gain.linearRampToValueAtTime(volume, t0 + 0.005);
        gain.gain.setValueAtTime(volume, t0 + 0.10);
        gain.gain.exponentialRampToValueAtTime(0.001, t0 + 0.12);
        osc.connect(gain).connect(ctx.destination);
        osc.start(t0);
        osc.stop(t0 + 0.13);
    }
    return true;
}

// Dispatcher: maps a `sound` name (one of bell / alarm / single_beep /
// double_beep / none) to its play fn. Centralizes audio choice so
// callers stay agnostic.
export function playSound(name, opts = {}) {
    switch (name) {
        case 'none':         return true;
        case 'bell':         return playBell(opts);
        case 'alarm':        return playAlarm(opts);
        case 'single_beep':  return playBeep(opts);
        case 'double_beep':  return playDoubleBeep(opts);
        default:             return playBell(opts);
    }
}

// Speaks the given text via the browser's TTS engine. Voice + rate +
// pitch configurable; returns false when speechSynthesis is missing.
export function speakAlert(text, { rate = 1.1, pitch = 1.0, volume = 1.0, voice = null } = {}) {
    if (typeof window === 'undefined' || !window.speechSynthesis) return false;
    if (!text || typeof text !== 'string') return false;
    try {
        const u = new SpeechSynthesisUtterance(text);
        u.rate = rate;
        u.pitch = pitch;
        u.volume = volume;
        if (voice) {
            const voices = window.speechSynthesis.getVoices();
            const match = voices.find(v => v.name === voice || v.lang === voice);
            if (match) u.voice = match;
        }
        window.speechSynthesis.speak(u);
        return true;
    } catch {
        return false;
    }
}

// Detects browser audio capability — useful for greying out a "test
// sound" button on Node-style hosts.
export function audioCapabilities() {
    if (typeof window === 'undefined') {
        return { audio: false, tts: false };
    }
    const audio = !!(window.AudioContext || window.webkitAudioContext);
    const tts = !!window.speechSynthesis;
    return { audio, tts };
}

// Compose a short spoken-alert phrase from a squeeze event. Keeps the
// phrasing TTS-friendly (no Unicode glyphs, percentages spelled out,
// short clauses).
export function formatSqueezeSpeech(ev) {
    if (!ev || !ev.symbol) return '';
    const pct = Number.isFinite(ev.price_change_pct)
        ? `up ${(ev.price_change_pct * 100).toFixed(1)} percent`
        : null;
    const vol = Number.isFinite(ev.volume_multiplier)
        ? `${ev.volume_multiplier.toFixed(1)} times average volume`
        : null;
    const parts = [`${ev.symbol} squeezing`];
    if (pct) parts.push(pct);
    if (vol) parts.push(`on ${vol}`);
    return parts.join(', ');
}
