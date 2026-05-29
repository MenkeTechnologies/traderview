// News Event Handler helpers: position parser, event parser (multi-token
// with auto-detected impact position), validator, body shape, trim
// fractions, badges, summary, demo, formatters.

import { test, expect } from 'vitest';
import {
    parsePositions, parseEvents, validateInputs, buildBody,
    trimFractionFor, impactBadge, summarize, makeDemoData,
    fmtN, fmtInt, fmtPct,
} from '../js/_news_event_inputs.js';

// ── parsePositions ────────────────────────────────────────────────

test('parsePositions uppercases + comma-tolerant', () => {
    const r = parsePositions('# header\naapl 100\nspy, 50');
    expect(r.errors).toEqual([]);
    expect(r.positions).toEqual([
        { symbol: 'AAPL', current_qty: 100 },
        { symbol: 'SPY',  current_qty: 50 },
    ]);
});

test('parsePositions rejects wrong token count', () => {
    expect(parsePositions('AAPL').errors[0].message).toMatch(/expected 2 tokens/);
});

test('parsePositions rejects bad symbol + non-positive qty', () => {
    expect(parsePositions('A!P 100').errors[0].message).toMatch(/bad symbol/);
    expect(parsePositions('AAPL 0').errors[0].message).toMatch(/qty/);
    expect(parsePositions('AAPL -1').errors[0].message).toMatch(/qty/);
});

test('parsePositions non-string returns 1 error', () => {
    expect(parsePositions(null).errors.length).toBe(1);
});

// ── parseEvents — multi-token event-name handling ────────────────

test('parseEvents accepts market-wide event (no affected symbols)', () => {
    const r = parseEvents('FOMC critical');
    expect(r.errors).toEqual([]);
    expect(r.events).toEqual([
        { event_name: 'FOMC', impact: 'critical', affected_symbols: [] },
    ]);
});

test('parseEvents accepts symbol-specific event (comma-sep)', () => {
    const r = parseEvents('CPI high TSLA,SPY');
    expect(r.errors).toEqual([]);
    expect(r.events[0].affected_symbols).toEqual(['TSLA', 'SPY']);
});

test('parseEvents handles multi-word event names with later impact token', () => {
    const r = parseEvents('Retail sales medium MSFT');
    expect(r.errors).toEqual([]);
    expect(r.events[0]).toEqual({
        event_name: 'Retail sales', impact: 'medium', affected_symbols: ['MSFT'],
    });
});

test('parseEvents handles 3-word event name', () => {
    const r = parseEvents('Bank of England high');
    expect(r.errors).toEqual([]);
    expect(r.events[0].event_name).toBe('Bank of England');
});

test('parseEvents accepts whitespace-separated affected symbols', () => {
    const r = parseEvents('FOMC critical AAPL TSLA SPY');
    expect(r.errors).toEqual([]);
    expect(r.events[0].affected_symbols).toEqual(['AAPL', 'TSLA', 'SPY']);
});

test('parseEvents rejects missing impact', () => {
    expect(parseEvents('Some event AAPL').errors[0].message).toMatch(/expected/);
});

test('parseEvents rejects impact as first token (no event name)', () => {
    expect(parseEvents('critical AAPL').errors[0].message).toMatch(/expected/);
});

test('parseEvents rejects bad affected symbol', () => {
    expect(parseEvents('CPI high A!P,SPY').errors[0].message).toMatch(/bad symbol/);
});

test('parseEvents skips comments + blanks', () => {
    const r = parseEvents('# header\n\nFOMC critical\n# tail\nCPI high');
    expect(r.errors).toEqual([]);
    expect(r.events.length).toBe(2);
});

test('parseEvents non-string returns 1 error', () => {
    expect(parseEvents(null).errors.length).toBe(1);
});

test('parseEvents impact match is case-insensitive', () => {
    const r = parseEvents('FOMC CRITICAL');
    expect(r.errors).toEqual([]);
    expect(r.events[0].impact).toBe('critical');
});

