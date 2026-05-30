// Audio alerts helpers — node-env tests for the speech-text formatter
// (engine functions are no-ops without browser APIs but must NOT crash).

import { test, expect } from 'vitest';
import {
    playBell, playAlarm, playBeep, playDoubleBeep, playSound,
    speakAlert, audioCapabilities, formatSqueezeSpeech,
} from '../js/_audio_alerts.js';

// ── audioCapabilities ────────────────────────────────────────────

test('audioCapabilities in Node env reports false for both channels', () => {
    const caps = audioCapabilities();
    expect(caps.audio).toBe(false);
    expect(caps.tts).toBe(false);
});

// ── No-op safety (no window, no SpeechSynthesis) ─────────────────

test('playBell returns false without browser', () => {
    expect(playBell()).toBe(false);
});

test('playAlarm returns false without browser', () => {
    expect(playAlarm()).toBe(false);
});

test('playBeep / playDoubleBeep return false without browser', () => {
    expect(playBeep()).toBe(false);
    expect(playDoubleBeep()).toBe(false);
});

test('playSound dispatcher accepts every valid name without crash', () => {
    for (const name of ['none', 'bell', 'alarm', 'single_beep', 'double_beep', 'unknown-fallthrough']) {
        // All should return either true (no-op for 'none') or false (no audio context) without throwing.
        const r = playSound(name);
        expect(typeof r).toBe('boolean');
    }
});

test('playSound(none) returns true (treated as silent OK)', () => {
    expect(playSound('none')).toBe(true);
});

test('speakAlert returns false without browser', () => {
    expect(speakAlert('hello')).toBe(false);
});

test('speakAlert rejects empty/non-string before touching browser API', () => {
    expect(speakAlert('')).toBe(false);
    expect(speakAlert(null)).toBe(false);
    expect(speakAlert(42)).toBe(false);
});

// ── formatSqueezeSpeech ──────────────────────────────────────────

test('formatSqueezeSpeech includes symbol + pct + volume mult when all present', () => {
    const phrase = formatSqueezeSpeech({
        symbol: 'AAPL', price_change_pct: 0.052, volume_multiplier: 3.5,
    });
    expect(phrase).toMatch(/AAPL squeezing/);
    expect(phrase).toMatch(/5\.2 percent/);
    expect(phrase).toMatch(/3\.5 times/);
});

test('formatSqueezeSpeech omits missing fields gracefully', () => {
    expect(formatSqueezeSpeech({ symbol: 'TSLA' })).toBe('TSLA squeezing');
});

test('formatSqueezeSpeech returns empty string on null', () => {
    expect(formatSqueezeSpeech(null)).toBe('');
    expect(formatSqueezeSpeech({})).toBe('');
});

test('formatSqueezeSpeech speech-friendly (no Unicode glyphs, no symbols)', () => {
    const phrase = formatSqueezeSpeech({
        symbol: 'SMID', price_change_pct: 0.123, volume_multiplier: 6.7,
    });
    expect(phrase).not.toMatch(/[%×]/);
    expect(phrase).toMatch(/percent/);
    expect(phrase).toMatch(/times/);
});

// ── formatSqueezeSpeech edge cases ─────────────────────────────────

test('formatSqueezeSpeech: non-finite pct/volume are omitted, not rendered as NaN', () => {
    const phrase = formatSqueezeSpeech({
        symbol: 'NVDA', price_change_pct: NaN, volume_multiplier: Infinity,
    });
    expect(phrase).toBe('NVDA squeezing');
    expect(phrase).not.toMatch(/NaN|Infinity/);
});

test('formatSqueezeSpeech: only pct (no volume) → includes pct, omits times clause', () => {
    const phrase = formatSqueezeSpeech({
        symbol: 'SPY', price_change_pct: -0.025,
    });
    // Note: pct is signed; speech says "up -2.5 percent" — captures current behavior
    // so future signed-sign improvement (e.g. "down 2.5") can update the test.
    expect(phrase).toMatch(/SPY squeezing/);
    expect(phrase).toMatch(/-2\.5 percent/);
    expect(phrase).not.toMatch(/times/);
});

test('formatSqueezeSpeech: only volume_multiplier → includes vol clause', () => {
    const phrase = formatSqueezeSpeech({ symbol: 'QQQ', volume_multiplier: 4.0 });
    expect(phrase).toBe('QQQ squeezing, on 4.0 times average volume');
});

test('formatSqueezeSpeech: parts joined with commas (TTS pacing)', () => {
    const phrase = formatSqueezeSpeech({
        symbol: 'GOOG', price_change_pct: 0.01, volume_multiplier: 2.0,
    });
    // Two commas = three clauses ("X squeezing", "up Y percent", "on Z times…")
    expect(phrase.split(', ').length).toBe(3);
});

test('formatSqueezeSpeech: empty symbol → empty string (guards bare "squeezing" output)', () => {
    expect(formatSqueezeSpeech({ symbol: '' })).toBe('');
    expect(formatSqueezeSpeech({ price_change_pct: 0.05 })).toBe('');
});

// ── playSound dispatcher ──────────────────────────────────────────

test('playSound: unknown name falls through to bell', () => {
    // bell returns false without browser AudioContext; whatever the dispatch
    // chose, the contract is that unknown names don't crash and don't return true.
    const r = playSound('made-up-sound');
    expect(typeof r).toBe('boolean');
    expect(r).toBe(false);  // same as playBell()
});

test('playSound: passes opts object through to underlying play fn', () => {
    // The dispatcher should NOT crash when opts has unknown fields.
    expect(() => playSound('bell',         { volume: 0.5, extra: 'ignored' })).not.toThrow();
    expect(() => playSound('alarm',        { volume: 0.5 })).not.toThrow();
    expect(() => playSound('single_beep',  { freq: 800 })).not.toThrow();
    expect(() => playSound('double_beep',  { volume: 0.5 })).not.toThrow();
});

// ── speakAlert input guards ───────────────────────────────────────

test('speakAlert: whitespace-only text NOT rejected (Node env still returns false)', () => {
    // '   ' is a non-empty truthy string. The guard only rejects empty + non-string.
    // In Node env, the speechSynthesis missing check fires first, so we get false.
    expect(speakAlert('   ')).toBe(false);
});

test('speakAlert: undefined / array / object rejected', () => {
    expect(speakAlert(undefined)).toBe(false);
    expect(speakAlert([])).toBe(false);
    expect(speakAlert({})).toBe(false);
});
