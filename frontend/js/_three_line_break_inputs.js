// Three-Line Break (TLB) chart helpers.
//
// Backend body: { closes: number[], num_lines: number }
// Returns: TlbLine[] = { direction: 'Up' | 'Down', open, close, source_index }
//
// Algorithm: A new line continues in the current direction whenever the
// close extends past the prior line's close. To flip direction, the close
// must break beyond the OPEN of the last `num_lines` lines in the current
// direction (the "N-line break" rule).

export const DEFAULT_NUM_LINES = 3;

export const DEFAULT_INPUTS = {
    closes: [],
    num_lines: DEFAULT_NUM_LINES,
};

export function validateInputs(input) {
    if (!Array.isArray(input.closes))                       return 'closes must be an array';
    for (let i = 0; i < input.closes.length; i++) {
        const v = input.closes[i];
        if (!Number.isFinite(v))                            return `closes[${i}] not finite`;
    }
    if (!Number.isInteger(input.num_lines))                 return 'num_lines must be an integer';
    if (input.num_lines < 1)                                return 'num_lines must be ≥ 1';
    return null;
}

export function buildBody(input) {
    return {
        closes:    input.closes,
        num_lines: input.num_lines,
    };
}

// Pure-JS mirror of crates/traderview-core/src/three_line_break.rs::compute.
// Returns same enum strings ('Up'/'Down') as Rust on the wire.
export function localCompute(closes, num_lines) {
    const out = [];
    if (!Array.isArray(closes) || closes.length === 0) return out;
    if (!Number.isInteger(num_lines) || num_lines < 1) return out;
    for (const v of closes) if (!Number.isFinite(v)) return out;
    let direction = null;
    let last_close = closes[0];
    for (let i = 1; i < closes.length; i++) {
        const px = closes[i];
        if (direction === null) {
            if (px > last_close) {
                out.push({ direction: 'Up',   open: last_close, close: px, source_index: i });
                last_close = px;
                direction = 'Up';
            } else if (px < last_close) {
                out.push({ direction: 'Down', open: last_close, close: px, source_index: i });
                last_close = px;
                direction = 'Down';
            }
            continue;
        }
        if (direction === 'Up') {
            if (px > last_close) {
                out.push({ direction: 'Up', open: last_close, close: px, source_index: i });
                last_close = px;
            } else {
                const recent = collectRecent(out, 'Up', num_lines);
                if (recent.length >= num_lines) {
                    let break_level = Infinity;
                    for (const l of recent) if (l.open < break_level) break_level = l.open;
                    if (px < break_level) {
                        out.push({ direction: 'Down', open: last_close, close: px, source_index: i });
                        last_close = px;
                        direction = 'Down';
                    }
                }
            }
        } else {
            // direction === 'Down'
            if (px < last_close) {
                out.push({ direction: 'Down', open: last_close, close: px, source_index: i });
                last_close = px;
            } else {
                const recent = collectRecent(out, 'Down', num_lines);
                if (recent.length >= num_lines) {
                    let break_level = -Infinity;
                    for (const l of recent) if (l.open > break_level) break_level = l.open;
                    if (px > break_level) {
                        out.push({ direction: 'Up', open: last_close, close: px, source_index: i });
                        last_close = px;
                        direction = 'Up';
                    }
                }
            }
        }
    }
    return out;
}

function collectRecent(lines, dir, n) {
    const out = [];
    for (let i = lines.length - 1; i >= 0 && out.length < n; i--) {
        if (lines[i].direction === dir) out.push(lines[i]);
    }
    return out;
}

// Parse comma/whitespace-separated price series; # comments + blanks ignored.
export function parseCloses(blob) {
    const out = { closes: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: 'input must be a string' });
        return out;
    }
    const tokens = blob
        .split('\n')
        .map(l => l.split('#')[0])
        .join(' ')
        .split(/[\s,]+/)
        .filter(t => t.length > 0);
    for (let i = 0; i < tokens.length; i++) {
        const v = Number(tokens[i]);
        if (!Number.isFinite(v)) {
            out.errors.push({ line_no: i + 1, message: `token "${tokens[i]}" not finite` });
            continue;
        }
        out.closes.push(v);
    }
    return out;
}

export function closesToBlob(closes) {
    return closes.join('\n');
}

