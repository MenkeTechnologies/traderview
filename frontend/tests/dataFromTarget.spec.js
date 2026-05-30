// Spec for `dataFromTarget(detail, key)` — the dataset-chain helper
// every row-context handler in context_menu.js uses to read payload
// from the right-clicked element.

import { test, expect } from 'vitest';
import { dataFromTarget } from '../js/_context_menu.js';

test('returns the attribute when chain is intact', () => {
    const detail = { target: { dataset: { id: 'abc-123', symbol: 'AAPL' } } };
    expect(dataFromTarget(detail, 'id')).toBe('abc-123');
    expect(dataFromTarget(detail, 'symbol')).toBe('AAPL');
});

test('returns null when detail is null/undefined', () => {
    expect(dataFromTarget(null,      'id')).toBeNull();
    expect(dataFromTarget(undefined, 'id')).toBeNull();
});

test('returns null when target is null/undefined', () => {
    expect(dataFromTarget({ target: null },      'id')).toBeNull();
    expect(dataFromTarget({ target: undefined }, 'id')).toBeNull();
    expect(dataFromTarget({},                    'id')).toBeNull();
});

test('returns null when dataset is missing', () => {
    expect(dataFromTarget({ target: {} },                  'id')).toBeNull();
    expect(dataFromTarget({ target: { dataset: null } },   'id')).toBeNull();
});

test('returns null when the specific key is missing', () => {
    const detail = { target: { dataset: { id: 'x' } } };
    expect(dataFromTarget(detail, 'symbol')).toBeNull();
});

test('returns null for empty-string dataset values (falsy)', () => {
    // Mirrors how data-trade-id="" appears when no linked trade — the
    // handler should treat empty as "absent" and warn the user.
    const detail = { target: { dataset: { tradeId: '' } } };
    expect(dataFromTarget(detail, 'tradeId')).toBeNull();
});

test('camelCase key matches dataset HTML kebab-case attribute', () => {
    // <tr data-rule-id="X"> → dataset.ruleId; the caller passes the
    // camelCase form.
    const detail = { target: { dataset: { ruleId: 'r-42', wid: 'w-9' } } };
    expect(dataFromTarget(detail, 'ruleId')).toBe('r-42');
    expect(dataFromTarget(detail, 'wid')).toBe('w-9');
});
