// Per-view race-token semantics. The bugs this guards against:
//   * View A's await resolves AFTER the user navigated to View B → must bail.
//   * Two views captured at different dispatches must NOT match the same token.
//   * After dispatch bumps, an old captured token must read stale.
//
// Run: `node --test frontend/tests/view_token.test.mjs`

import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
    bumpViewToken,
    currentViewToken,
    viewIsCurrent,
} from '../js/view_token.js';

test('viewIsCurrent returns true for the captured token before any nav', () => {
    const tok = currentViewToken();
    assert.equal(viewIsCurrent(tok), true);
});

test('bumpViewToken invalidates the prior captured token', () => {
    const tok = currentViewToken();
    bumpViewToken();
    assert.equal(viewIsCurrent(tok), false,
        'captured token must read stale after a bump');
});

test('the new token after bump is current', () => {
    const newTok = bumpViewToken();
    assert.equal(viewIsCurrent(newTok), true);
});

test('two views captured at different dispatches do not match', () => {
    bumpViewToken();
    const tokA = currentViewToken();
    bumpViewToken();
    const tokB = currentViewToken();
    assert.notEqual(tokA, tokB);
    assert.equal(viewIsCurrent(tokA), false);
    assert.equal(viewIsCurrent(tokB), true);
});

test('1000 rapid bumps still leave the latest token current and all prior stale', () => {
    const oldTok = currentViewToken();
    let last = oldTok;
    for (let i = 0; i < 1000; i++) last = bumpViewToken();
    assert.equal(viewIsCurrent(last), true);
    assert.equal(viewIsCurrent(oldTok), false);
});
