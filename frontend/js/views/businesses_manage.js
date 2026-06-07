// Full CRUD page for the user's businesses (Schedule C entities etc.).
//
// Lists every business with inline edit (name, EIN, entity type, NAICS,
// address), set-default, delete actions. Mounted at `#businesses`.
// Adds new businesses via the same wizard the expense pages use.

import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';
import { tConfirm } from '../dialog.js';
import { showToast } from '../toast.js';
import { openSetupWizard } from '../setup_wizard.js';
import { refreshBusinesses } from '../business_context.js';

const ENTITY_TYPES = ['sole_prop', 'llc', 's_corp', 'c_corp', 'partnership'];

export async function renderBusinessesManage(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `<h1 data-i18n="view.businesses_manage.h1" class="view-title">// BUSINESSES</h1>
        <div class="brokers-toolbar">
            <button class="btn primary" id="bzm-new" data-i18n="view.businesses_manage.btn_new">+ New business</button>
        </div>
        <div id="bzm-body" class="brokers-body"></div>`;
    await reload(mount, tok);
    mount.querySelector('#bzm-new').addEventListener('click', async () => {
        const created = await openSetupWizard({ kind: 'business' });
        if (created) {
            await refreshBusinesses();
            showToast(t('biz.created', { name: created.name }), { level: 'success' });
            await reload(mount, tok);
        }
    });
}

async function reload(mount, tok) {
    const body = mount.querySelector('#bzm-body');
    if (!body) return;
    let rows;
    try { rows = await api.businessesList(); } catch (e) {
        body.innerHTML = `<p class="muted">${esc(t('view.businesses_manage.load_failed', { err: e.message || String(e) }))}</p>`;
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!rows.length) {
        body.innerHTML = `<p class="muted">${esc(t('view.businesses_manage.empty'))}</p>`;
        return;
    }
    body.innerHTML = `<table class="trades brokers-table">
        <thead><tr>
            <th data-i18n="view.businesses_manage.col.default">Default</th>
            <th data-i18n="view.businesses_manage.col.name">Name</th>
            <th data-i18n="view.businesses_manage.col.entity">Entity</th>
            <th data-i18n="view.businesses_manage.col.ein">EIN</th>
            <th data-i18n="view.businesses_manage.col.naics">NAICS</th>
            <th data-i18n="view.businesses_manage.col.started">Started</th>
            <th data-i18n="view.businesses_manage.col.created">Created</th>
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
            ? `<span class="brk-default" title="${esc(t('view.businesses_manage.is_default'))}">★</span>`
            : `<button class="link" data-act="default" data-id="${esc(b.id)}" data-i18n="view.brokers_manage.btn_set_default">set</button>`}</td>
        <td><strong>${esc(b.name)}</strong></td>
        <td><code class="muted small">${esc(b.entity_type || 'sole_prop')}</code></td>
        <td>${b.ein ? esc(b.ein) : '<span class="muted">—</span>'}</td>
        <td>${b.naics_code ? esc(b.naics_code) : '<span class="muted">—</span>'}</td>
        <td class="muted small">${b.started_at ? esc(b.started_at) : '—'}</td>
        <td class="muted small">${fmtDateTime(b.created_at)}</td>
        <td>
            <button class="link" data-act="edit" data-id="${esc(b.id)}" data-i18n="view.brokers_manage.btn_edit">edit</button>
            <button class="link danger" data-act="delete" data-id="${esc(b.id)}" data-i18n="view.brokers_manage.btn_delete">delete</button>
        </td>
    </tr>`;
}

async function editRow(b, mount, tok) {
    const root = document.getElementById('tv-dialog-root') || (() => {
        const r = document.createElement('div'); r.id = 'tv-dialog-root';
        document.body.appendChild(r); return r;
    })();
    const entityOpts = ENTITY_TYPES
        .map(et => `<option value="${et}" ${et === (b.entity_type || 'sole_prop') ? 'selected' : ''}>${esc(t('wiz.biz.et.' + et))}</option>`)
        .join('');
    root.innerHTML = `<div class="tv-dialog-overlay" role="dialog" aria-modal="true">
        <div class="tv-dialog-card tv-dialog-info tv-wiz-card">
            <div class="tv-dialog-title" data-i18n="view.businesses_manage.edit_title">Edit business</div>
            <div class="tv-wiz-form">
                <div class="tv-wiz-row">
                    <label class="tv-wiz-label" data-i18n="wiz.biz.name">Business name</label>
                    <input id="eb-name" class="tv-dialog-input" value="${esc(b.name)}">
                </div>
                <div class="tv-wiz-row">
                    <label class="tv-wiz-label" data-i18n="wiz.biz.entity_type">Entity type</label>
                    <select id="eb-entity" class="tv-dialog-input">${entityOpts}</select>
                </div>
                <div class="tv-wiz-row">
                    <label class="tv-wiz-label" data-i18n="wiz.biz.ein">EIN</label>
                    <input id="eb-ein" class="tv-dialog-input" value="${esc(b.ein || '')}">
                </div>
                <div class="tv-wiz-row">
                    <label class="tv-wiz-label" data-i18n="wiz.biz.naics">NAICS code</label>
                    <input id="eb-naics" class="tv-dialog-input" value="${esc(b.naics_code || '')}">
                </div>
                <div class="tv-wiz-row">
                    <label class="tv-wiz-label" data-i18n="view.businesses_manage.principal_addr">Address</label>
                    <input id="eb-addr" class="tv-dialog-input" value="${esc(b.principal_addr || '')}">
                </div>
                <div class="tv-wiz-row">
                    <label class="tv-wiz-label" data-i18n="wiz.biz.started_at">Started</label>
                    <input id="eb-started" class="tv-dialog-input" type="date" value="${esc(b.started_at || '')}">
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
        try {
            // PATCH route accepts the field directly (its Option<Option<T>>
            // shape lets us send `{field: null}` to clear, or omit to leave).
            await api.businessPatch(b.id, {
                name: root.querySelector('#eb-name').value.trim() || b.name,
                entity_type: root.querySelector('#eb-entity').value,
                ein: root.querySelector('#eb-ein').value.trim() || null,
                naics_code: root.querySelector('#eb-naics').value.trim() || null,
                principal_addr: root.querySelector('#eb-addr').value.trim() || null,
                started_at: root.querySelector('#eb-started').value || null,
            });
            await refreshBusinesses();
            showToast(t('view.businesses_manage.toast_saved', { name: b.name }), { level: 'success' });
            close();
            await reload(mount, tok);
        } catch (e) {
            showToast(t('toast.error.api', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    setTimeout(() => root.querySelector('#eb-name')?.focus(), 30);
}

async function setDefault(b, mount, tok) {
    await api.businessSetDefault(b.id);
    await refreshBusinesses();
    showToast(t('view.businesses_manage.toast_default', { name: b.name }), { level: 'success' });
    await reload(mount, tok);
}

async function deleteRow(b, mount, tok) {
    const ok = await tConfirm('view.businesses_manage.confirm_delete', { name: b.name }, { level: 'danger' });
    if (!ok) return;
    await api.businessDelete(b.id);
    await refreshBusinesses();
    showToast(t('view.businesses_manage.toast_deleted', { name: b.name }), { level: 'success' });
    await reload(mount, tok);
}
