// Outbound webhooks: Discord, Slack, generic HTTP.
import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderWebhooks(mount) {
    const tok = currentViewToken();
    const rows = await api.webhooks();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 data-i18n="view.webhooks.h1.webhooks" class="view-title">// WEBHOOKS</h1>
        <p class="muted small">Fan-out alerts to Discord, Slack, or any generic HTTP endpoint.
            Wire a webhook here, then reference its ID from an alert rule's <code>webhook_ids[]</code>
            field — alert fires call POST automatically.</p>

        <div class="chart-panel">
            <h2 data-i18n="view.webhooks.h2.add_webhook">Add webhook</h2>
            <form id="wf" class="inline-form">
                <input name="name" placeholder="name" required>
                <select name="kind">
                    <option data-i18n="view.webhooks.opt.discord" value="discord">Discord</option>
                    <option data-i18n="view.webhooks.opt.slack" value="slack">Slack</option>
                    <option data-i18n="view.webhooks.opt.generic_raw_json" value="generic">Generic (raw JSON)</option>
                </select>
                <input name="url" placeholder="webhook URL" required style="min-width:340px">
                <input name="secret" placeholder="X-Webhook-Secret (optional, generic only)" style="min-width:240px">
                <button data-i18n="view.webhooks.btn.create" class="primary" type="submit">Create</button>
            </form>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.webhooks.h2.current_webhooks">Current webhooks</h2>
            ${rows.length ? `<table class="trades">
                <thead><tr><th data-i18n="view.webhooks.th.name">Name</th><th data-i18n="view.webhooks.th.kind">Kind</th><th data-i18n="view.webhooks.th.url">URL</th><th data-i18n="view.webhooks.th.on">On</th>
                    <th data-i18n="view.webhooks.th.fires">Fires</th><th data-i18n="view.webhooks.th.last_status">Last status</th><th data-i18n="view.webhooks.th.last_fired">Last fired</th><th></th></tr></thead>
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
                            <button data-i18n="view.webhooks.btn.test" class="link" data-test="${w.id}">test</button>
                            <button class="link" data-tog="${w.id}" data-en="${w.enabled}">${w.enabled ? 'disable' : 'enable'}</button>
                            <button data-i18n="view.webhooks.btn.delete" class="link" data-del="${w.id}">delete</button>
                        </td>
                    </tr>`).join('')}</tbody></table>` : '<p data-i18n="view.webhooks.hint.no_webhooks_yet" class="muted">No webhooks yet.</p>'}
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.webhooks.h2.provider_payloads">Provider payloads</h2>
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
    mount.querySelector('#wf').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        await api.createWebhook({
            name: fd.get('name'),
            kind: fd.get('kind'),
            url:  fd.get('url'),
            secret: fd.get('secret') || null,
        });
        if (!viewIsCurrent(tok)) return;
        renderWebhooks(mount);
    });
    mount.querySelectorAll('[data-test]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.testWebhook(b.dataset.test);
            if (!viewIsCurrent(tok)) return;
            alert('Test fired — check your Discord/Slack/endpoint.');
            renderWebhooks(mount);
        }));
    mount.querySelectorAll('[data-tog]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.toggleWebhook(b.dataset.tog, b.dataset.en !== 'true');
            if (!viewIsCurrent(tok)) return;
            renderWebhooks(mount);
        }));
    mount.querySelectorAll('[data-del]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteWebhook(b.dataset.del);
            if (!viewIsCurrent(tok)) return;
            renderWebhooks(mount);
        }));
}

function redact(url) {
    // Hide the secret-y part of Discord/Slack webhook URLs.
    return url.replace(/(\/(?:webhooks|services)\/[^/]+\/)[^/]+/, '$1***');
}
