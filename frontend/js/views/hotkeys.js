// Hotkeys configuration — Warrior-Trading-style key bindings, repurposed for
// journal/research UX. The actual key-listening lives in hotkey_engine.js.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

const ACTIONS = [
    { id: 'go_dashboard',   label: 'Jump to Dashboard' },
    { id: 'go_trades',      label: 'Jump to Trades' },
    { id: 'go_journal',     label: 'Jump to Journal (today)' },
    { id: 'go_research',    label: 'Jump to Research (prompt symbol)' },
    { id: 'go_scanners',    label: 'Jump to Scanners' },
    { id: 'go_paper',       label: 'Jump to Paper' },
    { id: 'go_watchlists',  label: 'Jump to Watchlists' },
    { id: 'paper_buy_100',  label: 'Paper: BUY 100 of prompt symbol' },
    { id: 'paper_sell_all', label: 'Paper: SELL all of current symbol' },
    { id: 'add_journal_quick', label: 'New journal note (today, prompt body)' },
];

export async function renderHotkeys(mount) {
    const tok = currentViewToken();
    const keys = await api.hotkeys();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 data-i18n="view.hotkeys.h1.hotkeys" class="view-title">// HOTKEYS</h1>
        <p data-i18n="view.hotkeys.hint.das_style_key_bindings_click_capture_then_press_th" class="muted small">DAS-style key bindings. Click "capture" then press the desired combo.</p>

        <div class="chart-panel">
            <h2 data-i18n="view.hotkeys.h2.new_binding">New binding</h2>
            <form id="hk-form" class="inline-form">
                <input name="name" placeholder="binding name" required>
                <button data-i18n="view.hotkeys.btn.capture_combo" type="button" id="capture" class="primary"
                    style="background:linear-gradient(180deg,var(--magenta),#7f00b5);border-color:var(--magenta)">
                    Capture combo
                </button>
                <input name="combo" placeholder="ctrl+shift+z" required readonly>
                <select name="action" required>
                    ${ACTIONS.map(a => `<option value="${a.id}">${esc(a.label)}</option>`).join('')}
                </select>
                <button data-i18n="view.hotkeys.btn.save" class="primary" type="submit">Save</button>
            </form>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.hotkeys.h2.current_bindings">Current bindings</h2>
            ${keys.length ? `<table class="trades">
                <thead><tr><th data-i18n="view.hotkeys.th.name">Name</th><th data-i18n="view.hotkeys.th.combo">Combo</th><th data-i18n="view.hotkeys.th.action">Action</th><th></th></tr></thead>
                <tbody>${keys.map(k => `
                    <tr><td>${esc(k.name)}</td>
                    <td><code>${esc(k.combo)}</code></td>
                    <td>${esc(actionLabel(k.action))}</td>
                    <td><button data-i18n="view.hotkeys.btn.delete" class="link" data-del="${k.id}">delete</button></td></tr>
                `).join('')}</tbody></table>` : '<p data-i18n="view.hotkeys.hint.no_bindings_yet" class="muted">No bindings yet.</p>'}
        </div>
    `;
    const comboInput = mount.querySelector('[name=combo]');
    mount.querySelector('#capture').addEventListener('click', () => {
        comboInput.value = '';
        comboInput.placeholder = t('view.hotkeys.placeholder.press_a_key');
        const handler = (e) => {
            const parts = [];
            if (e.ctrlKey)  parts.push('ctrl');
            if (e.altKey)   parts.push('alt');
            if (e.shiftKey) parts.push('shift');
            if (e.metaKey)  parts.push('meta');
            const key = e.key.length === 1 ? e.key.toLowerCase() : e.key.toLowerCase();
            if (key !== 'control' && key !== 'shift' && key !== 'alt' && key !== 'meta') {
                parts.push(key);
                comboInput.value = parts.join('+');
                window.removeEventListener('keydown', handler, true);
                e.preventDefault();
            }
        };
        window.addEventListener('keydown', handler, true);
    });
    mount.querySelector('#hk-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        await api.upsertHotkey({
            name: fd.get('name'),
            combo: fd.get('combo'),
            action: fd.get('action'),
            payload: {},
        });
        if (!viewIsCurrent(tok)) return;
        renderHotkeys(mount);
    });
    mount.querySelectorAll('[data-del]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteHotkey(b.dataset.del);
            if (!viewIsCurrent(tok)) return;
            renderHotkeys(mount);
        }));
}

function actionLabel(id) {
    return ACTIONS.find(a => a.id === id)?.label || id;
}
