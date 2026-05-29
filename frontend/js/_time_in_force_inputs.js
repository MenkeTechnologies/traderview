// Time-in-Force helpers shared by view + vitest.
//
// Backend body shape: { order: { tif, original_qty, filled_qty,
//   placed_at: ISO-8601 UTC, good_until: 'YYYY-MM-DD' | null },
//   now: ISO-8601 UTC, session_open: 'YYYY-MM-DD' }.
//
// TIF enum: 'day' | 'gtc' | 'ioc' | 'fok' | 'gtd' (snake_case).
// Returns: { action: 'keep' | 'cancel' | 'completed', reason: string }.

import { t } from './i18n.js';

export const TIF_KINDS = ['day', 'gtc', 'ioc', 'fok', 'gtd'];

export function validateInputs(order, nowIso, sessionOpenIso) {
    if (!TIF_KINDS.includes(order.tif)) return t('view.time_in_force.validate.tif', { list: TIF_KINDS.join(',') });
    if (!Number.isFinite(order.original_qty) || order.original_qty <= 0)
        return t('view.time_in_force.validate.original_qty');
    if (!Number.isFinite(order.filled_qty) || order.filled_qty < 0)
        return t('view.time_in_force.validate.filled_qty');
    if (order.filled_qty > order.original_qty)
        return t('view.time_in_force.validate.filled_le_original');
    if (!isValidUtcIso(order.placed_at)) return t('view.time_in_force.validate.placed_at');
    if (!isValidUtcIso(nowIso)) return t('view.time_in_force.validate.now');
    if (!isValidDate(sessionOpenIso)) return t('view.time_in_force.validate.session_open');
    if (order.tif === 'gtd' && order.good_until != null && !isValidDate(order.good_until))
        return t('view.time_in_force.validate.good_until');
    return null;
}

export function isValidUtcIso(s) {
    if (typeof s !== 'string' || !s.length) return false;
    const t = Date.parse(s);
    return Number.isFinite(t);
}

export function isValidDate(s) {
    if (typeof s !== 'string' || !/^\d{4}-\d{2}-\d{2}$/.test(s)) return false;
    const [y, m, d] = s.split('-').map(Number);
    const dt = new Date(Date.UTC(y, m - 1, d));
    return dt.getUTCFullYear() === y && dt.getUTCMonth() === m - 1 && dt.getUTCDate() === d;
}

export function buildBody(order, nowIso, sessionOpenIso) {
    return {
        order: {
            tif: order.tif,
            original_qty: order.original_qty,
            filled_qty: order.filled_qty,
            placed_at: order.placed_at,
            good_until: order.good_until ?? null,
        },
        now: nowIso,
        session_open: sessionOpenIso,
    };
}

// Pure-JS mirror of crates/traderview-core/src/time_in_force.rs::evaluate.
// Same reasons-text as Rust so the local pre-flight matches the backend
// exactly. Uses Date-based diffing in UTC.
export function localEvaluate(order, nowIso, sessionOpenIso) {
    const remaining = order.original_qty - order.filled_qty;
    if (remaining <= 0) {
        return { action: 'completed', reason: 'fully filled' };
    }
    const placedDate = dateOnlyFromIso(order.placed_at);
    const sessionOpen = sessionOpenIso;
    switch (order.tif) {
        case 'day':
            if (cmpDate(sessionOpen, placedDate) > 0) {
                return { action: 'cancel', reason: 'DAY order rolled into new session — expire' };
            }
            return { action: 'keep', reason: 'DAY order still in session' };
        case 'gtc': {
            const ageDays = wholeDaysBetween(order.placed_at, nowIso);
            if (ageDays > 90) {
                return { action: 'cancel', reason: 'GTC order exceeded 90-day broker timeout' };
            }
            return { action: 'keep', reason: `GTC order, age ${ageDays} days` };
        }
        case 'ioc':
            return { action: 'cancel', reason: `IOC: cancel ${remaining} unfilled qty` };
        case 'fok':
            if (order.filled_qty === 0) {
                return { action: 'cancel', reason: 'FOK: no fill available, cancel entire order' };
            }
            if (remaining > 0) {
                return { action: 'cancel', reason: 'FOK: partial fill not allowed, cancel rest' };
            }
            return { action: 'completed', reason: 'FOK: fully filled' };
        case 'gtd': {
            const good = order.good_until;
            if (good == null) return { action: 'cancel', reason: 'GTD missing good_until date' };
            if (cmpDate(sessionOpen, good) > 0) {
                return { action: 'cancel', reason: `GTD order past good_until date ${good}` };
            }
            return { action: 'keep', reason: `GTD valid until ${good}` };
        }
        default:
            return { action: 'cancel', reason: `unknown TIF: ${order.tif}` };
    }
}

