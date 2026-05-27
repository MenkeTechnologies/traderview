// Developer tab — personal access token management.
// New tokens are shown ONCE at creation time, then never again.

import { api } from '../api.js';
import { esc } from '../util.js';

export async function renderDeveloper(mount) {
    mount.innerHTML = `
        <h1 class="view-title">// DEVELOPER — PUBLIC API</h1>
        <p class="muted small">Personal Access Tokens authenticate third-party integrations
            against the same endpoints the UI uses. Pass them as
            <code>Authorization: Bearer pat_&lt;prefix&gt;_&lt;secret&gt;</code>. Tokens are
            argon2-hashed at rest — the secret is shown <strong>once</strong> at creation
            time and never recoverable afterwards. Revoke a token to cut access immediately.</p>

        <div class="chart-panel">
            <h2>Create token</h2>
            <form id="tok-form" class="inline-form">
                <label>Name <input name="name" placeholder="n8n staging" required style="min-width:220px;"></label>
                <label>Scopes
                    <select name="scopes" multiple size="3" style="min-width:120px;">
                        <option value="read" selected>read</option>
                        <option value="write">write</option>
                        <option value="admin">admin</option>
                    </select>
                </label>
                <label>Expires (optional)
                    <input name="expires_at" type="date" style="width:160px;">
                </label>
                <label>Rate limit (req/min)
                    <input name="rate_limit_per_min" type="number" min="1" max="10000"
                           value="60" style="width:90px;">
                </label>
                <button class="primary" type="submit">Generate</button>
            </form>
            <div id="tok-new"></div>
        </div>

        <div class="chart-panel">
            <h2>Active tokens</h2>
            <div id="tok-list"><div class="boot">loading…</div></div>
        </div>

        <div class="chart-panel">
            <h2>Example use (curl)</h2>
            <pre style="background:#0d0d22;padding:12px;overflow:auto;font-size:11px;">curl -H "Authorization: Bearer pat_xxx_yyy" \\
     ${esc(window.location.origin)}/api/trades?status=closed&amp;limit=50</pre>
        </div>
    `;

    document.getElementById('tok-form').addEventListener('submit', async (e) => {
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
        const out = document.getElementById('tok-new');
        out.innerHTML = '<p class="muted small">generating…</p>';
        try {
            const r = await api.createApiToken(body);
            out.innerHTML = `
                <div class="chart-panel" style="background:#0d0d22;border-left:3px solid #ff7a1f;">
                    <p><strong>Save this token now — it will never be shown again:</strong></p>
                    <pre style="background:#070714;padding:8px;font-size:13px;overflow:auto;">${esc(r.token)}</pre>
                    <p class="muted small">Stored as: ${esc(r.summary.prefix)} (prefix) · ${r.summary.scopes.join(', ')} · created ${new Date(r.summary.created_at).toLocaleString()}</p>
                </div>
            `;
            e.target.reset();
            await loadList();
        } catch (err) {
            out.innerHTML = `<p class="boot">${esc(err.message)}</p>`;
        }
    });

    await loadList();
}

async function loadList() {
    const el = document.getElementById('tok-list');
    try {
        const rows = await api.listApiTokens();
        if (!rows.length) {
            el.innerHTML = '<p class="muted small">No tokens yet.</p>';
            return;
        }
        el.innerHTML = `
            <table class="trades">
                <thead><tr>
                    <th>Name</th><th>Prefix</th><th>Scopes</th><th>Rate/min</th>
                    <th>Created</th><th>Last used</th><th>Uses</th><th>Expires</th>
                    <th>Status</th><th></th>
                </tr></thead>
                <tbody>
                    ${rows.map(t => `<tr>
                        <td>${esc(t.name)}</td>
                        <td><code>${esc(t.prefix)}</code></td>
                        <td class="small">${t.scopes.join(', ')}</td>
                        <td class="small">
                            ${t.revoked_at ? t.rate_limit_per_min :
                              `<input type="number" min="1" max="10000" value="${t.rate_limit_per_min}"
                                      class="rate-input" data-id="${t.id}" style="width:70px;">`}
                        </td>
                        <td class="small">${new Date(t.created_at).toLocaleDateString()}</td>
                        <td class="small">${t.last_used_at ? new Date(t.last_used_at).toLocaleString() : '—'}</td>
                        <td>${t.use_count}</td>
                        <td class="small">${t.expires_at ? new Date(t.expires_at).toLocaleDateString() : 'never'}</td>
                        <td class="small ${t.revoked_at ? 'neg' : 'pos'}">${t.revoked_at ? 'revoked' : 'active'}</td>
                        <td>${t.revoked_at
                            ? ''
                            : `<button class="btn revoke-btn" data-id="${t.id}">Revoke</button>`}</td>
                    </tr>`).join('')}
                </tbody>
            </table>
        `;
        el.querySelectorAll('.revoke-btn').forEach(b => {
            b.addEventListener('click', async () => {
                if (!confirm('Revoke this token? Integrations using it will lose access immediately.')) return;
                try { await api.revokeApiToken(b.dataset.id); await loadList(); }
                catch (e) { alert(e.message); }
            });
        });
        el.querySelectorAll('.rate-input').forEach(input => {
            input.addEventListener('change', async () => {
                const v = Number(input.value);
                if (!Number.isFinite(v) || v < 1 || v > 10000) {
                    alert('rate must be 1..=10000'); return;
                }
                try { await api.setApiTokenRateLimit(input.dataset.id, v); }
                catch (e) { alert(e.message); await loadList(); }
            });
        });
    } catch (e) {
        el.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}
