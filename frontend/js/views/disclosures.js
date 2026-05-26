// Insider Form 4 + Senate / House STOCK Act stream + watcher rules.
import { api } from '../api.js';
import { playSound } from '../alert_engine.js';
import { esc, fmt, fmtDateTime } from '../util.js';

const KINDS = [
    { id: 'insider_form4', label: 'Insider (Form 4)' },
    { id: 'senate_stock',  label: 'Senate' },
    { id: 'house_stock',   label: 'House' },
];

let pollTimer = null;
let lastSeen = '';

export async function renderDisclosures(mount) {
    const [filings, watchers] = await Promise.all([
        api.disclosures(null, null, 100),
        api.disclosureWatchers(),
    ]);
    mount.innerHTML = `
        <h1 class="view-title">// DISCLOSURES — INSIDER + CONGRESS</h1>
        <p class="muted small">
            Polls SEC EDGAR Form 4 + Senate eFD + House Clerk every 20 seconds.
            Sub-30s alerts vs Quiver Quant's ~5min. Configure watchers to fire
            audio + browser-push when your rule matches.
        </p>

        <div class="chart-panel">
            <div class="inline-form">
                <button class="primary" id="poll-now">Poll now</button>
                <span class="muted" id="poll-status"></span>
            </div>
        </div>

        <div class="panel-grid">
            <div class="chart-panel">
                <h2>New watcher</h2>
                <form id="w-form" class="inline-form">
                    <input name="name" placeholder="name" required>
                    <input name="symbols" placeholder="symbols (CSV, blank = any)">
                    <input name="filers" placeholder="filer names (CSV, blank = any)">
                    <input name="min_amount_usd" type="number" placeholder="min $ amount">
                    <button class="primary" type="submit">Create</button>
                </form>
                ${watchers.length ? `<table class="trades" style="margin-top:10px">
                    <thead><tr><th>Name</th><th>Symbols</th><th>Filers</th>
                        <th>Min $</th><th>On</th><th></th></tr></thead>
                    <tbody>${watchers.map(w => `
                        <tr><td>${esc(w.name)}</td>
                        <td>${esc((w.symbols || []).join(', '))}</td>
                        <td>${esc((w.filers || []).join(', '))}</td>
                        <td>${w.min_amount_usd != null ? '$' + fmt(w.min_amount_usd) : '—'}</td>
                        <td>${w.enabled ? '✓' : '—'}</td>
                        <td><button class="link" data-del-w="${w.id}">delete</button></td></tr>
                    `).join('')}</tbody></table>` : '<p class="muted small" style="margin-top:8px">No watchers yet.</p>'}
            </div>

            <div class="chart-panel" style="grid-column: 1 / -1">
                <h2>Live feed · auto-refreshes every 20s</h2>
                <div id="feed">${renderFeed(filings)}</div>
            </div>
        </div>
    `;

    document.getElementById('poll-now').addEventListener('click', async () => {
        const status = document.getElementById('poll-status');
        status.textContent = 'polling…';
        try {
            const r = await api.disclosuresPollNow();
            status.textContent =
                `${r.edgar_inserted} EDGAR / ${r.senate_inserted} Senate / ${r.house_inserted} House new`;
            await refreshFeed(watchers);
        } catch (e) {
            status.textContent = 'error: ' + e.message;
        }
    });

    document.getElementById('w-form').addEventListener('submit', async (e) => {
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
        renderDisclosures(mount);
    });
    document.querySelectorAll('[data-del-w]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteDisclosureWatcher(b.dataset.delW);
            renderDisclosures(mount);
        }));

    // Poll loop.
    if (pollTimer) clearInterval(pollTimer);
    lastSeen = filings[0]?.id || '';
    pollTimer = setInterval(() => refreshFeed(watchers), 20_000);
    window.addEventListener('hashchange', () => {
        if (!window.location.hash.startsWith('#disclosures')) {
            clearInterval(pollTimer); pollTimer = null;
        }
    }, { once: true });
}

async function refreshFeed(watchers) {
    try {
        const all = await api.disclosures(null, null, 100);
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
        const feedEl = document.getElementById('feed');
        if (feedEl) feedEl.innerHTML = renderFeed(all);
    } catch (_) {}
}

function renderFeed(items) {
    if (!items.length) return '<p class="muted">No disclosures cached yet — hit "Poll now" to fetch.</p>';
    return `<table class="trades">
        <thead><tr><th>Filed</th><th>Kind</th><th>Filer</th><th>Symbol</th>
            <th>Tx</th><th>Shares</th><th>Amount</th><th>Source</th></tr></thead>
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
    return KINDS.find(x => x.id === k)?.label || k;
}
