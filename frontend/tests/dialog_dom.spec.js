// @vitest-environment jsdom
//
// DOM spec for dialog.js — the tConfirm/tPrompt modal glue. The pure
// helpers live in dialog_inputs.spec.js; this pins the rendered DOM:
// the `detail` option (raw dynamic context under the message), default
// values, and resolution semantics that the paper/algo views rely on
// after the native-prompt() conversion.

import { describe, test, expect, beforeEach } from 'vitest';
import { tConfirm, tPrompt } from '../js/dialog.js';

beforeEach(() => { document.body.innerHTML = ''; });

const root = () => document.getElementById('tv-dialog-root');

describe('tPrompt DOM', () => {
    test('renders detail text under the message, escaped', async () => {
        const p = tPrompt('view.paper.prompt.move_to', {}, {
            detail: '1. Main\n2. <img src=x onerror=alert(1)>',
        });
        const det = root().querySelector('.tv-dialog-detail');
        expect(det).toBeTruthy();
        // Escaped: the markup is text, not an element.
        expect(det.querySelector('img')).toBeNull();
        expect(det.textContent).toContain('2. <img src=x onerror=alert(1)>');
        root().querySelector('.tv-dialog-cancel').click();
        expect(await p).toBeNull();
    });

    test('no detail option → no detail node', async () => {
        const p = tPrompt('view.paper.prompt.replace_qty');
        expect(root().querySelector('.tv-dialog-detail')).toBeNull();
        root().querySelector('.tv-dialog-cancel').click();
        await p;
    });

    test('defaultValue pre-fills the input and confirm resolves it', async () => {
        const p = tPrompt('view.paper.prompt.trail', {}, { defaultValue: '5%' });
        const input = root().querySelector('#tv-dialog-input');
        expect(input.value).toBe('5%');
        root().querySelector('.tv-dialog-confirm').click();
        expect(await p).toBe('5%');
    });

    test('cancel resolves null — Number(null) stays falsy like Number(NaN) was', async () => {
        const p = tPrompt('view.paper.prompt.replace_limit');
        root().querySelector('.tv-dialog-cancel').click();
        const v = await p;
        expect(v).toBeNull();
        expect(Number(v)).toBeFalsy();
    });

    test('typed value resolves trimmed', async () => {
        const p = tPrompt('view.paper.prompt.roll_to');
        const input = root().querySelector('#tv-dialog-input');
        input.value = '  spy260619c00600000  ';
        root().querySelector('.tv-dialog-confirm').click();
        expect(await p).toBe('spy260619c00600000');
    });

    test('invalid detail option resolves null without rendering', async () => {
        const v = await tPrompt('view.paper.prompt.replace_qty', {}, { detail: 42 });
        expect(v).toBeNull();
        // Validation rejects before the mount is even created — nothing
        // rendered anywhere.
        expect(document.querySelector('.tv-dialog-overlay')).toBeNull();
    });
});

describe('tConfirm DOM', () => {
    test('confirm resolves true, cancel resolves false', async () => {
        let p = tConfirm('view.paper.prompt.replace_qty');
        root().querySelector('.tv-dialog-confirm').click();
        expect(await p).toBe(true);
        p = tConfirm('view.paper.prompt.replace_qty');
        root().querySelector('.tv-dialog-cancel').click();
        expect(await p).toBe(false);
    });

    test('overlay click dismisses as cancel', async () => {
        const p = tConfirm('view.paper.prompt.replace_qty');
        const overlay = root().querySelector('.tv-dialog-overlay');
        overlay.dispatchEvent(new MouseEvent('click', { bubbles: true }));
        expect(await p).toBe(false);
    });

    test('opening a second dialog resolves the first as cancelled', async () => {
        // Previously the second innerHTML write orphaned the first
        // dialog's promise and left its document keydown listener live —
        // Enter would resolve the stale promise and wipe the new dialog.
        const first = tConfirm('view.paper.prompt.replace_qty');
        const second = tPrompt('view.paper.prompt.trail', {}, { defaultValue: '5%' });
        expect(await first).toBe(false);
        expect(document.querySelectorAll('.tv-dialog-overlay')).toHaveLength(1);
        root().querySelector('.tv-dialog-confirm').click();
        expect(await second).toBe('5%');
    });
});
