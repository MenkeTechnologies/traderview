import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { tConfirm } from '../dialog.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

// Single page that lists trade- and journal-scoped note templates with an
// inline editor at the bottom. The Insert template buttons on journal and
// trade-detail already pull defaults via api.defaultNoteTemplate(scope).
export async function renderNoteTemplates(mount) {
    const tok = currentViewToken();
    const all = await api.noteTemplates();
    if (!viewIsCurrent(tok)) return;

    const trade   = (all || []).filter(t => t.scope === 'trade');
    const journal = (all || []).filter(t => t.scope === 'journal');

    mount.innerHTML = `
        <h1 class="view-title">// ${esc(t('view.note_templates.h1'))}</h1>
        <p class="muted small" data-i18n="view.note_templates.hint">
            Templates auto-populate "Insert template" buttons in Journal and
            Trade Detail. Mark one per scope as default — that's the one
            pulled when you hit Insert.
        </p>

        ${section('trade',   t('view.note_templates.section.trade'),   trade)}
        ${section('journal', t('view.note_templates.section.journal'), journal)}

        <div class="chart-panel" id="tpl-editor">
            <h2 data-i18n="view.note_templates.h2.new_or_edit">New / edit template</h2>
            <form id="tpl-form" class="tpl-form">
                <input type="hidden" name="id" value="">
                <label><span data-i18n="view.note_templates.field.name">Name</span>
                    <input type="text" name="name" required placeholder="e.g. ABCD breakout review" /></label>
                <label><span data-i18n="view.note_templates.field.scope">Scope</span>
                    <select name="scope">
                        <option value="trade"   data-i18n="view.note_templates.scope.trade">Trade-level (per-trade journal)</option>
                        <option value="journal" data-i18n="view.note_templates.scope.journal">Daily journal</option>
                    </select>
                </label>
                <label class="tpl-default">
                    <input type="checkbox" name="is_default" />
                    <span data-i18n="view.note_templates.field.default">Set as default for this scope</span>
                </label>
                <label><span data-i18n="view.note_templates.field.body">Body (markdown)</span>
                    <textarea name="body_md" rows="14" required
                        placeholder="# Setup\n- entry trigger:\n- thesis:\n\n# Execution\n- size:\n- stop:\n\n# Review\n- mistakes:\n- lessons:"></textarea></label>
                <div class="inline-form">
                    <button type="submit" class="primary" data-i18n="view.note_templates.btn.save">Save</button>
                    <button type="button" class="btn btn-secondary" id="tpl-cancel" data-i18n="view.note_templates.btn.reset">Reset</button>
                </div>
            </form>
        </div>
    `;

    const list = mount;
    list.querySelectorAll('[data-edit-tpl]').forEach(btn => {
        btn.addEventListener('click', () => {
            const tpl = (all || []).find(x => x.id === btn.dataset.editTpl);
            if (!tpl) return;
            const form = mount.querySelector('#tpl-form');
            form.id.value = tpl.id;
            form.name.value = tpl.name;
            form.scope.value = tpl.scope;
            form.is_default.checked = !!tpl.is_default;
            form.body_md.value = tpl.body_md || '';
            mount.querySelector('#tpl-editor').scrollIntoView({ behavior: 'smooth' });
        });
    });
    list.querySelectorAll('[data-del-tpl]').forEach(btn => {
        btn.addEventListener('click', async () => {
            if (!await tConfirm('view.note_templates.confirm.delete', {}, { level: 'danger' })) return;
            try {
                await api.deleteNoteTemplate(btn.dataset.delTpl);
                if (!viewIsCurrent(tok)) return;
                showToast(t('view.note_templates.toast.deleted'), { level: 'success' });
                renderNoteTemplates(mount);
            } catch (e) {
                showToast(t('toast.error.api', { err: e.message }), { level: 'error' });
            }
        });
    });

    const form = mount.querySelector('#tpl-form');
    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(form);
        try {
            await api.upsertNoteTemplate(
                fd.get('name'),
                fd.get('scope'),
                fd.get('body_md'),
                fd.get('is_default') === 'on',
            );
            if (!viewIsCurrent(tok)) return;
            showToast(t('view.note_templates.toast.saved'), { level: 'success' });
            renderNoteTemplates(mount);
        } catch (err) {
            showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
        }
    });
    mount.querySelector('#tpl-cancel').addEventListener('click', () => {
        form.reset();
        form.id.value = '';
    });
}

function section(scope, label, rows) {
    return `
        <div class="chart-panel">
            <h2>${esc(label)} <span class="muted small">(${rows.length})</span></h2>
            ${rows.length ? `
                <table class="trades">
                    <thead><tr>
                        <th data-i18n="view.note_templates.col.name">Name</th>
                        <th data-i18n="view.note_templates.col.default">Default</th>
                        <th data-i18n="view.note_templates.col.updated">Updated</th>
                        <th></th>
                    </tr></thead>
                    <tbody>${rows.map(r => `
                        <tr>
                            <td>${esc(r.name)}</td>
                            <td>${r.is_default ? '★' : ''}</td>
                            <td>${r.updated_at ? fmtDateTime(r.updated_at) : '—'}</td>
                            <td>
                                <button class="link" data-edit-tpl="${esc(r.id)}" data-i18n="view.note_templates.btn.edit">edit</button>
                                <button class="link" data-del-tpl="${esc(r.id)}" data-i18n="view.note_templates.btn.delete">delete</button>
                            </td>
                        </tr>
                    `).join('')}</tbody>
                </table>
            ` : `<p class="muted small" data-i18n="view.note_templates.empty">No ${scope} templates yet.</p>`}
        </div>
    `;
}
