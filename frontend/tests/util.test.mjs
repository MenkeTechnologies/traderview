// Display formatters from frontend/js/util.js. These render numbers, money,
// percents, durations, dates, and HTML into every screen — a wrong format
// means a wrong number shown to a trader. The bugs these guards against:
//   * XSS via unescaped HTML in user-controlled labels (esc).
//   * Silent NaN/Infinity rendering instead of an explicit sentinel (fmt).
//   * Wrong sign / wrong magnitude on percent + currency.
//   * Duration switching unit thresholds at the wrong boundary.
//   * Invalid ISO strings rendering as "Invalid Date" without a guard.
//   * pnlClass picking the wrong CSS class at zero / negative-zero.
//   * statCard skipping HTML-escape on the label and enabling injection.
//
// Run: `node --test frontend/tests/util.test.mjs`

import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
    esc,
    fmt,
    fmtDate,
    fmtDateTime,
    fmtMoney,
    fmtPct,
    fmtSecs,
    html,
    pnlClass,
    statCard,
} from '../js/util.js';

// ─── esc ──────────────────────────────────────────────────────────────────

test('esc escapes ampersand first to avoid double-encoding', () => {
    assert.equal(esc('Tom & Jerry'), 'Tom &amp; Jerry');
});

test('esc escapes less-than and greater-than (script tag guard)', () => {
    assert.equal(esc('<script>alert(1)</script>'),
        '&lt;script&gt;alert(1)&lt;/script&gt;');
});

test('esc escapes double quotes (attribute-context guard)', () => {
    assert.equal(esc('say "hi"'), 'say &quot;hi&quot;');
});

test('esc escapes single quotes as numeric entity', () => {
    assert.equal(esc("it's"), 'it&#39;s');
});

test('esc escapes all five chars in one pass without double-encoding', () => {
    assert.equal(esc(`<a href="x">&'</a>`),
        '&lt;a href=&quot;x&quot;&gt;&amp;&#39;&lt;/a&gt;');
});

test('esc maps null to empty string (no "null" leaking into DOM)', () => {
    assert.equal(esc(null), '');
});

test('esc maps undefined to empty string', () => {
    assert.equal(esc(undefined), '');
});

test('esc coerces numbers to string', () => {
    assert.equal(esc(42), '42');
});

test('esc preserves plain ASCII unchanged', () => {
    assert.equal(esc('hello world'), 'hello world');
});

// ─── fmt ──────────────────────────────────────────────────────────────────

test('fmt rounds to 2 decimals by default', () => {
    assert.equal(fmt(1234.5), '1,234.50');
});

test('fmt with decimals=0 drops the fractional part', () => {
    assert.equal(fmt(1234.5, 0), '1,235');
});

test('fmt with decimals=4 widens precision', () => {
    assert.equal(fmt(1.23456, 4), '1.2346');
});

test('fmt formats negative numbers with leading minus', () => {
    assert.equal(fmt(-1234.5), '-1,234.50');
});

test('fmt formats integer input with trailing zeros to decimals', () => {
    assert.equal(fmt(7), '7.00');
});

test('fmt formats large numbers with grouping separators', () => {
    assert.equal(fmt(1000000), '1,000,000.00');
});

test('fmt returns em-dash sentinel for null', () => {
    assert.equal(fmt(null), '—');
});

test('fmt returns em-dash sentinel for undefined', () => {
    assert.equal(fmt(undefined), '—');
});

test('fmt returns em-dash for empty string (form-field idle state)', () => {
    assert.equal(fmt(''), '—');
});

test('fmt returns em-dash for NaN (not the string "NaN")', () => {
    assert.equal(fmt(NaN), '—');
});

test('fmt returns em-dash for +Infinity', () => {
    assert.equal(fmt(Infinity), '—');
});

test('fmt returns em-dash for -Infinity', () => {
    assert.equal(fmt(-Infinity), '—');
});

test('fmt coerces numeric string input', () => {
    assert.equal(fmt('42.5'), '42.50');
});

test('fmt returns em-dash for non-numeric string', () => {
    // Number('abc') === NaN, Number.isFinite(NaN) === false → '—' (was '∞').
    assert.equal(fmt('abc'), '—');
});

// ─── fmtMoney ─────────────────────────────────────────────────────────────

