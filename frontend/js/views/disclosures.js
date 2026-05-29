// Insider Form 4 + Senate / House STOCK Act stream + watcher rules.
import { api } from '../api.js';
import { playSound } from '../alert_engine.js';
import { esc, fmt, fmtDateTime } from '../util.js';
import { on as onWsEvent } from '../ws.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

let wsUnsub = null;

const KINDS = [
    { id: 'insider_form4', label: 'Insider (Form 4)' },
    { id: 'senate_stock',  label: 'Senate' },
    { id: 'house_stock',   label: 'House' },
];

let pollTimer = null;
let lastSeen = '';

export async function renderDisclosures(mount) {
    const tok = currentViewToken();
    const [filings, watchers] = await Promise.all([
        api.disclosures(null, null, 100),
        api.disclosureWatchers(),
    ]);
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 data-i18n="view.disclosures.h1.disclosures_insider_congress" class="view-title">// DISCLOSURES — INSIDER + CONGRESS</h1>
        <p data-i18n="view.disclosures.hint.polls_sec_edgar_form_4_senate_efd_house_clerk_ever" class="muted small">
            Polls SEC EDGAR Form 4 + Senate eFD + House Clerk every 20 seconds.
            Sub-30s alerts vs Quiver Quant's ~5min. Configure watchers to fire
            audio + browser-push when your rule matches.
        </p>

        <div class="chart-panel">
            <div class="inline-form">
                <button data-i18n="view.disclosures.btn.poll_now" class="primary" id="poll-now">Poll now</button>
                <span class="muted" id="poll-status"></span>
            </div>
        </div>

        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.disclosures.h2.new_watcher">New watcher</h2>
                <form id="w-form" class="inline-form">
                    <input name="name" placeholder="name" data-i18n-placeholder="common.placeholder.name" required>
                    <input name="symbols" placeholder="symbols (CSV, blank = any)" data-i18n-placeholder="view.disclosures.placeholder.symbols">
                    <input name="filers" placeholder="filer names (CSV, blank = any)" data-i18n-placeholder="view.disclosures.placeholder.filers">
                    <input name="min_amount_usd" type="number" placeholder="min $ amount" data-i18n-placeholder="view.disclosures.placeholder.min_amount">
                    <button data-i18n="view.disclosures.btn.create" class="primary" type="submit">Create</button>
                </form>
                ${watchers.length ? `<table class="trades" style="margin-top:10px">
                    <thead><tr><th data-i18n="view.disclosures.th.name">Name</th><th data-i18n="view.disclosures.th.symbols">Symbols</th><th data-i18n="view.disclosures.th.filers">Filers</th>
                        <th data-i18n="view.disclosures.th.min">Min $</th><th data-i18n="view.disclosures.th.on">On</th><th></th></tr></thead>
                    <tbody>${watchers.map(w => `
                        <tr><td>${esc(w.name)}</td>
                        <td>${esc((w.symbols || []).join(', '))}</td>
                        <td>${esc((w.filers || []).join(', '))}</td>
                        <td>${w.min_amount_usd != null ? '$' + fmt(w.min_amount_usd) : '—'}</td>
                        <td>${w.enabled ? '✓' : '—'}</td>
                        <td><button data-i18n="view.disclosures.btn.delete" class="link" data-del-w="${w.id}">delete</button></td></tr>
                    `).join('')}</tbody></table>` : '<p data-i18n="view.disclosures.hint.no_watchers_yet" class="muted small" style="margin-top:8px">No watchers yet.</p>'}
            </div>

            <div class="chart-panel" style="grid-column: 1 / -1">
                <h2 data-i18n="view.disclosures.h2.live_feed_auto_refreshes_every_20s">Live feed · auto-refreshes every 20s</h2>
                <div id="feed">${renderFeed(filings)}</div>
            </div>
        </div>
    `;

    mount.querySelector('#poll-now').addEventListener('click', async () => {
        const status = mount.querySelector('#poll-status');
        if (status) status.textContent = t('common.status.polling');
        try {
            const r = await api.disclosuresPollNow();
            if (!viewIsCurrent(tok)) return;
            const status2 = mount.querySelector('#poll-status');
            if (status2) status2.textContent =
                `${r.edgar_inserted} EDGAR / ${r.senate_inserted} Senate / ${r.house_inserted} House new`;
            await refreshFeed(watchers, mount, tok);
        } catch (e) {
            if (!viewIsCurrent(tok)) return;
            const status2 = mount.querySelector('#poll-status');
            if (status2) status2.textContent = t('common.error', { err: e.message });
        }
    });

    mount.querySelector('#w-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const symbols = (fd.get('symbols') || '').split(',').map(s => s.trim().toUpperCase()).filter(Boolean);
        const filers = (fd.get('filers') || '').split(',').map(s => s.trim()).filter(Boolean);
        await api.createDisclosureWatcher({
            name: fd.get('name'),
            symbols: symbols.length ? symbols : null,
            filers: filers.length ? filers : null,
            min_amount_usd: fd.get('min_amount_usd') ? Number(fd.get('min_amount_usd')) : null,
        });
        if (!viewIsCurrent(tok)) return;
        renderDisclosures(mount);
    });
    mount.querySelectorAll('[data-del-w]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteDisclosureWatcher(b.dataset.delW);
            if (!viewIsCurrent(tok)) return;
            renderDisclosures(mount);
        }));

    // Poll loop — kept as a 60s safety net; WS push handles the real-time path.
    if (pollTimer) clearInterval(pollTimer);
    lastSeen = filings[0]?.id || '';
    pollTimer = setInterval(() => {
        if (!viewIsCurrent(tok)) { clearInterval(pollTimer); pollTimer = null; return; }
        refreshFeed(watchers, mount, tok);
    }, 60_000);

    // Live push: refresh immediately on any disclosure event from the server.
    if (wsUnsub) wsUnsub();
    wsUnsub = onWsEvent('disclosure', () => {
        if (!viewIsCurrent(tok)) { if (wsUnsub) { wsUnsub(); wsUnsub = null; } return; }
        refreshFeed(watchers, mount, tok);
    });

    window.addEventListener('hashchange', () => {
        if (!window.location.hash.startsWith('#disclosures')) {
            clearInterval(pollTimer); pollTimer = null;
            if (wsUnsub) { wsUnsub(); wsUnsub = null; }
        }
    }, { once: true });
}

async function refreshFeed(watchers, mount, tok) {
    try {
        const all = await api.disclosures(null, null, 100);
        if (tok != null && !viewIsCurrent(tok)) return;
        const top = all[0]?.id || '';
        // Fire watchers on any new rows we haven't seen.
        const fresh = [];
        for (const d of all) {
            if (d.id === lastSeen) break;
            fresh.push(d);
        }
        if (fresh.length && lastSeen) {
            for (const d of fresh) {
                for (const w of watchers) {
                    if (!w.enabled) continue;
                    if (w.kinds.length && !w.kinds.includes(d.kind)) continue;
                    if (w.symbols && d.symbol && !w.symbols.includes(d.symbol)) continue;
                    if (w.filers && !w.filers.some(f =>
                        d.filer_name.toLowerCase().includes(f.toLowerCase()))) continue;
                    if (w.min_amount_usd && (!d.amount_usd || Number(d.amount_usd) < Number(w.min_amount_usd))) continue;
                    playSound(w.sound || 'bell');
                    if ('Notification' in window && Notification.permission === 'granted') {
                        new Notification(`TraderView · ${kindLabel(d.kind)}`, {
                            body: `${d.filer_name}${d.symbol ? ' · ' + d.symbol : ''}`,
                        });
                    }
                    break;
                }
            }
        }
        lastSeen = top;
        const feedEl = mount ? mount.querySelector('#feed') : document.getElementById('feed');
        if (feedEl) feedEl.innerHTML = renderFeed(all);
    } catch (_) {}
}

function renderFeed(items) {
    if (!items.length) return '<p data-i18n="view.disclosures.hint.no_disclosures_cached_yet_hit_poll_now_to_fetch" class="muted">No disclosures cached yet — hit "Poll now" to fetch.</p>';
    return `<table class="trades">
        <thead><tr><th data-i18n="view.disclosures.th.filed">Filed</th><th data-i18n="view.disclosures.th.kind">Kind</th><th data-i18n="view.disclosures.th.filer">Filer</th><th data-i18n="view.disclosures.th.symbol">Symbol</th>
            <th data-i18n="view.disclosures.th.tx">Tx</th><th data-i18n="view.disclosures.th.shares">Shares</th><th data-i18n="view.disclosures.th.amount">Amount</th><th data-i18n="view.disclosures.th.source">Source</th></tr></thead>
        <tbody>${items.map(d => `
            <tr>
                <td>${fmtDateTime(d.filed_at)}</td>
                <td><span class="tape-sym">${esc(kindLabel(d.kind))}</span></td>
                <td>${esc(d.filer_name)}${d.filer_role ? ` <span class="muted small">(${esc(d.filer_role)})</span>` : ''}</td>
                <td>${d.symbol ? `<a href="#research/${encodeURIComponent(d.symbol)}">${esc(d.symbol)}</a>` : '—'}</td>
                <td>${esc(d.txn_type || '')}</td>
                <td>${d.shares != null ? Number(d.shares).toLocaleString() : '—'}</td>
                <td>${d.amount_usd != null ? '$' + fmt(d.amount_usd) : (d.amount_range || '—')}</td>
                <td>${d.source_url ? `<a href="${esc(d.source_url)}" target="_blank" rel="noopener noreferrer">view</a>` : '—'}</td>
            </tr>`).join('')}</tbody></table>`;
}

function kindLabel(k) {
    const fallback = KINDS.find(x => x.id === k)?.label || k;
    const key = `view.disclosures.kind.${k}.label`;
    const v = t(key);
    return (v && v !== key) ? v : fallback;
}