// Trend verdict from line list — biased toward last direction.
export function trendBadge(lines) {
    if (!Array.isArray(lines) || lines.length === 0) return { key: 'view.tlb.badge.flat', cls: '' };
    const last = lines[lines.length - 1];
    if (last.direction === 'Up')   return { key: 'view.tlb.badge.uptrend',   cls: 'pos' };
    if (last.direction === 'Down') return { key: 'view.tlb.badge.downtrend', cls: 'neg' };
    return { key: 'view.tlb.badge.flat', cls: '' };
}

// Direction-flip count — how many direction changes in the line sequence.
export function flipCount(lines) {
    if (!Array.isArray(lines) || lines.length < 2) return 0;
    let flips = 0;
    for (let i = 1; i < lines.length; i++) {
        if (lines[i].direction !== lines[i - 1].direction) flips++;
    }
    return flips;
}

// Run-length of the FINAL direction.
export function finalRunLength(lines) {
    if (!Array.isArray(lines) || lines.length === 0) return 0;
    const dir = lines[lines.length - 1].direction;
    let n = 0;
    for (let i = lines.length - 1; i >= 0; i--) {
        if (lines[i].direction === dir) n++;
        else break;
    }
    return n;
}

// Aggregate stats for the summary cards.
export function summarize(lines) {
    if (!Array.isArray(lines) || lines.length === 0) {
        return { count: 0, ups: 0, downs: 0, avg_up: NaN, avg_down: NaN,
                 last_dir: null, last_close: NaN };
    }
    let ups = 0, downs = 0, sumUp = 0, sumDown = 0;
    for (const l of lines) {
        const move = l.close - l.open;
        if (l.direction === 'Up')   { ups++;   sumUp   += move; }
        if (l.direction === 'Down') { downs++; sumDown += -move; }
    }
    return {
        count:     lines.length,
        ups, downs,
        avg_up:    ups > 0   ? sumUp / ups : NaN,
        avg_down:  downs > 0 ? sumDown / downs : NaN,
        last_dir:  lines[lines.length - 1].direction,
        last_close: lines[lines.length - 1].close,
    };
}

// Convert TLB lines to uPlot-friendly stepped polyline (open → close at each line).
export function linesToPolyline(lines) {
    if (!Array.isArray(lines) || lines.length === 0) return { xs: [], ys: [] };
    const xs = [];
    const ys = [];
    let cursor = 0;
    for (const l of lines) {
        xs.push(cursor); ys.push(l.open);
        xs.push(cursor); ys.push(l.close);
        cursor++;
    }
    return { xs, ys };
}

// Deterministic demos.
export function makeDemoInput(kind = 'uptrend') {
    switch (kind) {
        case 'uptrend':       return { closes: range(100, 110, 1),  num_lines: 3 };
        case 'downtrend':     return { closes: range(110, 100, -1), num_lines: 3 };
        case 'small-pullback': return { closes: [100, 102, 104, 106, 105.5], num_lines: 3 };
        case 'deep-pullback':  return { closes: [100, 102, 104, 106, 99],    num_lines: 3 };
        case 'choppy':        return { closes: chop(100, 50, 1.5),  num_lines: 3 };
        case 'flat':          return { closes: Array(20).fill(100), num_lines: 3 };
        case 'two-line':      return { closes: [100, 102, 104, 99],         num_lines: 2 };
        case 'five-line':     return { closes: [100, 102, 104, 106, 108, 95], num_lines: 5 };
        default:              return makeDemoInput('uptrend');
    }
}

function range(start, end, step) {
    const out = [];
    if (step > 0) for (let v = start; v <= end + 1e-9; v += step) out.push(round(v));
    else          for (let v = start; v >= end - 1e-9; v += step) out.push(round(v));
    return out;
}

function chop(center, n, amp) {
    const out = [];
    for (let i = 0; i < n; i++) out.push(round(center + Math.sin(i * 0.7) * amp + Math.cos(i * 1.3) * amp * 0.6));
    return out;
}

function round(v) { return Math.round(v * 10000) / 10000; }

export function fmtUSD(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return '$' + v.toFixed(d);
}

export function fmtMove(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '-') + '$' + Math.abs(v).toFixed(d);
}

export function fmtInt(v) {
    if (!Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}

export function dirLabelKey(dir) {
    if (dir === 'Up')   return 'view.tlb.dir.up';
    if (dir === 'Down') return 'view.tlb.dir.down';
    return 'view.tlb.dir.unknown';
}
