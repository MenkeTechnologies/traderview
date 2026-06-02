// Regression: custom hotkeys (the api.hotkeys() engine) must still fire from
// inside focused text fields when the chord uses a command modifier, so e.g.
// Cmd+E -> go_trades works while the Home tab's auto-focused filter box has
// focus. Plain keys / Shift-only stay as normal text entry.
import { describe, test, expect } from 'vitest';
import { hotkeyAllowedForTarget } from '../js/hotkey_engine.js';

const ev = (o = {}) => ({
    ctrlKey: false, altKey: false, shiftKey: false, metaKey: false,
    target: { tagName: 'BODY' },
    ...o,
});

describe('hotkeyAllowedForTarget', () => {
    test('non-editable target: any key allowed', () => {
        expect(hotkeyAllowedForTarget(ev())).toBe(true);
        expect(hotkeyAllowedForTarget(ev({ target: { tagName: 'BUTTON' } }))).toBe(true);
    });

    test('inside input: command-modifier chords fire (Cmd+E)', () => {
        const input = { tagName: 'INPUT' };
        expect(hotkeyAllowedForTarget(ev({ target: input, metaKey: true }))).toBe(true);
        expect(hotkeyAllowedForTarget(ev({ target: input, ctrlKey: true }))).toBe(true);
        expect(hotkeyAllowedForTarget(ev({ target: input, altKey: true }))).toBe(true);
    });

    test('inside input: plain key / shift-only are normal typing, suppressed', () => {
        const input = { tagName: 'INPUT' };
        expect(hotkeyAllowedForTarget(ev({ target: input }))).toBe(false);
        expect(hotkeyAllowedForTarget(ev({ target: input, shiftKey: true }))).toBe(false);
    });

    test('textarea / select / contentEditable behave like inputs', () => {
        expect(hotkeyAllowedForTarget(ev({ target: { tagName: 'TEXTAREA' } }))).toBe(false);
        expect(hotkeyAllowedForTarget(ev({ target: { tagName: 'SELECT' } }))).toBe(false);
        expect(hotkeyAllowedForTarget(ev({ target: { tagName: 'DIV', isContentEditable: true } }))).toBe(false);
        expect(hotkeyAllowedForTarget(ev({ target: { tagName: 'DIV', isContentEditable: true }, metaKey: true }))).toBe(true);
    });
});
