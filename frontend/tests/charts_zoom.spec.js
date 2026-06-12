// @vitest-environment jsdom
//
// Spec for charts.js zoom machinery: `idxAxisValues` (index-scale axis
// labels) and `zoomPlugin` (wheel-zoom / drag-pan / spacebar-hand /
// button controls). uPlot itself isn't loaded — the plugin is exercised
// against a minimal mock `u` with the same surface (over, scales, data,
// posToVal, setScale), which is exactly the contract the plugin codes
// against.

import { describe, test, expect, beforeEach } from 'vitest';
import { idxAxisValues, zoomPlugin } from '../js/charts.js';

describe('idxAxisValues', () => {
    const labels = ['06/09', '06/10', '06/11'];
    const fn = idxAxisValues(labels);

    test('integer splits map to their label', () => {
        expect(fn(null, [0, 1, 2])).toEqual(['06/09', '06/10', '06/11']);
    });

    test('fractional splits render blank — not the rounded neighbor', () => {
        // The 1-bar daily-volume regression: range [-0.5, 0.5] makes
        // uPlot emit fractional splits that all rounded to index 0 and
        // repeated "06/11" across the whole axis.
        expect(fn(null, [-0.4, -0.2, 0.2, 0.4])).toEqual(['', '', '', '']);
    });

    test('near-integer float noise still labels', () => {
        expect(fn(null, [0.9999999999, 2.0000000001])).toEqual(['06/10', '06/11']);
    });

    test('out-of-range integers render blank', () => {
        expect(fn(null, [-1, 3, 99])).toEqual(['', '', '']);
    });
});

// ─── zoomPlugin against a mock uPlot instance ──────────────────────────────

function mockU({ data = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9], width = 600 } = {}) {
    const over = document.createElement('div');
    Object.defineProperty(over, 'clientWidth', { value: width });
    const wrap = document.createElement('div');
    wrap.appendChild(over);
    document.body.appendChild(wrap);
    const u = {
        over,
        data: [data],
        scales: { x: { min: data[0], max: data[data.length - 1] } },
        setScaleCalls: [],
        posToVal(px) {
            const sx = this.scales.x;
            return sx.min + (px / width) * (sx.max - sx.min);
        },
        setScale(_key, { min, max }) {
            this.scales.x.min = min;
            this.scales.x.max = max;
            this.setScaleCalls.push([min, max]);
        },
    };
    return u;
}

function initPlugin(u, opts) {
    const p = zoomPlugin(opts);
    p.hooks.init(u);
    return p;
}

const lastScale = (u) => u.setScaleCalls[u.setScaleCalls.length - 1];

beforeEach(() => { document.body.innerHTML = ''; });

