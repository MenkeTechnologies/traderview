// Trade-Plan Checklist helpers: validator, body shape, local evaluator
// (mirrors the Rust gate emitter), demo presets, formatters.

import { test, expect } from 'vitest';
import {
    DEFAULT_CONFIG, validateInputs, buildBody, localEvaluate,
    gateLabel, gateCls, gateIcon, makeDemoData,
    fmtPct, fmtR,
} from '../js/_trade_plan_checklist_inputs.js';

const goodPlan = () => makeDemoData('good');
const cfg = () => ({ ...DEFAULT_CONFIG });

// ── validateInputs ───────────────────────────────────────────────

test('validate accepts the good demo plan', () => {
    expect(validateInputs(goodPlan(), cfg())).toBe(null);
});

test('validate rejects non-string thesis', () => {
    expect(validateInputs({ ...goodPlan(), thesis: 42 }, cfg())).toMatch(/thesis/);
});

test('validate rejects non-positive entry / risk_dollars < 0 / non-positive equity', () => {
    expect(validateInputs({ ...goodPlan(), entry_price: 0 }, cfg())).toMatch(/entry_price/);
    expect(validateInputs({ ...goodPlan(), risk_dollars: -1 }, cfg())).toMatch(/risk_dollars/);
    expect(validateInputs({ ...goodPlan(), account_equity: 0 }, cfg())).toMatch(/account_equity/);
});

test('validate rejects bad stop / target shapes but accepts null', () => {
    expect(validateInputs({ ...goodPlan(), stop_price: 0 }, cfg())).toMatch(/stop_price/);
    expect(validateInputs({ ...goodPlan(), target_price: 0 }, cfg())).toMatch(/target_price/);
    expect(validateInputs({ ...goodPlan(), stop_price: null, target_price: null }, cfg())).toBe(null);
});

test('validate rejects non-boolean is_long', () => {
    expect(validateInputs({ ...goodPlan(), is_long: 'yes' }, cfg())).toMatch(/is_long/);
});

test('validate rejects bad config thresholds', () => {
    expect(validateInputs(goodPlan(), { ...cfg(), min_thesis_words: -1 })).toMatch(/min_thesis_words/);
    expect(validateInputs(goodPlan(), { ...cfg(), min_thesis_words: 1.5 })).toMatch(/min_thesis_words/);
    expect(validateInputs(goodPlan(), { ...cfg(), min_r_multiple: -1 })).toMatch(/min_r_multiple/);
    expect(validateInputs(goodPlan(), { ...cfg(), max_risk_pct_per_trade: -0.01 })).toMatch(/max_risk_pct/);
    expect(validateInputs(goodPlan(), { ...cfg(), max_risk_pct_per_trade: 1.5 })).toMatch(/max_risk_pct/);
});

// ── buildBody ────────────────────────────────────────────────────

test('buildBody nests { plan, config } per backend route contract', () => {
    const p = goodPlan(); const c = cfg();
    const body = buildBody(p, c);
    expect(body).toEqual({
        plan: {
            thesis: p.thesis, entry_price: p.entry_price, stop_price: p.stop_price,
            target_price: p.target_price, risk_dollars: p.risk_dollars,
            account_equity: p.account_equity, is_long: p.is_long,
        },
        config: { ...c },
    });
});

// ── localEvaluate ────────────────────────────────────────────────

test('localEvaluate: good plan passes all 7 gates', () => {
    const r = localEvaluate(goodPlan(), cfg());
    expect(r.gates.length).toBe(7);
    expect(r.all_passed).toBe(true);
    // entry=100, stop=98, target=106 → R = (106-100)/(100-98) = 3.0
    expect(r.computed_r_multiple).toBeCloseTo(3.0, 6);
    // risk=200 / equity=50_000 = 0.004 = 0.4%
    expect(r.risk_pct).toBeCloseTo(200 / 50_000, 10);
});

test('localEvaluate: no stop → only thesis + target + risk gates (5 total)', () => {
    const r = localEvaluate(makeDemoData('no-stop'), cfg());
    expect(r.gates.map(g => g.gate)).toEqual([
        'thesis_present', 'stop_loss_set', 'target_set', 'risk_within_max',
    ]);
    expect(r.gates.find(g => g.gate === 'stop_loss_set').passed).toBe(false);
    expect(r.computed_r_multiple).toBe(null);
    expect(r.all_passed).toBe(false);
});

test('localEvaluate: weak R-multiple fails the r gate (1R vs 1.5 min)', () => {
    const r = localEvaluate(makeDemoData('weak-r'), cfg());
    // entry=100, stop=98, target=102 → R = 2/2 = 1.0
    expect(r.computed_r_multiple).toBeCloseTo(1.0, 6);
    const g = r.gates.find(x => x.gate === 'r_multiple_meets_minimum');
    expect(g.passed).toBe(false);
    expect(r.all_passed).toBe(false);
});

