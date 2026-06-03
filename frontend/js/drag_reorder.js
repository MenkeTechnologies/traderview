// Trello-style drag-to-reorder engine, ported from MenkeTechnologies'
// audio_haxor. Pure pointer-driven (no HTML5 dragstart/dragover/drop), so it
// dodges all the WebView/Tauri interception traps and works through child
// canvases (uPlot) that would otherwise eat mouse events.
//
// One global mousemove + mouseup pair. Each container registers a single
// mousedown listener via initDragReorder().
//
// Usage:
//   import { initDragReorder } from './drag_reorder.js';
//   initDragReorder(containerEl, '.row-selector', 'localStorageKey', {
//       direction:        'vertical' | 'horizontal',  // default vertical
//       handleSelector:   '.drag-handle',             // optional restrict
//       getKey:           (el, i) => el.dataset.id || String(i),
//       onReorder:        (newKeys) => { ... },       // optional callback
//       persist:          (newKeys) => fetch(...),    // optional server save
//       toastMessage:     'Reordered',                // optional toast
//   });

import { showToast } from './toast.js';

// Single shared drag state — only one drag at a time.
let _drag = null;

/**
 * Resolve a click target to a top-level direct child of the container that
 * matches childSelector. Required because clicks land deep inside the panel
 * (canvas, text inside a card, etc) but we need to grab the outermost item.
 */
function resolveDragChild(container, childSelector, target) {
    if (!target || !container.contains(target)) return null;
    const hit = target.closest(childSelector);
    if (!hit || !container.contains(hit)) return null;
    let cur = hit;
    while (cur && cur.parentElement !== container) cur = cur.parentElement;
    return cur && cur.parentElement === container && cur.matches(childSelector) ? cur : null;
}

function listDragChildren(container, childSelector) {
    try {
        return [...container.querySelectorAll(childSelector)]
            .filter(c => c.parentElement === container);
    } catch {
        return [];
    }
}

// ---------------------------------------------------------------------------
// Global mousemove — runs only when a drag is in flight.
// ---------------------------------------------------------------------------
document.addEventListener('mousemove', (e) => {
    if (!_drag) return;
    const d = _drag;
    const dx = e.clientX - d.startX;
    const dy = e.clientY - d.startY;

    // Threshold — promote click → drag once cursor moves >3px in the active
    // axis. Below this, we treat it as a click (handlers still fire).
    if (!d.isDragging && Math.abs(d.direction === 'horizontal' ? dx : dy) > 3) {
        d.isDragging = true;
        document.body.style.userSelect = 'none';
        document.body.style.cursor = 'grabbing';

        const rect = d.dragged.getBoundingClientRect();

        // Placeholder = visible slot showing where the drop will land.
        d.placeholder = document.createElement(d.dragged.tagName);
        d.placeholder.className = 'trello-placeholder';
        for (const cls of d.dragged.classList) {
            if (cls.includes('wide') || cls.includes('span')) {
                d.placeholder.classList.add(cls);
            }
        }
        if (d.direction === 'horizontal') {
            d.placeholder.style.width = rect.width + 'px';
            d.placeholder.style.height = rect.height + 'px';
            d.placeholder.style.display = 'inline-block';
        } else {
            d.placeholder.style.height = rect.height + 'px';
            d.placeholder.style.width = rect.width + 'px';
        }
        d.dragged.parentNode.insertBefore(d.placeholder, d.dragged);

        // Ghost = floating clone that follows the cursor.
        d.ghost = d.dragged.cloneNode(true);
        d.ghost.classList.add('trello-ghost');
        d.ghost.style.cssText =
            `position:fixed;z-index:20000;width:${rect.width}px;height:${rect.height}px;` +
            `left:${rect.left}px;top:${rect.top}px;pointer-events:none;opacity:0.9;` +
            `transform:rotate(2deg) scale(1.05);will-change:transform,left,top;` +
            `box-shadow:0 8px 32px rgba(0,0,0,0.5),0 0 20px rgba(0,229,255,0.3);` +
            // Hardcode colors instead of var(--cyan)/var(--bg-card) — release
            // WebKit doesn't reliably resolve CSS custom properties on
            // dynamically inserted DOM nodes appended to document.body.
            `border:2px solid #00e5ff;border-radius:4px;background:#0d1a26;transition:none;`;
        document.body.appendChild(d.ghost);
        d.dragged.style.display = 'none';
    }

    if (!d.isDragging || !d.ghost) return;

    d.ghost.style.left = (e.clientX - d.offsetX) + 'px';
    d.ghost.style.top  = (e.clientY - d.offsetY) + 'px';

    // Hide ghost briefly to find the element underneath the cursor.
    d.ghost.style.display = 'none';
    const el = document.elementFromPoint(e.clientX, e.clientY);
    d.ghost.style.display = '';
    const target = d.resolveDragChild ? d.resolveDragChild(el) : el?.closest(d.childSelector);

    if (target && target !== d.dragged && target !== d.placeholder && d.container.contains(target)) {
        try {
            const r = target.getBoundingClientRect();
            const mid = d.direction === 'horizontal' ? r.left + r.width / 2 : r.top + r.height / 2;
            const pos = d.direction === 'horizontal' ? e.clientX : e.clientY;
            const ref = pos < mid ? target : target.nextSibling;
            if (ref === null || d.container.contains(ref)) {
                d.container.insertBefore(d.placeholder, ref);
            }
        } catch { /* swallow — DOM mutated between getBoundingClientRect and insertBefore */ }
    }
});

