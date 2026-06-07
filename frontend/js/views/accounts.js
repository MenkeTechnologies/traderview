import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';
import { tConfirm } from '../dialog.js';
import { showToast } from '../toast.js';
import { currencyOptions } from '../_currencies.js';

export async function renderAccounts(mount, _state, onChange) {
    const tok = currentViewToken();
    // Pull accounts + the live brokers list in parallel — the broker
    // dropdown is now sourced from the user's brokers table (so custom
    // "Other" brokers from the wizard show up here too), not a hardcoded
    // import-parser list.
    const [accounts, brokers] = await Promise.all([api.accounts(), api.brokersList().catch(() => [])]);
    if (!viewIsCurrent(tok)) return;
    const brokerOpts = brokers.length
        ? brokers.map(b => `<option value="${esc(b.slug)}">${esc(b.display_name)}</option>`).join('')
        : `<option value="manual" data-i18n="view.accounts.opt.manual_other">Manual / Other</option>`;
    mount.innerHTML = `
        <h1 data-i18n="view.accounts.h1.accounts" class="view-title">// ACCOUNTS</h1>
        <div class="chart-panel">
            <h2 data-i18n="view.accounts.h2.add_account">Add account</h2>
            <form id="acct-form" class="inline-form">
                <select name="broker" data-tip="view.accounts.tip.broker">
                    ${brokerOpts}
                </select>
                <input name="name" placeholder="account name (e.g. Margin)" data-i18n-placeholder="view.accounts.placeholder.name"
                       data-tip="view.accounts.tip.name" data-shortcut="accounts_focus_name" required>
                <select name="base_currency" data-tip="view.accounts.tip.base_currency">${currencyOptions('USD')}</select>
                <button data-i18n="view.accounts.btn.create" data-tip="view.accounts.tip.create" class="primary" type="submit">Create</button>
            </form>
        </div>

        <table class="trades">
            <thead><tr><th data-i18n="view.accounts.th.broker">Broker</th><th data-i18n="view.accounts.th.name">Name</th><th data-i18n="view.accounts.th.currency">Currency</th><th data-i18n="view.accounts.th.created">Created</th><th></th></tr></thead>
            <tbody>${accounts.map(a => `
                <tr data-context-scope="account-row" data-id="${esc(a.id)}" data-name="${esc(a.name)}"><td>${esc(a.broker)}</td><td>${esc(a.name)}</td>
                <td>${esc(a.base_currency)}</td>
                <td>${fmtDateTime(a.created_at)}</td>
                <td>
                    <button class="link" data-edit="${a.id}" data-i18n="view.accounts.btn.edit" style="margin-right:8px">edit</button>
                    <button class="link" data-rebuild="${a.id}" data-i18n="view.accounts.btn.rebuild" style="margin-right:8px">rebuild trades</button>
                    <button data-i18n="view.accounts.btn.delete" class="link danger" data-del="${a.id}">delete</button>
                </td></tr>
            `).join('') || `<tr><td colspan="5" class="muted">${esc(t('view.accounts.empty'))}</td></tr>`}
            </tbody>
        </table>

        <div class="chart-panel">
            <h2 data-i18n="view.accounts.h2.broker_chart">Accounts by broker</h2>
            <div id="acct-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.accounts.h2.currency_chart">Accounts by currency</h2>
            <div id="acct-ccy-chart" style="width:100%;height:200px"></div>
        </div>
    `;
    renderBrokerChart(accounts);
    renderCurrencyChart(accounts);

    mount.querySelector('#acct-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const name = String(fd.get('name') || '').trim();
        try {
            await api.createAccount(fd.get('broker'), name, fd.get('base_currency'));
            if (!viewIsCurrent(tok)) return;
            showToast(t('view.accounts.toast.created', { name }), { level: 'success' });
            if (onChange) onChange();
            renderAccounts(mount, _state, onChange);
        } catch (err) {
            showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
        }
    });
    mount.querySelectorAll('[data-edit]').forEach(b =>
        b.addEventListener('click', async () => {
            const acct = accounts.find(a => a.id === b.dataset.edit);
            if (!acct) return;
            await openAccountEdit(acct, brokers, async () => {
                if (!viewIsCurrent(tok)) return;
                if (onChange) onChange();
                renderAccounts(mount, _state, onChange);
            });
        }));
    mount.querySelectorAll('[data-del]').forEach(b =>
        b.addEventListener('click', async () => {
            if (!await tConfirm('view.accounts.confirm.delete', {}, { level: 'danger' })) return;
            try {
                await api.deleteAccount(b.dataset.del);
                if (!viewIsCurrent(tok)) return;
                const tr = b.closest('tr');
                const name = tr?.dataset?.name || '';
                showToast(t('view.accounts.toast.deleted', { name }), { level: 'success' });
                if (onChange) onChange();
                renderAccounts(mount, _state, onChange);
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        }));
    mount.querySelectorAll('[data-rebuild]').forEach(b =>
        b.addEventListener('click', async () => {
            try {
                b.disabled = true;
                const r = await api.rebuildTrades(b.dataset.rebuild);
                if (!viewIsCurrent(tok)) return;
                showToast(t('view.accounts.toast.rebuilt', { n: r.trades_rolled }), { level: 'success' });
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            } finally {
                b.disabled = false;
            }
        }));
}

