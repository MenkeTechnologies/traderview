import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';

export async function renderSettings(mount, state) {
    const [s, filters, templates] = await Promise.all([
        api.settings(),
        api.listFilters(),
        api.noteTemplates(),
    ]);
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
                <label>Starting cash <input type="number" step="any" name="starting_cash" value="${s.starting_cash}"></label>
                <label>Commission / share
                    <input type="number" step="any" name="commission_per_share" value="${s.commission_per_share}">
                </label>
                <label>Commission / contract
                    <input type="number" step="any" name="commission_per_contract" value="${s.commission_per_contract}">
                </label>
                <label>Auto-flatten (new trade after going flat)
                    <input type="checkbox" name="auto_flatten" ${s.auto_flatten ? 'checked' : ''}>
                </label>
                <label>Always require account tag on import
                    <input type="checkbox" name="require_account_tag" ${s.require_account_tag ? 'checked' : ''}>
                </label>
                <button class="primary" type="submit">Save</button>
            </form>
            <p class="muted small">
                Commission rates fill in only when the broker file omits fees (fee = 0).
                Mirrors TraderVue's "manual rate" behavior — won't double-count.
            </p>
        </div>

        <div class="chart-panel">
            <h2>Notes Templates</h2>
            <table class="trades">
                <thead><tr><th>Name</th><th>Scope</th><th>Default</th><th>Updated</th><th></th></tr></thead>
                <tbody>${templates.map(t => `
                    <tr><td>${esc(t.name)}</td>
                    <td>${esc(t.scope)}</td>
                    <td>${t.is_default ? '✓' : ''}</td>
                    <td>${fmtDateTime(t.updated_at)}</td>
                    <td>
                        <button class="link" data-edit-tpl='${esc(JSON.stringify(t))}'>edit</button>
                        <button class="link" data-del-tpl="${t.id}">delete</button>
                    </td></tr>
                `).join('') || '<tr><td colspan="5" class="muted">No templates yet.</td></tr>'}
                </tbody>
            </table>
            <form id="tpl-form" class="inline-form" style="margin-top:10px">
                <input name="name" placeholder="template name" required>
                <select name="scope">
                    <option value="trade">trade</option>
                    <option value="journal">journal</option>
                </select>
                <label style="flex-direction:row;align-items:center;gap:6px">
                    <input type="checkbox" name="is_default"> default
                </label>
                <textarea name="body_md" placeholder="markdown body — used as default when creating notes for the selected scope" rows="4" style="flex:1 1 100%"></textarea>
                <button class="primary" type="submit">Save template</button>
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
            commission_per_share: Number(fd.get('commission_per_share') || 0),
            commission_per_contract: Number(fd.get('commission_per_contract') || 0),
            auto_flatten: !!fd.get('auto_flatten'),
            require_account_tag: !!fd.get('require_account_tag'),
        });
        await api.updateSettings(body);
        renderSettings(mount, state);
    });

    document.getElementById('tpl-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        await api.upsertNoteTemplate(
            fd.get('name'),
            fd.get('scope'),
            fd.get('body_md') || '',
            !!fd.get('is_default'),
        );
        renderSettings(mount, state);
    });
    document.querySelectorAll('[data-edit-tpl]').forEach(b =>
        b.addEventListener('click', () => {
            const t = JSON.parse(b.dataset.editTpl);
            const f = document.getElementById('tpl-form');
            f.name.value = t.name;
            f.scope.value = t.scope;
            f.body_md.value = t.body_md;
            f.is_default.checked = t.is_default;
        }));
    document.querySelectorAll('[data-del-tpl]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteNoteTemplate(b.dataset.delTpl);
            renderSettings(mount, state);
        }));
    document.querySelectorAll('[data-del-f]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteFilter(b.dataset.delF);
            renderSettings(mount, state);
        }));
}
