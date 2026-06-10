// Global table enhancement — column resize + column sort + persistence.
//
// Auto-arms via a single MutationObserver on document.body that wires
// every <table> added to the DOM (including ones replaced via innerHTML
// re-renders). Idempotent: a table is enhanced once, tracked in a WeakSet
// so subsequent enhancement calls are no-ops.
//
// Per-table config via data-* attributes on the <table>:
//   data-table-key="<id>"   — persistence key (column widths + sort state)
//   data-no-enhance="1"     — skip this table entirely
// Per-th config:
//   data-no-sort="1"        — skip sort on this column
//   data-sort="number|text" — force sort type (default: auto-detect)
//   data-sort-value="<v>"   — override cell value used for sort comparison
//                             (set on the td, not the th)

const ENHANCED = new WeakSet();
const ORIGINAL_ORDER = new WeakMap(); // tbody → original row sequence for "none" sort

// ── public API ────────────────────────────────────────────────────────────

export function enhanceTable(table, opts = {}) {
    if (!table || !(table instanceof HTMLElement)) return;
    if (table.tagName !== 'TABLE' || ENHANCED.has(table)) return;
    if (table.dataset.noEnhance === '1') return;
    const ths = headerCells(table);
    if (ths.length === 0) return;
    ENHANCED.add(table);
    table.classList.add('te-enhanced');

    const key = opts.key ?? table.dataset.tableKey ?? null;

    ths.forEach((th, colIdx) => wireHeader(table, th, colIdx, key));

    if (key) {
        restoreColumnWidths(table, key);
        restoreSort(table, key);
    }
}

export function enhanceAll(root = document) {
    root.querySelectorAll('table').forEach(t => enhanceTable(t));
}

export function startAutoEnhance() {
    // Run once on call; subsequent calls are no-ops via the WeakSet guard.
    enhanceAll(document);
    if (startAutoEnhance._armed) return;
    startAutoEnhance._armed = true;
    const obs = new MutationObserver(mutations => {
        for (const m of mutations) {
            for (const node of m.addedNodes) {
                if (!(node instanceof HTMLElement)) continue;
                if (node.tagName === 'TABLE') enhanceTable(node);
                if (node.querySelectorAll) {
                    node.querySelectorAll('table').forEach(t => enhanceTable(t));
                }
            }
        }
    });
    obs.observe(document.body, { childList: true, subtree: true });
}

// Self-arm when imported. Idempotent.
if (typeof document !== 'undefined') {
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', startAutoEnhance, { once: true });
    } else {
        startAutoEnhance();
    }
}

// ── header wiring ────────────────────────────────────────────────────────

function headerCells(table) {
    // Prefer <thead><tr><th> but fall back to first <tr>'s ths for tables
    // that skip <thead>. We grab only the FIRST header row — sub-headers
    // (colspan="N") would confuse the per-column resize/sort math.
    const thead = table.tHead;
    if (thead && thead.rows.length > 0) {
        return Array.from(thead.rows[0].cells).filter(c => c.tagName === 'TH');
    }
    const firstRow = table.rows[0];
    return firstRow ? Array.from(firstRow.cells).filter(c => c.tagName === 'TH') : [];
}

function wireHeader(table, th, colIdx, key) {
    // Sort wiring (unless opted out)
    if (th.dataset.noSort !== '1') {
        th.classList.add('te-sortable');
        th.addEventListener('click', (e) => {
            if (e.target.closest('.te-resize-handle')) return;
            cycleSort(table, colIdx, th, key);
        });
    }
    // Resize handle on right edge — narrow strip, abs-positioned within th.
    // th needs position:relative for this to anchor; CSS rule handles that.
    const handle = document.createElement('span');
    handle.className = 'te-resize-handle';
    handle.setAttribute('aria-hidden', 'true');
    th.appendChild(handle);
    handle.addEventListener('mousedown', (e) => startResize(e, table, th, colIdx, key));
}

// ── sort ──────────────────────────────────────────────────────────────────

function cycleSort(table, colIdx, th, key) {
    const cur = th.dataset.sortDir || 'none';
    const next = cur === 'asc' ? 'desc' : (cur === 'desc' ? 'none' : 'asc');
    headerCells(table).forEach(h => delete h.dataset.sortDir);
    if (next !== 'none') th.dataset.sortDir = next;
    applySort(table, colIdx, next, th);
    if (key) saveSort(table, key, colIdx, next);
}

function applySort(table, colIdx, dir, th) {
    const tbody = table.tBodies[0];
    if (!tbody) return;
    // Snapshot original order on first sort so 'none' restores it
    if (!ORIGINAL_ORDER.has(tbody)) {
        ORIGINAL_ORDER.set(tbody, Array.from(tbody.rows));
    }
    if (dir === 'none') {
        const original = ORIGINAL_ORDER.get(tbody);
        if (original) original.forEach(r => tbody.appendChild(r));
        return;
    }
    const rows = Array.from(tbody.rows);
    const type = (th && th.dataset.sort) || detectColumnType(rows, colIdx);
    const asc = dir === 'asc';
    rows.sort((a, b) => compareCells(a, b, colIdx, type, asc));
    rows.forEach(r => tbody.appendChild(r));
}

