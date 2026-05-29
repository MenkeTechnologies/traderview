import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderSettings(mount, state) {
    const tok = currentViewToken();
    const [s, filters, templates] = await Promise.all([
        api.settings(),
        api.listFilters(),
        api.noteTemplates(),
    ]);
    if (!viewIsCurrent(tok)) return;
    const accountOptions = state.accounts.map(a =>
        `<option value="${a.id}" ${a.id === s.default_account_id ? 'selected' : ''}>${esc(a.broker)} · ${esc(a.name)}</option>`
    ).join('');
    mount.innerHTML = `
        <h1 data-i18n="view.settings.h1.settings" class="view-title">// SETTINGS</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.settings.h2.appearance">Appearance</h2>
            <p data-i18n="view.settings.hint.crt_scanlines_neon_border_pulse_and_dark_light_the" class="muted small">CRT scanlines, neon-border pulse, and dark/light theme are toggled from the buttons in the topbar. Color scheme switches the whole HUD palette — picks below.</p>
            <div class="settings-scheme">
                <div class="scheme-grid" id="hudSchemeGrid"></div>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.settings.h2.profile">Profile</h2>
            <form id="settings-form" class="inline-form">
                <label><span data-i18n="view.settings.label.default_account">Default account</span>
                    <select name="default_account_id">
                        <option data-i18n="view.settings.opt.none" value="">(none)</option>${accountOptions}
                    </select>
                </label>
                <label><span data-i18n="view.settings.label.base_currency">Base currency</span>
                    <input name="base_currency" value="${esc(s.base_currency)}"></label>
                <label><span data-i18n="view.settings.label.timezone">Timezone</span>
                    <input name="timezone" value="${esc(s.timezone)}"></label>
                <label><span data-i18n="view.settings.label.theme">Theme</span>
                    <select name="theme">
                        <option data-i18n="view.settings.opt.cyberpunk" value="cyberpunk" ${s.theme === 'cyberpunk' ? 'selected' : ''}>Cyberpunk</option>
                        <option data-i18n="view.settings.opt.dark" value="dark" ${s.theme === 'dark' ? 'selected' : ''}>Dark</option>
                    </select>
                </label>
                <label><span data-i18n="view.settings.label.starting_cash">Starting cash</span>
                    <input type="number" step="any" name="starting_cash" value="${s.starting_cash}"></label>
                <label><span data-i18n="view.settings.label.commission_per_share">Commission / share</span>
                    <input type="number" step="any" name="commission_per_share" value="${s.commission_per_share}">
                </label>
                <label><span data-i18n="view.settings.label.commission_per_contract">Commission / contract</span>
                    <input type="number" step="any" name="commission_per_contract" value="${s.commission_per_contract}">
                </label>
                <label><span data-i18n="view.settings.label.auto_flatten">Auto-flatten (new trade after going flat)</span>
                    <input type="checkbox" name="auto_flatten" ${s.auto_flatten ? 'checked' : ''}>
                </label>
                <label><span data-i18n="view.settings.label.require_account_tag">Always require account tag on import</span>
                    <input type="checkbox" name="require_account_tag" ${s.require_account_tag ? 'checked' : ''}>
                </label>
                <button data-i18n="view.settings.btn.save" class="primary" type="submit">Save</button>
            </form>
            <p data-i18n="view.settings.hint.commission_rates_fill_in_only_when_the_broker_file" class="muted small">
                Commission rates fill in only when the broker file omits fees (fee = 0).
                Mirrors TraderVue's "manual rate" behavior — won't double-count.
            </p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.settings.h2.notes_templates">Notes Templates</h2>
            <table class="trades">
                <thead><tr><th data-i18n="view.settings.th.name">Name</th><th data-i18n="view.settings.th.scope">Scope</th><th data-i18n="view.settings.th.default">Default</th><th data-i18n="view.settings.th.updated">Updated</th><th></th></tr></thead>
                <tbody>${templates.map(tpl => `
                    <tr><td>${esc(tpl.name)}</td>
                    <td>${esc(tpl.scope)}</td>
                    <td>${tpl.is_default ? '✓' : ''}</td>
                    <td>${fmtDateTime(tpl.updated_at)}</td>
                    <td>
                        <button data-i18n="view.settings.btn.edit" class="link" data-edit-tpl='${esc(JSON.stringify(tpl))}'>edit</button>
                        <button data-i18n="view.settings.btn.delete" class="link" data-del-tpl="${tpl.id}">delete</button>
                    </td></tr>
                `).join('') || `<tr><td colspan="5" class="muted">${esc(t('view.settings.empty.templates'))}</td></tr>`}
                </tbody>
            </table>
            <form id="tpl-form" class="inline-form" style="margin-top:10px">
                <input name="name" placeholder="template name" data-i18n-placeholder="view.settings.placeholder.template_name" required>
                <select name="scope">
                    <option data-i18n="view.settings.opt.trade" value="trade">trade</option>
                    <option data-i18n="view.settings.opt.journal" value="journal">journal</option>
                </select>
                <label style="flex-direction:row;align-items:center;gap:6px">
                    <input type="checkbox" name="is_default"> default
                </label>
                <textarea name="body_md" placeholder="markdown body — used as default when creating notes for the selected scope" data-i18n-placeholder="view.settings.placeholder.template_body" rows="4" style="flex:1 1 100%"></textarea>
                <button data-i18n="view.settings.btn.save_template" class="primary" type="submit">Save template</button>
            </form>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.settings.h2.saved_filter_sets">Saved filter sets</h2>
            ${filters.length ? `<table class="trades">
                <thead><tr><th data-i18n="view.settings.th.name_2">Name</th><th data-i18n="view.settings.th.default_2">Default</th><th data-i18n="view.settings.th.created">Created</th><th></th></tr></thead>
                <tbody>${filters.map(f => `
                    <tr><td>${esc(f.name)}</td><td>${f.is_default ? '✓' : ''}</td>
                    <td>${fmtDateTime(f.created_at)}</td>
                    <td><button data-i18n="view.settings.btn.delete_2" class="link" data-del-f="${f.id}">delete</button></td></tr>
                `).join('')}</tbody></table>` : '<p data-i18n="view.settings.hint.no_saved_filters" class="muted">No saved filters.</p>'}
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.settings.h2.your_user_id">Your user ID</h2>
            <p data-i18n="view.settings.hint.share_this_with_someone_if_they_want_to_mentor_you">Share this with someone if they want to mentor you.</p>
            <code>${esc(state.me?.id || '')}</code>
        </div>
    `;

    // Repaint the color-scheme grid into the Appearance panel.
    if (window.tvHud && typeof window.tvHud.remountSchemeGrid === 'function') {
        window.tvHud.remountSchemeGrid();
    }

    mount.querySelector('#settings-form').addEventListener('submit', async (e) => {
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
        if (!viewIsCurrent(tok)) return;
        renderSettings(mount, state);
    });

    mount.querySelector('#tpl-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        await api.upsertNoteTemplate(
            fd.get('name'),
            fd.get('scope'),
            fd.get('body_md') || '',
            !!fd.get('is_default'),
        );
        if (!viewIsCurrent(tok)) return;
        renderSettings(mount, state);
    });
    mount.querySelectorAll('[data-edit-tpl]').forEach(b =>
        b.addEventListener('click', () => {
            const t = JSON.parse(b.dataset.editTpl);
            const f = mount.querySelector('#tpl-form');
            if (!f) return;
            f.name.value = t.name;
            f.scope.value = t.scope;
            f.body_md.value = t.body_md;
            f.is_default.checked = t.is_default;
        }));
    mount.querySelectorAll('[data-del-tpl]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteNoteTemplate(b.dataset.delTpl);
            if (!viewIsCurrent(tok)) return;
            renderSettings(mount, state);
        }));
    mount.querySelectorAll('[data-del-f]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteFilter(b.dataset.delF);
            if (!viewIsCurrent(tok)) return;
            renderSettings(mount, state);
        }));
}