// Match chrono::DateTime::date_naive() — convert ISO-8601 UTC to its
// UTC date component as 'YYYY-MM-DD'.
export function dateOnlyFromIso(iso) {
    const d = new Date(iso);
    if (!Number.isFinite(d.getTime())) return '';
    const y = d.getUTCFullYear();
    const m = String(d.getUTCMonth() + 1).padStart(2, '0');
    const day = String(d.getUTCDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
}

// Strict date string compare 'YYYY-MM-DD'.
export function cmpDate(a, b) {
    if (a < b) return -1; if (a > b) return 1; return 0;
}

// Mirror chrono::Duration::num_days — whole-days truncation. Matches
// Rust's (now - placed_at).num_days() which truncates toward zero.
export function wholeDaysBetween(fromIso, toIso) {
    const ms = Date.parse(toIso) - Date.parse(fromIso);
    return Math.trunc(ms / 86_400_000);
}

const ACTION_BADGES = {
    keep:      { key: 'keep',      cls: 'pos' },
    cancel:    { key: 'cancel',    cls: 'neg' },
    completed: { key: 'completed', cls: 'pos' },
};
export function actionBadge(a) {
    const x = ACTION_BADGES[a];
    if (!x) return { label: String(a || '—').toUpperCase(), cls: '', hint: '—' };
    return {
        label: t(`view.time_in_force.action.${x.key}.label`),
        cls: x.cls,
        hint: t(`view.time_in_force.action.${x.key}.hint`),
    };
}

// Demo presets — one per TIF verdict path so the user can step through
// every branch by clicking. Uses today as the anchor where applicable.
export function makeDemoOrder(kind, today = new Date()) {
    const todayIso = today.toISOString();
    const yesterday = new Date(today.getTime() - 86_400_000);
    const yesterdayIso = yesterday.toISOString();
    const sessionToday = isoToDate(todayIso);
    const sessionTomorrow = isoToDate(new Date(today.getTime() + 86_400_000).toISOString());
    const placed91Ago = new Date(today.getTime() - 91 * 86_400_000).toISOString();
    const placed30Ago = new Date(today.getTime() - 30 * 86_400_000).toISOString();
    const goodFuture = isoToDate(new Date(today.getTime() + 7 * 86_400_000).toISOString());
    const goodPast   = isoToDate(new Date(today.getTime() - 7 * 86_400_000).toISOString());
    switch (kind) {
        case 'day-keep':       return mk('day', todayIso,     todayIso, sessionToday, 100, 0);
        case 'day-cancel':     return mk('day', yesterdayIso, todayIso, sessionToday, 100, 0);
        case 'gtc-keep':       return mk('gtc', placed30Ago,  todayIso, sessionToday, 100, 0);
        case 'gtc-cancel':     return mk('gtc', placed91Ago,  todayIso, sessionToday, 100, 0);
        case 'ioc-cancel':     return mk('ioc', todayIso,     todayIso, sessionToday, 100, 50);
        case 'fok-no-fill':    return mk('fok', todayIso,     todayIso, sessionToday, 100, 0);
        case 'fok-partial':    return mk('fok', todayIso,     todayIso, sessionToday, 100, 50);
        case 'fok-completed':  return mk('fok', todayIso,     todayIso, sessionToday, 100, 100);
        case 'gtd-keep':       return { ...mk('gtd', todayIso, todayIso, sessionToday, 100, 0), good_until_in_order: goodFuture };
        case 'gtd-cancel':     return { ...mk('gtd', placed30Ago, todayIso, sessionTomorrow, 100, 0), good_until_in_order: goodPast };
        case 'gtd-missing':    return { ...mk('gtd', todayIso, todayIso, sessionToday, 100, 0), good_until_in_order: null };
        case 'completed':      return mk('gtc', placed30Ago,  todayIso, sessionToday, 100, 100);
        default:               return mk('day', yesterdayIso, todayIso, sessionToday, 100, 0);
    }
}

function mk(tif, placedIso, nowIso, sessionDate, originalQty, filledQty) {
    return {
        order: {
            tif, original_qty: originalQty, filled_qty: filledQty,
            placed_at: placedIso, good_until: null,
        },
        now: nowIso, session_open: sessionDate,
        good_until_in_order: null,
    };
}

export function isoToDate(iso) {
    return dateOnlyFromIso(iso);
}

// Local <input type="datetime-local"> works in the user's tz; the backend
// expects ISO-8601 UTC. Helpers to flip between the two.
export function localDtToIsoUtc(localDt) {
    if (!localDt) return '';
    const d = new Date(localDt);
    if (!Number.isFinite(d.getTime())) return '';
    return d.toISOString();
}

export function isoUtcToLocalDt(iso) {
    if (!iso) return '';
    const d = new Date(iso);
    if (!Number.isFinite(d.getTime())) return '';
    const pad = (n) => String(n).padStart(2, '0');
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}