function compareCells(a, b, colIdx, type, asc) {
    const av = cellSortValue(a, colIdx, type);
    const bv = cellSortValue(b, colIdx, type);
    let cmp;
    if (av === null && bv === null) cmp = 0;
    else if (av === null) cmp = 1;       // nulls always sort to bottom
    else if (bv === null) cmp = -1;
    else if (type === 'number') cmp = av - bv;
    else cmp = String(av).localeCompare(String(bv), undefined, { numeric: true, sensitivity: 'base' });
    return asc ? cmp : -cmp;
}

function cellSortValue(row, colIdx, type) {
    const cell = row.cells[colIdx];
    if (!cell) return null;
    const raw = (cell.dataset.sortValue ?? cell.textContent ?? '').trim();
    if (raw === '' || raw === '—' || raw === '-') return null;
    if (type === 'number') {
        const n = parseHumanNumber(raw);
        return Number.isFinite(n) ? n : null;
    }
    return raw;
}

// Parse "$1.2M", "(123.45)", "12.3%", "1,234.56" into a Number.
function parseHumanNumber(s) {
    let str = String(s).replace(/[ \s]/g, '');
    // Accounting parens = negative
    const paren = str.startsWith('(') && str.endsWith(')');
    if (paren) str = str.slice(1, -1);
    // Strip currency + commas
    str = str.replace(/[$,]/g, '');
    // Strip trailing % (still treat as plain number magnitude)
    const pct = str.endsWith('%');
    if (pct) str = str.slice(0, -1);
    // K/M/B/T suffix
    const m = str.match(/^(-?[\d.]+)([KMBT])$/i);
    let n;
    if (m) {
        const base = Number(m[1]);
        const mult = { K: 1e3, M: 1e6, B: 1e9, T: 1e12 }[m[2].toUpperCase()];
        n = base * mult;
    } else {
        n = Number(str);
    }
    if (!Number.isFinite(n)) return NaN;
    return paren ? -n : n;
}

function detectColumnType(rows, colIdx) {
    // Sample up to 10 non-empty cells. If >60% parse as numbers, treat as numeric.
    let numCount = 0, total = 0;
    for (let i = 0; i < Math.min(rows.length, 10); i++) {
        const cell = rows[i].cells[colIdx];
        if (!cell) continue;
        const raw = (cell.dataset.sortValue ?? cell.textContent ?? '').trim();
        if (raw === '' || raw === '—' || raw === '-') continue;
        total++;
        if (Number.isFinite(parseHumanNumber(raw))) numCount++;
    }
    return total > 0 && numCount / total > 0.6 ? 'number' : 'text';
}

// ── resize ────────────────────────────────────────────────────────────────

function startResize(ev, table, th, colIdx, key) {
    ev.preventDefault();
    ev.stopPropagation();
    if (ev.button !== 0) return;
    const startX = ev.clientX;
    const startW = th.getBoundingClientRect().width;
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
    th.classList.add('te-resizing');

    const onMove = (mv) => {
        const dx = mv.clientX - startX;
        const w = Math.max(28, startW + dx);
        setColumnWidth(th, w);
    };
    const onUp = () => {
        document.body.style.cursor = '';
        document.body.style.userSelect = '';
        th.classList.remove('te-resizing');
        document.removeEventListener('mousemove', onMove);
        document.removeEventListener('mouseup', onUp);
        if (key) saveColumnWidths(table, key);
    };
    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onUp);
}

function setColumnWidth(th, w) {
    const px = `${w}px`;
    th.style.width = px;
    th.style.minWidth = px;
    th.style.maxWidth = px;
}

// ── persistence ───────────────────────────────────────────────────────────

const NS = 'tv:tableCols:';
const NS_SORT = 'tv:tableSort:';

function saveColumnWidths(table, key) {
    const widths = headerCells(table).map(th => {
        const w = th.style.width;
        return w && w.endsWith('px') ? parseInt(w, 10) : null;
    });
    try { localStorage.setItem(NS + key, JSON.stringify(widths)); } catch {}
}

function restoreColumnWidths(table, key) {
    let widths;
    try {
        const raw = localStorage.getItem(NS + key);
        if (!raw) return;
        widths = JSON.parse(raw);
    } catch { return; }
    if (!Array.isArray(widths)) return;
    headerCells(table).forEach((th, i) => {
        const w = widths[i];
        if (Number.isFinite(w) && w > 0) setColumnWidth(th, w);
    });
}

function saveSort(table, key, colIdx, dir) {
    try { localStorage.setItem(NS_SORT + key, JSON.stringify({ col: colIdx, dir })); } catch {}
}

function restoreSort(table, key) {
    let saved;
    try {
        const raw = localStorage.getItem(NS_SORT + key);
        if (!raw) return;
        saved = JSON.parse(raw);
    } catch { return; }
    if (!saved || saved.dir === 'none' || !Number.isFinite(saved.col)) return;
    const ths = headerCells(table);
    const th = ths[saved.col];
    if (!th || th.dataset.noSort === '1') return;
    th.dataset.sortDir = saved.dir;
    applySort(table, saved.col, saved.dir, th);
}