// ---------------------------------------------------------------------------
// Global mouseup — finalize drop or cancel non-drag click.
// ---------------------------------------------------------------------------
document.addEventListener('mouseup', () => {
    if (!_drag) return;
    const d = _drag;
    if (d.isDragging) {
        document.body.style.userSelect = '';
        document.body.style.cursor = '';
        if (d.placeholder?.parentNode) {
            d.placeholder.parentNode.insertBefore(d.dragged, d.placeholder);
            d.placeholder.remove();
        }
        d.dragged.style.display = '';
        if (d.ghost) d.ghost.remove();

        const newKeys = listDragChildren(d.container, d.childSelector).map((c, i) => d.getKey(c, i));

        // Persist to localStorage (default) + optional server-side persist.
        if (d.prefsKey) {
            try { localStorage.setItem(d.prefsKey, JSON.stringify(newKeys)); }
            catch (_) { /* quota exceeded — non-fatal */ }
        }
        if (typeof d.persist === 'function') {
            try { Promise.resolve(d.persist(newKeys)).catch(err => console.warn('drag persist failed', err)); }
            catch (err) { console.warn('drag persist threw', err); }
        }
        if (typeof d.onReorder === 'function') {
            try { d.onReorder(newKeys); } catch (err) { console.warn('drag onReorder threw', err); }
        }
        if (d.toastMessage) {
            try { showToast(d.toastMessage, { level: 'success' }); } catch (_) {}
        }
    }
    _drag = null;
});

// ---------------------------------------------------------------------------
// Cancel-on-blur / context menu so an aborted drag doesn't leave the page
// in a stuck userSelect=none / cursor=grabbing state.
// ---------------------------------------------------------------------------
function abort() {
    if (!_drag) return;
    const d = _drag;
    if (d.isDragging) {
        document.body.style.userSelect = '';
        document.body.style.cursor = '';
        if (d.placeholder?.parentNode) d.placeholder.remove();
        if (d.dragged) d.dragged.style.display = '';
        if (d.ghost) d.ghost.remove();
    }
    _drag = null;
}
window.addEventListener('blur', abort);
document.addEventListener('contextmenu', abort);

// ---------------------------------------------------------------------------
// Public API.
// ---------------------------------------------------------------------------
/**
 * Wire a container so its direct children matching `childSelector` become
 * Trello-style draggable. Returns void; safe to call multiple times on the
 * same container (subsequent calls are no-ops via _trelloDragInit flag).
 *
 * @param {Element} container   Parent element whose children get drag wires
 * @param {string}  childSelector  CSS selector matching draggable direct children
 * @param {string}  prefsKey    localStorage key for order persistence (or null)
 * @param {object}  [opts]      direction, getKey, handleSelector, onReorder, persist, toastMessage
 */
export function initDragReorder(container, childSelector, prefsKey, opts = {}) {
    if (!container || container._trelloDragInit) return;
    container._trelloDragInit = true;

    const direction      = opts.direction       || 'vertical';
    const onReorder      = opts.onReorder       || null;
    const persist        = opts.persist         || null;
    const handleSelector = opts.handleSelector  || null;
    const toastMessage   = opts.toastMessage    || null;
    const getKey         = opts.getKey
        || ((el, i) => el.dataset.dragKey || el.dataset.id || el.textContent.trim().slice(0, 30) || String(i));

    const resolveChild = (node) => resolveDragChild(container, childSelector, node);

    // Restore saved order from localStorage on init. Server-side persist
    // can hydrate via a separate path (caller passes a pre-ordered DOM).
    if (prefsKey) {
        try {
            const raw = localStorage.getItem(prefsKey);
            if (raw) {
                const saved = JSON.parse(raw);
                if (Array.isArray(saved)) {
                    const children = listDragChildren(container, childSelector);
                    const map = {};
                    children.forEach((c, i) => { map[getKey(c, i)] = c; });
                    // Re-append in saved order; unknown / new keys append at end.
                    for (const key of saved) {
                        if (map[key]) container.appendChild(map[key]);
                    }
                    children.forEach((c, i) => {
                        if (!saved.includes(getKey(c, i))) container.appendChild(c);
                    });
                }
            }
        } catch (_) { /* corrupt prefs — ignore */ }
    }

    container.addEventListener('mousedown', (e) => {
        if (e.button !== 0 || _drag) return;
        const child = resolveChild(e.target);
        if (!child || !container.contains(child)) return;
        if (handleSelector && !e.target.closest(handleSelector)) return;
        // Skip drag init when the user is clicking interactive elements —
        // otherwise drag eats their click.
        const skipSelector = direction === 'horizontal'
            ? 'input, select, textarea'
            : 'input, button, select, textarea, a';
        if (e.target.closest(skipSelector)) return;
        e.preventDefault();
        const rect = child.getBoundingClientRect();
        _drag = {
            container, childSelector, direction, onReorder, persist,
            prefsKey, getKey, toastMessage,
            resolveDragChild: resolveChild,
            dragged: child, ghost: null, placeholder: null, isDragging: false,
            startX: e.clientX, startY: e.clientY,
            offsetX: e.clientX - rect.left, offsetY: e.clientY - rect.top,
        };
    });
}

/**
 * Clear the `_trelloDragInit` flag so initDragReorder can re-wire after a
 * full re-render of the container's contents. Without this, a re-render
 * leaves new children without drag wires because the container already
 * thinks it's initialized.
 */
export function resetDragReorder(container) {
    if (container) container._trelloDragInit = false;
}
