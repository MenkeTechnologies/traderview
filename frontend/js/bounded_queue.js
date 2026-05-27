// Bounded FIFO queue with oldest-first eviction. Extracted from
// error_reporter so the eviction policy can be unit-tested under
// `node --test` without DOM globals.
//
// Used by error_reporter to cap the in-flight reporting queue at MAX entries
// so a `console.error` hot loop before the backend is reachable can't OOM
// the renderer.

export class BoundedQueue {
    constructor(max) {
        if (!Number.isFinite(max) || max < 1) {
            throw new TypeError('BoundedQueue: max must be a positive integer');
        }
        this.max = max;
        this.items = [];
        this.dropped = 0;
    }

    /// Append `item`. If at capacity, evicts the oldest entry first and
    /// bumps the dropped counter. Returns true if an eviction happened.
    push(item) {
        let evicted = false;
        if (this.items.length >= this.max) {
            this.items.shift();
            this.dropped++;
            evicted = true;
        }
        this.items.push(item);
        return evicted;
    }

    shift() { return this.items.shift(); }
    get length() { return this.items.length; }
    peek(i = 0) { return this.items[i]; }
}
