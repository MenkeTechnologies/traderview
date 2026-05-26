import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';

export async function renderSettings(mount, state) {
    const [s, filters] = await Promise.all([api.settings(), api.listFilters()]);
    const accountOptions = state.accounts.map(a =>
        `<option value="${a.id}" ${a.id === s.default_account_id ? 'selected' : ''}>${esc(a.broker)} · ${esc(a.name)}</option>`
    ).join('');
    mount.innerHTML = `
        <h1 class="view-title">// SETTINGS</h1>
        <div class="chart-panel">
            <h2>Profile</h2>
            <form id="settings-form" class="inline-form">
                <label>Default account
                    <select name="default_account_id">
                        <option value="">(none)</option>${accountOptions}
                    </select>
                </label>
                <label>Base currency <input name="base_currency" value="${esc(s.base_currency)}"></label>
                <label>Timezone <input name="timezone" value="${esc(s.timezone)}"></label>
                <label>Theme
                    <select name="theme">
                        <option value="cyberpunk" ${s.theme === 'cyberpunk' ? 'selected' : ''}>Cyberpunk</option>
                        <option value="dark" ${s.theme === 'dark' ? 'selected' : ''}>Dark</option>
                    </select>
                </label>
                <label>Starting cash
                    <input type="number" step="any" name="starting_cash" value="${s.starting_cash}">
                </label>
                <button class="primary" type="submit">Save</button>
            </form>
        </div>

        <div class="chart-panel">
            <h2>Saved filter sets</h2>
            ${filters.length ? `<table class="trades">
                <thead><tr><th>Name</th><th>Default</th><th>Created</th><th></th></tr></thead>
                <tbody>${filters.map(f => `
                    <tr><td>${esc(f.name)}</td><td>${f.is_default ? '✓' : ''}</td>
                    <td>${fmtDateTime(f.created_at)}</td>
                    <td><button class="link" data-del-f="${f.id}">delete</button></td></tr>
                `).join('')}</tbody></table>` : '<p class="muted">No saved filters.</p>'}
        </div>

        <div class="chart-panel">
            <h2>Your user ID</h2>
            <p>Share this with someone if they want to mentor you.</p>
            <code>${esc(state.me?.id || '')}</code>
        </div>
    `;
    document.getElementById('settings-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = Object.assign({}, s, {
            default_account_id: fd.get('default_account_id') || null,
            base_currency: fd.get('base_currency'),
            timezone: fd.get('timezone'),
            theme: fd.get('theme'),
            starting_cash: Number(fd.get('starting_cash')),
        });
        await api.updateSettings(body);
        renderSettings(mount, state);
    });
    document.querySelectorAll('[data-del-f]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteFilter(b.dataset.delF);
            renderSettings(mount, state);
        }));
}
