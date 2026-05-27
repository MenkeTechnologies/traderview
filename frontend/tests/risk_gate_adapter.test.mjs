// Risk-gate form adapter tests. Pins the form→ProposedTrade shape so a
// rename in new_trade.js doesn't silently break the engine call.
//
// Run: `node --test frontend/tests/risk_gate_adapter.test.mjs`

import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
    executionSideToTradeSide,
    buildProposedTrade,
    splitViolations,
} from '../js/_risk_gate_adapter.js';

// ─── executionSideToTradeSide ────────────────────────────────────────────

test('buy + sell are LONG-side operations', () => {
    assert.equal(executionSideToTradeSide('buy'),  'long');
    assert.equal(executionSideToTradeSide('sell'), 'long');
});

test('short + cover are SHORT-side operations', () => {
    assert.equal(executionSideToTradeSide('short'), 'short');
    assert.equal(executionSideToTradeSide('cover'), 'short');
});

test('unknown side falls through to short (safety: no false-long)', () => {
    // Anything that isn't a recognized buy/sell would have already been
    // rejected by the form's <select>; this is just a defensive default.
    assert.equal(executionSideToTradeSide('???'), 'short');
});

// ─── buildProposedTrade ──────────────────────────────────────────────────

test('buildProposedTrade pulls every field through correctly', () => {
    const p = buildProposedTrade({
        symbol: 'AAPL', side: 'buy', qty: 100, price: 150,
        stop_loss: '149', asset_class: 'stock', multiplier: 1,
        has_attached_plan: true,
    });
    assert.deepEqual(p, {
        symbol: 'AAPL',
        side: 'long',
        qty: 100,
        entry_price: 150,
        stop_loss: 149,
        asset_class: 'stock',
        multiplier: 1,
        tick_size: null,
        tick_value: null,
        has_attached_plan: true,
    });
});

test('buildProposedTrade nulls stop_loss when missing', () => {
    const p = buildProposedTrade({
        symbol: 'X', side: 'sell', qty: 1, price: 1,
    });
    assert.equal(p.stop_loss, null,
        'missing stop must be null, not the empty string');
});

test('buildProposedTrade nulls stop_loss when empty string', () => {
    const p = buildProposedTrade({
        symbol: 'X', side: 'buy', qty: 1, price: 1, stop_loss: '',
    });
    assert.equal(p.stop_loss, null);
});

test('buildProposedTrade coerces stop_loss string to number', () => {
    const p = buildProposedTrade({
        symbol: 'X', side: 'buy', qty: 1, price: 1, stop_loss: '49.50',
    });
    assert.equal(p.stop_loss, 49.5);
    assert.equal(typeof p.stop_loss, 'number');
});

test('buildProposedTrade defaults asset_class to stock', () => {
    const p = buildProposedTrade({ symbol: 'X', side: 'buy', qty: 1, price: 1 });
    assert.equal(p.asset_class, 'stock');
});

test('buildProposedTrade defaults multiplier to 1 when missing', () => {
    const p = buildProposedTrade({ symbol: 'X', side: 'buy', qty: 1, price: 1 });
    assert.equal(p.multiplier, 1);
});

test('buildProposedTrade preserves option multiplier', () => {
    const p = buildProposedTrade({
        symbol: 'AAPL', side: 'buy', qty: 1, price: 5,
        asset_class: 'option', multiplier: 100,
    });
    assert.equal(p.multiplier, 100);
});

test('has_attached_plan coerces to boolean', () => {
    assert.equal(buildProposedTrade({ has_attached_plan: 'on' }).has_attached_plan, true);
    assert.equal(buildProposedTrade({ has_attached_plan: false }).has_attached_plan, false);
    assert.equal(buildProposedTrade({}).has_attached_plan, false);
});

// ─── splitViolations ─────────────────────────────────────────────────────

test('splitViolations partitions by severity', () => {
    const d = { allow: false, violations: [
        { rule: 'a', severity: 'block',   message: 'x' },
        { rule: 'b', severity: 'warning', message: 'y' },
        { rule: 'c', severity: 'block',   message: 'z' },
    ]};
    const { blocks, warnings } = splitViolations(d);
    assert.equal(blocks.length, 2);
    assert.equal(warnings.length, 1);
    assert.equal(blocks[0].rule, 'a');
    assert.equal(warnings[0].rule, 'b');
});

test('splitViolations handles null decision gracefully', () => {
    // The gate call can fail (network down) and we set decision=null. The
    // adapter must not throw — both arrays empty.
    const { blocks, warnings } = splitViolations(null);
    assert.equal(blocks.length, 0);
    assert.equal(warnings.length, 0);
});

test('splitViolations handles missing violations array', () => {
    const { blocks, warnings } = splitViolations({ allow: true });
    assert.equal(blocks.length, 0);
    assert.equal(warnings.length, 0);
});

test('splitViolations on empty violations returns empty arrays', () => {
    const { blocks, warnings } = splitViolations({ allow: true, violations: [] });
    assert.equal(blocks.length, 0);
    assert.equal(warnings.length, 0);
});