async function openAccountEdit(acct, brokers, onSaved) {
    const root = document.getElementById('tv-dialog-root') || (() => {
        const r = document.createElement('div'); r.id = 'tv-dialog-root';
        document.body.appendChild(r); return r;
    })();
    const brokerOpts = brokers
        .map(b => `<option value="${esc(b.slug)}" ${b.slug === acct.broker ? 'selected' : ''}>${esc(b.display_name)}</option>`)
        .join('') || `<option value="manual">Manual / Other</option>`;
    root.innerHTML = `<div class="tv-dialog-overlay" role="dialog" aria-modal="true">
        <div class="tv-dialog-card tv-dialog-info tv-wiz-card">
            <div class="tv-dialog-title" data-i18n="view.accounts.edit_title">Edit account</div>
            <div class="tv-wiz-form">
                <div class="tv-wiz-row">
                    <label class="tv-wiz-label" data-i18n="wiz.acct.broker">Broker</label>
                    <select id="ea-broker" class="tv-dialog-input">${brokerOpts}</select>
                </div>
                <div class="tv-wiz-row">
                    <label class="tv-wiz-label" data-i18n="wiz.acct.name">Account name</label>
                    <input id="ea-name" class="tv-dialog-input" value="${esc(acct.name)}">
                </div>
                <div class="tv-wiz-row">
                    <label class="tv-wiz-label" data-i18n="wiz.acct.base_currency">Base currency</label>
                    <select id="ea-ccy" class="tv-dialog-input">${currencyOptions(acct.base_currency)}</select>
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
            await api.patchAccount(acct.id, {
                broker: root.querySelector('#ea-broker').value,
                name:   root.querySelector('#ea-name').value.trim() || acct.name,
                base_currency: root.querySelector('#ea-ccy').value || acct.base_currency,
            });
            showToast(t('view.accounts.toast.saved', { name: acct.name }), { level: 'success' });
            close();
            await onSaved?.();
        } catch (e) {
            showToast(t('toast.error.api', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    setTimeout(() => root.querySelector('#ea-name')?.focus(), 30);
}

// Render a horizontal category bar chart as plain DOM. uPlot was the
// previous tool here, but it defaults to a time-scale x-axis — N=2
// integer x-values rendered with split labels mod-mapped to the
// category list ("webull webull webull ...") and the cursor tooltip
// showed `1969-12-31` (Unix epoch 1 + tz offset). For 2 brokers / 1
// currency it's also pure overkill.
function renderCategoryBars(el, pairs, color) {
    const max = pairs.reduce((m, [, n]) => Math.max(m, n), 0) || 1;
    el.innerHTML = `<ul class="acct-bars">${
        pairs.map(([label, n]) => {
            const pct = Math.round((n / max) * 100);
            return `<li>
                <span class="acct-bars-label">${esc(label)}</span>
                <span class="acct-bars-track">
                    <span class="acct-bars-fill" style="width:${pct}%;background:${color}"></span>
                </span>
                <span class="acct-bars-count">${esc(String(n))}</span>
            </li>`;
        }).join('')
    }</ul>`;
}

function renderCurrencyChart(accounts) {
    const el = document.getElementById('acct-ccy-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!accounts || !accounts.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.accounts.empty_ccy_chart">${esc(t('view.accounts.empty_ccy_chart'))}</div>`;
        return;
    }
    const counts = new Map();
    for (const a of accounts) {
        const key = (a.base_currency || '?').toUpperCase();
        counts.set(key, (counts.get(key) || 0) + 1);
    }
    const pairs = Array.from(counts.entries()).sort((a, b) => b[1] - a[1]);
    renderCategoryBars(el, pairs, '#7af0a8');
}

function renderBrokerChart(accounts) {
    const el = document.getElementById('acct-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!accounts || !accounts.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.accounts.empty_chart">${esc(t('view.accounts.empty_chart'))}</div>`;
        return;
    }
    const counts = new Map();
    for (const a of accounts) {
        const key = a.broker || '?';
        counts.set(key, (counts.get(key) || 0) + 1);
    }
    const pairs = Array.from(counts.entries()).sort((a, b) => b[1] - a[1]);
    renderCategoryBars(el, pairs, '#00e5ff');
}

