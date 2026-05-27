// Sentiment-tagged news — searchable history + per-symbol feed.
import { api } from '../api.js';
import { esc } from '../util.js';
import { on as onWsEvent } from '../ws.js';

let timer = null;
let wsUnsub = null;
let lastQuery = { mode: 'recent', sym: '', q: '' };

export async function renderNews(mount) {
    mount.innerHTML = `
        <h1 class="view-title">// NEWS</h1>
        <p class="muted small">Yahoo headlines polled per watchlist symbol every 5 minutes,
            scored with the same WSB-aware sentiment lexicon used for social feeds, and
            indexed for full-text search via Postgres tsvector / websearch_to_tsquery.
            Color stripe on each row maps to sentiment: red (negative) → grey → green (positive).</p>

        <div class="chart-panel">
            <form id="n-form" class="inline-form">
                <select name="mode">
                    <option value="recent">recent (global)</option>
                    <option value="symbol">by symbol</option>
                    <option value="search">full-text search</option>
                </select>
                <input name="value" placeholder="symbol or query" style="min-width:200px;">
                <label>Limit <input name="limit" type="number" min="10" max="200" value="40" style="width:80px;"></label>
                <button class="primary" type="submit">Fetch</button>
                <button type="button" class="btn" id="n-poll-now">Poll now</button>
                <span id="n-status" class="muted small"></span>
            </form>
        </div>

        <div id="n-list"><div class="boot">loading…</div></div>
    `;
    document.getElementById('n-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        lastQuery = {
            mode: fd.get('mode'),
            sym: fd.get('value').trim().toUpperCase(),
            q: fd.get('value').trim(),
            limit: Number(fd.get('limit')) || 40,
        };
        await refresh();
    });
    document.getElementById('n-poll-now').addEventListener('click', async () => {
        const status = document.getElementById('n-status');
        status.textContent = 'polling watchlists…';
        try {
            const s = await api.newsPollNow();
            status.textContent = `${s.symbols_polled} symbols / ${s.inserted} new`;
            await refresh();
        } catch (e) { status.textContent = 'error: ' + e.message; }
    });

    if (timer) clearInterval(timer);
    timer = setInterval(refresh, 120_000);
    if (wsUnsub) wsUnsub();
    wsUnsub = onWsEvent('news', () => refresh());

    window.addEventListener('hashchange', () => {
        if (!window.location.hash.startsWith('#news')) {
            clearInterval(timer); timer = null;
            if (wsUnsub) { wsUnsub(); wsUnsub = null; }
        }
    }, { once: true });

    await refresh();
}

async function refresh() {
    const list = document.getElementById('n-list');
    if (!list) return;
    const limit = lastQuery.limit || 40;
    try {
        let items;
        if (lastQuery.mode === 'symbol' && lastQuery.sym) {
            items = await api.newsBySymbol(lastQuery.sym, limit);
        } else if (lastQuery.mode === 'search' && lastQuery.q) {
            items = await api.newsSearch(lastQuery.q, limit);
        } else {
            items = await api.newsRecent(limit);
        }
        renderList(list, items);
    } catch (e) { list.innerHTML = `<p class="boot">${esc(e.message)}</p>`; }
}

function renderList(el, items) {
    if (!items.length) { el.innerHTML = '<p class="muted small">no items</p>'; return; }
    el.innerHTML = items.map(n => row(n)).join('');
}

function sentimentBar(s) {
    if (s == null) return '<span style="display:inline-block;width:8px;height:18px;background:#444;margin-right:8px;"></span>';
    const t = Math.max(-1, Math.min(1, s));
    const color = t > 0.1 ? '#7af0a8' : t < -0.1 ? '#ff1f7a' : '#9aa0c8';
    return `<span title="sentiment ${t.toFixed(2)}" style="display:inline-block;width:8px;height:18px;background:${color};margin-right:8px;vertical-align:middle;"></span>`;
}

function row(n) {
    const when = n.published_at || n.fetched_at;
    const ago = relativeTime(when);
    const link = n.link
        ? `<a href="${esc(n.link)}" target="_blank" rel="noopener">${esc(n.title)}</a>`
        : esc(n.title);
    return `<div class="chart-panel" style="padding:8px 10px;margin-bottom:6px;">
        <div style="display:flex;align-items:flex-start;">
            ${sentimentBar(n.sentiment)}
            <div style="flex:1 1 auto;min-width:0;">
                <div style="font-size:13px;">${link}</div>
                <div class="muted small">
                    <strong>${esc(n.symbol)}</strong> ·
                    ${esc(n.publisher || 'unknown')} · ${esc(ago)}
                    ${n.sentiment != null ? ` · sentiment ${n.sentiment.toFixed(2)}` : ''}
                </div>
            </div>
        </div>
    </div>`;
}

function relativeTime(iso) {
    if (!iso) return '';
    const t = new Date(iso).getTime();
    const diff = (Date.now() - t) / 1000;
    if (diff < 60)   return `${Math.floor(diff)}s ago`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return `${Math.floor(diff / 86400)}d ago`;
}
