// Hotkey engine — listens globally, dispatches to actions.
import { api } from './api.js';
import { localToday } from './local_date.js';
import { buildCombo } from './_pure.js';
import { t } from './i18n.js';

let bindings = [];

export async function reloadHotkeys() {
    try { bindings = await api.hotkeys(); } catch (_) { bindings = []; }
}

export function installHotkeyEngine() {
    window.addEventListener('keydown', handler, true);
    reloadHotkeys();
}

// Whether a keydown event should be allowed to trigger a custom hotkey even
// though it may target a text field. Plain keys / Shift-only combos inside an
// editable element are normal text entry and must NOT fire hotkeys; chords
// using a command modifier (Ctrl/Alt/Cmd) always may. Exported for tests.
export function hotkeyAllowedForTarget(e) {
    const tag = (e.target?.tagName || '').toLowerCase();
    const inEditable = tag === 'input' || tag === 'textarea' || tag === 'select'
        || !!(e.target && e.target.isContentEditable);
    if (!inEditable) return true;
    return !!(e.ctrlKey || e.altKey || e.metaKey);
}

function handler(e) {
    const combo = buildCombo(e);
    if (!combo) return;
    // While the user is typing in a text field, only fire chords that use a
    // command modifier (Ctrl/Alt/Cmd). Plain keys and Shift-only combos are
    // normal text entry and must reach the input. This lets custom hotkeys
    // like Cmd+E work even when a search box is focused (e.g. the Home tab,
    // which auto-focuses its filter input).
    if (!hotkeyAllowedForTarget(e)) return;

    const hit = bindings.find(b => b.combo === combo);
    if (hit) {
        e.preventDefault();
        run(hit.action);
    }
}

function run(action) {
    switch (action) {
        case 'go_dashboard':  window.location.hash = 'dashboard'; break;
        case 'go_trades':     window.location.hash = 'trades'; break;
        // Use LOCAL date — toISOString() is UTC and rolls over at 7-8pm ET,
        // putting the user on yesterday's journal page during evening review.
        case 'go_journal':    window.location.hash = 'journal/' + localToday(); break;
        case 'go_scanners':   window.location.hash = 'scanners'; break;
        case 'go_paper':      window.location.hash = 'paper'; break;
        case 'go_watchlists': window.location.hash = 'watchlists'; break;
        case 'go_research': {
            const s = prompt(t('hotkey.prompt.research_symbol'));
            if (s) window.location.hash = 'research/' + encodeURIComponent(s.toUpperCase());
            break;
        }
        case 'paper_buy_100': paperOrderPrompt('buy', 100); break;
        case 'paper_sell_all': paperSellAllCurrent(); break;
        case 'add_journal_quick': journalQuick(); break;
        default: console.warn('unknown hotkey action', action);
    }
}

async function paperOrderPrompt(side, qty) {
    const sym = prompt(t('hotkey.prompt.paper_order', { side, qty }));
    if (!sym) return;
    const accts = await api.paperAccounts();
    if (!accts.length) { alert(t('hotkey.alert.open_paper_tab')); return; }
    try {
        const o = await api.paperSubmit(accts[0].id, {
            symbol: sym.toUpperCase(), side, qty, order_type: 'market',
            limit_price: null, stop_price: null,
        });
        alert(t('hotkey.alert.order_fill', { status: o.status, symbol: o.symbol, side: o.side, qty: o.qty, price: o.filled_price ?? '—' }));
    } catch (e) { alert(t('hotkey.alert.order_failed', { err: e.message })); }
}

async function paperSellAllCurrent() {
    // "current symbol" = last research view's symbol from hash.
    const m = (window.location.hash || '').match(/^#research\/([^/]+)/);
    if (!m) { alert(t('hotkey.alert.nav_research_first')); return; }
    const sym = decodeURIComponent(m[1]);
    const accts = await api.paperAccounts();
    if (!accts.length) return;
    const pos = (await api.paperPositions(accts[0].id)).find(p => p.symbol === sym);
    if (!pos) { alert(t('hotkey.alert.no_paper_position', { sym })); return; }
    const qty = Math.abs(Number(pos.qty));
    const side = Number(pos.qty) > 0 ? 'sell' : 'cover';
    try {
        await api.paperSubmit(accts[0].id, {
            symbol: sym, side, qty, order_type: 'market',
            limit_price: null, stop_price: null,
        });
    } catch (e) { alert(e.message); }
}

async function journalQuick() {
    const body = prompt(t('hotkey.prompt.journal_quick'));
    if (!body) return;
    try {
        await api.createJournal({ day: localToday(), body_md: body });
    } catch (e) { alert(e.message); }
}