test('localEvaluate: oversize risk fails risk_within_max (4% vs 2% cap)', () => {
    const r = localEvaluate(makeDemoData('oversize'), cfg());
    // 2000 / 50_000 = 0.04
    expect(r.risk_pct).toBeCloseTo(0.04, 10);
    const g = r.gates.find(x => x.gate === 'risk_within_max');
    expect(g.passed).toBe(false);
    expect(r.all_passed).toBe(false);
});

test('localEvaluate: wrong-direction long target fails target_in_direction', () => {
    const r = localEvaluate(makeDemoData('wrong-target'), cfg());
    const g = r.gates.find(x => x.gate === 'target_in_direction');
    expect(g.passed).toBe(false);
    // Stop direction still valid for long (stop 98 < entry 100).
    const s = r.gates.find(x => x.gate === 'stop_in_direction');
    expect(s.passed).toBe(true);
});

test('localEvaluate: short trade with stop above / target below passes direction gates', () => {
    const r = localEvaluate(makeDemoData('short-trade'), cfg());
    // entry=100, stop=102, target=94 → R = 6/2 = 3.0
    expect(r.computed_r_multiple).toBeCloseTo(3.0, 6);
    expect(r.gates.find(g => g.gate === 'target_in_direction').passed).toBe(true);
    expect(r.gates.find(g => g.gate === 'stop_in_direction').passed).toBe(true);
    expect(r.all_passed).toBe(true);
});

test('localEvaluate: short trade with stop BELOW entry fails stop_in_direction', () => {
    const bad = { ...makeDemoData('short-trade'), stop_price: 98 };
    const r = localEvaluate(bad, cfg());
    expect(r.gates.find(g => g.gate === 'stop_in_direction').passed).toBe(false);
});

test('localEvaluate: no thesis fails thesis_present (1 word < 10 min)', () => {
    const r = localEvaluate(makeDemoData('no-thesis'), cfg());
    expect(r.gates.find(g => g.gate === 'thesis_present').passed).toBe(false);
});

test('localEvaluate: thesis word count counts non-blank tokens only', () => {
    const r = localEvaluate(
        { ...goodPlan(), thesis: '  one   two\tthree\n\nfour ' },
        { ...cfg(), min_thesis_words: 4 });
    expect(r.gates.find(g => g.gate === 'thesis_present').reason).toMatch(/^4 words/);
});

test('localEvaluate: equal-to-min thresholds pass (≥, not >)', () => {
    // R = 1.5 exactly: target = 100 + 1.5 * (100-98) = 103
    const r = localEvaluate({ ...goodPlan(), target_price: 103 }, cfg());
    expect(r.computed_r_multiple).toBeCloseTo(1.5, 6);
    expect(r.gates.find(g => g.gate === 'r_multiple_meets_minimum').passed).toBe(true);
    // Risk = 2% exactly.
    const r2 = localEvaluate({ ...goodPlan(), risk_dollars: 1000 }, cfg());
    expect(r2.risk_pct).toBeCloseTo(0.02, 10);
    expect(r2.gates.find(g => g.gate === 'risk_within_max').passed).toBe(true);
});

test('localEvaluate: stop = entry → risk distance zero → r-multiple gate fails', () => {
    const r = localEvaluate({ ...goodPlan(), stop_price: 100, target_price: 106 }, cfg());
    expect(r.computed_r_multiple).toBe(0);
    expect(r.gates.find(g => g.gate === 'r_multiple_meets_minimum').passed).toBe(false);
});

// ── presentation helpers ─────────────────────────────────────────

test('gateLabel maps each known gate id to a friendly label', () => {
    expect(gateLabel('thesis_present')).toMatch(/Thesis/);
    expect(gateLabel('stop_loss_set')).toMatch(/Stop/);
    expect(gateLabel('r_multiple_meets_minimum')).toMatch(/R-multiple/);
    expect(gateLabel('unknown')).toBe('unknown');
});

test('gateCls / gateIcon pick pos vs neg', () => {
    expect(gateCls(true)).toBe('pos');
    expect(gateCls(false)).toBe('neg');
    expect(gateIcon(true)).toBe('✓');
    expect(gateIcon(false)).toBe('×');
});

test('fmtPct / fmtR format and guard non-finite', () => {
    expect(fmtPct(0.0234)).toBe('2.34%');
    expect(fmtR(1.5)).toBe('1.50R');
    expect(fmtPct(NaN)).toBe('—');
    expect(fmtR(null)).toBe('—');
});

test('DEFAULT_CONFIG matches backend defaults (10 / 1.5 / 0.02)', () => {
    expect(DEFAULT_CONFIG).toEqual({
        min_thesis_words: 10, min_r_multiple: 1.5, max_risk_pct_per_trade: 0.02,
    });
});
