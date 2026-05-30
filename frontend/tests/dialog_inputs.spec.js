// Spec for `_dialog.js` — the pure-JS surface of the tConfirm / tPrompt
// modal primitive. DOM glue (in dialog.js) is exercised by hand-testing
// in the live app; this spec pins the helpers that decide buttons,
// keybinds, classes, and result normalization.

import { describe, test, expect } from 'vitest';
import {
    validateOptions, defaultButtons, classFor,
    isConfirmKey, isCancelKey, normalizePromptResult,
} from '../js/_dialog.js';

describe('validateOptions', () => {
    test('null / undefined / {} accepted', () => {
        expect(validateOptions(null)).toBeNull();
        expect(validateOptions(undefined)).toBeNull();
        expect(validateOptions({})).toBeNull();
    });
    test('non-object rejected', () => {
        expect(validateOptions('hi')).toMatch(/object/);
        expect(validateOptions(42)).toMatch(/object/);
    });
    test('valid levels accepted', () => {
        for (const lvl of ['info', 'warning', 'danger']) {
            expect(validateOptions({ level: lvl })).toBeNull();
        }
    });
    test('invalid level rejected', () => {
        expect(validateOptions({ level: 'error' })).toMatch(/level must be/);
    });
    test('confirmKey / cancelKey must be strings', () => {
        expect(validateOptions({ confirmKey: 'dialog.btn.ok' })).toBeNull();
        expect(validateOptions({ confirmKey: 42 })).toMatch(/confirmKey/);
        expect(validateOptions({ cancelKey:  {} })).toMatch(/cancelKey/);
    });
    test('defaultValue / placeholder must be strings', () => {
        expect(validateOptions({ defaultValue: '' })).toBeNull();
        expect(validateOptions({ defaultValue: 42 })).toMatch(/defaultValue/);
        expect(validateOptions({ placeholder: 42 })).toMatch(/placeholder/);
    });
});

describe('defaultButtons', () => {
    test('prompt → ok / cancel', () => {
        expect(defaultButtons('prompt', 'info')).toEqual({
            confirmKey: 'dialog.btn.ok', cancelKey: 'dialog.btn.cancel',
        });
    });
    test('confirm + danger → delete / cancel', () => {
        expect(defaultButtons('confirm', 'danger')).toEqual({
            confirmKey: 'dialog.btn.delete', cancelKey: 'dialog.btn.cancel',
        });
    });
    test('confirm + info → confirm / cancel', () => {
        expect(defaultButtons('confirm', 'info')).toEqual({
            confirmKey: 'dialog.btn.confirm', cancelKey: 'dialog.btn.cancel',
        });
    });
});

describe('classFor', () => {
    test('info → cyan', () => {
        expect(classFor('info')).toBe('tv-dialog-card tv-dialog-info');
    });
    test('warning → amber', () => {
        expect(classFor('warning')).toBe('tv-dialog-card tv-dialog-warning');
    });
    test('danger → red', () => {
        expect(classFor('danger')).toBe('tv-dialog-card tv-dialog-danger');
    });
    test('unknown level falls back to info', () => {
        expect(classFor('mystery')).toBe('tv-dialog-card tv-dialog-info');
    });
});

describe('isConfirmKey / isCancelKey', () => {
    test('Enter → confirm', () => {
        expect(isConfirmKey({ key: 'Enter' })).toBe(true);
    });
    test('Escape → cancel', () => {
        expect(isCancelKey({ key: 'Escape' })).toBe(true);
    });
    test('letter keys not confirm/cancel', () => {
        expect(isConfirmKey({ key: 'a' })).toBe(false);
        expect(isCancelKey({ key: 'a' })).toBe(false);
    });
    test('null / undefined / missing key safe', () => {
        expect(isConfirmKey(null)).toBe(false);
        expect(isConfirmKey({})).toBe(false);
        expect(isCancelKey(null)).toBe(false);
    });
});

describe('normalizePromptResult', () => {
    test('trims whitespace', () => {
        expect(normalizePromptResult('  hi  ')).toBe('hi');
    });
    test('null / undefined → empty string when not required', () => {
        expect(normalizePromptResult(null)).toBe('');
        expect(normalizePromptResult(undefined)).toBe('');
    });
    test('required + empty → null (signal validation failure)', () => {
        expect(normalizePromptResult('  ', { required: true })).toBeNull();
        expect(normalizePromptResult('',   { required: true })).toBeNull();
    });
    test('required + non-empty → trimmed string', () => {
        expect(normalizePromptResult(' foo ', { required: true })).toBe('foo');
    });
});