test('fmtMoney prefixes dollar sign and formats with 2 decimals', () => {
    assert.equal(fmtMoney(1234.5), '$1,234.50');
});

test('fmtMoney formats zero as $0.00', () => {
    assert.equal(fmtMoney(0), '$0.00');
});

test('fmtMoney puts minus sign BEFORE the dollar sign', () => {
    // Accounting convention: -$100.00, not $-100.00.
    assert.equal(fmtMoney(-100), '-$100.00');
});

test('fmtMoney formats null as em-dash', () => {
    assert.equal(fmtMoney(null), '—');
});

test('fmtMoney formats undefined as em-dash', () => {
    assert.equal(fmtMoney(undefined), '—');
});

test('fmtMoney handles small negative decimals correctly', () => {
    assert.equal(fmtMoney(-0.5), '-$0.50');
});

test('fmtMoney formats large positive with grouping', () => {
    assert.equal(fmtMoney(1234567.89), '$1,234,567.89');
});

// ─── fmtPct ───────────────────────────────────────────────────────────────

test('fmtPct multiplies by 100 and appends % with 1 decimal', () => {
    assert.equal(fmtPct(0.1234), '12.3%');
});

test('fmtPct formats zero as 0.0%', () => {
    assert.equal(fmtPct(0), '0.0%');
});

test('fmtPct formats negative with leading minus', () => {
    assert.equal(fmtPct(-0.05), '-5.0%');
});

test('fmtPct rounds near-zero up to one decimal place', () => {
    // 0.0001 * 100 = 0.01 → toFixed(1) → "0.0"
    assert.equal(fmtPct(0.0001), '0.0%');
});

test('fmtPct returns em-dash for null', () => {
    assert.equal(fmtPct(null), '—');
});

test('fmtPct returns em-dash for undefined', () => {
    assert.equal(fmtPct(undefined), '—');
});

test('fmtPct returns em-dash for NaN', () => {
    // Fixed: NaN now hits the !isFinite branch and returns '—' like null.
    assert.equal(fmtPct(NaN), '—');
});

test('fmtPct returns em-dash for Infinity', () => {
    assert.equal(fmtPct(Infinity), '—');
});

test('fmtPct handles 1.0 as 100.0%', () => {
    assert.equal(fmtPct(1), '100.0%');
});

// ─── fmtSecs ──────────────────────────────────────────────────────────────

test('fmtSecs formats 0 as "0s"', () => {
    assert.equal(fmtSecs(0), '0s');
});

test('fmtSecs formats sub-minute as Ns', () => {
    assert.equal(fmtSecs(45), '45s');
});

test('fmtSecs switches to minutes at exactly 60s', () => {
    assert.equal(fmtSecs(60), '1.0m');
});

test('fmtSecs formats minutes with 1 decimal', () => {
    assert.equal(fmtSecs(90), '1.5m');
});

test('fmtSecs switches to hours at exactly 3600s', () => {
    assert.equal(fmtSecs(3600), '1.0h');
});

test('fmtSecs formats hours with 1 decimal', () => {
    assert.equal(fmtSecs(5400), '1.5h');
});

test('fmtSecs switches to days at exactly 86400s', () => {
    assert.equal(fmtSecs(86400), '1.0d');
});

test('fmtSecs formats multi-day with 1 decimal', () => {
    assert.equal(fmtSecs(86400 * 2.5), '2.5d');
});

test('fmtSecs returns em-dash for null', () => {
    assert.equal(fmtSecs(null), '—');
});

test('fmtSecs returns em-dash for undefined', () => {
    assert.equal(fmtSecs(undefined), '—');
});

test('fmtSecs coerces numeric string', () => {
    assert.equal(fmtSecs('90'), '1.5m');
});

// ─── fmtDate ──────────────────────────────────────────────────────────────

test('fmtDate slices ISO to YYYY-MM-DD', () => {
    assert.equal(fmtDate('2026-05-27T15:30:00Z'), '2026-05-27');
});

test('fmtDate returns empty string for null', () => {
    assert.equal(fmtDate(null), '');
});

test('fmtDate returns empty string for undefined', () => {
    assert.equal(fmtDate(undefined), '');
});

test('fmtDate passes through already-truncated date', () => {
    assert.equal(fmtDate('2026-05-27'), '2026-05-27');
});

// ─── fmtDateTime ──────────────────────────────────────────────────────────

