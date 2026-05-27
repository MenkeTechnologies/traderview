// Pure-compute helpers extracted from DOM-bound modules so node --test can
// verify the logic without a DOM stub. Each function MUST stay pure (no
// document/window access). The originals re-export from here.

// ---------------------------------------------------------------------------
// launcher.matchesQuery — filter tile by id/label/desc substring (lowercase)
// ---------------------------------------------------------------------------

/** Tile tuple shape: [id, label, glyph, desc, badge]. */
export function matchesQuery(tile, query) {
    if (!query) return true;
    const q = String(query).toLowerCase();
    return tile[1].toLowerCase().includes(q)
        || tile[0].toLowerCase().includes(q)
        || (tile[3] || '').toLowerCase().includes(q);
}

// ---------------------------------------------------------------------------
// alert_engine.matches — evaluate an alert rule against a current quote
// ---------------------------------------------------------------------------

/**
 * Rule: `{ trigger, threshold }`.
 * Quote: `{ price, change_pct, volume, day_high, day_low }`.
 *
 * Trigger semantics:
 *   price_above       fires when price >= threshold
 *   price_below       fires when price <= threshold
 *   pct_up            fires when change_pct >= threshold (positive)
 *   pct_down          fires when change_pct <= -threshold (i.e. drop)
 *   new_high_of_day   fires when price >= day_high
 *   new_low_of_day    fires when price <= day_low
 *   volume_surge      fires when threshold > 0 AND volume >= threshold
 */
export function matchesAlert(rule, quote) {
    const price = Number(quote.price);
    const ch    = Number(quote.change_pct ?? 0);
    const vol   = Number(quote.volume ?? 0);
    const thr   = Number(rule.threshold ?? 0);
    switch (rule.trigger) {
        case 'price_above':     return price >= thr;
        case 'price_below':     return price <= thr;
        case 'pct_up':          return ch    >=  thr;
        case 'pct_down':        return ch    <= -thr;
        case 'new_high_of_day': return quote.day_high != null && price >= Number(quote.day_high);
        case 'new_low_of_day':  return quote.day_low  != null && price <= Number(quote.day_low);
        case 'volume_surge':    return thr > 0 && vol >= thr;
        default:                return false;
    }
}

// ---------------------------------------------------------------------------
// hotkey_engine.buildCombo — turn a keydown event into a "ctrl+alt+key" string
// ---------------------------------------------------------------------------

/**
 * Build the canonical "modifier+modifier+key" string used for hotkey
 * matching. Order is fixed: ctrl, alt, shift, meta. Returns null when the
 * event is a bare modifier press (Control/Shift/Alt/Meta on its own).
 */
export function buildCombo(event) {
    const parts = [];
    if (event.ctrlKey)  parts.push('ctrl');
    if (event.altKey)   parts.push('alt');
    if (event.shiftKey) parts.push('shift');
    if (event.metaKey)  parts.push('meta');
    const key = String(event.key || '').toLowerCase();
    if (key === 'control' || key === 'shift' || key === 'alt' || key === 'meta') return null;
    if (!key) return null;
    parts.push(key);
    return parts.join('+');
}
