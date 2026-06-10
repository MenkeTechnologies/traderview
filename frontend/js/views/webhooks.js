// Outbound webhooks: Discord, Slack, generic HTTP.
import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderWebhooks(mount) {
    const tok = currentViewToken();
    const rows = await api.webhooks();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 data-i18n="view.webhooks.h1.webhooks" class="view-title">// WEBHOOKS</h1>
        <p class="muted small" data-i18n="view.webhooks.hint.intro">Fan-out alerts to Discord, Slack, or any generic HTTP endpoint. Wire a webhook here, then reference its ID from an alert rule's webhook_ids[] field — alert fires call POST automatically.</p>

        <div class="chart-panel">
            <h2 data-i18n="view.webhooks.h2.add_webhook">Add webhook</h2>
            <form id="wf" class="inline-form">
                <input name="name" placeholder="name" data-i18n-placeholder="common.placeholder.name" required>
                <select name="kind">
                    <option data-i18n="view.webhooks.opt.discord" value="discord">Discord</option>
                    <option data-i18n="view.webhooks.opt.slack" value="slack">Slack</option>
                    <option data-i18n="view.webhooks.opt.generic_raw_json" value="generic">Generic (raw JSON)</option>
                </select>
                <input name="url" placeholder="webhook URL" data-i18n-placeholder="view.webhooks.placeholder.url" required style="min-width:340px">
                <input name="secret" placeholder="X-Webhook-Secret (optional, generic only)" data-i18n-placeholder="view.webhooks.placeholder.secret" style="min-width:240px">
                <button data-i18n="view.webhooks.btn.create" class="primary" type="submit">Create</button>
            </form>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.webhooks.h2.current_webhooks">Current webhooks</h2>
            ${rows.length ? `<table class="trades">
                <thead><tr><th data-i18n="view.webhooks.th.name">Name</th><th data-i18n="view.webhooks.th.kind">Kind</th><th data-i18n="view.webhooks.th.url">URL</th><th data-i18n="view.webhooks.th.on">On</th>
                    <th data-i18n="view.webhooks.th.fires">Fires</th><th data-i18n="view.webhooks.th.last_status">Last status</th><th data-i18n="view.webhooks.th.last_fired">Last fired</th><th></th></tr></thead>
                <tbody>${rows.map(w => `
                    <tr data-context-scope="webhook-row" data-id="${esc(w.id)}" data-enabled="${w.enabled ? 'true' : 'false'}">
                        <td>${esc(w.name)}</td>
                        <td><span class="tape-sym">${esc(w.kind)}</span></td>
                        <td class="muted small">${esc(redact(w.url))}</td>
                        <td>${w.enabled ? '✓' : '—'}</td>
                        <td>${w.fire_count}</td>
                        <td class="muted small">${esc(w.last_status || '')}</td>
                        <td class="muted small">${w.last_fired_at ? fmtDateTime(w.last_fired_at) : '—'}</td>
                        <td>
                            <button data-i18n="view.webhooks.btn.test" class="link" data-test="${w.id}">test</button>
                            <button class="link" data-tog="${w.id}" data-en="${w.enabled}">${w.enabled ? t('common.btn.disable_lc') : t('common.btn.enable_lc')}</button>
                            <button data-i18n="view.webhooks.btn.delete" class="link" data-del="${w.id}">delete</button>
                        </td>
                    </tr>`).join('')}</tbody></table>` : '<p data-i18n="view.webhooks.hint.no_webhooks_yet" class="muted">No webhooks yet.</p>'}
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.webhooks.h2.fires_chart">Fires per webhook</h2>
            <div id="wh-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.webhooks.h2.kind_chart">Webhook count by provider</h2>
            <div id="wh-kind-chart" style="width:100%;height:200px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.webhooks.h2.status_chart">Last status distribution (success / 4xx / 5xx / pending)</h2>
            <div id="wh-status-chart" style="width:100%;height:200px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.webhooks.h2.provider_payloads">Provider payloads</h2>
            <details>
                <summary data-i18n="view.webhooks.summary.discord">Discord embed</summary>
                <pre class="result">{ "username": "TraderView", "embeds": [{ "title": "...", "description": "...", "color": 0x00e5ff, "fields": [...] }] }</pre>
            </details>
            <details>
                <summary data-i18n="view.webhooks.summary.slack">Slack blocks</summary>
                <pre class="result">{ "text": "header\\nbody", "blocks": [ {"type":"header", ...}, {"type":"section", ...}, {"type":"context", ...} ] }</pre>
            </details>
            <details>
                <summary data-i18n="view.webhooks.summary.generic">Generic (raw AlertPayload JSON)</summary>
                <pre class="result">{ "title": "...", "message": "...", "symbol": "AAPL", "kind": "price_alert", "url": "...", "fired_at": "2026-..." }</pre>
            </details>
        </div>
    `;
    renderFiresChart(rows);
    renderKindChart(rows);
    renderStatusChart(rows);
    mount.querySelector('#wf').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const name = fd.get('name');
        try {
            await api.createWebhook({
                name,
                kind: fd.get('kind'),
                url:  fd.get('url'),
                secret: fd.get('secret') || null,
            });
            if (!viewIsCurrent(tok)) return;
            showToast(t('view.webhooks.toast.created', { name }), { level: 'success' });
            renderWebhooks(mount);
        } catch (err) {
            showToast(t('toast.error.api', { err: err.message || err }), { level: 'error' });
        }
    });
    mount.querySelectorAll('[data-test]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.testWebhook(b.dataset.test);
            if (!viewIsCurrent(tok)) return;
            showToast(t('view.webhooks.alert.test_fired'), { level: 'success' });
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
            try {
                await api.deleteWebhook(b.dataset.del);
                if (!viewIsCurrent(tok)) return;
                showToast(t('view.webhooks.toast.deleted'), { level: 'success' });
                renderWebhooks(mount);
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message || err }), { level: 'error' });
            }
        }));
}

function renderStatusChart(rows) {
    const el = document.getElementById('wh-status-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    let ok = 0, clientErr = 0, serverErr = 0, pending = 0;
    for (const w of rows || []) {
        const code = parseInt(String(w.last_status || '').replace(/[^0-9]/g, ''), 10);
        if (!Number.isFinite(code)) pending++;
        else if (code >= 200 && code < 300) ok++;
        else if (code >= 400 && code < 500) clientErr++;
        else if (code >= 500 && code < 600) serverErr++;
        else pending++;
    }
    if (ok + clientErr + serverErr + pending < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.webhooks.empty_status_chart">${esc(t('view.webhooks.empty_status_chart'))}</div>`;
        return;
    }
    const labels = [
        t('view.webhooks.chart.ok'),
        t('view.webhooks.chart.client_err'),
        t('view.webhooks.chart.server_err'),
        t('view.webhooks.chart.pending'),
    ];
    const xs = labels.map((_, i) => i + 1);
    const ok2 = [ok,  null, null, null];
    const ce  = [null, clientErr, null, null];
    const se  = [null, null, serverErr, null];
    const pe  = [null, null, null, pending];
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.webhooks.chart.bucket') },
            { label: t('view.webhooks.chart.ok'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 18, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.webhooks.chart.client_err'),
              stroke: '#ffd84a', width: 0,
              points: { show: true, size: 18, fill: '#ffd84a', stroke: '#ffd84a' } },
            { label: t('view.webhooks.chart.server_err'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 18, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.webhooks.chart.pending'),
              stroke: '#aab',    width: 0,
              points: { show: true, size: 18, fill: '#aab',    stroke: '#aab'    } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ok2, ce, se, pe], el);
}

