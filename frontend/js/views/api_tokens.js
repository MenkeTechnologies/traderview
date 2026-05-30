// Developer tab — personal access token management.
// New tokens are shown ONCE at creation time, then never again.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { tConfirm } from '../dialog.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderDeveloper(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.api_tokens.h1.developer_public_api" class="view-title">// DEVELOPER — PUBLIC API</h1>
        <p class="muted small" data-i18n="view.api_tokens.hint.intro">Personal Access Tokens authenticate third-party integrations against the same endpoints the UI uses. Pass them as Authorization: Bearer pat_&lt;prefix&gt;_&lt;secret&gt;. Tokens are argon2-hashed at rest — the secret is shown once at creation time and never recoverable afterwards. Revoke a token to cut access immediately.</p>

        <div class="chart-panel">
            <h2 data-i18n="view.api_tokens.h2.create_token">Create token</h2>
            <form id="tok-form" class="inline-form">
                <label><span data-i18n="view.api_tokens.label.name">Name</span>
                    <input name="name" placeholder="n8n staging" data-i18n-placeholder="view.api_tokens.placeholder.name" required style="min-width:220px;"></label>
                <label><span data-i18n="view.api_tokens.label.scopes">Scopes</span>
                    <select name="scopes" multiple size="3" style="min-width:120px;">
                        <option data-i18n="view.api_tokens.opt.read" value="read" selected>read</option>
                        <option data-i18n="view.api_tokens.opt.write" value="write">write</option>
                        <option data-i18n="view.api_tokens.opt.admin" value="admin">admin</option>
                    </select>
                </label>
                <label><span data-i18n="view.api_tokens.label.expires">Expires (optional)</span>
                    <input name="expires_at" type="date" style="width:160px;">
                </label>
                <label><span data-i18n="view.api_tokens.label.rate_limit">Rate limit (req/min)</span>
                    <input name="rate_limit_per_min" type="number" min="1" max="10000"
                           value="60" style="width:90px;">
                </label>
                <button data-i18n="view.api_tokens.btn.generate" class="primary" type="submit">Generate</button>
            </form>
            <div id="tok-new"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.api_tokens.h2.active_tokens">Active tokens</h2>
            <div id="tok-list"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.loading">loading…</div></div></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.api_tokens.h2.example_use_curl">Example use (curl)</h2>
            <pre style="background:#0d0d22;padding:12px;overflow:auto;font-size:11px;">curl -H "Authorization: Bearer pat_xxx_yyy" \\
     ${esc(window.location.origin)}/api/trades?status=closed&amp;limit=50</pre>
        </div>
    `;

    mount.querySelector('#tok-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const scopes = Array.from(e.target.scopes.selectedOptions).map(o => o.value);
        const expIso = fd.get('expires_at');
        const body = {
            name: fd.get('name').trim(),
            scopes,
            expires_at: expIso ? new Date(expIso).toISOString() : null,
            rate_limit_per_min: Number(fd.get('rate_limit_per_min')) || 60,
        };
        const out = mount.querySelector('#tok-new');
        if (!out) return;
        out.innerHTML = '<p data-i18n="view.api_tokens.hint.generating" class="muted small">generating…</p>';
        try {
            const r = await api.createApiToken(body);
            if (!viewIsCurrent(tok)) return;
            const out2 = mount.querySelector('#tok-new');
            if (out2) out2.innerHTML = `
                <div class="chart-panel" style="background:#0d0d22;border-left:3px solid #ff7a1f;">
                    <p><strong data-i18n="view.api_tokens.warn.save_now">Save this token now — it will never be shown again:</strong></p>
                    <pre style="background:#070714;padding:8px;font-size:13px;overflow:auto;">${esc(r.token)}</pre>
                    <p class="muted small">${esc(t('view.api_tokens.stored_as', { prefix: r.summary.prefix, scopes: r.summary.scopes.join(', '), created: new Date(r.summary.created_at).toLocaleString() }))}</p>
                </div>
            `;
            e.target.reset();
            await loadList(mount, tok);
        } catch (err) {
            if (!viewIsCurrent(tok)) return;
            const out2 = mount.querySelector('#tok-new');
            if (out2) out2.innerHTML = `<p class="boot">${esc(err.message)}</p>`;
        }
    });

    await loadList(mount, tok);
}

async function loadList(mount, tok) {
    const el = mount.querySelector('#tok-list');
    if (!el) return;
    try {
        const rows = await api.listApiTokens();
        if (!viewIsCurrent(tok)) return;
        const el2 = mount.querySelector('#tok-list');
        if (!el2) return;
        if (!rows.length) {
            el2.innerHTML = '<p data-i18n="view.api_tokens.hint.no_tokens_yet" class="muted small">No tokens yet.</p>';
            return;
        }
        el2.innerHTML = `
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.api_tokens.th.name">Name</th><th data-i18n="view.api_tokens.th.prefix">Prefix</th><th data-i18n="view.api_tokens.th.scopes">Scopes</th><th data-i18n="view.api_tokens.th.rate_min">Rate/min</th>
                    <th data-i18n="view.api_tokens.th.created">Created</th><th data-i18n="view.api_tokens.th.last_used">Last used</th><th data-i18n="view.api_tokens.th.uses">Uses</th><th data-i18n="view.api_tokens.th.expires">Expires</th>
                    <th data-i18n="view.api_tokens.th.status">Status</th><th></th>
                </tr></thead>
                <tbody>
                    ${rows.map(tk => `<tr>
                        <td>${esc(tk.name)}</td>
                        <td><code>${esc(tk.prefix)}</code></td>
                        <td class="small">${tk.scopes.join(', ')}</td>
                        <td class="small">
                            ${tk.revoked_at ? tk.rate_limit_per_min :
                              `<input type="number" min="1" max="10000" value="${tk.rate_limit_per_min}"
                                      class="rate-input" data-id="${tk.id}" style="width:70px;">`}
                        </td>
                        <td class="small">${new Date(tk.created_at).toLocaleDateString()}</td>
                        <td class="small">${tk.last_used_at ? new Date(tk.last_used_at).toLocaleString() : '—'}</td>
                        <td>${tk.use_count}</td>
                        <td class="small">${tk.expires_at ? new Date(tk.expires_at).toLocaleDateString() : t('common.status.never')}</td>
                        <td class="small ${tk.revoked_at ? 'neg' : 'pos'}">${tk.revoked_at ? t('common.status.revoked') : t('common.status.active')}</td>
                        <td>${tk.revoked_at
                            ? ''
                            : `<button data-i18n="view.api_tokens.btn.revoke" class="btn revoke-btn" data-id="${tk.id}">Revoke</button>`}</td>
                    </tr>`).join('')}
                </tbody>
            </table>
        `;
        el2.querySelectorAll('.revoke-btn').forEach(b => {
            b.addEventListener('click', async () => {
                if (!await tConfirm('view.api_tokens.confirm.revoke', {}, { level: 'danger' })) return;
                try { await api.revokeApiToken(b.dataset.id); if (viewIsCurrent(tok)) await loadList(mount, tok); }
                catch (e) { showToast(t('common.error', { err: e.message }), { level: 'error' }); }
            });
        });
        el2.querySelectorAll('.rate-input').forEach(input => {
            input.addEventListener('change', async () => {
                const v = Number(input.value);
                if (!Number.isFinite(v) || v < 1 || v > 10000) {
                    showToast(t('view.api_tokens.alert.rate_range'), { level: 'warning' }); return;
                }
                try { await api.setApiTokenRateLimit(input.dataset.id, v); }
                catch (e) { showToast(t('common.error', { err: e.message }), { level: 'error' }); if (viewIsCurrent(tok)) await loadList(mount, tok); }
            });
        });
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const el2 = mount.querySelector('#tok-list');
        if (el2) el2.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}
