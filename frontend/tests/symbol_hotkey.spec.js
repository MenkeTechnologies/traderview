// Global symbol-hotkey pure logic. DOM wiring lives in
// symbol_hotkey_install.js and is exercised in the browser; here we
// cover the buffer rules, ticker-char predicate, route-decision logic,
// and keydown classifier (with mocked event-like objects).

import { test, expect } from 'vitest';
import {
    isTickerChar, SymbolBuffer,
    decideTargetHash, classifyKey,
} from '../js/_symbol_hotkey.js';

// ── isTickerChar ────────────────────────────────────────────────────

test('isTickerChar accepts letters', () => {
    for (const c of 'ABCabc') expect(isTickerChar(c)).toBe(true);
});

test('isTickerChar accepts digits', () => {
    for (const c of '0123456789') expect(isTickerChar(c)).toBe(true);
});

test('isTickerChar accepts "." and "-"', () => {
    expect(isTickerChar('.')).toBe(true);
    expect(isTickerChar('-')).toBe(true);
});

test('isTickerChar rejects space, punctuation, multi-char', () => {
    expect(isTickerChar(' ')).toBe(false);
    expect(isTickerChar(',')).toBe(false);
    expect(isTickerChar('AB')).toBe(false);
    expect(isTickerChar('')).toBe(false);
    expect(isTickerChar(null)).toBe(false);
});

// ── SymbolBuffer ────────────────────────────────────────────────────

test('SymbolBuffer.appendChar uppercases letters', () => {
    const b = new SymbolBuffer();
    b.appendChar('a').appendChar('a').appendChar('p').appendChar('l');
    expect(b.value).toBe('AAPL');
});

test('SymbolBuffer.appendChar skips non-ticker chars', () => {
    const b = new SymbolBuffer();
    b.appendChar('A').appendChar(' ').appendChar('B');
    expect(b.value).toBe('AB');
});

test('SymbolBuffer caps at 8 chars', () => {
    const b = new SymbolBuffer();
    for (const c of 'ABCDEFGHIJ') b.appendChar(c);
    expect(b.value.length).toBe(8);
    expect(b.value).toBe('ABCDEFGH');
});

test('SymbolBuffer.backspace removes one char', () => {
    const b = new SymbolBuffer();
    b.appendChar('A').appendChar('B').backspace();
    expect(b.value).toBe('A');
});

test('SymbolBuffer.backspace on empty is a no-op', () => {
    const b = new SymbolBuffer();
    b.backspace();
    expect(b.value).toBe('');
});

test('SymbolBuffer.reset clears the value', () => {
    const b = new SymbolBuffer();
    b.appendChar('X').reset();
    expect(b.value).toBe('');
});

test('SymbolBuffer.isValid requires at least one letter', () => {
    const b = new SymbolBuffer();
    expect(b.isValid()).toBe(false);    // empty
    b.appendChar('1').appendChar('2').appendChar('3');
    expect(b.isValid()).toBe(false);    // all digits
    b.appendChar('A');
    expect(b.isValid()).toBe(true);
});

test('SymbolBuffer.isValid accepts BRK.B', () => {
    const b = new SymbolBuffer();
    for (const c of 'BRK.B') b.appendChar(c);
    expect(b.value).toBe('BRK.B');
    expect(b.isValid()).toBe(true);
});

// ── decideTargetHash ────────────────────────────────────────────────

test('decideTargetHash routes to research/ by default', () => {
    expect(decideTargetHash('#dashboard', 'AAPL')).toBe('research/AAPL');
    expect(decideTargetHash('#launcher',  'NVDA')).toBe('research/NVDA');
    expect(decideTargetHash('',           'TSLA')).toBe('research/TSLA');
});

test('decideTargetHash preserves the current view when it is symbol-aware', () => {
    expect(decideTargetHash('#charts/MSFT',     'AAPL')).toBe('charts/AAPL');
    expect(decideTargetHash('#options/SPY',     'QQQ')).toBe('options/QQQ');
    expect(decideTargetHash('#earnings-iv/NFLX', 'META')).toBe('earnings-iv/META');
});

test('decideTargetHash uppercases the symbol', () => {
    expect(decideTargetHash('#research', 'aapl')).toBe('research/AAPL');
});

test('decideTargetHash returns null on empty / non-string symbol', () => {
    expect(decideTargetHash('#research', '')).toBe(null);
    expect(decideTargetHash('#research', null)).toBe(null);
    expect(decideTargetHash('#research', 42)).toBe(null);
});

test('decideTargetHash tolerates trailing slashes / params in current hash', () => {
    expect(decideTargetHash('#research/OLD/extra', 'NEW')).toBe('research/NEW');
});

// ── classifyKey ─────────────────────────────────────────────────────

function evt(overrides = {}) {
    return {
        target: { tagName: 'BODY', isContentEditable: false },
        ctrlKey: false, metaKey: false, altKey: false, shiftKey: false,
        key: '',
        ...overrides,
    };
}

test('classifyKey returns null when target is an input', () => {
    expect(classifyKey(evt({ key: 'a', target: { tagName: 'INPUT' } }))).toBe(null);
    expect(classifyKey(evt({ key: 'a', target: { tagName: 'TEXTAREA' } }))).toBe(null);
    expect(classifyKey(evt({ key: 'a', target: { tagName: 'SELECT' } }))).toBe(null);
});

test('classifyKey returns null when target is contenteditable', () => {
    expect(classifyKey(evt({ key: 'a', target: { tagName: 'DIV', isContentEditable: true } }))).toBe(null);
});

test('classifyKey returns null on modifier-key combos', () => {
    expect(classifyKey(evt({ key: 'a', ctrlKey: true }))).toBe(null);
    expect(classifyKey(evt({ key: 'a', metaKey: true }))).toBe(null);
    expect(classifyKey(evt({ key: 'a', altKey: true }))).toBe(null);
});

test('classifyKey allows Shift+letter for capital input', () => {
    expect(classifyKey(evt({ key: 'A', shiftKey: true }))).toBe('append');
});

test('classifyKey maps Enter / Backspace / Escape / Space', () => {
    expect(classifyKey(evt({ key: 'Enter' }))).toBe('enter');
    expect(classifyKey(evt({ key: 'Backspace' }))).toBe('backspace');
    expect(classifyKey(evt({ key: 'Escape' }))).toBe('escape');
    expect(classifyKey(evt({ key: ' ' }))).toBe('escape');
});

test('classifyKey returns "append" for ticker chars', () => {
    expect(classifyKey(evt({ key: 'A' }))).toBe('append');
    expect(classifyKey(evt({ key: '5' }))).toBe('append');
    expect(classifyKey(evt({ key: '.' }))).toBe('append');
});

test('classifyKey returns null for non-ticker keys (arrow, F-keys, etc.)', () => {
    expect(classifyKey(evt({ key: 'ArrowDown' }))).toBe(null);
    expect(classifyKey(evt({ key: 'F5' }))).toBe(null);
    expect(classifyKey(evt({ key: 'Tab' }))).toBe(null);
    expect(classifyKey(evt({ key: '/' }))).toBe(null);
});

test('classifyKey returns null on a null event', () => {
    expect(classifyKey(null)).toBe(null);
});
