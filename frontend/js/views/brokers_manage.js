// Full CRUD page for the user's brokers.
//
// Lists every broker with inline edit (display name, home URL, notes),
// set-default, and delete actions. Mounted at `#brokers`. Adds new
// brokers via the same wizard the topbar selector uses.

import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';
import { tConfirm } from '../dialog.js';
import { showToast } from '../toast.js';
import { openSetupWizard } from '../setup_wizard.js';
import { refreshBrokers } from '../broker_context.js';

export async function renderBrokersManage(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `<h1 data-i18n="view.brokers_manage.h1" class="view-title">// BROKERS</h1>
        <div class="brokers-toolbar">
            <button class="btn primary" id="bm-new" data-i18n="view.brokers_manage.btn_new">+ New broker</button>
        </div>
        <div id="bm-body" class="brokers-body"></div>`;
    await reload(mount, tok);
    mount.querySelector('#bm-new').addEventListener('click', async () => {
        const created = await openSetupWizard({ kind: 'broker' });
        if (created) {
            await refreshBrokers();
            showToast(t('broker.created', { name: created.display_name }), { level: 'success' });
            await reload(mount, tok);
        }
    });
}

async function reload(mount, tok) {
    const body = mount.querySelector('#bm-body');
    if (!body) return;
    let rows;
    try { rows = await api.brokersList(); } catch (e) {
        body.innerHTML = `<p class="muted">${esc(t('view.brokers_manage.load_failed', { err: e.message || String(e) }))}</p>`;
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!rows.length) {
        body.innerHTML = `<p class="muted">${esc(t('view.brokers_manage.empty'))}</p>`;
        return;
    }
    body.innerHTML = `<table class="trades brokers-table">
        <thead><tr>
            <th data-i18n="view.brokers_manage.col.default">Default</th>
            <th data-i18n="view.brokers_manage.col.name">Name</th>
            <th data-i18n="view.brokers_manage.col.slug">Slug</th>
            <th data-i18n="view.brokers_manage.col.home">Home URL</th>
            <th data-i18n="view.brokers_manage.col.notes">Notes</th>
            <th data-i18n="view.brokers_manage.col.created">Created</th>
            <th></th>
        </tr></thead>
        <tbody>${rows.map(rowHtml).join('')}</tbody>
    </table>`;
    body.querySelectorAll('[data-act]').forEach(btn => {
        btn.addEventListener('click', async (e) => {
            e.preventDefault();
            const id = btn.dataset.id;
            const row = rows.find(r => r.id === id);
            if (!row) return;
            try {
                if (btn.dataset.act === 'edit') await editRow(row, mount, tok);
                else if (btn.dataset.act === 'default') await setDefault(row, mount, tok);
                else if (btn.dataset.act === 'delete') await deleteRow(row, mount, tok);
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message || String(err) }), { level: 'error' });
            }
        });
    });
}

function rowHtml(b) {
    return `<tr data-id="${esc(b.id)}">
        <td>${b.is_default
            ? `<span class="brk-default" title="${esc(t('view.brokers_manage.is_default'))}">★</span>`
            : `<button class="link" data-act="default" data-id="${esc(b.id)}" data-i18n="view.brokers_manage.btn_set_default">set</button>`}</td>
        <td><strong>${esc(b.display_name)}</strong></td>
        <td><code class="muted small">${esc(b.slug)}</code></td>
        <td>${b.home_url ? `<a href="${esc(b.home_url)}" target="_blank" rel="noopener">${esc(b.home_url)}</a>` : '<span class="muted">—</span>'}</td>
        <td class="brk-notes">${b.notes ? esc(b.notes) : '<span class="muted">—</span>'}</td>
        <td class="muted small">${fmtDateTime(b.created_at)}</td>
        <td>
            <button class="link" data-act="edit" data-id="${esc(b.id)}" data-i18n="view.brokers_manage.btn_edit">edit</button>
            <button class="link danger" data-act="delete" data-id="${esc(b.id)}" data-i18n="view.brokers_manage.btn_delete">delete</button>
        </td>
    </tr>`;
}

