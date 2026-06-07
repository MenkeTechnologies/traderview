// Hotkeys configuration — Warrior-Trading-style key bindings, repurposed for
// journal/research UX. The actual key-listening lives in hotkey_engine.js.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { tConfirm } from '../dialog.js';

const ACTIONS = [
    { id: 'go_dashboard',      get label() { return t('view.hotkeys.action.go_dashboard'); } },
    { id: 'go_trades',         get label() { return t('view.hotkeys.action.go_trades'); } },
    { id: 'go_journal',        get label() { return t('view.hotkeys.action.go_journal'); } },
    { id: 'go_research',       get label() { return t('view.hotkeys.action.go_research'); } },
    { id: 'go_scanners',       get label() { return t('view.hotkeys.action.go_scanners'); } },
    { id: 'go_paper',          get label() { return t('view.hotkeys.action.go_paper'); } },
    { id: 'go_watchlists',     get label() { return t('view.hotkeys.action.go_watchlists'); } },
    { id: 'paper_buy_100',     get label() { return t('view.hotkeys.action.paper_buy_100'); } },
    { id: 'paper_sell_all',    get label() { return t('view.hotkeys.action.paper_sell_all'); } },
    { id: 'add_journal_quick', get label() { return t('view.hotkeys.action.add_journal_quick'); } },
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
                <input name="name" placeholder="binding name" data-i18n-placeholder="view.hotkeys.placeholder.name"
                       data-tip="view.hotkeys.tip.name" data-shortcut="hotkeys_focus_name" required>
                <button data-i18n="view.hotkeys.btn.capture_combo" data-tip="view.hotkeys.tip.capture" data-shortcut="hotkeys_capture" type="button" id="capture" class="primary"
                    style="background:linear-gradient(180deg,var(--magenta),#7f00b5);border-color:var(--magenta)">
                    Capture combo
                </button>
                <input name="combo" placeholder="ctrl+shift+z" data-i18n-placeholder="view.hotkeys.placeholder.combo" data-tip="view.hotkeys.tip.combo" required readonly>
                <select name="action" data-tip="view.hotkeys.tip.action" required>
                    ${ACTIONS.map(a => `<option value="${a.id}">${esc(a.label)}</option>`).join('')}
                </select>
                <button data-i18n="view.hotkeys.btn.save" data-tip="view.hotkeys.tip.save" class="primary" type="submit">Save</button>
            </form>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.hotkeys.h2.current_bindings">Current bindings</h2>
            ${keys.length ? `<table class="trades">
                <thead><tr><th data-i18n="view.hotkeys.th.name">Name</th><th data-i18n="view.hotkeys.th.combo">Combo</th><th data-i18n="view.hotkeys.th.action">Action</th><th></th></tr></thead>
                <tbody>${keys.map(k => `
                    <tr data-context-scope="hotkey-row" data-id="${esc(k.id)}" data-combo="${esc(k.combo)}"><td>${esc(k.name)}</td>
                    <td><code>${esc(k.combo)}</code></td>
                    <td>${esc(actionLabel(k.action))}</td>
                    <td><button data-i18n="view.hotkeys.btn.delete" data-tip="view.hotkeys.tip.delete_row" class="link" data-del="${k.id}">delete</button></td></tr>
                `).join('')}</tbody></table>` : '<p data-i18n="view.hotkeys.hint.no_bindings_yet" class="muted">No bindings yet.</p>'}
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.hotkeys.h2.action_chart">Bindings per action</h2>
            <div id="hk-chart" style="width:100%;height:240px"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.hotkeys.h2.modifier_chart">Modifier-key usage across bindings</h2>
            <div id="hk-mod-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.hotkeys.hint.modifier_chart" class="muted small">How many bindings include each modifier key (ctrl / alt / shift / meta / plain — no modifier). Reveals layout balance: many plain-key bindings risk accidental triggers; ctrl-heavy maps collide with browser shortcuts. Orthogonal to per-action distribution.</p>
        </div>
    `;
    renderActionChart(keys);
    renderModifierChart(keys);
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
        const name = String(fd.get('name') || '').trim();
        const combo = String(fd.get('combo') || '').trim();
        try {
            await api.upsertHotkey({
                name, combo,
                action: fd.get('action'),
                payload: {},
            });
            if (!viewIsCurrent(tok)) return;
            showToast(t('view.hotkeys.toast.bound', { name, combo }), { level: 'success' });
            renderHotkeys(mount);
        } catch (err) {
            showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
        }
    });
    mount.querySelectorAll('[data-del]').forEach(b =>
        b.addEventListener('click', async () => {
            if (!await tConfirm('view.hotkeys.confirm.delete', {}, { level: 'danger' })) return;
            try {
                await api.deleteHotkey(b.dataset.del);
                if (!viewIsCurrent(tok)) return;
                showToast(t('view.hotkeys.toast.deleted'), { level: 'success' });
                renderHotkeys(mount);
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        }));
}

function actionLabel(id) {
    return ACTIONS.find(a => a.id === id)?.label || id;
}

function renderModifierChart(keys) {
    const el = document.getElementById('hk-mod-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!keys || !keys.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.hotkeys.empty_mod_chart">${esc(t('view.hotkeys.empty_mod_chart'))}</div>`;
        return;
    }
    const modKeys = ['ctrl', 'alt', 'shift', 'meta'];
    const counts = { ctrl: 0, alt: 0, shift: 0, meta: 0, plain: 0 };
    for (const k of keys) {
        const parts = String(k.combo || '').toLowerCase().split('+').map(p => p.trim()).filter(Boolean);
        let touched = false;
        for (const m of modKeys) {
            if (parts.includes(m)) { counts[m] += 1; touched = true; }
        }
        if (!touched) counts.plain += 1;
    }
    const order = ['ctrl', 'alt', 'shift', 'meta', 'plain'];
    const labels = order.map(k => t(`view.hotkeys.chart.mod.${k}`));
    const ys = order.map(k => counts[k]);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.hotkeys.chart.modifier') },
            { label: t('view.hotkeys.chart.count'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 14, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderActionChart(keys) {
    const el = document.getElementById('hk-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!keys || !keys.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.hotkeys.empty_chart">${esc(t('view.hotkeys.empty_chart'))}</div>`;
        return;
    }
    const counts = new Map();
    for (const k of keys) counts.set(k.action, (counts.get(k.action) || 0) + 1);
    const pairs = Array.from(counts.entries()).sort((a, b) => b[1] - a[1]);
    const labels = pairs.map(([id]) => actionLabel(id));
    const ys = pairs.map(([, n]) => n);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.hotkeys.chart.action_idx') },
            { label: t('view.hotkeys.chart.count'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}
