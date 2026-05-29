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