// ── validateInputs / buildBody ────────────────────────────────────

test('validate requires ≥1 position + events array', () => {
    expect(validateInputs([], [])).toMatch(/at least 1 position/);
    expect(validateInputs([{ symbol: 'X', current_qty: 1 }], 'not-array')).toMatch(/events/);
});

test('validate accepts empty events (legal "calendar clear" check)', () => {
    expect(validateInputs([{ symbol: 'X', current_qty: 1 }], [])).toBe(null);
});

test('buildBody emits backend NewsEventBody shape', () => {
    const p = [{ symbol: 'A', current_qty: 1 }];
    const e = [{ event_name: 'X', impact: 'low', affected_symbols: [] }];
    expect(buildBody(p, e)).toEqual({ positions: p, events: e });
});

// ── trimFractionFor (backend parity) ─────────────────────────────

test('trimFractionFor: Low=0 / Medium=0.25 / High=0.50 / Critical=1.0', () => {
    expect(trimFractionFor('low')).toBe(0.0);
    expect(trimFractionFor('medium')).toBe(0.25);
    expect(trimFractionFor('high')).toBe(0.50);
    expect(trimFractionFor('critical')).toBe(1.0);
});

test('trimFractionFor unknown → 0 (safe default)', () => {
    expect(trimFractionFor('garbage')).toBe(0.0);
    expect(trimFractionFor(null)).toBe(0.0);
});

// ── impactBadge ──────────────────────────────────────────────────

test('impactBadge: high/critical → neg, low → pos, medium → empty', () => {
    expect(impactBadge('low').cls).toBe('pos');
    expect(impactBadge('medium').cls).toBe('');
    expect(impactBadge('high').cls).toBe('neg');
    expect(impactBadge('critical').cls).toBe('neg');
    expect(impactBadge('garbage').label).toBe('garbage');
});

// ── summarize ────────────────────────────────────────────────────

test('summarize aggregates action counts + total trim + critical-action count', () => {
    const positions = [
        { symbol: 'A', current_qty: 100 },
        { symbol: 'B', current_qty: 50 },
        { symbol: 'C', current_qty: 200 },
    ];
    const report = {
        actions: [
            { symbol: 'A', trim_amount: 100, reason: 'trim 100% due to Critical impact ...' },
            { symbol: 'B', trim_amount: 25,  reason: 'trim 50% due to High impact ...' },
        ],
    };
    const s = summarize(report, positions);
    expect(s.positionCount).toBe(3);
    expect(s.actionCount).toBe(2);
    expect(s.unchanged).toBe(1);
    expect(s.totalTrim).toBe(125);
    expect(s.critical).toBe(1);
});

test('summarize null-report safe', () => {
    const s = summarize(null, []);
    expect(s.actionCount).toBe(0);
    expect(s.totalTrim).toBe(0);
});

// ── makeDemoData ──────────────────────────────────────────────────

test('makeDemoData: 5 positions + 4 events spanning all 4 impact tiers', () => {
    const { positions, events } = makeDemoData();
    expect(positions.length).toBe(5);
    expect(events.length).toBe(4);
    const impacts = events.map(e => e.impact).sort();
    expect(impacts).toEqual(['critical', 'high', 'low', 'medium']);
});

test('makeDemoData includes a market-wide event AND a symbol-specific event', () => {
    const { events } = makeDemoData();
    expect(events.some(e => e.affected_symbols.length === 0)).toBe(true);
    expect(events.some(e => e.affected_symbols.length > 0)).toBe(true);
});

// ── formatters ────────────────────────────────────────────────────

test('formatters', () => {
    expect(fmtN(1.234)).toBe('1.23');
    expect(fmtN(NaN)).toBe('—');
    expect(fmtInt(1234)).toBe('1,234');
    expect(fmtInt(NaN)).toBe('—');
    expect(fmtPct(0.25)).toBe('25%');
    expect(fmtPct(NaN)).toBe('—');
});
