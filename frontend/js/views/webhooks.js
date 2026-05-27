// Outbound webhooks: Discord, Slack, generic HTTP.
import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';

export async function renderWebhooks(mount) {
    const rows = await api.webhooks();
    mount.innerHTML = `
        <h1 class="view-title">// WEBHOOKS</h1>
        <p class="muted small">Fan-out alerts to Discord, Slack, or any generic HTTP endpoint.
            Wire a webhook here, then reference its ID from an alert rule's <code>webhook_ids[]</code>
            field — alert fires call POST automatically.</p>

        <div class="chart-panel">
            <h2>Add webhook</h2>
            <form id="wf" class="inline-form">
                <input name="name" placeholder="name" required>
                <select name="kind">
                    <option value="discord">Discord</option>
                    <option value="slack">Slack</option>
                    <option value="generic">Generic (raw JSON)</option>
                </select>
                <input name="url" placeholder="webhook URL" required style="min-width:340px">
                <input name="secret" placeholder="X-Webhook-Secret (optional, generic only)" style="min-width:240px">
                <button class="primary" type="submit">Create</button>
            </form>
        </div>

        <div class="chart-panel">
            <h2>Current webhooks</h2>
            ${rows.length ? `<table class="trades">
                <thead><tr><th>Name</th><th>Kind</th><th>URL</th><th>On</th>
                    <th>Fires</th><th>Last status</th><th>Last fired</th><th></th></tr></thead>
                <tbody>${rows.map(w => `
                    <tr>
                        <td>${esc(w.name)}</td>
                        <td><span class="tape-sym">${esc(w.kind)}</span></td>
                        <td class="muted small">${esc(redact(w.url))}</td>
                        <td>${w.enabled ? '✓' : '—'}</td>
                        <td>${w.fire_count}</td>
                        <td class="muted small">${esc(w.last_status || '')}</td>
                        <td class="muted small">${w.last_fired_at ? fmtDateTime(w.last_fired_at) : '—'}</td>
                        <td>
                            <button class="link" data-test="${w.id}">test</button>
                            <button class="link" data-tog="${w.id}" data-en="${w.enabled}">${w.enabled ? 'disable' : 'enable'}</button>
                            <button class="link" data-del="${w.id}">delete</button>
                        </td>
                    </tr>`).join('')}</tbody></table>` : '<p class="muted">No webhooks yet.</p>'}
        </div>

        <div class="chart-panel">
            <h2>Provider payloads</h2>
            <details>
                <summary>Discord embed</summary>
                <pre class="result">{ "username": "TraderView", "embeds": [{ "title": "...", "description": "...", "color": 0x00e5ff, "fields": [...] }] }</pre>
            </details>
            <details>
                <summary>Slack blocks</summary>
                <pre class="result">{ "text": "header\\nbody", "blocks": [ {"type":"header", ...}, {"type":"section", ...}, {"type":"context", ...} ] }</pre>
            </details>
            <details>
                <summary>Generic (raw AlertPayload JSON)</summary>
                <pre class="result">{ "title": "...", "message": "...", "symbol": "AAPL", "kind": "price_alert", "url": "...", "fired_at": "2026-..." }</pre>
            </details>
        </div>
    `;
    document.getElementById('wf').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        await api.createWebhook({
            name: fd.get('name'),
            kind: fd.get('kind'),
            url:  fd.get('url'),
            secret: fd.get('secret') || null,
        });
        renderWebhooks(mount);
    });
    document.querySelectorAll('[data-test]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.testWebhook(b.dataset.test);
            alert('Test fired — check your Discord/Slack/endpoint.');
            renderWebhooks(mount);
        }));
    document.querySelectorAll('[data-tog]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.toggleWebhook(b.dataset.tog, b.dataset.en !== 'true');
            renderWebhooks(mount);
        }));
    document.querySelectorAll('[data-del]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteWebhook(b.dataset.del);
            renderWebhooks(mount);
        }));
}

function redact(url) {
    // Hide the secret-y part of Discord/Slack webhook URLs.
    return url.replace(/(\/(?:webhooks|services)\/[^/]+\/)[^/]+/, '$1***');
}
