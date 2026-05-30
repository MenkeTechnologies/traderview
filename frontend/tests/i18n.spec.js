// i18n module: t(), setMap, extendMap, applyUiI18n, decodeNewlines.

import { test, expect, beforeEach } from 'vitest';
import {
    t, setMap, extendMap, getMap, applyUiI18n, decodeNewlines, currentLocale,
} from '../js/i18n.js';

beforeEach(() => { setMap({}); });

test('t returns key on miss (visible untranslated string)', () => {
    expect(t('some.unknown.key')).toBe('some.unknown.key');
});

test('t interpolates {placeholder} params', () => {
    setMap({ 'greeting': 'Hello {name} ({count})' });
    expect(t('greeting', { name: 'Jake', count: 7 })).toBe('Hello Jake (7)');
});

test('t leaves unfilled {placeholder} intact', () => {
    setMap({ 'greeting': 'Hello {name}' });
    expect(t('greeting', {})).toBe('Hello {name}');
});

test('t skips interpolation when no params object', () => {
    setMap({ 'k': 'static' });
    expect(t('k')).toBe('static');
});

test('setMap replaces map; extendMap merges', () => {
    setMap({ a: '1' });
    extendMap({ b: '2' });
    expect(getMap()).toEqual({ a: '1', b: '2' });
    setMap({ c: '3' });
    expect(getMap()).toEqual({ c: '3' });
});

test('decodeNewlines: &#10; / &#13; → \\n / \\r', () => {
    expect(decodeNewlines('a&#10;b&#13;c')).toBe('a\nb\rc');
    expect(decodeNewlines(42)).toBe(42);
});

test('applyUiI18n: walks fake DOM and updates textContent', () => {
    const elText = makeEl('SPAN', { 'i18n': 'k1' });
    const root = fakeRoot({ '[data-i18n]': [elText] });
    setMap({ k1: 'one' });
    expect(applyUiI18n(root)).toBe(1);
    expect(elText.textContent).toBe('one');
});

test('applyUiI18n: placeholder + title + aria-label dispatched correctly', () => {
    const elPh    = makeEl('INPUT',  { 'i18n-placeholder': 'ph' });
    const elTitle = makeEl('SPAN',   { 'i18n-title': 'tt' });
    const elAria  = makeEl('BUTTON', { 'i18n-aria-label': 'lbl' });
    const root = fakeRoot({
        '[data-i18n]': [],
        '[data-i18n-placeholder]': [elPh],
        '[data-i18n-title]':       [elTitle],
        '[data-i18n-aria-label]':  [elAria],
    });
    setMap({ ph: 'search…', tt: 'tooltip', lbl: 'click me' });
    applyUiI18n(root);
    expect(elPh.placeholder).toBe('search…');
    expect(elTitle.title).toBe('tooltip');
    expect(elAria._attrs['aria-label']).toBe('click me');
});

test('applyUiI18n: data-i18n-html sets innerHTML so child markup survives', () => {
    const el = makeEl('P', { 'i18n-html': 'k_html' });
    const root = fakeRoot({ '[data-i18n-html]': [el] });
    setMap({ k_html: 'Engine lives in <code>traderview_core::risk_gate</code>.' });
    applyUiI18n(root);
    expect(el.innerHTML).toBe('Engine lives in <code>traderview_core::risk_gate</code>.');
});

test('applyUiI18n: data-i18n-html missing key leaves innerHTML intact', () => {
    const el = makeEl('P', { 'i18n-html': 'no_such_key' });
    el.innerHTML = '<em>stays as-is</em>';
    const root = fakeRoot({ '[data-i18n-html]': [el] });
    setMap({});
    applyUiI18n(root);
    expect(el.innerHTML).toBe('<em>stays as-is</em>');
});

test('applyUiI18n: missing key leaves textContent untouched', () => {
    const el = makeEl('SPAN', { 'i18n': 'missing' });
    el.textContent = 'stays';
    const root = fakeRoot({ '[data-i18n]': [el] });
    setMap({});
    applyUiI18n(root);
    expect(el.textContent).toBe('stays');
});

test('applyUiI18n: explicit null root returns 0 (no global doc fallback)', () => {
    expect(applyUiI18n(null)).toBe(0);
    expect(applyUiI18n({})).toBe(0);
});

function fakeRoot(selectorMap) {
    return {
        querySelectorAll(sel) { return selectorMap[sel] || []; },
    };
}
function makeEl(tag, dataAttrs) {
    const dataset = {};
    for (const [k, v] of Object.entries(dataAttrs)) {
        const camel = k.replace(/-([a-z])/g, (_, c) => c.toUpperCase());
        dataset[camel] = v;
    }
    return {
        tagName: tag, textContent: '', placeholder: '', title: '',
        dataset, _attrs: {},
        setAttribute(k, v) { this._attrs[k] = v; },
        getAttribute(k) { return this._attrs[k]; },
    };
}

test('currentLocale defaults to "en"', () => {
    expect(currentLocale()).toBe('en');
});

// ── edge cases ────────────────────────────────────────────────────

test('t handles same placeholder appearing twice', () => {
    setMap({ k: '{x} and {x} again' });
    expect(t('k', { x: 'foo' })).toBe('foo and foo again');
});

