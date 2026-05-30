// Spec for `dataFromTarget(detail, key)` — the dataset-chain helper
// every row-context handler in context_menu.js uses to read payload
// from the right-clicked element.

import { test, expect, describe } from 'vitest';
import { dataFromTarget, symbolFromTarget } from '../js/_context_menu.js';

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

describe('symbolFromTarget', () => {
    test('reads data-symbol and uppercases', () => {
        expect(symbolFromTarget({ target: { dataset: { symbol: 'aapl' } } })).toBe('AAPL');
    });
    test('already-uppercase symbol passes through', () => {
        expect(symbolFromTarget({ target: { dataset: { symbol: 'NVDA' } } })).toBe('NVDA');
    });
    test('mixed-case from broker feeds normalizes', () => {
        expect(symbolFromTarget({ target: { dataset: { symbol: 'Brk.B' } } })).toBe('BRK.B');
    });
    test('crypto symbols preserve the suffix', () => {
        expect(symbolFromTarget({ target: { dataset: { symbol: 'btc-usd' } } })).toBe('BTC-USD');
    });
    test('index symbols preserve the caret', () => {
        expect(symbolFromTarget({ target: { dataset: { symbol: '^gspc' } } })).toBe('^GSPC');
    });
    test('missing data-symbol → null', () => {
        expect(symbolFromTarget({ target: { dataset: {} } })).toBeNull();
    });
    test('empty data-symbol → null (treated as missing)', () => {
        expect(symbolFromTarget({ target: { dataset: { symbol: '' } } })).toBeNull();
    });
    test('null detail / target safe', () => {
        expect(symbolFromTarget(null)).toBeNull();
        expect(symbolFromTarget({ target: null })).toBeNull();
    });
});