async function editRow(b, mount, tok) {
    // Reuse the dialog scaffold from setup_wizard via a lightweight
    // inline form — the wizard supports create only, so the edit dialog
    // is built ad hoc here. Keeps the modal styling and Esc/Enter behavior
    // consistent with the rest of the app.
    const root = document.getElementById('tv-dialog-root') || (() => {
        const r = document.createElement('div'); r.id = 'tv-dialog-root';
        document.body.appendChild(r); return r;
    })();
    root.innerHTML = `<div class="tv-dialog-overlay" role="dialog" aria-modal="true">
        <div class="tv-dialog-card tv-dialog-info tv-wiz-card">
            <div class="tv-dialog-title" data-i18n="view.brokers_manage.edit_title">Edit broker</div>
            <div class="tv-wiz-form">
                <div class="tv-wiz-row">
                    <label class="tv-wiz-label" data-i18n="wiz.broker.name">Display name</label>
                    <input id="em-name" class="tv-dialog-input" value="${esc(b.display_name)}">
                </div>
                <div class="tv-wiz-row">
                    <label class="tv-wiz-label" data-i18n="wiz.broker.home_url">Home URL</label>
                    <input id="em-home" class="tv-dialog-input" value="${esc(b.home_url || '')}">
                </div>
                <div class="tv-wiz-row">
                    <label class="tv-wiz-label" data-i18n="wiz.broker.notes">Notes</label>
                    <textarea id="em-notes" class="tv-dialog-input tv-wiz-textarea" rows="2">${esc(b.notes || '')}</textarea>
                </div>
            </div>
            <div class="tv-dialog-actions">
                <button class="tv-dialog-btn tv-dialog-cancel" data-i18n="dialog.btn.cancel">Cancel</button>
                <button class="tv-dialog-btn tv-dialog-confirm" data-i18n="view.brokers_manage.save">Save</button>
            </div>
        </div></div>`;
    const ov = root.querySelector('.tv-dialog-overlay');
    const close = () => { root.innerHTML = ''; };
    root.querySelector('.tv-dialog-cancel').addEventListener('click', close);
    ov.addEventListener('click', (e) => { if (e.target === ov) close(); });
    root.querySelector('.tv-dialog-confirm').addEventListener('click', async () => {
        const body = {
            display_name: root.querySelector('#em-name').value.trim() || b.display_name,
            home_url: [root.querySelector('#em-home').value.trim() || null],
            notes: [root.querySelector('#em-notes').value.trim() || null],
        };
        // The PATCH route uses double-Option for nullable fields — wrap
        // in an array as JSON [value] / [null] hits Option<Option<String>>.
        try {
            await api.brokerPatch(b.id, {
                display_name: body.display_name,
                home_url: body.home_url[0],
                notes: body.notes[0],
            });
            await refreshBrokers();
            showToast(t('view.brokers_manage.toast_saved', { name: body.display_name }), { level: 'success' });
            close();
            await reload(mount, tok);
        } catch (e) {
            showToast(t('toast.error.api', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    setTimeout(() => root.querySelector('#em-name')?.focus(), 30);
}

async function setDefault(b, mount, tok) {
    await api.brokerSetDefault(b.id);
    await refreshBrokers();
    showToast(t('view.brokers_manage.toast_default', { name: b.display_name }), { level: 'success' });
    await reload(mount, tok);
}

async function deleteRow(b, mount, tok) {
    const ok = await tConfirm('view.brokers_manage.confirm_delete', { name: b.display_name }, { level: 'danger' });
    if (!ok) return;
    await api.brokerDelete(b.id);
    await refreshBrokers();
    showToast(t('view.brokers_manage.toast_deleted', { name: b.display_name }), { level: 'success' });
    await reload(mount, tok);
}
