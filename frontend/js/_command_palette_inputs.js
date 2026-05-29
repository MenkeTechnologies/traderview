// Command-palette pure helpers shared with vitest.
//
// Items shape:
//   { id, label, hint, icon, category, viewId, score?, kind: 'view'|'favorite'|'bookmark'|'action' }
//
// Fuzzy scoring: subsequence match + prefix bonus + exact-token bonus.
// Optimized for ~200-item catalogs (every TILE in launcher) — O(n·m) is fine.

export function buildTileItems(tiles, categoriesByViewId) {
    if (!Array.isArray(tiles)) return [];
    return tiles.map(t => ({
        id: `view:${t[0]}`,
        kind: 'view',
        viewId: t[0],
        label: t[1],
        icon: t[2] || '',
        hint: t[3] || '',
        category: categoriesByViewId.get(t[0]) || '',
        badge: t[4] || null,
    }));
}

// Map of viewId → category label, built from CATEGORIES const layout.
// CATEGORIES is a flat array of [catId, catLabel, viewIds[]].
export function categoriesByViewId(categories) {
    const map = new Map();
    if (!Array.isArray(categories)) return map;
    for (const cat of categories) {
        if (!Array.isArray(cat) || cat.length < 3) continue;
        const label = cat[1];
        const viewIds = cat[2];
        if (!Array.isArray(viewIds)) continue;
        for (const vid of viewIds) map.set(vid, label);
    }
    return map;
}

// Append favorite + bookmark items to the catalog so they're palette-
// searchable too.
export function buildFavoriteItems(favorites, tilesByViewId) {
    if (!Array.isArray(favorites)) return [];
    return favorites.map(vid => {
        const t = tilesByViewId.get(vid);
        return {
            id: `fav:${vid}`,
            kind: 'favorite',
            viewId: vid,
            label: t ? t[1] : vid,
            icon: '★',
            hint: t ? t[3] : '',
            category: 'Favorites',
        };
    }).filter(it => !!it.viewId);
}

export function buildBookmarkItems(bookmarks, tilesByViewId) {
    if (!Array.isArray(bookmarks)) return [];
    return bookmarks.map(b => {
        const t = tilesByViewId.get(b.viewId);
        return {
            id: `bm:${b.id}`,
            kind: 'bookmark',
            viewId: b.viewId,
            label: b.name || (t ? t[1] : b.viewId),
            icon: '📌',
            hint: t ? t[3] : '',
            category: 'Bookmarks',
        };
    }).filter(it => !!it.viewId);
}

export function tilesByViewId(tiles) {
    const map = new Map();
    if (!Array.isArray(tiles)) return map;
    for (const t of tiles) if (Array.isArray(t) && t.length >= 2) map.set(t[0], t);
    return map;
}

// Build palette items from the shortcut registry. Each shortcut becomes
// a searchable "action" command — selecting it dispatches its actionKey
// CustomEvent (same path the shortcut press would take).
//
// `translate(key)` lets the caller pass i18n's t(). It's optional; if
// omitted, the descKey is returned verbatim. `formatChip(shortcut)` is
// also optional and returns the visible keyboard chip.
export function buildActionItems(shortcuts, translate, formatChip) {
    if (!Array.isArray(shortcuts)) return [];
    const tr = typeof translate === 'function' ? translate : (k => k);
    const fmt = typeof formatChip === 'function' ? formatChip : (() => '');
    return shortcuts.map(sc => {
        const chip = fmt(sc) || '';
        return {
            id: `action:${sc.id}`,
            kind: 'action',
            actionKey: sc.actionKey,
            label: sc.descKey ? tr(sc.descKey) : sc.id,
            icon: '⚡',
            hint: chip,
            category: 'Actions',
            scope: sc.scope || 'global',
        };
    }).filter(it => !!it.actionKey);
}

// Subsequence-fuzzy + bonuses. Higher = better match.
// 0 = no match (caller filters those out).
export function fuzzyScore(query, item) {
    if (!item || !item.label) return 0;
    const q = String(query || '').trim().toLowerCase();
    if (!q) return 1; // empty query → all items pass with neutral score
    const haystack = [item.label, item.viewId || '', item.hint || '', item.category || '']
        .join(' ').toLowerCase();
    let qi = 0;
    let lastMatchAt = -1;
    let consecutive = 0;
    let bonuses = 0;
    for (let i = 0; i < haystack.length && qi < q.length; i++) {
        const c = q[qi];
        if (haystack[i] === c) {
            if (lastMatchAt === i - 1) consecutive++;
            else consecutive = 1;
            bonuses += consecutive;
            // Word-boundary bonus.
            if (i === 0 || /\s|\W/.test(haystack[i - 1])) bonuses += 2;
            lastMatchAt = i;
            qi++;
        }
    }
    if (qi < q.length) return 0;
    // Prefix bonus: query starts the label.
    if (item.label.toLowerCase().startsWith(q)) bonuses += 25;
    if ((item.viewId || '').toLowerCase().startsWith(q)) bonuses += 15;
    // Exact label match: huge bonus.
    if (item.label.toLowerCase() === q) bonuses += 100;
    // Kind tiebreakers.
    if (item.kind === 'favorite') bonuses += 4;
    if (item.kind === 'bookmark') bonuses += 3;
    if (item.kind === 'action')   bonuses += 2;
    if (item.kind === 'recent')   bonuses += 1;
    return bonuses + 1;
}

// Filter + rank + cap. Stable on score (preserves insertion order for ties).
export function filterAndRank(items, query, max = 50) {
    if (!Array.isArray(items)) return [];
    const scored = [];
    for (let i = 0; i < items.length; i++) {
        const s = fuzzyScore(query, items[i]);
        if (s > 0) scored.push({ item: items[i], score: s, idx: i });
    }
    scored.sort((a, b) => (b.score - a.score) || (a.idx - b.idx));
    return scored.slice(0, max).map(x => ({ ...x.item, score: x.score }));
}

// Mark the matched characters in `label` against `query`. Returns an
// array of `{ ch, hit }` segments for the renderer to wrap matched
// chars in <mark>.
export function highlightLabel(label, query) {
    const out = [];
    if (!label) return out;
    const q = String(query || '').trim().toLowerCase();
    if (!q) return [{ ch: label, hit: false }];
    let qi = 0;
    let buf = '';
    let inHit = false;
    for (let i = 0; i < label.length; i++) {
        const c = label[i];
        const matches = qi < q.length && c.toLowerCase() === q[qi];
        if (matches !== inHit && buf.length > 0) {
            out.push({ ch: buf, hit: inHit });
            buf = '';
        }
        buf += c;
        inHit = matches;
        if (matches) qi++;
    }
    if (buf) out.push({ ch: buf, hit: inHit });
    return out;
}

// Keyboard navigation: clamp selected index into bounds, handle ↑↓ + Enter.
export function moveSelection(current, delta, total) {
    if (total <= 0) return 0;
    let next = current + delta;
    if (next < 0) next = total - 1;
    if (next >= total) next = 0;
    return next;
}