describe('zoomPlugin', () => {
    test('opts hook disables uPlot native drag-select', () => {
        const p = zoomPlugin();
        const opts = p.opts(null, {});
        expect(opts.cursor.drag).toEqual({ x: false, y: false });
    });

    test('init mounts the +/−/⟳ button overlay with i18n titles', () => {
        const u = mockU();
        initPlugin(u);
        const bar = u.over.parentNode.querySelector('.u-zoom-actions');
        expect(bar).toBeTruthy();
        const btns = bar.querySelectorAll('button.tv-chart-icon');
        expect(btns).toHaveLength(3);
        expect(btns[0].title).toBe('Zoom in');
        expect(btns[1].title).toBe('Zoom out');
        expect(btns[2].title).toBe('Reset zoom');
    });

    test('zoom-in button shrinks the span around center', () => {
        const u = mockU();
        initPlugin(u);
        u.over.parentNode.querySelector('[title="Zoom in"]').click();
        // span 9 → 5.4 centered on 4.5: [1.8, 7.2]
        const [min, max] = lastScale(u);
        expect(min).toBeCloseTo(1.8, 10);
        expect(max).toBeCloseTo(7.2, 10);
    });

    test('zoom-out at full range resets instead of overshooting bounds', () => {
        const u = mockU();
        initPlugin(u);
        u.over.parentNode.querySelector('[title="Zoom out"]').click();
        expect(lastScale(u)).toEqual([0, 9]);
        expect(u._zoomRange).toBeNull();
    });

    test('reset button restores data extremes and clears _zoomRange', () => {
        const u = mockU();
        initPlugin(u);
        u.over.parentNode.querySelector('[title="Zoom in"]').click();
        expect(u._zoomRange).not.toBeNull();
        u.over.parentNode.querySelector('[title="Reset zoom"]').click();
        expect(lastScale(u)).toEqual([0, 9]);
        expect(u._zoomRange).toBeNull();
    });

    test('wheel zooms toward the cursor and prevents page scroll', () => {
        const u = mockU();
        initPlugin(u);
        const e = new WheelEvent('wheel', { deltaY: -100, clientX: 0, cancelable: true });
        u.over.dispatchEvent(e);
        expect(e.defaultPrevented).toBe(true);
        // cursor at left edge: frac 0 → min stays 0, span 9 → 6.75
        const [min, max] = lastScale(u);
        expect(min).toBeCloseTo(0, 10);
        expect(max).toBeCloseTo(6.75, 10);
    });

    test('wheel zoom-in refuses to go below 1/200 of full span', () => {
        const u = mockU();
        initPlugin(u);
        u.scales.x = { min: 4, max: 4 + 9 / 250 }; // already past the floor
        u.over.dispatchEvent(new WheelEvent('wheel', { deltaY: -100, clientX: 300, cancelable: true }));
        expect(u.setScaleCalls).toHaveLength(0);
    });

    test('drag pans the zoomed window and clamps at the data edge', () => {
        const u = mockU();
        initPlugin(u);
        u.scales.x = { min: 2, max: 5 };
        u.over.dispatchEvent(new MouseEvent('mousedown', { button: 0, clientX: 300 }));
        // 100px left over 600px width of a 3-unit span → +0.5
        window.dispatchEvent(new MouseEvent('mousemove', { clientX: 200 }));
        let [min, max] = lastScale(u);
        expect(min).toBeCloseTo(2.5, 10);
        expect(max).toBeCloseTo(5.5, 10);
        // Keep dragging far past the right edge — clamps to [6, 9].
        window.dispatchEvent(new MouseEvent('mousemove', { clientX: -9000 }));
        [min, max] = lastScale(u);
        expect(min).toBeCloseTo(6, 10);
        expect(max).toBeCloseTo(9, 10);
        window.dispatchEvent(new MouseEvent('mouseup'));
        // After mouseup, movement must not pan.
        const calls = u.setScaleCalls.length;
        window.dispatchEvent(new MouseEvent('mousemove', { clientX: 50 }));
        expect(u.setScaleCalls).toHaveLength(calls);
    });

    test('drag at full range is a no-op (nothing to pan)', () => {
        const u = mockU();
        initPlugin(u);
        u.over.dispatchEvent(new MouseEvent('mousedown', { button: 0, clientX: 300 }));
        window.dispatchEvent(new MouseEvent('mousemove', { clientX: 100 }));
        expect(u.setScaleCalls).toHaveLength(0);
        window.dispatchEvent(new MouseEvent('mouseup'));
    });

    test('dblclick resets to full range', () => {
        const u = mockU();
        initPlugin(u);
        u.scales.x = { min: 3, max: 4 };
        u.over.dispatchEvent(new MouseEvent('dblclick'));
        expect(lastScale(u)).toEqual([0, 9]);
    });

    test('spacebar over the chart shows the hand cursor; keyup clears it', () => {
        const u = mockU();
        initPlugin(u);
        u.over.dispatchEvent(new MouseEvent('mouseenter'));
        const down = new KeyboardEvent('keydown', { code: 'Space', cancelable: true });
        window.dispatchEvent(down);
        expect(u.over.style.cursor).toBe('grab');
        expect(down.defaultPrevented).toBe(true); // page must not scroll
        window.dispatchEvent(new KeyboardEvent('keyup', { code: 'Space' }));
        expect(u.over.style.cursor).toBe('');
    });

    test('spacebar while typing in an input is left alone', () => {
        const u = mockU();
        initPlugin(u);
        u.over.dispatchEvent(new MouseEvent('mouseenter'));
        const input = document.createElement('input');
        document.body.appendChild(input);
        input.focus();
        const down = new KeyboardEvent('keydown', { code: 'Space', cancelable: true });
        window.dispatchEvent(down);
        expect(down.defaultPrevented).toBe(false);
        expect(u.over.style.cursor).toBe('');
    });

    test('getBounds override clamps zoom and reset to the inset range', () => {
        const u = mockU();
        initPlugin(u, { getBounds: () => [-0.5, 9.5] });
        u.over.parentNode.querySelector('[title="Reset zoom"]').click();
        expect(lastScale(u)).toEqual([-0.5, 9.5]);
    });

    test('destroy removes the window listeners', () => {
        const u = mockU();
        const p = initPlugin(u);
        u.over.dispatchEvent(new MouseEvent('mouseenter'));
        p.hooks.destroy(u);
        window.dispatchEvent(new KeyboardEvent('keydown', { code: 'Space', cancelable: true }));
        expect(u.over.style.cursor).toBe('');
        u.over.dispatchEvent(new MouseEvent('mousedown', { button: 0, clientX: 300 }));
        window.dispatchEvent(new MouseEvent('mousemove', { clientX: 100 }));
        // mousedown listener is on `over` (removed with the chart DOM in
        // real life) but the window mousemove must be dead after destroy.
        expect(u.setScaleCalls).toHaveLength(0);
    });
});