test('fmtDateTime returns em-dash for null', () => {
    assert.equal(fmtDateTime(null), '—');
});

test('fmtDateTime returns em-dash for undefined', () => {
    assert.equal(fmtDateTime(undefined), '—');
});

test('fmtDateTime returns em-dash for empty string', () => {
    assert.equal(fmtDateTime(''), '—');
});

test('fmtDateTime returns em-dash for malformed ISO', () => {
    // Fixed: isNaN(date.getTime()) → '—' instead of "Invalid Date".
    assert.equal(fmtDateTime('not-an-iso'), '—');
});

test('fmtDateTime renders a string containing the year for a valid ISO', () => {
    // Exact format is locale-dependent; pin the year only.
    const out = fmtDateTime('2026-05-27T15:30:00Z');
    assert.equal(typeof out, 'string');
    assert.match(out, /2026/);
});

test('fmtDateTime uses 24-hour clock (hour12: false) — no AM/PM', () => {
    const out = fmtDateTime('2026-05-27T15:30:00Z');
    assert.doesNotMatch(out, /\bAM\b|\bPM\b/);
});

// ─── pnlClass ─────────────────────────────────────────────────────────────

test('pnlClass returns "pos" for positive number', () => {
    assert.equal(pnlClass(100), 'pos');
});

test('pnlClass returns "neg" for negative number', () => {
    assert.equal(pnlClass(-100), 'neg');
});

test('pnlClass treats exact zero as "flat" (break-even)', () => {
    assert.equal(pnlClass(0), 'flat');
});

test('pnlClass treats -0 as "flat" (break-even)', () => {
    assert.equal(pnlClass(-0), 'flat');
});

test('pnlClass coerces numeric string', () => {
    assert.equal(pnlClass('-1.5'), 'neg');
});

test('pnlClass returns "flat" for NaN (missing data, not loss)', () => {
    // Fixed: NaN no longer colors red as if it were a loss.
    assert.equal(pnlClass(NaN), 'flat');
});

test('pnlClass returns "flat" for null/undefined', () => {
    assert.equal(pnlClass(null), 'flat');
    assert.equal(pnlClass(undefined), 'flat');
});

// ─── html (tagged template helper) ────────────────────────────────────────

test('html concatenates static + interpolated parts in order', () => {
    const x = 1, y = 2;
    assert.equal(html`a=${x},b=${y}`, 'a=1,b=2');
});

test('html replaces undefined interpolations with empty string', () => {
    assert.equal(html`x=${undefined}`, 'x=');
});

test('html does NOT escape — caller is responsible (contract check)', () => {
    // This is intentional: html` ` is a string-concat helper, not an
    // auto-escaper. Pinning so a future "auto-escape" change is deliberate.
    assert.equal(html`<b>${'<x>'}</b>`, '<b><x></b>');
});

// ─── statCard ─────────────────────────────────────────────────────────────

test('statCard returns a div.card containing the label', () => {
    const out = statCard('PnL', '$100');
    assert.match(out, /<div class="card">/);
    assert.match(out, /PnL/);
    assert.match(out, /\$100/);
});

test('statCard applies the optional modifier class on the value div', () => {
    const out = statCard('PnL', '$100', 'pos');
    assert.match(out, /<div class="value pos">/);
});

test('statCard renders empty modifier class when omitted', () => {
    const out = statCard('PnL', '$100');
    assert.match(out, /<div class="value ">/);
});

test('statCard HTML-escapes the label (XSS guard)', () => {
    const out = statCard('<script>', 'safe');
    assert.match(out, /&lt;script&gt;/);
    assert.doesNotMatch(out, /<script>/);
});

test('statCard escapes the value to prevent XSS', () => {
    // Fixed: value now passes through esc(). Pre-rendered safe HTML (e.g.
    // fmtMoney output, which contains no `<>"`) survives the escape
    // unchanged; user-controlled strings are neutralized.
    const out = statCard('label', '<b>raw</b>');
    assert.doesNotMatch(out, /<b>raw<\/b>/, 'must not contain raw <b> tag');
    assert.match(out, /&lt;b&gt;raw&lt;\/b&gt;/, 'must contain escaped form');
});

test('statCard escapes scripts in the value (XSS regression)', () => {
    const out = statCard('label', '<script>alert(1)</script>');
    assert.doesNotMatch(out, /<script>/);
});
