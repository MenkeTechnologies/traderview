// Toast pure helpers: validateOptions / normalizeOptions / classFor /
// animationFor / coalesceKey.

import { test, expect } from 'vitest';
import {
    DEFAULT_DURATION_MS, LEVELS,
    validateOptions, normalizeOptions, classFor, animationFor, coalesceKey,
    shouldToastApiError, apiErrorToastDetail, iconFor,
} from '../js/_toast.js';

test('LEVELS includes the canonical four', () => {
    expect(LEVELS).toEqual(['info', 'success', 'warning', 'error']);
});

test('DEFAULT_DURATION_MS is 2500 (audio-haxor parity)', () => {
    expect(DEFAULT_DURATION_MS).toBe(2500);
});

test('validateOptions: null/empty/undefined accept (use defaults)', () => {
    expect(validateOptions()).toBe(null);
    expect(validateOptions(null)).toBe(null);
    expect(validateOptions({})).toBe(null);
});

test('validateOptions: rejects bad duration', () => {
    expect(validateOptions({ duration: -1 })).toMatch(/duration/);
    expect(validateOptions({ duration: NaN })).toMatch(/duration/);
});

test('validateOptions: rejects bad level', () => {
    expect(validateOptions({ level: 'fatal' })).toMatch(/level/);
});

test('validateOptions: accepts each known level', () => {
    for (const lvl of LEVELS) expect(validateOptions({ level: lvl })).toBe(null);
});

test('normalizeOptions: defaults applied', () => {
    expect(normalizeOptions(null)).toEqual({
        duration: DEFAULT_DURATION_MS, level: 'info', extraClass: '',
    });
    expect(normalizeOptions({})).toEqual({
        duration: DEFAULT_DURATION_MS, level: 'info', extraClass: '',
    });
});

test('normalizeOptions: passes through caller values', () => {
    expect(normalizeOptions({ duration: 1000, level: 'error', extraClass: 'sticky' })).toEqual({
        duration: 1000, level: 'error', extraClass: 'sticky',
    });
});

test('classFor: info → no level suffix', () => {
    expect(classFor('info', '')).toBe('tv-toast');
});

test('classFor: error/warning/success → appended class', () => {
    expect(classFor('error', '')).toBe('tv-toast tv-toast-error');
    expect(classFor('warning', '')).toBe('tv-toast tv-toast-warning');
    expect(classFor('success', '')).toBe('tv-toast tv-toast-success');
});

test('classFor: extra class appended', () => {
    expect(classFor('info', 'sticky')).toBe('tv-toast sticky');
    expect(classFor('error', 'big')).toBe('tv-toast tv-toast-error big');
});

test('animationFor: 2500ms → out at 2.2s', () => {
    expect(animationFor(2500)).toMatch(/tv-toast-out 0\.3s ease-in 2\.2s forwards/);
});

test('animationFor: 1000ms → out at 0.7s', () => {
    expect(animationFor(1000)).toMatch(/tv-toast-out 0\.3s ease-in 0\.7s forwards/);
});

test('animationFor: too-short durations clamp at 0s delay (no negative)', () => {
    expect(animationFor(100)).toMatch(/tv-toast-out 0\.3s ease-in 0s forwards/);
});

test('coalesceKey: same level + same message → same key', () => {
    expect(coalesceKey('hello', 'info')).toBe(coalesceKey('hello', 'info'));
});

test('coalesceKey: different level → different key', () => {
    expect(coalesceKey('hello', 'info')).not.toBe(coalesceKey('hello', 'error'));
});

test('coalesceKey: defaults to info on missing level', () => {
    expect(coalesceKey('hello')).toBe(coalesceKey('hello', 'info'));
});

test('coalesceKey: empty message handled', () => {
    expect(coalesceKey()).toBe('info|');
});

// ── apiErrorToastDetail / shouldToastApiError ─────────────────────

test('shouldToastApiError: 5xx and 0 → true; 4xx and 2xx → false', () => {
    expect(shouldToastApiError({ status: 0 })).toBe(true);
    expect(shouldToastApiError({ status: 500 })).toBe(true);
    expect(shouldToastApiError({ status: 503 })).toBe(true);
    expect(shouldToastApiError({ status: 599 })).toBe(true);
    expect(shouldToastApiError({ status: 200 })).toBe(false);
    expect(shouldToastApiError({ status: 400 })).toBe(false);
    expect(shouldToastApiError({ status: 404 })).toBe(false);
    expect(shouldToastApiError({ status: 499 })).toBe(false);
});

test('shouldToastApiError: /client-errors always suppressed', () => {
    expect(shouldToastApiError({ status: 500, path: '/client-errors' })).toBe(false);
    expect(shouldToastApiError({ status: 0, path: '/client-errors' })).toBe(false);
});

test('shouldToastApiError: null/undefined safe', () => {
    expect(shouldToastApiError()).toBe(false);
    expect(shouldToastApiError(null)).toBe(false);
});

test('apiErrorToastDetail: 5xx → payload with httpLabel', () => {
    const p = apiErrorToastDetail({ status: 502, method: 'POST', path: '/analytics/foo' });
    expect(p).not.toBe(null);
    expect(p.level).toBe('error');
    expect(p.messageKey).toBe('toast.api_failed');
    expect(p.params.method).toBe('POST');
    expect(p.params.path).toBe('/analytics/foo');
    expect(p.params.httpLabel).toBe('HTTP 502');
    expect(p.params.labelKey).toBe(null);
});

test('apiErrorToastDetail: status 0 → labelKey set, httpLabel null', () => {
    const p = apiErrorToastDetail({ status: 0, method: 'GET', path: '/x' });
    expect(p.params.labelKey).toBe('toast.network_down');
    expect(p.params.httpLabel).toBe(null);
});

test('apiErrorToastDetail: 4xx → null (no toast)', () => {
    expect(apiErrorToastDetail({ status: 404, path: '/x' })).toBe(null);
});

test('apiErrorToastDetail: /client-errors → null even for 5xx', () => {
    expect(apiErrorToastDetail({ status: 500, path: '/client-errors' })).toBe(null);
});

test('apiErrorToastDetail: missing method/path → ? defaults', () => {
    const p = apiErrorToastDetail({ status: 500 });
    expect(p.params.method).toBe('?');
    expect(p.params.path).toBe('?');
});

// ── iconFor ────────────────────────────────────────────────────────

test('iconFor: distinct glyph per level', () => {
    const glyphs = LEVELS.map(iconFor);
    expect(new Set(glyphs).size).toBe(LEVELS.length);
});

test('iconFor: known levels map correctly', () => {
    expect(iconFor('success')).toBe('✓');
    expect(iconFor('warning')).toBe('⚠');
    expect(iconFor('error')).toBe('✕');
    expect(iconFor('info')).toBe('ⓘ');
});

test('iconFor: unknown level falls back to info glyph', () => {
    expect(iconFor('bogus')).toBe('ⓘ');
    expect(iconFor()).toBe('ⓘ');
});
