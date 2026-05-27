// Hotkey engine — listens globally, dispatches to actions.
import { api } from './api.js';
import { localToday } from './local_date.js';

let bindings = [];

export async function reloadHotkeys() {
    try { bindings = await api.hotkeys(); } catch (_) { bindings = []; }
}

export function installHotkeyEngine() {
    window.addEventListener('keydown', handler, true);
    reloadHotkeys();
}

function handler(e) {
    // Ignore when typing in inputs.
    const tag = (e.target?.tagName || '').toLowerCase();
    if (tag === 'input' || tag === 'textarea' || tag === 'select') return;

    const parts = [];
    if (e.ctrlKey)  parts.push('ctrl');
    if (e.altKey)   parts.push('alt');
    if (e.shiftKey) parts.push('shift');
    if (e.metaKey)  parts.push('meta');
    const key = e.key.length === 1 ? e.key.toLowerCase() : e.key.toLowerCase();
    if (key === 'control' || key === 'shift' || key === 'alt' || key === 'meta') return;
    parts.push(key);
    const combo = parts.join('+');
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
            const s = prompt('Research symbol:');
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
    const sym = prompt('Symbol to ' + side + ' ' + qty + ':');
    if (!sym) return;
    const accts = await api.paperAccounts();
    if (!accts.length) { alert('Open the Paper tab first to initialize an account.'); return; }
    try {
        const o = await api.paperSubmit(accts[0].id, {
            symbol: sym.toUpperCase(), side, qty, order_type: 'market',
            limit_price: null, stop_price: null,
        });
        alert(`${o.status}: ${o.symbol} ${o.side} ${o.qty} @ ${o.filled_price ?? '—'}`);
    } catch (e) { alert('Order failed: ' + e.message); }
}

async function paperSellAllCurrent() {
    // "current symbol" = last research view's symbol from hash.
    const m = (window.location.hash || '').match(/^#research\/([^/]+)/);
    if (!m) { alert('Navigate to a Research page first.'); return; }
    const sym = decodeURIComponent(m[1]);
    const accts = await api.paperAccounts();
    if (!accts.length) return;
    const pos = (await api.paperPositions(accts[0].id)).find(p => p.symbol === sym);
    if (!pos) { alert('No paper position in ' + sym); return; }
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
    const body = prompt('Journal note for today:');
    if (!body) return;
    try {
        await api.createJournal({ day: localToday(), body_md: body });
    } catch (e) { alert(e.message); }
}
