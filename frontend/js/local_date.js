// Local-timezone "today" helper. Extracted so node --test can verify the
// month-off-by-one + zero-padding logic without a DOM.
//
// Why local: `new Date().toISOString().slice(0, 10)` is always UTC, so
// during ET/PT evening review it rolls over to "tomorrow" and lands the
// user on the wrong journal page.

export function localToday(date = new Date()) {
    const y = date.getFullYear();
    const m = String(date.getMonth() + 1).padStart(2, '0');
    const d = String(date.getDate()).padStart(2, '0');
    return `${y}-${m}-${d}`;
}