test('t coerces non-string param values via String()', () => {
    setMap({ k: '{n}' });
    expect(t('k', { n: 0 })).toBe('0');
    expect(t('k', { n: false })).toBe('false');
    expect(t('k', { n: null })).toBe('{n}');
});

test('t with non-object params skips interpolation', () => {
    setMap({ k: '{x}' });
    // Number / string / null — t() guards via `typeof params === "object"`.
    // null is typeof object but the guard's && short-circuit handles it.
    expect(t('k', 42)).toBe('{x}');
    expect(t('k', 'string')).toBe('{x}');
    expect(t('k', null)).toBe('{x}');
});

test('extendMap merges over existing keys (later wins)', () => {
    setMap({ a: '1', b: '2' });
    extendMap({ b: '99', c: '3' });
    expect(getMap()).toEqual({ a: '1', b: '99', c: '3' });
});

test('extendMap with non-object input is a no-op', () => {
    setMap({ a: '1' });
    extendMap(null);
    extendMap(undefined);
    extendMap(42);
    expect(getMap()).toEqual({ a: '1' });
});

test('decodeNewlines: only converts entities; leaves other text untouched', () => {
    expect(decodeNewlines('plain text')).toBe('plain text');
    expect(decodeNewlines('&amp; stays')).toBe('&amp; stays');
});

test('applyUiI18n: empty-string value treated as miss (textContent preserved)', () => {
    const el = makeEl('SPAN', { 'i18n': 'k' });
    el.textContent = 'original';
    const root = fakeRoot({ '[data-i18n]': [el] });
    setMap({ k: '' });
    applyUiI18n(root);
    expect(el.textContent).toBe('original');
});

test('applyUiI18n: missing data-i18n attribute → element skipped', () => {
    const el = makeEl('SPAN', {});  // no data-i18n
    el.textContent = 'untouched';
    const root = fakeRoot({ '[data-i18n]': [el] });
    setMap({});
    expect(applyUiI18n(root)).toBe(0);
    expect(el.textContent).toBe('untouched');
});

test('applyUiI18n: count return value reflects only successful updates', () => {
    const e1 = makeEl('SPAN', { 'i18n': 'a' });
    const e2 = makeEl('SPAN', { 'i18n': 'b' });
    const e3 = makeEl('SPAN', { 'i18n': 'missing' });
    const root = fakeRoot({ '[data-i18n]': [e1, e2, e3] });
    setMap({ a: 'A', b: 'B' });  // 'missing' not in map
    expect(applyUiI18n(root)).toBe(2);
});

// ── loadLocale: merge-over-en + fetch failure handling ────────────

import { loadLocale } from '../js/i18n.js';

test('loadLocale: non-en locale merges over en (en keys preserved, locale keys overlay)', async () => {
    const enCatalog = { 'view.x': 'X-en', 'view.y': 'Y-en', 'common.ok': 'OK' };
    const esCatalog = { 'view.x': 'X-es' /* y omitted */ };
    globalThis.fetch = async (url) => {
        if (url.includes('app_i18n_es.json')) return { ok: true, json: async () => esCatalog };
        if (url.includes('app_i18n_en.json')) return { ok: true, json: async () => enCatalog };
        return { ok: false };
    };
    globalThis.localStorage = { setItem: () => {} };
    globalThis.document = { querySelectorAll: () => [] };
    try {
        const keyCount = await loadLocale('es');
        expect(keyCount).toBe(3);            // 3 unique keys after merge
        expect(t('view.x')).toBe('X-es');    // es overlay wins
        expect(t('view.y')).toBe('Y-en');    // missing in es → falls through to en
        expect(t('common.ok')).toBe('OK');   // en-only key still visible
    } finally {
        delete globalThis.fetch;
        delete globalThis.localStorage;
        delete globalThis.document;
    }
});

test('loadLocale: returns 0 on network failure (existing map intact)', async () => {
    setMap({ 'before': 'before-value' });
    globalThis.fetch = async () => { throw new Error('network'); };
    try {
        const r = await loadLocale('de');
        expect(r).toBe(0);
        expect(t('before')).toBe('before-value');  // map unchanged
    } finally {
        delete globalThis.fetch;
    }
});

test('loadLocale: returns 0 on HTTP error response', async () => {
    setMap({ 'before': 'before-value' });
    globalThis.fetch = async () => ({ ok: false, status: 404 });
    try {
        expect(await loadLocale('xx')).toBe(0);
        expect(t('before')).toBe('before-value');
    } finally {
        delete globalThis.fetch;
    }
});

test('loadLocale: en code skips the en-merge step (only ONE fetch)', async () => {
    const calls = [];
    globalThis.fetch = async (url) => {
        calls.push(url);
        return { ok: true, json: async () => ({ 'k': 'v' }) };
    };
    globalThis.localStorage = { setItem: () => {} };
    globalThis.document = { querySelectorAll: () => [] };
    try {
        await loadLocale('en');
        expect(calls.filter(u => u.includes('i18n')).length).toBe(1);
    } finally {
        delete globalThis.fetch;
        delete globalThis.localStorage;
        delete globalThis.document;
    }
});

