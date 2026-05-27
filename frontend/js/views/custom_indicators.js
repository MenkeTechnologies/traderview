// Custom indicator registry — manage saved indicator presets.
// The Charts tab consumes these via the eval endpoint to overlay series.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const KINDS = [
    { id: 'sma',       label: 'SMA',       params: { period: 20 } },
    { id: 'ema',       label: 'EMA',       params: { period: 20 } },
    { id: 'rsi',       label: 'RSI',       params: { period: 14 } },
    { id: 'bollinger', label: 'Bollinger', params: { period: 20, k: 2 } },
    { id: 'macd',      label: 'MACD',      params: { fast: 12, slow: 26, signal: 9 } },
];

export async function renderCustomIndicators(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// CUSTOM INDICATORS</h1>
        <p class="muted small">Save named indicator + parameter combos (SMA, EMA, RSI, Bollinger,
            MACD). The Charts tab gets a multi-select to overlay any of them on the SVG cursor.
            Backend evaluates the chosen presets against cached bars and returns one series
            per output line (Bollinger emits 3, MACD emits 3, scalars emit 1).</p>

        <div class="chart-panel">
            <h2>Create / update preset</h2>
            <form id="ci-form" class="inline-form">
                <input name="name" placeholder="name (e.g. 'EMA-21 trend')" required style="min-width:200px;">
                <select name="kind">
                    ${KINDS.map(k => `<option value="${k.id}">${esc(k.label)}</option>`).join('')}
                </select>
                <span id="ci-params"></span>
                <label>Color
                    <input name="color" type="color" value="#00e5ff" style="width:48px;height:28px;padding:0;">
                </label>
                <label><input name="is_default" type="checkbox"> default</label>
                <button class="primary" type="submit">Save</button>
                <span id="ci-status" class="muted small"></span>
            </form>
        </div>

        <div class="chart-panel">
            <h2>Saved presets</h2>
            <div id="ci-list"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text">loading…</div></div></div>
        </div>
    `;
    const kindSel = mount.querySelector('#ci-form [name=kind]');
    const renderParams = () => {
        const k = KINDS.find(x => x.id === kindSel.value);
        const params = mount.querySelector('#ci-params');
        if (params) params.innerHTML = Object.entries(k.params).map(
            ([key, val]) => `<label>${esc(key)}
                <input name="param_${key}" type="number" step="any" value="${val}" style="width:70px;">
            </label>`).join('');
    };
    kindSel.addEventListener('change', renderParams);
    renderParams();
    mount.querySelector('#ci-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const k = KINDS.find(x => x.id === fd.get('kind'));
        const params = {};
        for (const key of Object.keys(k.params)) {
            const raw = fd.get(`param_${key}`);
            params[key] = raw == null ? null : Number(raw);
        }
        const body = {
            name: fd.get('name').trim(),
            definition: { kind: k.id, params },
            color: fd.get('color') || '#00e5ff',
            is_default: !!fd.get('is_default'),
        };
        const status = mount.querySelector('#ci-status');
        if (status) status.textContent = 'saving…';
        try {
            await api.createCustomIndicator(body);
            if (!viewIsCurrent(tok)) return;
            e.target.reset();
            renderParams();
            const status2 = mount.querySelector('#ci-status');
            if (status2) status2.textContent = '';
            await refresh(mount, tok);
        }
        catch (err) {
            if (!viewIsCurrent(tok)) return;
            const status2 = mount.querySelector('#ci-status');
            if (status2) status2.textContent = 'error: ' + err.message;
        }
    });
    await refresh(mount, tok);
}

async function refresh(mount, tok) {
    const el = mount.querySelector('#ci-list');
    if (!el) return;
    try {
        const rows = await api.listCustomIndicators();
        if (!viewIsCurrent(tok)) return;
        const el2 = mount.querySelector('#ci-list');
        if (!el2) return;
        if (!rows.length) { el2.innerHTML = '<p class="muted small">No saved indicators yet.</p>'; return; }
        el2.innerHTML = `<table class="trades">
            <thead><tr><th>Name</th><th>Definition</th><th>Color</th><th>Default</th><th></th></tr></thead>
            <tbody>
            ${rows.map(r => `<tr>
                <td>${esc(r.name)}</td>
                <td class="small"><code>${esc(JSON.stringify(r.definition))}</code></td>
                <td><span style="display:inline-block;width:16px;height:16px;background:${esc(r.color)};border-radius:2px;border:1px solid var(--border);"></span></td>
                <td>${r.is_default ? '<span class="pos">★</span>' : ''}</td>
                <td><button class="btn ci-del" data-id="${r.id}">Delete</button></td>
            </tr>`).join('')}
            </tbody></table>`;
        el2.querySelectorAll('.ci-del').forEach(b =>
            b.addEventListener('click', async () => {
                if (!confirm('Delete preset?')) return;
                try { await api.deleteCustomIndicator(b.dataset.id); if (viewIsCurrent(tok)) await refresh(mount, tok); }
                catch (e) { alert(e.message); }
            }));
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const el2 = mount.querySelector('#ci-list');
        if (el2) el2.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}
