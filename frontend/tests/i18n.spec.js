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