function renderKindChart(rows) {
    const el = document.getElementById('wh-kind-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const counts = new Map();
    for (const w of rows || []) {
        const k = w.kind || 'unknown';
        counts.set(k, (counts.get(k) || 0) + 1);
    }
    const entries = [...counts.entries()].sort((a, b) => b[1] - a[1]);
    if (entries.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.webhooks.empty_kind_chart">${esc(t('view.webhooks.empty_kind_chart'))}</div>`;
        return;
    }
    const labels = entries.map(e => e[0]);
    const xs = labels.map((_, i) => i + 1);
    const ys = entries.map(e => e[1]);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.webhooks.chart.kind') },
            { label: t('view.webhooks.chart.kind_count'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 16, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderFiresChart(rows) {
    const el = document.getElementById('wh-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const valid = (rows || []).filter(w => Number.isFinite(Number(w.fire_count)));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.webhooks.empty_chart">${esc(t('view.webhooks.empty_chart'))}</div>`;
        return;
    }
    valid.sort((a, b) => Number(b.fire_count) - Number(a.fire_count));
    const labels = valid.map(w => w.name || w.id);
    const xs = labels.map((_, i) => i + 1);
    const onY  = valid.map(w => w.enabled  ? Number(w.fire_count) : null);
    const offY = valid.map(w => !w.enabled ? Number(w.fire_count) : null);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.webhooks.chart.webhook') },
            { label: t('view.webhooks.chart.enabled'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.webhooks.chart.disabled'),
              stroke: '#aab',    width: 0,
              points: { show: true, size: 12, fill: '#aab',    stroke: '#aab'    } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, onY, offY], el);
}

function redact(url) {
    // Hide the secret-y part of Discord/Slack webhook URLs.
    return url.replace(/(\/(?:webhooks|services)\/[^/]+\/)[^/]+/, '$1***');
}
