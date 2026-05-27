// BoundedQueue (used by error_reporter to cap the in-flight queue). Bugs
// this guards against:
//   * Unbounded growth when the drain endpoint never resolves — OOM risk.
//   * Eviction order: must drop OLDEST first (FIFO), not newest.
//   * Dropped counter must reflect every eviction.
//
// Run: `node --test frontend/tests/bounded_queue.test.mjs`

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { BoundedQueue } from '../js/bounded_queue.js';

test('push below cap does not evict', () => {
    const q = new BoundedQueue(5);
    for (let i = 0; i < 5; i++) {
        assert.equal(q.push(i), false);
    }
    assert.equal(q.length, 5);
    assert.equal(q.dropped, 0);
});

test('push at cap evicts oldest', () => {
    const q = new BoundedQueue(3);
    q.push('a'); q.push('b'); q.push('c');
    assert.equal(q.length, 3);
    assert.equal(q.push('d'), true);
    assert.equal(q.length, 3, 'cap respected');
    assert.equal(q.dropped, 1);
    assert.equal(q.peek(0), 'b', 'oldest evicted; b is now head');
    assert.equal(q.peek(2), 'd', 'newest is tail');
});

test('eviction is FIFO across many pushes', () => {
    const q = new BoundedQueue(10);
    for (let i = 0; i < 100; i++) q.push(i);
    assert.equal(q.length, 10);
    assert.equal(q.dropped, 90);
    // The last 10 items (90..99) should remain.
    for (let i = 0; i < 10; i++) {
        assert.equal(q.peek(i), 90 + i);
    }
});

test('shift removes oldest', () => {
    const q = new BoundedQueue(5);
    q.push('a'); q.push('b'); q.push('c');
    assert.equal(q.shift(), 'a');
    assert.equal(q.length, 2);
    assert.equal(q.peek(0), 'b');
});

test('OOM safety: 100,000 pushes stay capped', () => {
    const q = new BoundedQueue(200);
    for (let i = 0; i < 100_000; i++) q.push(i);
    assert.equal(q.length, 200);
    assert.equal(q.dropped, 99_800);
});

test('constructor rejects non-positive max', () => {
    assert.throws(() => new BoundedQueue(0), TypeError);
    assert.throws(() => new BoundedQueue(-1), TypeError);
    assert.throws(() => new BoundedQueue('abc'), TypeError);
});
